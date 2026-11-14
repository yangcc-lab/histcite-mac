use histcite_core::{collection::Collection, parse_wos, safety};

fn text(year: u32, gcs: u64) -> String {
    format!("PT J\nUT synthetic:limits\nTI Input validation\nPY {year}\nTC {gcs}\nNR 0\nER\n")
}

#[test]
fn invalid_years_and_counters_are_rejected_in_text_and_hcm() {
    for year in [0, 10_000, u32::MAX] {
        assert!(parse_wos(&text(year, 0), "synthetic.txt").is_err());
    }
    assert!(parse_wos(&text(2026, u64::MAX), "synthetic.txt").is_err());
    let mut collection = Collection::from_text(&text(2026, 3), "synthetic.txt").unwrap();
    collection.records[0].year = Some(u32::MAX);
    assert!(
        Collection::from_text(
            &serde_json::to_string(&collection).unwrap(),
            "synthetic.hcm"
        )
        .is_err()
    );
    collection.records[0].year = Some(2026);
    collection.records[0].gcs = Some(u64::MAX);
    assert!(collection.validate().is_err());
    assert!(parse_wos(&text(9999, u32::MAX as u64), "synthetic.txt").is_ok());
}

#[test]
fn settings_keep_enough_json_depth_for_recovery_round_trip() {
    let mut collection = Collection::from_text(&text(2026, 3), "synthetic.txt").unwrap();
    let mut value = serde_json::Value::Null;
    for _ in 0..16 {
        value = serde_json::json!([value]);
    }
    collection.settings.graph_settings = value.clone();
    collection.validate().unwrap();
    let recovery = serde_json::json!({"collection": collection});
    let encoded = serde_json::to_string(&recovery).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    let reopened: Collection = serde_json::from_value(decoded["collection"].clone()).unwrap();
    reopened.validate().unwrap();
    collection.settings.graph_settings = serde_json::json!([value.clone()]);
    assert!(collection.validate().is_err());
    collection.settings.graph_settings = serde_json::json!({});
    collection
        .graphs
        .push(histcite_core::collection::SavedGraph {
            id: "synthetic".into(),
            title: "Synthetic".into(),
            description: "".into(),
            records: vec![],
            settings: serde_json::json!([value]),
        });
    assert!(collection.validate().is_err());
}

#[cfg(unix)]
#[test]
fn writes_are_private_and_protected_source_aliases_are_rejected() {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!(
        "histcite-security-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("source.txt");
    let alias = directory.join("alias.txt");
    let saved = directory.join("saved.hcm");
    std::fs::write(&source, b"original synthetic data").unwrap();
    std::fs::hard_link(&source, &alias).unwrap();
    assert!(safety::atomic_write_protected(&alias, b"replacement", &[source.clone()]).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), b"original synthetic data");
    safety::atomic_write_protected(&saved, b"first", &[]).unwrap();
    assert_eq!(
        std::fs::metadata(&saved).unwrap().permissions().mode() & 0o777,
        0o600
    );
    safety::atomic_write_protected(&saved, b"second", &[]).unwrap();
    assert_eq!(std::fs::read(&saved).unwrap(), b"second");
    assert_eq!(
        std::fs::metadata(&saved).unwrap().permissions().mode() & 0o777,
        0o600
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn oversized_regular_file_is_rejected_before_reading_its_contents() {
    let path = std::env::temp_dir().join(format!(
        "histcite-size-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .unwrap();
    // Sparse metadata-only file: no large allocation or large data write.
    file.set_len(safety::MAX_INPUT_BYTES as u64 + 1).unwrap();
    assert!(safety::read_text(&path).is_err());
    drop(file);
    std::fs::remove_file(path).unwrap();
    assert!(safety::read_text(&std::env::temp_dir()).is_err());
}
