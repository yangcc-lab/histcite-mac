use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::LazyLock;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
pub mod analytics;
pub mod collection;
pub mod duplicates;
pub mod safety;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    #[serde(default)]
    pub fields: BTreeMap<String, Vec<String>>,
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub journal: String,
    pub journal_names: Vec<String>,
    pub year: Option<u32>,
    pub volume: String,
    pub page: String,
    pub article_number: String,
    pub doi: String,
    pub gcs: Option<u64>,
    pub declared_references: Option<usize>,
    pub references: Vec<String>,
    pub abstract_text: String,
    pub source_file: String,
    pub full_authors: Vec<String>,
    pub publication_date: String,
    pub issue: String,
    pub end_page: String,
    pub document_type: String,
    pub language: String,
    pub addresses: String,
    pub author_keywords: String,
    pub keywords_plus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub citing: usize,
    pub cited: usize,
    pub method: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ambiguity {
    pub citing: usize,
    pub reference: String,
    pub candidates: Vec<usize>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchingMode {
    #[default]
    Histcite,
    Doi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    #[serde(default)]
    pub metrics: Vec<analytics::Metrics>,
    #[serde(default)]
    pub indexes: BTreeMap<String, Vec<analytics::IndexEntry>>,
    pub matching_mode: MatchingMode,
    pub records: Vec<Record>,
    pub edges: Vec<Edge>,
    pub lcs: Vec<u64>,
    pub lcr: Vec<u64>,
    pub ambiguities: Vec<Ambiguity>,
    pub warnings: Vec<String>,
    pub references_complete: bool,
    pub reference_count: usize,
    pub unmatched_reference_count: usize,
    pub duplicate_records: usize,
    #[serde(default)]
    pub merge_log: Vec<duplicates::MergeRecord>,
    pub year_counts: BTreeMap<u32, usize>,
    pub journal_counts: BTreeMap<String, usize>,
}

type Fields = BTreeMap<String, Vec<String>>;
fn field(f: &Fields, key: &str) -> String {
    f.get(key).map(|v| v.join(" ")).unwrap_or_default()
}
pub fn normalize(s: &str) -> String {
    s.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_uppercase)
        .filter(|c| c.is_alphanumeric())
        .collect()
}
fn doi_normalize(s: &str) -> String {
    s.trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://dx.doi.org/")
        .trim_end_matches(['.', ';', ']'])
        .to_lowercase()
}
fn numeric<T: std::str::FromStr>(
    f: &Fields,
    key: &str,
    source: &str,
    line: usize,
) -> Result<Option<T>, String> {
    let s = field(f, key);
    if s.is_empty() {
        return Ok(None);
    }
    s.parse()
        .map(Some)
        .map_err(|_| format!("{source}:{line}：{key} 不是有效數字：{s}"))
}
fn make_record(f: &Fields, source: &str, line: usize) -> Result<Record, String> {
    let title = field(f, "TI");
    let doi = doi_normalize(&field(f, "DI"));
    let authors = f.get("AU").cloned().unwrap_or_default();
    let year = numeric(f, "PY", source, line)?;
    let mut id = field(f, "UT");
    if id.is_empty() {
        id = if !doi.is_empty() {
            format!("doi:{doi}")
        } else {
            format!(
                "bib:{}:{}:{}",
                normalize(&title),
                year.unwrap_or(0),
                normalize(authors.first().map(String::as_str).unwrap_or(""))
            )
        };
        if title.is_empty() && doi.is_empty() {
            return Err(format!(
                "{source}:{line}：缺少 UT、DOI 與標題，無法識別文獻"
            ));
        }
    }
    Ok(Record {
        fields: f.clone(),
        id,
        title,
        authors,
        journal: field(f, "SO"),
        journal_names: ["J9", "JI", "SO"]
            .iter()
            .map(|k| field(f, k))
            .filter(|s| !s.is_empty())
            .collect(),
        year,
        volume: field(f, "VL"),
        page: field(f, "BP"),
        article_number: field(f, "AR"),
        doi,
        gcs: numeric(f, "TC", source, line)?,
        declared_references: numeric(f, "NR", source, line)?,
        references: f.get("CR").cloned().unwrap_or_default(),
        abstract_text: field(f, "AB"),
        source_file: source.to_string(),
        full_authors: f.get("AF").cloned().unwrap_or_default(),
        publication_date: field(f, "PD"),
        issue: field(f, "IS"),
        end_page: field(f, "EP"),
        document_type: field(f, "DT"),
        language: field(f, "LA"),
        addresses: f.get("C1").map(|v| v.join("\n")).unwrap_or_default(),
        author_keywords: field(f, "DE"),
        keywords_plus: field(f, "ID"),
    })
}

pub fn parse_wos(text: &str, source: &str) -> Result<Vec<Record>, String> {
    safety::check_text_size(text)?;
    let histcite = text.lines().any(|l| l.starts_with("MG HistCite"));
    let mut records = Vec::new();
    let mut current = Fields::new();
    let mut key = String::new();
    for (idx, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.starts_with("PT ") {
            if !current.is_empty() {
                return Err(format!("{source}:{}：上一筆記錄缺少 ER 結束標記", idx + 1));
            }
            key = "PT".into();
            current.insert(key.clone(), vec![line[3..].into()]);
        } else if line == "ER" {
            if current.is_empty() {
                if histcite {
                    continue;
                }
                return Err(format!("{source}:{}：ER 前沒有記錄", idx + 1));
            }
            if !(histcite && current.values().flatten().all(|s| s.trim().is_empty())) {
                if records.len() >= safety::MAX_RECORDS {
                    return Err("集合超過 100,000 篇文獻；請縮小集合".into());
                }
                records.push(make_record(&current, source, idx + 1)?);
            }
            current.clear();
            key.clear();
        } else if !current.is_empty() && line.starts_with("   ") {
            if key.is_empty() {
                return Err(format!("{source}:{}：沒有欄位名稱的延續行", idx + 1));
            }
            current
                .entry(key.clone())
                .or_default()
                .push(line[3..].to_string());
        } else if !current.is_empty()
            && line.as_bytes().get(2) == Some(&b' ')
            && line.as_bytes()[..2].iter().all(u8::is_ascii_alphanumeric)
        {
            key = line[..2].into();
            current
                .entry(key.clone())
                .or_default()
                .push(line[3..].into());
        } else if !current.is_empty() && !line.is_empty() {
            return Err(format!(
                "{source}:{}：無法辨識記錄內容：{}",
                idx + 1,
                line.chars().take(60).collect::<String>()
            ));
        }
    }
    if !current.is_empty() {
        return Err(format!("{source}：最後一筆記錄缺少 ER，檔案可能不完整"));
    }
    if records.is_empty() {
        return Err(format!(
            "{source}：找不到 WoS 標記式純文字記錄，請匯出「全記錄與引用的參考文獻」TXT"
        ));
    }
    safety::validate_records(&records)?;
    Ok(records)
}

#[derive(Debug)]
struct Reference {
    author: String,
    year: u32,
    journal: String,
    volume: String,
    page: String,
    dois: Vec<String>,
}
static DOI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b10\.\d{4,9}/[^\s,;\]]+").unwrap());
fn parse_reference(raw: &str) -> Reference {
    let parts: Vec<&str> = raw.split(',').map(str::trim).collect();
    let year = parts
        .get(1)
        .and_then(|s| s.get(..4))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut volume = String::new();
    let mut page = String::new();
    for p in parts.iter().skip(3) {
        let upper = p.to_ascii_uppercase();
        if upper.starts_with('V') && !upper.starts_with("VOL") {
            volume = normalize(&p[1..]);
        } else if upper.starts_with('P') && !upper.starts_with("PMID") {
            page = normalize(&p[1..]);
        }
    }
    Reference {
        author: normalize(parts.first().copied().unwrap_or("")),
        year,
        journal: normalize(parts.get(2).copied().unwrap_or("")),
        volume,
        page,
        dois: DOI
            .find_iter(raw)
            .map(|m| doi_normalize(m.as_str()))
            .collect(),
    }
}
fn ref_key(author: &str, year: u32, journal: &str, volume: &str, page: &str) -> String {
    format!("{author}|{year}|{journal}|{volume}|{page}")
}

// These rules reproduce the observed HistCite 12.03.17 behavior. They are
// deliberately separate from DOI matching: the original ignores DOI/journal,
// uses eight surname characters plus one initial, and prioritizes AR over BP.
fn legacy_name(surname: &str, initial: char) -> String {
    let name: String = surname
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_uppercase)
        .take(8)
        .collect();
    format!("{name}:{}", initial.to_uppercase())
}
fn legacy_record_key(r: &Record) -> Option<String> {
    if !r.article_number.is_empty() || r.page.is_empty() {
        return None;
    }
    let (surname, initials) = r.authors.first()?.split_once(',')?;
    let initial = initials.trim().chars().next()?;
    Some(ref_key(
        &legacy_name(surname, initial),
        r.year?,
        "",
        &normalize(&r.volume),
        &normalize(&r.page),
    ))
}
fn legacy_reference_key(raw: &str, r: &Reference) -> Option<String> {
    let author = raw.split(',').next()?;
    let mut parts = author.split_whitespace();
    let surname = parts.next()?;
    if !surname.chars().all(char::is_alphanumeric) || r.year == 0 || r.page.is_empty() {
        return None;
    }
    let initial = parts.next()?.chars().next()?;
    Some(ref_key(
        &legacy_name(surname, initial),
        r.year,
        "",
        &r.volume,
        &r.page,
    ))
}

pub fn analyze(records: Vec<Record>) -> Analysis {
    analyze_with_mode(records, MatchingMode::Histcite)
}
pub fn analyze_with_mode(records: Vec<Record>, mode: MatchingMode) -> Analysis {
    let dedup = duplicates::deduplicate(records);
    analyze_retained(dedup.records, mode, dedup.merges)
}
pub(crate) fn analyze_retained(
    records: Vec<Record>,
    mode: MatchingMode,
    merge_log: Vec<duplicates::MergeRecord>,
) -> Analysis {
    let duplicate_records = merge_log.len();
    let mut doi_index: HashMap<String, BTreeSet<usize>> = HashMap::new();
    let mut signature_index: HashMap<String, BTreeSet<usize>> = HashMap::new();
    let mut legacy_index: HashMap<String, BTreeSet<usize>> = HashMap::new();
    for (i, r) in records.iter().enumerate() {
        if let Some(key) = legacy_record_key(r) {
            legacy_index.entry(key).or_default().insert(i);
        }
        if !r.doi.is_empty() {
            doi_index.entry(r.doi.clone()).or_default().insert(i);
        }
        if let (Some(author), Some(year)) = (r.authors.first(), r.year) {
            for journal in &r.journal_names {
                let key = ref_key(
                    &normalize(author),
                    year,
                    &normalize(journal),
                    &normalize(&r.volume),
                    &normalize(&r.page),
                );
                signature_index.entry(key).or_default().insert(i);
            }
        }
    }
    let mut edges = Vec::new();
    let mut ambiguities = Vec::new();
    let mut lcs = vec![0; records.len()];
    let mut lcr = lcs.clone();
    let mut unmatched = 0;
    for (i, r) in records.iter().enumerate() {
        let mut cited = BTreeSet::new();
        for raw in &r.references {
            let reference = parse_reference(raw);
            let mut candidates = BTreeSet::new();
            let mut method = "doi";
            if mode == MatchingMode::Histcite {
                method = "histcite_key";
                if let Some(key) = legacy_reference_key(raw, &reference) {
                    if let Some(found) = legacy_index.get(&key) {
                        candidates.extend(found);
                    }
                }
            } else {
                for doi in &reference.dois {
                    if let Some(found) = doi_index.get(doi) {
                        candidates.extend(found);
                    }
                }
                let unique_dois: BTreeSet<_> = reference.dois.iter().collect();
                if unique_dois.len() > 1 && !candidates.is_empty() {
                    ambiguities.push(Ambiguity {
                        citing: i,
                        reference: raw.clone(),
                        candidates: candidates.into_iter().collect(),
                        reason: "參考文獻包含多個不同 DOI，需人工確認".into(),
                    });
                    continue;
                }
                if candidates.is_empty() {
                    method = "bibliographic";
                    let key = ref_key(
                        &reference.author,
                        reference.year,
                        &reference.journal,
                        &reference.volume,
                        &reference.page,
                    );
                    if reference.year > 0 {
                        if let Some(found) = signature_index.get(&key) {
                            candidates.extend(found);
                        }
                    }
                }
            }
            if candidates.len() == 1 {
                let target = *candidates.iter().next().unwrap();
                if cited.insert(target) {
                    edges.push(Edge {
                        citing: i,
                        cited: target,
                        method: method.into(),
                        reference: raw.clone(),
                    });
                    lcs[target] += 1;
                    lcr[i] += 1;
                }
            } else if candidates.len() > 1 {
                ambiguities.push(Ambiguity {
                    citing: i,
                    reference: raw.clone(),
                    candidates: candidates.into_iter().collect(),
                    reason: "有多個符合的集合內文獻".into(),
                });
            } else {
                unmatched += 1;
            }
        }
    }
    let mut warnings = Vec::new();
    let missing = records
        .iter()
        .filter(|r| r.references.is_empty() && r.declared_references.unwrap_or(1) > 0)
        .count();
    let mismatches = records
        .iter()
        .filter(|r| {
            r.declared_references
                .is_some_and(|n| n != r.references.len())
        })
        .count();
    if missing > 0 {
        warnings.push(format!(
            "{missing} 篇缺少參考文獻清單；目前 LCS 與引用圖不完整。"
        ));
    }
    if mismatches > 0 {
        warnings.push(format!(
            "{mismatches} 篇的 NR 數量與實際 CR 清單長度不同；以清單建立引用關係，保留原始 NR。"
        ));
    }
    if duplicate_records > 0 {
        warnings.push(format!(
            "依 HistCite 書目規則合併 {duplicate_records} 筆記錄，保留首筆內容與較高 GCS；原始版本保存在合併紀錄。"
        ));
    }
    if !ambiguities.is_empty() {
        warnings.push(format!(
            "{} 條參考文獻需要人工確認，尚未建立連線。",
            ambiguities.len()
        ));
    }
    let mut year_counts = BTreeMap::new();
    let mut journal_counts = BTreeMap::new();
    for r in &records {
        if let Some(y) = r.year {
            *year_counts.entry(y).or_insert(0) += 1;
        }
        *journal_counts.entry(r.journal.clone()).or_insert(0) += 1;
    }
    let reference_count = records.iter().map(|r| r.references.len()).sum();
    let mut analysis = Analysis {
        metrics: Vec::new(),
        indexes: BTreeMap::new(),
        matching_mode: mode,
        records,
        edges,
        lcs,
        lcr,
        ambiguities,
        warnings,
        references_complete: missing == 0,
        reference_count,
        unmatched_reference_count: unmatched,
        duplicate_records,
        merge_log,
        year_counts,
        journal_counts,
    };
    analytics::refresh(&mut analysis, &analytics::Settings::default());
    analysis
}

pub fn analyze_files(paths: &[String]) -> Result<Analysis, String> {
    analyze_files_with_mode(paths, MatchingMode::Histcite)
}
pub fn analyze_files_with_mode(paths: &[String], mode: MatchingMode) -> Result<Analysis, String> {
    if paths.is_empty() {
        return Err("請選取至少一個 WoS TXT 檔案".into());
    }
    let mut records = Vec::new();
    for path in paths {
        let text = safety::read_text(std::path::Path::new(path))
            .map_err(|e| format!("無法讀取 {path}：{e}"))?;
        records.extend(parse_wos(&text, path)?);
    }
    safety::validate_records(&records)?;
    Ok(analyze_with_mode(records, mode))
}
