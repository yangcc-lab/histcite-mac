mod background;

use histcite_core::{
    Analysis, MatchingMode,
    collection::{Collection, SavedGraph, Tag, atomic_write},
    safety::{atomic_write_protected, read_text},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use tauri::{Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Workspace {
    initialized: bool,
    collection: Option<Collection>,
    analysis: Option<Analysis>,
    revision: u64,
    dirty: bool,
    network_pending: bool,
    path: Option<PathBuf>,
    sources: Vec<String>,
    undo: Vec<Snapshot>,
    marks: BTreeSet<String>,
    log: Vec<String>,
}
#[derive(Clone)]
struct Snapshot {
    collection: Collection,
    path: Option<PathBuf>,
    sources: Vec<String>,
    marks: BTreeSet<String>,
}
type DataState = background::Background<Workspace>;
#[derive(Serialize, Deserialize)]
struct Recovery {
    collection: Option<Collection>,
    path: Option<PathBuf>,
    sources: Vec<String>,
    dirty: bool,
}
fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
fn recovery_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("workspace-v1.json"))
}
fn persist(
    app: &tauri::AppHandle,
    c: Option<&Collection>,
    path: Option<&PathBuf>,
    sources: &[String],
    dirty: bool,
) -> Result<(), String> {
    let data = Recovery {
        collection: c.cloned(),
        path: path.cloned(),
        sources: sources.to_vec(),
        dirty,
    };
    histcite_core::safety::check_serialized_size(&data)?;
    atomic_write(
        &recovery_path(app)?,
        &serde_json::to_vec(&data).map_err(|e| e.to_string())?,
    )
}
impl Workspace {
    fn snapshot(&self) -> Option<Snapshot> {
        self.collection.as_ref().map(|c| Snapshot {
            collection: c.clone(),
            path: self.path.clone(),
            sources: self.sources.clone(),
            marks: self.marks.clone(),
        })
    }
    fn remember(&mut self, old: Snapshot) {
        self.undo.push(old);
        if self.undo.len() > 20 {
            self.undo.remove(0);
        }
    }
    fn view(&self) -> Result<Value, String> {
        let Some(a) = &self.analysis else {
            return Ok(Value::Null);
        };
        let c = self.collection.as_ref().ok_or("集合尚未載入")?;
        let mut v = serde_json::to_value(a).map_err(|e| e.to_string())?;
        v["collection"] = json!({"title":c.title,"description":c.description,"created":c.created,"comment":c.comment,"tags":c.tags,"graphs":c.graphs,"dirty":self.dirty,"save_path":self.path,"revision":self.revision,"network_pending":self.network_pending,"undo_count":self.undo.len(),"marks":self.marks,"log":self.log});
        v["settings"] = serde_json::to_value(&c.settings).map_err(|e| e.to_string())?;
        Ok(v)
    }
    fn commit(
        &mut self,
        app: &tauri::AppHandle,
        c: Collection,
        label: &str,
        remember: bool,
        defer: bool,
    ) -> Result<Value, String> {
        c.validate()?;
        let analysis = if defer {
            if let Some(old) = &self.analysis {
                let mut a = old.clone();
                a.records = c.records.clone();
                histcite_core::analytics::refresh(&mut a, &c.settings);
                a
            } else {
                c.analyze()
            }
        } else {
            c.analyze()
        };
        persist(app, Some(&c), self.path.as_ref(), &self.sources, true)?;
        if remember {
            if let Some(old) = self.snapshot() {
                self.remember(old);
            }
        }
        let old_merges = self
            .collection
            .as_ref()
            .map(|old| old.merge_log.len())
            .unwrap_or(0);
        for entry in c.merge_log.iter().skip(old_merges) {
            if self.marks.remove(&entry.incoming.id) {
                self.marks.insert(entry.kept_id.clone());
            }
        }
        self.network_pending = defer;
        self.collection = Some(c);
        self.analysis = Some(analysis);
        self.revision += 1;
        self.dirty = true;
        self.log.push(format!("{} {label}", now()));
        if self.log.len() > 500 {
            self.log.remove(0);
        }
        let ids: BTreeSet<_> = self
            .collection
            .as_ref()
            .ok_or("集合尚未載入")?
            .records
            .iter()
            .map(|r| r.id.clone())
            .collect();
        self.marks.retain(|id| ids.contains(id));
        self.view()
    }
}
fn notify(app: &tauri::AppHandle, revision: u64) {
    if let Err(error) = app.emit("collection-changed", revision) {
        eprintln!("Collection notification failed at revision {revision}: {error}");
    }
}
fn read_collection(path: &str) -> Result<Collection, String> {
    let text = read_text(Path::new(path)).map_err(|e| format!("無法讀取 {path}：{e}"))?;
    Collection::from_text(&text, path)
}
#[tauri::command]
async fn initial_analysis(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
) -> Result<Value, String> {
    state
        .run(move |w| {
            if w.initialized {
                return w.view();
            }
            let mut paths: Vec<String> = std::env::args()
                .skip(1)
                .filter(|p| [".txt", ".hci", ".hcm"].iter().any(|ext| p.ends_with(ext)))
                .collect();
            if paths.is_empty() {
                let recovery_file = recovery_path(&app)?;
                if recovery_file.exists() {
                    let text =
                        read_text(&recovery_file).map_err(|e| format!("無法讀取工作副本：{e}"))?;
                    let recovery: Recovery = serde_json::from_str(&text)
                        .map_err(|e| format!("無法讀取工作副本：{e}"))?;
                    if let Some(c) = &recovery.collection {
                        c.validate()?;
                        w.analysis = Some(c.analyze());
                    }
                    w.collection = recovery.collection;
                    w.path = recovery.path;
                    w.sources = recovery.sources;
                    w.dirty = recovery.dirty;
                    w.initialized = true;
                    return w.view();
                }
                if let Ok(dir) = app.path().app_config_dir() {
                    if let Ok(bytes) = std::fs::read(dir.join("last-files.json")) {
                        paths = serde_json::from_slice(&bytes).unwrap_or_default();
                    }
                }
            }
            if !paths.is_empty() {
                let mut c = read_collection(&paths[0])?;
                for path in paths.iter().skip(1) {
                    c.merge(read_collection(path)?);
                }
                c.validate()?;
                w.analysis = Some(c.analyze());
                w.path = if paths.len() == 1 && paths[0].ends_with(".hcm") {
                    Some(PathBuf::from(&paths[0]))
                } else {
                    None
                };
                persist(&app, Some(&c), w.path.as_ref(), &paths, false)?;
                w.collection = Some(c);
                w.sources = paths;
            }
            w.initialized = true;
            w.view()
        })
        .await
}
#[tauri::command]
async fn current_analysis(state: State<'_, DataState>) -> Result<Value, String> {
    state.run(|w| w.view()).await
}
#[tauri::command]
async fn current_revision(state: State<'_, DataState>) -> Result<u64, String> {
    state.run(|w| Ok(w.revision)).await
}
#[tauri::command]
async fn choose_files(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    mode: MatchingMode,
    append: Option<bool>,
) -> Result<Value, String> {
    let opened_revision = state.run(|w| Ok(w.revision)).await?;
    let picker = app.clone();
    let files = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title("Open HistCite / WoS collection")
            .add_filter("HistCite and Web of Science", &["txt", "hci", "hcm"])
            .blocking_pick_files()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(files) = files else {
        return Ok(Value::Null);
    };
    let paths: Vec<String> = files
        .into_iter()
        .map(|f| {
            f.into_path()
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| e.to_string())
        })
        .collect::<Result<_, _>>()?;
    state
        .run(move |w| {
            let first = paths.first().ok_or("沒有選擇檔案")?;
            let mut imported = read_collection(first)?;
            for path in paths.iter().skip(1) {
                imported.merge(read_collection(path)?);
            }
            if w.revision != opened_revision {
                return Err("開啟檔案期間集合已更新；請重新選擇檔案。".into());
            }
            let mut c = if append.unwrap_or(false) {
                let mut previous = w.collection.clone().unwrap_or_default();
                previous.merge(imported);
                previous
            } else {
                imported
            };
            c.mode = mode;
            let previous_snapshot = w.snapshot();
            let previous_sources = w.sources.clone();
            let previous_path = w.path.clone();
            if append.unwrap_or(false) {
                w.sources.extend(paths.clone());
            } else {
                w.sources = paths.clone();
                w.path = if paths.len() == 1 && paths[0].ends_with(".hcm") {
                    Some(PathBuf::from(&paths[0]))
                } else {
                    None
                };
            }
            let mut result = w.commit(&app, c, "Imported files", false, false);
            if result.is_ok() {
                if let Some(old) = previous_snapshot {
                    w.remember(old);
                }
                result = w.view();
            }
            if result.is_err() {
                w.sources = previous_sources;
                w.path = previous_path;
            }
            let revision = w.revision;
            if result.is_ok() {
                notify(&app, revision);
            }
            result
        })
        .await
}
#[tauri::command]
async fn change_mode(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    mode: MatchingMode,
) -> Result<Value, String> {
    state
        .run(move |w| {
            let mut c = w.collection.clone().ok_or("請先匯入資料")?;
            c.mode = mode;
            let v = w.commit(&app, c, "Changed matching mode", true, false)?;
            let rev = w.revision;
            notify(&app, rev);
            Ok(v)
        })
        .await
}
#[tauri::command]
async fn mutate_collection(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    revision: u64,
    operation: Value,
) -> Result<Value, String> {
    state
        .run(move |w| {
            if w.revision != revision && !(w.collection.is_none() && operation["action"] == "new") {
                return Err("集合已在另一個視窗更新；請重新載入後再套用變更。".into());
            }
            let op = operation["action"].as_str().ok_or("缺少操作名稱")?;
            let mut c = w.collection.clone().unwrap_or_default();
            let string = |name: &str| operation[name].as_str().unwrap_or("").to_string();
            let ids: BTreeSet<String> =
                serde_json::from_value(operation.get("ids").cloned().unwrap_or(json!([])))
                    .map_err(|e| e.to_string())?;
            let existing: BTreeSet<_> = c.records.iter().map(|r| r.id.clone()).collect();
            if !ids.is_subset(&existing) {
                return Err("選取的記錄已不存在".into());
            }
            match op {
                "mark" | "unmark" => {
                    if op == "mark" {
                        w.marks.extend(ids);
                    } else {
                        w.marks.retain(|id| !ids.contains(id));
                    }
                    w.revision += 1;
                    let v = w.view()?;
                    let rev = w.revision;
                    notify(&app, rev);
                    return Ok(v);
                }
                "record" => c.edit_record(operation["id"].as_str(), &string("text"))?,
                "delete" => c.delete(&ids),
                "deduplicate" => c.consolidate(),
                "properties" => {
                    c.title = string("title");
                    c.description = string("description");
                    c.created = string("created");
                    c.comment = string("comment");
                }
                "settings" => {
                    c.settings = serde_json::from_value(operation["settings"].clone())
                        .map_err(|e| e.to_string())?;
                    c.settings.validate()?;
                }
                "tag" => {
                    let name = string("name").trim().to_uppercase();
                    if name.is_empty()
                        || name.len() > 200
                        || name.chars().any(char::is_whitespace)
                        || name.contains([';', ':'])
                    {
                        return Err("請輸入有效標籤名稱（不含空白、分號或冒號）".into());
                    }
                    let t = c.tags.entry(name).or_insert_with(Tag::default);
                    t.description = string("description");
                    t.records.extend(ids);
                }
                "untag" => {
                    let names: Vec<String> = operation
                        .get("names")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_else(|| vec![string("name")]);
                    for name in names {
                        if let Some(t) = c.tags.get_mut(&name.to_uppercase()) {
                            t.records.retain(|id| !ids.contains(id));
                        }
                    }
                    c.tags.retain(|_, t| !t.records.is_empty());
                }
                "clear_tags" => {
                    for t in c.tags.values_mut() {
                        t.records.retain(|id| !ids.contains(id));
                    }
                }
                "rename" => {
                    let names: BTreeSet<String> =
                        serde_json::from_value(operation["names"].clone())
                            .map_err(|e| e.to_string())?;
                    c.replace_items(&string("kind"), &names, &string("replacement"))?;
                }
                "save_graph" => {
                    let mut g: SavedGraph = serde_json::from_value(operation["graph"].clone())
                        .map_err(|e| e.to_string())?;
                    if g.id.is_empty() {
                        g.id = now();
                    }
                    if g.records.iter().any(|id| !existing.contains(id)) {
                        return Err("圖中包含不存在的文獻".into());
                    }
                    if let Some(old) = c.graphs.iter_mut().find(|x| x.id == g.id) {
                        *old = g;
                    } else {
                        c.graphs.push(g);
                    }
                }
                "delete_graph" => c.graphs.retain(|g| g.id != string("id")),
                "undo" => {
                    c = w.undo.last().ok_or("沒有可復原的變更")?.collection.clone();
                }
                "new" => c = Collection::default(),
                "update_network" => {}
                "reset_settings" => c.settings = Default::default(),
                _ => return Err("不支援的集合操作".into()),
            }
            let defer = op == "rename" && operation["defer"].as_bool().unwrap_or(false);
            let previous_location = (w.path.clone(), w.sources.clone(), w.marks.clone());
            if op == "undo" {
                let saved = w.undo.last().ok_or("沒有可復原的變更")?.clone();
                w.path = saved.path;
                w.sources = saved.sources;
                w.marks = saved.marks;
            }
            if op == "new" {
                w.path = None;
                w.sources.clear();
                w.marks.clear();
            }
            let result = w.commit(&app, c, op, op != "undo" && op != "update_network", defer);
            if result.is_err() {
                w.path = previous_location.0;
                w.sources = previous_location.1;
                w.marks = previous_location.2;
            }
            let v = result?;
            if op == "undo" {
                w.undo.pop();
            }
            let rev = w.revision;
            let v = if op == "undo" { w.view()? } else { v };
            notify(&app, rev);
            Ok(v)
        })
        .await
}
#[tauri::command]
async fn close_collection(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
) -> Result<(), String> {
    state
        .run(move |w| {
            persist(&app, None, None, &[], false)?;
            let rev = w.revision + 1;
            *w = Workspace {
                initialized: true,
                revision: rev,
                ..Default::default()
            };
            notify(&app, rev);
            Ok(())
        })
        .await
}
#[tauri::command]
async fn save_collection(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    save_as: bool,
) -> Result<Option<String>, String> {
    let (c, previous, revision) = state
        .run(|w| {
            Ok((
                w.collection.clone().ok_or("請先匯入資料")?,
                w.path.clone(),
                w.revision,
            ))
        })
        .await?;
    let path = if let Some(previous) = previous.filter(|_| !save_as) {
        previous
    } else {
        let picker = app.clone();
        let selected = tauri::async_runtime::spawn_blocking(move || {
            picker
                .dialog()
                .file()
                .add_filter("HistCite Mac collection", &["hcm"])
                .set_file_name("collection.hcm")
                .blocking_save_file()
        })
        .await
        .map_err(|e| e.to_string())?;
        let Some(file) = selected else {
            return Ok(None);
        };
        file.into_path().map_err(|e| e.to_string())?
    };
    state
        .run(move |w| {
            if w.revision != revision {
                return Err("儲存期間集合已變更；請再按一次儲存。".into());
            }
            protect_source(&w, &path)?;
            atomic_write_protected(
                &path,
                &serde_json::to_vec(&c).map_err(|e| e.to_string())?,
                &protected_sources(&w),
            )?;
            persist(&app, Some(&c), Some(&path), &w.sources, false)?;
            w.path = Some(path.clone());
            w.dirty = false;
            w.log.push(format!("{} Saved {}", now(), path.display()));
            w.revision += 1;
            notify(&app, w.revision);
            Ok(Some(path.to_string_lossy().to_string()))
        })
        .await
}
fn protect_source(w: &Workspace, path: &Path) -> Result<(), String> {
    let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if w.sources.iter().filter(|s| !s.ends_with(".hcm")).any(|s| {
        Path::new(s)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(s))
            == target
    }) {
        return Err("請另存新檔，保留原始 WoS／HCI 資料。".into());
    }
    Ok(())
}
fn protected_sources(w: &Workspace) -> Vec<PathBuf> {
    w.sources
        .iter()
        .filter(|s| !s.ends_with(".hcm"))
        .map(PathBuf::from)
        .collect()
}
#[tauri::command]
async fn save_export(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    format: String,
    content: Option<String>,
    svg: Option<String>,
    ids: Option<Vec<String>>,
) -> Result<Option<String>, String> {
    let data_format = format.clone();
    let (data, mut sources) = state
        .run(move |w| {
            let format = data_format;
            if w.network_pending && format == "json" {
                return Err("請先按 Update lists 更新引用網絡。".into());
            }
            let data = match format.as_str() {
                "json" => serde_json::to_vec_pretty(w.analysis.as_ref().ok_or("請先匯入文獻")?)
                    .map_err(|e| e.to_string())?,
                "hci" => w
                    .collection
                    .as_ref()
                    .ok_or("請先匯入文獻")?
                    .hci(
                        ids.as_ref()
                            .map(|v| v.iter().cloned().collect::<BTreeSet<_>>())
                            .as_ref(),
                    )
                    .into_bytes(),
                "png" => {
                    use base64::Engine;
                    let text = content.ok_or("沒有匯出內容")?;
                    base64::engine::general_purpose::STANDARD
                        .decode(text)
                        .map_err(|e| e.to_string())?
                }
                "csv" | "svg" | "dot" | "net" | "ps" => {
                    content.or(svg).ok_or("沒有匯出內容")?.into_bytes()
                }
                _ => return Err("不支援的匯出格式".into()),
            };
            // Keep source identities from the exported snapshot as well as
            // the current workspace if another window imports during the picker.
            Ok((data, protected_sources(w)))
        })
        .await?;
    if data.len() > 200_000_000 {
        return Err("匯出內容超過 200 MB".into());
    }
    let picker = app.clone();
    let ext = format.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .add_filter("HistCite export", &[ext.as_str()])
            .set_file_name(format!("histcite-analysis.{ext}"))
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(file) = selected else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    state
        .run(move |w| {
            protect_source(w, &path)?;
            sources.extend(protected_sources(w));
            atomic_write_protected(&path, &data, &sources)?;
            Ok(Some(path.to_string_lossy().to_string()))
        })
        .await
}
#[tauri::command]
async fn open_record(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    index: usize,
    number: usize,
) -> Result<(), String> {
    let id = state
        .run(move |w| {
            Ok(w.analysis
                .as_ref()
                .and_then(|a| a.records.get(index))
                .ok_or("找不到這筆文獻")?
                .id
                .clone())
        })
        .await?;
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let label = format!("record-{:x}", hasher.finish());
    if let Some(window) = app.get_webview_window(&label) {
        return window.set_focus().map_err(|e| e.to_string());
    }
    let query = urlencoding::encode(&id);
    tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::App(format!("detail.html?id={query}&index={index}").into()),
    )
    .title(format!("HistCite - Record {number}"))
    .inner_size(900.0, 750.0)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
async fn open_graph(app: tauri::AppHandle, graph_id: Option<String>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("graph") {
        if let Some(id) = &graph_id {
            app.emit_to("graph", "load-graph", id)
                .map_err(|e| format!("無法同步圖形視窗：{e}"))?;
        }
        return window.set_focus().map_err(|e| e.to_string());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "graph",
        tauri::WebviewUrl::App(
            format!(
                "index.html?window=graph{}",
                graph_id
                    .map(|id| format!("&graph={}", urlencoding::encode(&id)))
                    .unwrap_or_default()
            )
            .into(),
        ),
    )
    .title("HistCite - Graph Maker")
    .inner_size(1200.0, 850.0)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
async fn open_list(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
    ids: Vec<String>,
    title: String,
) -> Result<(), String> {
    let query = state
        .run(move |w| {
            let existing: BTreeSet<_> = w
                .collection
                .as_ref()
                .ok_or("請先匯入資料")?
                .records
                .iter()
                .map(|r| r.id.as_str())
                .collect();
            let ids: Vec<_> = ids
                .into_iter()
                .filter(|id| existing.contains(id.as_str()))
                .collect();
            serde_json::to_string(&ids).map_err(|e| e.to_string())
        })
        .await?;
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    query.hash(&mut h);
    title.hash(&mut h);
    let label = format!("list-{:x}", h.finish());
    if let Some(window) = app.get_webview_window(&label) {
        return window.set_focus().map_err(|e| e.to_string());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::App(
            format!(
                "index.html?subset={}&title={}",
                urlencoding::encode(&query),
                urlencoding::encode(&title)
            )
            .into(),
        ),
    )
    .title(format!("HistCite - {title}"))
    .inner_size(1100.0, 750.0)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn print_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.print().map_err(|e| e.to_string())
}
#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) || url.contains(['\n', '\r']) {
        return Err("無效網頁位址".into());
    }
    let mut child = std::process::Command::new("/usr/bin/open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    // Reap LaunchServices' short-lived helper without blocking the UI thread.
    std::thread::spawn(move || {
        if let Err(error) = child.wait() {
            eprintln!("Unable to wait for browser helper: {error}");
        }
    });
    Ok(())
}
#[tauri::command]
async fn export_html(
    app: tauri::AppHandle,
    state: State<'_, DataState>,
) -> Result<Option<String>, String> {
    let data = state
        .run(|w| {
            if w.network_pending {
                return Err("請先按 Update lists 更新引用網絡。".into());
            }
            w.view()
        })
        .await?;
    if data.is_null() {
        return Err("請先匯入文獻".into());
    }
    let picker = app.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title("Choose folder for HTML presentation")
            .blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(file) = selected else {
        return Ok(None);
    };
    state
        .run(move |_| {
            let parent = file.into_path().map_err(|e| e.to_string())?;
            let dir = parent.join(format!("HistCite-presentation-{}", now()));
            std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
            let files = [
                ("index.html", include_str!("../../ui/index.html")),
                ("detail.html", include_str!("../../ui/detail.html")),
                ("style.css", include_str!("../../ui/style.css")),
                ("common.js", include_str!("../../ui/common.js")),
                ("app.js", include_str!("../../ui/app.js")),
                ("detail.js", include_str!("../../ui/detail.js")),
                ("graph.js", include_str!("../../ui/graph.js")),
                ("graph-layout.js", include_str!("../../ui/graph-layout.js")),
                ("graphviz.js", include_str!("../../ui/graphviz.js")),
                ("THIRD-PARTY.txt", include_str!("../../ui/THIRD-PARTY.txt")),
            ];
            for (name, text) in files {
                let text = if name.ends_with(".html") {
                    text.replace(
                        "<script src=\"common.js\">",
                        "<script src=\"data.js\"></script><script src=\"common.js\">",
                    )
                } else {
                    text.to_string()
                };
                atomic_write(&dir.join(name), text.as_bytes())?;
            }
            atomic_write(
                &dir.join("Graphviz-source.zip"),
                include_bytes!("../../ui/Graphviz-source.zip"),
            )?;
            let text = format!(
                "window.__HISTCITE_EXPORT__={};",
                serde_json::to_string(&data).map_err(|e| e.to_string())?
            );
            atomic_write(&dir.join("data.js"), text.as_bytes())?;
            Ok(Some(dir.to_string_lossy().to_string()))
        })
        .await
}
#[tauri::command]
async fn quit_app(app: tauri::AppHandle, state: State<'_, DataState>) -> Result<(), String> {
    state.finish().await;
    app.exit(0);
    Ok(())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DataState::default())
        .invoke_handler(tauri::generate_handler![
            initial_analysis,
            current_analysis,
            current_revision,
            choose_files,
            change_mode,
            mutate_collection,
            close_collection,
            save_collection,
            save_export,
            open_record,
            open_graph,
            open_list,
            print_window,
            open_external,
            export_html,
            quit_app
        ])
        .build(tauri::generate_context!())
        .expect("Unable to run HistCite Mac")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    app.state::<DataState>().finish().await;
                    app.exit(0);
                });
            }
        });
}
