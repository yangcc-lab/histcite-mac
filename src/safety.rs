//! Limits and filesystem boundaries for user-supplied collections.
use crate::Record;
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_INPUT_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_RECORDS: usize = 100_000;
pub const MAX_REFERENCES: usize = 2_000_000;

pub fn read_text(path: &Path) -> Result<String, String> {
    #[cfg(unix)]
    let file = {
        use rustix::fs::{CWD, Mode, OFlags, openat};
        std::fs::File::from(
            openat(
                CWD,
                path,
                OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK,
                Mode::empty(),
            )
            .map_err(|e| e.to_string())?,
        )
    };
    #[cfg(not(unix))]
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES as u64 {
        return Err("請選擇不超過 100 MiB 的一般文字檔案".into());
    }
    // The read limit also applies when the file grows after metadata was read.
    let mut text = String::new();
    file.take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    check_text_size(&text)?;
    Ok(text)
}

pub fn check_text_size(text: &str) -> Result<(), String> {
    if text.len() > MAX_INPUT_BYTES {
        Err("文獻檔案超過 100 MiB；請縮小集合後再匯入".into())
    } else {
        Ok(())
    }
}

pub fn check_serialized_size(value: &impl Serialize) -> Result<(), String> {
    struct Budget(usize);
    impl Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| std::io::Error::other("工作集合超過 100 MiB；請縮小集合"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(MAX_INPUT_BYTES), value).map_err(|e| e.to_string())
}

pub fn validate_records(records: &[Record]) -> Result<(), String> {
    if records.len() > MAX_RECORDS {
        return Err("集合超過 100,000 篇文獻；請縮小集合".into());
    }
    let mut references = 0usize;
    for record in records {
        if record.year.is_some_and(|y| !(1..=9999).contains(&y)) {
            return Err("發表年份必須介於 1 與 9999；未知年份請留白".into());
        }
        // Together with the serialized-size bound, this also bounds weighted totals.
        if record.gcs.is_some_and(|g| g > u32::MAX as u64) {
            return Err("GCS 超出有效範圍（0–4,294,967,295）".into());
        }
        references = references
            .checked_add(record.references.len())
            .ok_or("參考文獻過多")?;
        if references > MAX_REFERENCES {
            return Err("集合超過 2,000,000 條參考文獻".into());
        }
    }
    check_serialized_size(&records)
}

pub fn validate_json_depth(value: &Value) -> Result<(), String> {
    fn visit(value: &Value, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match value {
            Value::Array(items) => items.iter().all(|v| visit(v, depth + 1)),
            Value::Object(items) => items.values().all(|v| visit(v, depth + 1)),
            _ => true,
        }
    }
    if visit(value, 0) {
        Ok(())
    } else {
        Err("圖形設定巢狀層數過多".into())
    }
}

#[cfg(unix)]
struct Destination {
    directory: std::fs::File,
    name: std::ffi::OsString,
}

#[cfg(unix)]
impl Destination {
    fn open(path: &Path) -> Result<Self, String> {
        use rustix::fs::{CWD, Mode, OFlags, openat};
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let directory = openat(
            CWD,
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            directory: directory.into(),
            name: path.file_name().ok_or("無效存檔路徑")?.into(),
        })
    }
    fn write(&self, bytes: &[u8], protected: &[PathBuf]) -> Result<(), String> {
        use rustix::fs::{AtFlags, Mode, OFlags, openat, renameat, statat, unlinkat};
        use std::os::unix::fs::MetadataExt;
        // Check the target through the SAME pinned directory used for replacement.
        // A changed ancestor symlink cannot redirect a later pathname resolution.
        match statat(&self.directory, &self.name, AtFlags::empty()) {
            Ok(target) => {
                for source in protected {
                    match std::fs::metadata(source) {
                        Ok(m)
                            if m.dev() == target.st_dev as u64
                                && m.ino() == target.st_ino as u64 =>
                        {
                            return Err("請另存新檔，保留原始 WoS／HCI 資料。".into());
                        }
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => return Err(e.to_string()),
                    }
                }
            }
            Err(rustix::io::Errno::NOENT) => {}
            Err(e) => return Err(e.to_string()),
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = format!(".histcite-{}-{stamp}.tmp", std::process::id());
        let fd = openat(
            &self.directory,
            temporary.as_str(),
            OFlags::CREATE | OFlags::EXCL | OFlags::WRONLY | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|e| e.to_string())?;
        // Cleanup is attempted only after this process successfully creates the file.
        let mut file = std::fs::File::from(fd);
        let result = file
            .write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())
            .and_then(|_| {
                renameat(
                    &self.directory,
                    temporary.as_str(),
                    &self.directory,
                    &self.name,
                )
                .map_err(|e| e.to_string())
            });
        if let Err(original) = result {
            if let Err(cleanup) = unlinkat(&self.directory, temporary.as_str(), AtFlags::empty()) {
                return Err(format!("{original}；暫存檔清理失敗：{cleanup}"));
            }
            return Err(original);
        }
        Ok(())
    }
}

pub fn atomic_write_protected(
    path: &Path,
    bytes: &[u8],
    protected: &[PathBuf],
) -> Result<(), String> {
    #[cfg(unix)]
    {
        Destination::open(path)?.write(bytes, protected)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, bytes, protected);
        Err("安全的原子存檔目前僅支援 Unix 系統".into())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn replacement_uses_the_open_directory_even_after_it_is_renamed() {
        let root = std::env::temp_dir().join(format!(
            "histcite-directory-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let original = root.join("original");
        let moved = root.join("moved");
        std::fs::create_dir_all(&original).unwrap();
        let target = Destination::open(&original.join("result.hcm")).unwrap();
        std::fs::rename(&original, &moved).unwrap();
        std::fs::create_dir(&original).unwrap();
        target.write(b"synthetic result", &[]).unwrap();
        assert_eq!(
            std::fs::read(moved.join("result.hcm")).unwrap(),
            b"synthetic result"
        );
        assert!(!original.join("result.hcm").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
