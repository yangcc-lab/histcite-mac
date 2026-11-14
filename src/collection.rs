//! Editable, portable collections. Source WoS files are never rewritten.
use crate::{
    Analysis, MatchingMode, Record,
    analytics::{self, Settings},
    analyze_retained,
    duplicates::{self, MergeRecord},
    parse_wos,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tag {
    pub description: String,
    pub records: BTreeSet<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGraph {
    pub id: String,
    pub title: String,
    pub description: String,
    pub records: Vec<String>,
    pub settings: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Collection {
    pub schema_version: u32,
    pub title: String,
    pub description: String,
    pub created: String,
    pub comment: String,
    pub records: Vec<Record>,
    pub duplicate_records: usize,
    pub merge_log: Vec<MergeRecord>,
    pub mode: MatchingMode,
    pub settings: Settings,
    pub tags: BTreeMap<String, Tag>,
    pub graphs: Vec<SavedGraph>,
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            schema_version: 1,
            title: "Untitled Collection".into(),
            description: String::new(),
            created: String::new(),
            comment: String::new(),
            records: vec![],
            duplicate_records: 0,
            merge_log: vec![],
            mode: MatchingMode::Histcite,
            settings: Settings::default(),
            tags: BTreeMap::new(),
            graphs: vec![],
        }
    }
}
impl Collection {
    pub fn analyze(&self) -> Analysis {
        let mut a = analyze_retained(self.records.clone(), self.mode, self.merge_log.clone());
        a.duplicate_records = self.duplicate_records;
        analytics::refresh(&mut a, &self.settings);
        a
    }
    pub fn from_text(text: &str, source: &str) -> Result<Self, String> {
        crate::safety::check_text_size(text)?;
        if text.trim_start().starts_with('{') {
            let c: Self = serde_json::from_str(text).map_err(|e| format!("集合格式錯誤：{e}"))?;
            if c.schema_version != 1 {
                return Err("不支援的集合版本".into());
            }
            c.validate()?;
            return Ok(c);
        }
        let mut c = Self {
            records: parse_wos(text, source)?,
            ..Self::default()
        };
        let mut key = String::new();
        let mut meta: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for line in text.lines() {
            if line == "ER" || line.starts_with("PT ") {
                break;
            }
            if line.starts_with("   ") {
                meta.entry(key.clone()).or_default().push(line[3..].into());
            } else if line.as_bytes().get(2) == Some(&b' ') {
                key = line[..2].into();
                meta.entry(key.clone()).or_default().push(line[3..].into());
            }
        }
        for (tag, target) in [
            ("MT", &mut c.title),
            ("MC", &mut c.description),
            ("MO", &mut c.created),
            ("MN", &mut c.comment),
        ] {
            if let Some(lines) = meta.get(tag) {
                *target = lines.join("\n");
            }
        }
        if let Some(lines) = meta.get("TD") {
            for line in lines {
                let (name, desc) = line.split_once(':').unwrap_or((line, ""));
                c.tags.insert(
                    name.trim().to_string(),
                    Tag {
                        description: desc.trim().into(),
                        records: BTreeSet::new(),
                    },
                );
            }
        }
        let originals = c.records.clone();
        let dedup = duplicates::deduplicate(std::mem::take(&mut c.records));
        c.duplicate_records = dedup.merges.len();
        c.merge_log = dedup.merges;
        c.records = dedup.records;
        for (r, canonical_id) in originals.iter().zip(dedup.canonical_ids) {
            if let Some(tags) = r.fields.get("TG") {
                for name in tags
                    .iter()
                    .flat_map(|v| v.split(|ch: char| ch == ';' || ch.is_whitespace()))
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                {
                    c.tags
                        .entry(name.into())
                        .or_default()
                        .records
                        .insert(canonical_id.clone());
                }
            }
        }
        Ok(c)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.settings.validate()?;
        crate::safety::validate_records(&self.records)?;
        for graph in &self.graphs {
            crate::safety::validate_json_depth(&graph.settings)?;
        }
        let mut ids = BTreeSet::new();
        for r in &self.records {
            if r.id.is_empty() || !ids.insert(r.id.clone()) {
                return Err("集合含空白或重複識別碼".into());
            }
        }
        if self.graphs.len() > 1000 || self.tags.len() > 10000 {
            return Err("集合標籤或圖形過多".into());
        }
        crate::safety::check_serialized_size(self)
    }
    /// Reconcile imported records, and remap collection references to survivors.
    pub fn consolidate(&mut self) {
        let old_ids: Vec<_> = self.records.iter().map(|r| r.id.clone()).collect();
        let result = duplicates::deduplicate(std::mem::take(&mut self.records));
        let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (old, new) in old_ids.into_iter().zip(result.canonical_ids) {
            aliases.entry(old).or_default().insert(new);
        }
        for tag in self.tags.values_mut() {
            tag.records = tag
                .records
                .iter()
                .flat_map(|id| {
                    aliases
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| BTreeSet::from([id.clone()]))
                })
                .collect();
        }
        for graph in &mut self.graphs {
            graph.records = graph
                .records
                .iter()
                .flat_map(|id| {
                    aliases
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| BTreeSet::from([id.clone()]))
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        for entry in &mut self.merge_log {
            if let Some(ids) = aliases.get(&entry.kept_id) {
                if ids.len() == 1 {
                    entry.kept_id = ids.first().unwrap().clone();
                }
            }
        }
        self.duplicate_records += result.merges.len();
        self.merge_log.extend(result.merges);
        self.records = result.records;
    }
    pub fn merge(&mut self, other: Self) {
        self.duplicate_records += other.duplicate_records;
        // Incoming handles can collide with existing handles for genuinely distinct
        // bibliography. Give them temporary unique handles before mapping tags.
        let mut reserved: BTreeSet<_> = self
            .records
            .iter()
            .chain(other.records.iter())
            .map(|r| r.id.clone())
            .collect();
        let mut occupied: BTreeSet<_> = self.records.iter().map(|r| r.id.clone()).collect();
        let mut incoming_ids = BTreeMap::new();
        for mut r in other.records {
            let old = r.id.clone();
            if !occupied.insert(old.clone()) {
                let mut n = 2;
                loop {
                    let candidate = format!("{old}#histcite-{n}");
                    if reserved.insert(candidate.clone()) {
                        r.id = candidate;
                        occupied.insert(r.id.clone());
                        break;
                    }
                    n += 1;
                }
            }
            incoming_ids.insert(old, r.id.clone());
            self.records.push(r);
        }
        for mut entry in other.merge_log {
            if let Some(new) = incoming_ids.get(&entry.kept_id) {
                entry.kept_id = new.clone();
            }
            self.merge_log.push(entry);
        }
        for (name, tag) in other.tags {
            let entry = self.tags.entry(name).or_default();
            entry.records.extend(
                tag.records
                    .into_iter()
                    .map(|id| incoming_ids.get(&id).cloned().unwrap_or(id)),
            );
            if entry.description.is_empty() {
                entry.description = tag.description;
            }
        }
        for mut graph in other.graphs {
            for id in &mut graph.records {
                if let Some(new) = incoming_ids.get(id) {
                    *id = new.clone();
                }
            }
            if self.graphs.iter().any(|g| g.id == graph.id) {
                let base = graph.id.clone();
                let mut n = 2;
                while self.graphs.iter().any(|g| g.id == graph.id) {
                    graph.id = format!("{base}-{n}");
                    n += 1;
                }
            }
            self.graphs.push(graph);
        }
        self.consolidate();
    }
    pub fn edit_record(&mut self, id: Option<&str>, text: &str) -> Result<(), String> {
        let mut parsed = parse_wos(text, "Record editor")?;
        if parsed.len() != 1 {
            return Err("編輯器一次只能儲存一筆記錄".into());
        }
        let mut r = parsed.remove(0);
        if r.title.trim().is_empty() {
            return Err("請填入文獻標題".into());
        }
        if let Some(id) = id {
            let i = self
                .records
                .iter()
                .position(|r| r.id == id)
                .ok_or("此記錄已不存在")?;
            if r.fields.get("UT") == self.records[i].fields.get("UT") {
                r.id = id.to_string();
            }
            if r.id != id && self.records.iter().any(|x| x.id == r.id) {
                return Err("識別碼已存在".into());
            }
            for t in self.tags.values_mut() {
                if t.records.remove(id) {
                    t.records.insert(r.id.clone());
                }
            }
            for g in &mut self.graphs {
                for entry in &mut g.records {
                    if entry == id {
                        *entry = r.id.clone();
                    }
                }
            }
            for entry in &mut self.merge_log {
                if entry.kept_id == id {
                    entry.kept_id = r.id.clone();
                }
            }
            self.records[i] = r;
        } else {
            if self.records.iter().any(|x| x.id == r.id) {
                return Err("識別碼已存在".into());
            }
            self.records.push(r);
        }
        Ok(())
    }
    pub fn delete(&mut self, ids: &BTreeSet<String>) {
        self.records.retain(|r| !ids.contains(&r.id));
        for t in self.tags.values_mut() {
            t.records.retain(|id| !ids.contains(id));
        }
        for g in &mut self.graphs {
            g.records.retain(|id| !ids.contains(id));
        }
    }
    pub fn replace_items(
        &mut self,
        kind: &str,
        old: &BTreeSet<String>,
        replacement: &str,
    ) -> Result<usize, String> {
        if replacement.trim().is_empty() || replacement.contains(['\n', '\r']) {
            return Err("替換文字不可空白或包含換行".into());
        }
        let tag = match kind {
            "authors" => "AU",
            "references" => "CR",
            "institutions" | "subdivisions" => "C1",
            _ => return Err("此分析表不支援批次編修".into()),
        };
        let mut count = 0;
        for r in &mut self.records {
            let mut fields = r.fields.clone();
            let mut changed = false;
            let mut edit_tags = vec![tag];
            if tag == "C1"
                && (self.settings.reprint_address == "always"
                    || (self.settings.reprint_address == "missing" && !fields.contains_key("C1")))
            {
                edit_tags.push("RP");
            }
            for edit_tag in edit_tags {
                if let Some(values) = fields.get_mut(edit_tag) {
                    for value in values {
                        if kind == "authors" || kind == "references" {
                            let label = if kind == "authors" {
                                value.replace(", ", " ").replace(',', " ")
                            } else {
                                value.clone()
                            };
                            if old.iter().any(|s| {
                                (kind == "authors"
                                    && crate::normalize(s) == crate::normalize(value))
                                    || s.eq_ignore_ascii_case(&label)
                                    || s.eq_ignore_ascii_case(value)
                                    || (kind == "references"
                                        && analytics::reference_index_key(s)
                                            == analytics::reference_index_key(value))
                            }) {
                                *value = replacement.into();
                                changed = true;
                                count += 1;
                            }
                        } else {
                            let (updated, n) = replace_address(value, kind, old, replacement);
                            if n > 0 {
                                *value = updated;
                                count += n;
                                changed = true;
                            }
                        }
                    }
                }
            }
            if changed {
                if kind == "authors" {
                    fields.remove("AF");
                }
                let mut parsed = parse_wos(&fields_to_wos(&fields), &r.source_file)?;
                let id = r.id.clone();
                *r = parsed.remove(0);
                r.id = id;
            }
        }
        Ok(count)
    }
    pub fn hci(&self, selected: Option<&BTreeSet<String>>) -> String {
        let mut out =
            "FN Thomson Reuters Web of Knowledge\nVR 1.0\nMG HistCite Mac 0.3\n".to_string();
        for (key, value) in [
            ("MT", &self.title),
            ("MC", &self.description),
            ("MO", &self.created),
            ("MN", &self.comment),
        ] {
            write_field(
                &mut out,
                key,
                &value.lines().map(String::from).collect::<Vec<_>>(),
            );
        }
        let tags: Vec<_> = self
            .tags
            .iter()
            .map(|(name, t)| format!("{name}: {}", t.description.replace(['\n', '\r'], " ")))
            .collect();
        write_field(&mut out, "TD", &tags);
        out.push_str("ER\n\n");
        for r in &self.records {
            if selected.is_some_and(|ids| !ids.contains(&r.id)) {
                continue;
            }
            let mut fields = record_fields(r);
            let tags: Vec<_> = self
                .tags
                .iter()
                .filter(|(_, t)| t.records.contains(&r.id))
                .map(|(name, _)| name.clone())
                .collect();
            fields.remove("TG");
            if !tags.is_empty() {
                fields.insert("TG".into(), vec![tags.join(" ")]);
            }
            out.push_str(&fields_to_wos(&fields));
            out.push('\n');
        }
        out.push_str("EF\n");
        out
    }
}
fn replace_address(
    value: &str,
    kind: &str,
    old: &BTreeSet<String>,
    replacement: &str,
) -> (String, usize) {
    let mut segments = Vec::new();
    let mut bracket = 0usize;
    let mut start = 0;
    for (i, ch) in value.char_indices() {
        match ch {
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            ';' if bracket == 0 => {
                segments.push(&value[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(&value[start..]);
    let mut count = 0;
    let updated = segments
        .into_iter()
        .map(|segment| {
            let segment = segment.trim();
            let offset = if segment.starts_with('[') {
                segment.find(']').map(|i| i + 1).unwrap_or(0)
            } else {
                0
            };
            let body = segment[offset..].trim_start();
            let parts: Vec<_> = body.split(',').map(str::trim).collect();
            let n = if kind == "institutions" {
                1
            } else {
                parts.len().saturating_sub(2).clamp(1, 2)
            };
            let label = parts.iter().take(n).copied().collect::<Vec<_>>().join(", ");
            if old
                .iter()
                .any(|s| s.eq_ignore_ascii_case(label.trim_end_matches('.')))
            {
                count += 1;
                let tail = parts.iter().skip(n).copied().collect::<Vec<_>>().join(", ");
                format!(
                    "{}{}{}{}",
                    &segment[..offset],
                    if offset > 0 { " " } else { "" },
                    replacement,
                    if tail.is_empty() {
                        String::new()
                    } else {
                        format!(", {tail}")
                    }
                )
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    (updated, count)
}
pub fn record_fields(r: &Record) -> BTreeMap<String, Vec<String>> {
    let mut f = r.fields.clone();
    for (key, value) in [
        (
            "UT",
            r.fields
                .get("UT")
                .map(|v| v.join(" "))
                .unwrap_or_else(|| r.id.clone()),
        ),
        ("TI", r.title.clone()),
        ("SO", r.journal.clone()),
        ("DI", r.doi.clone()),
        ("PY", r.year.map(|v| v.to_string()).unwrap_or_default()),
        ("VL", r.volume.clone()),
        ("BP", r.page.clone()),
        ("EP", r.end_page.clone()),
        ("AR", r.article_number.clone()),
        ("TC", r.gcs.map(|v| v.to_string()).unwrap_or_default()),
        (
            "NR",
            r.declared_references
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        ("AB", r.abstract_text.clone()),
        ("PD", r.publication_date.clone()),
        ("IS", r.issue.clone()),
        ("DT", r.document_type.clone()),
        ("LA", r.language.clone()),
        ("DE", r.author_keywords.clone()),
        ("ID", r.keywords_plus.clone()),
    ] {
        if value.is_empty() {
            f.remove(key);
        } else {
            f.insert(key.into(), vec![value]);
        }
    }
    for (key, values) in [
        ("AU", r.authors.clone()),
        ("AF", r.full_authors.clone()),
        ("CR", r.references.clone()),
        ("C1", r.addresses.lines().map(String::from).collect()),
    ] {
        if values.is_empty() {
            f.remove(key);
        } else {
            f.insert(key.into(), values);
        }
    }
    f.entry("PT".into()).or_insert_with(|| vec!["J".into()]);
    f
}
fn write_field(out: &mut String, key: &str, values: &[String]) {
    for (i, v) in values.iter().enumerate() {
        out.push_str(if i == 0 { key } else { "  " });
        out.push(' ');
        out.push_str(&v.replace(['\n', '\r'], " "));
        out.push('\n');
    }
}
pub fn fields_to_wos(f: &BTreeMap<String, Vec<String>>) -> String {
    let mut out = String::new();
    write_field(
        &mut out,
        "PT",
        f.get("PT").map(Vec::as_slice).unwrap_or(&["J".into()]),
    );
    for (key, values) in f {
        if key != "PT" && key != "ER" && key != "EF" {
            write_field(&mut out, key, values);
        }
    }
    out.push_str("ER\n");
    out
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    crate::safety::atomic_write_protected(path, bytes, &[])
}
