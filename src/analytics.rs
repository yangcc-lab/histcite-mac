//! Bibliometric views and analytical indexes. Definitions follow the bundled HistCite manual.
use crate::{Analysis, Record, normalize};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub beginning_years: u32,
    pub ending_years: u32,
    pub title_words: bool,
    pub stop_words: bool,
    pub min_word_length: usize,
    pub split_hyphens: bool,
    pub author_keywords: bool,
    pub keywords_plus: bool,
    pub split_keywords: bool,
    pub split_keyword_hyphens: bool,
    pub reprint_address: String,
    pub unify_uk: bool,
    pub unify_ussr: bool,
    pub unify_frg: bool,
    pub unify_gdr: bool,
    pub record_page_size: usize,
    pub index_page_size: usize,
    pub author_limit: usize,
    pub font_size: u32,
    pub view: String,
    pub columns: Vec<String>,
    pub brief: bool,
    pub percentages: bool,
    pub title_word_index: bool,
    pub reference_index: bool,
    pub lowercase_words: bool,
    pub show_h_index: bool,
    pub distinct_keywords: bool,
    pub wos_link_template: String,
    pub html_limits: bool,
    pub html_lcs_min: u64,
    pub html_gcs_min: u64,
    pub html_pages: usize,
    pub html_limit_alpha: bool,
    pub html_popup_min: usize,
    pub html_popup_max: usize,
    pub show_page_index: bool,
    pub extended_page_index: bool,
    pub record_header_interval: usize,
    pub index_header_interval: usize,
    pub tip_on_start: bool,
    pub save_graph_settings: bool,
    pub graph_settings: serde_json::Value,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            beginning_years: 3,
            ending_years: 3,
            title_words: true,
            stop_words: true,
            min_word_length: 2,
            split_hyphens: true,
            author_keywords: false,
            keywords_plus: false,
            split_keywords: true,
            split_keyword_hyphens: false,
            reprint_address: "missing".into(),
            unify_uk: true,
            unify_ussr: true,
            unify_frg: true,
            unify_gdr: true,
            record_page_size: 100,
            index_page_size: 200,
            author_limit: 5,
            font_size: 10,
            view: "standard".into(),
            columns: ["lcs", "gcs", "lcr", "cr"].map(String::from).to_vec(),
            brief: false,
            percentages: true,
            title_word_index: true,
            reference_index: true,
            lowercase_words: false,
            show_h_index: true,
            distinct_keywords: true,
            wos_link_template: "https://www.webofscience.com/".into(),
            html_limits: false,
            html_lcs_min: 0,
            html_gcs_min: 0,
            html_pages: 10,
            html_limit_alpha: false,
            html_popup_min: 0,
            html_popup_max: 1000,
            show_page_index: false,
            extended_page_index: false,
            record_header_interval: 10,
            index_header_interval: 30,
            tip_on_start: false,
            save_graph_settings: true,
            graph_settings: serde_json::json!({}),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=100).contains(&self.beginning_years)
            || !(1..=100).contains(&self.ending_years)
            || !(1..=2000).contains(&self.record_page_size)
            || !(1..=2000).contains(&self.index_page_size)
            || !(8..=20).contains(&self.font_size)
            || self.author_limit > 1000
            || self.min_word_length > 100
            || self.record_header_interval > 2000
            || self.index_header_interval > 2000
            || self.html_pages == 0
            || self.html_pages > 10000
            || self.html_popup_max == 0
            || self.html_popup_max > 1000000
        {
            return Err("設定超出有效範圍".into());
        }
        if !["standard", "bibliometric", "custom"].contains(&self.view.as_str())
            || !["never", "missing", "always"].contains(&self.reprint_address.as_str())
        {
            return Err("無效的顯示或地址設定".into());
        }
        if !(self.wos_link_template.starts_with("https://")
            || self.wos_link_template.starts_with("http://"))
            || self.wos_link_template.contains(['\n', '\r'])
        {
            return Err("WoS 連結必須是 HTTP(S) 網址".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metrics {
    pub lcsx: u64,
    pub lcst: f64,
    pub gcst: Option<f64>,
    pub ocs: Option<u64>,
    pub na: usize,
    pub lcsb: i64,
    pub lcse: i64,
    pub ratio: Option<String>,
    pub citations_by_year: BTreeMap<u32, u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexEntry {
    pub name: String,
    pub occurrences: usize,
    pub variants: BTreeSet<String>,
    #[serde(skip)]
    seen: BTreeSet<(usize, String)>,
    #[serde(skip)]
    weights: BTreeMap<usize, u64>,
    pub indices: BTreeSet<usize>,
    pub local_targets: BTreeSet<usize>,
    pub lcs: u64,
    pub gcs: u64,
    pub lcsx: u64,
    pub lcr: u64,
    pub lcst: f64,
    pub gcst: f64,
    pub lcsb: i64,
    pub lcse: i64,
    pub title_word: bool,
    pub keyword: bool,
}
fn add(map: &mut BTreeMap<String, IndexEntry>, name: &str, i: usize) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    let g = map
        .entry(name.to_uppercase())
        .or_insert_with(|| IndexEntry {
            name: name.into(),
            ..Default::default()
        });
    if g.seen.insert((i, name.to_string())) {
        g.occurrences += 1;
        *g.weights.entry(i).or_default() += 1;
    }
    g.indices.insert(i);
}
static BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[[^\]]*\]\s*").unwrap());
static WORDS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]+(?:[-/][\p{L}\p{N}]+)*").unwrap());
static SIMPLE_WORDS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]+").unwrap());
pub fn addresses(r: &Record, s: &Settings) -> Vec<String> {
    let mut list = r
        .fields
        .get("C1")
        .cloned()
        .unwrap_or_else(|| r.addresses.lines().map(String::from).collect());
    if s.reprint_address == "always" || (s.reprint_address == "missing" && list.is_empty()) {
        if let Some(lines) = r.fields.get("RP") {
            for line in lines {
                let cleaned = line.clone();
                if !cleaned.is_empty() {
                    list.push(cleaned);
                }
            }
        }
    }
    list.into_iter()
        .flat_map(|line| {
            // Modern C1 uses bracketed author lists containing semicolons. Remove those first.
            let clean = BRACKETS.replace_all(&line, "");
            clean
                .split(';')
                .map(|s| s.trim().trim_end_matches('.').to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}
fn country(address: &str, s: &Settings) -> String {
    let part = address
        .rsplit(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('.');
    let up = part.to_uppercase();
    let state_zip=Regex::new(r"^(AL|AK|AZ|AR|CA|CO|CT|DE|DC|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY)\s+\d{5}(-\d{4})?$").unwrap();
    let mut c = if up.ends_with("USA") || state_zip.is_match(&up) {
        "USA".to_string()
    } else {
        part.to_string()
    };
    if s.unify_uk
        && [
            "ENGLAND",
            "SCOTLAND",
            "WALES",
            "NORTH IRELAND",
            "NORTHERN IRELAND",
            "UK",
        ]
        .contains(&up.as_str())
    {
        c = "UK".into();
    }
    if s.unify_ussr
        && [
            "ARSSR", "AZSSR", "BESSR", "ESSSR", "GESSR", "KASSR", "KISSR", "LASSR", "LISSR",
            "MOSSR", "TASSR", "TUSSR", "UKSSR", "UZSSR", "USSR",
        ]
        .contains(&up.as_str())
    {
        c = "USSR".into();
    }
    if s.unify_frg
        && [
            "BRD",
            "BUNDES REPUBLIK",
            "FED REP GER",
            "W GER",
            "WEST GERMANY",
            "FRG",
        ]
        .contains(&up.as_str())
    {
        c = "FRG".into();
    }
    if s.unify_gdr
        && [
            "DDR",
            "DEUTSCH DEM REP",
            "E GER",
            "EAST GERMANY",
            "GER DEM REP",
            "GDR",
        ]
        .contains(&up.as_str())
    {
        c = "GDR".into();
    }
    // A lone institution or city is not a country. Legacy data often lack a final country.
    if !address.contains(',') || c.is_empty() || c.chars().any(|c| c.is_ascii_digit()) {
        return "Unknown".into();
    }
    c
}
fn tokens(text: &str, split: bool) -> Vec<String> {
    (if split { &*SIMPLE_WORDS } else { &*WORDS })
        .find_iter(text)
        .filter(|m| m.as_str().chars().any(|c| c.is_ascii_alphabetic()))
        .map(|m| {
            m.as_str()
                .trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_uppercase()
        })
        .collect()
}
/// Legacy Cited References index identity, verified with controlled pairs and 54,294 groups.
/// This is intentionally distinct from the key used for local citation edges.
pub fn reference_index_key(raw: &str) -> String {
    static AUTHOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\w+)\W+(\w)").unwrap());
    let parts: Vec<_> = raw
        .trim()
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();
    if parts.first().is_none_or(|s| s.is_empty()) {
        return "INVALID".into();
    }
    let author = if let Some(m) = AUTHOR.captures(&parts[0]) {
        format!("{}{}", m[1].chars().take(8).collect::<String>(), &m[2])
    } else {
        parts[0].clone()
    };
    let has_year = parts
        .get(1)
        .is_some_and(|s| s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()));
    let year = if has_year { parts[1].as_str() } else { "" };
    let tail = &parts[if has_year { 2 } else { 1 }..];
    let mut signature = vec![
        author,
        year.to_string(),
        if tail.len() > 1 { "EXTRA" } else { "SOURCE" }.into(),
    ];
    if tail.len() > 1 {
        signature.push(tail[1].clone());
        if tail.len() > 2 && !tail[2].starts_with("DOI") {
            signature.push(tail[2].clone());
        }
    } else {
        signature.extend(tail.iter().cloned());
    }
    signature.join("|")
}
pub fn refresh(a: &mut Analysis, s: &Settings) {
    let end = a.records.iter().filter_map(|r| r.year).max().unwrap_or(0);
    let authors: Vec<BTreeSet<_>> = a
        .records
        .iter()
        .map(|r| r.authors.iter().map(|a| normalize(a)).collect())
        .collect();
    let mut metrics: Vec<Metrics> = a
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let age = end.saturating_sub(r.year.unwrap_or(end)) + 1;
            Metrics {
                lcst: a.lcs[i] as f64 / age as f64,
                gcst: r.gcs.map(|g| g as f64 / age as f64),
                ocs: r
                    .fields
                    .get("TS")
                    .and_then(|s| s.first())
                    .and_then(|v| v.parse().ok()),
                na: r.authors.len(),
                ..Default::default()
            }
        })
        .collect();
    for e in &a.edges {
        if authors[e.citing].is_disjoint(&authors[e.cited]) {
            metrics[e.cited].lcsx += 1;
        }
        if let Some(y) = a.records[e.citing].year {
            *metrics[e.cited].citations_by_year.entry(y).or_default() += 1;
            if let Some(start) = a.records[e.cited].year {
                if y < start.saturating_add(s.beginning_years) {
                    metrics[e.cited].lcsb += 1;
                }
            }
            if y > end.saturating_sub(s.ending_years) {
                metrics[e.cited].lcse += 1;
            }
        }
    }
    for (i, m) in metrics.iter_mut().enumerate() {
        if a.records[i]
            .year
            .is_some_and(|y| end.saturating_sub(y) + 1 >= s.beginning_years + s.ending_years)
        {
            m.ratio = Some(if m.lcsb == 0 && m.lcse == 0 {
                "0".into()
            } else if m.lcsb == 0 || m.lcse == 0 {
                format!("{}/{}", m.lcse, m.lcsb)
            } else {
                format!("{:.2}", m.lcse as f64 / m.lcsb as f64)
            });
        } else {
            m.lcsb = -1;
            m.lcse = -1;
        }
    }
    a.metrics = metrics;
    let names = [
        "authors",
        "journals",
        "references",
        "words",
        "years",
        "types",
        "languages",
        "institutions",
        "subdivisions",
        "countries",
    ];
    let mut all: BTreeMap<String, BTreeMap<String, IndexEntry>> = names
        .iter()
        .map(|k| (k.to_string(), BTreeMap::new()))
        .collect();
    let stops: BTreeSet<_> = include_str!("data/stopwords.txt").lines().collect();
    let local_refs: BTreeMap<_, _> =
        a.edges
            .iter()
            .fold(BTreeMap::<String, BTreeSet<usize>>::new(), |mut map, e| {
                map.entry(reference_index_key(&e.reference))
                    .or_default()
                    .insert(e.cited);
                map
            });
    let mut author_names: BTreeMap<String, String> = BTreeMap::new();
    for (i, r) in a.records.iter().enumerate() {
        for name in &r.authors {
            let label = author_names
                .entry(normalize(name))
                .or_insert_with(|| name.replace(", ", " ").replace(',', " "));
            let map = all.get_mut("authors").unwrap();
            let label = label.clone();
            if map
                .get(&label.to_uppercase())
                .is_some_and(|g| g.seen.contains(&(i, label.clone())))
            {
                let g = map.get_mut(&label.to_uppercase()).unwrap();
                g.occurrences += 1;
                *g.weights.entry(i).or_default() += 1;
            } else {
                add(map, &label, i);
            }
        }
        for (kind, name) in [
            ("journals", r.journal.clone()),
            (
                "years",
                r.year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "Unknown".into()),
            ),
            ("types", r.document_type.clone()),
            ("languages", r.language.clone()),
        ] {
            add(
                all.get_mut(kind).unwrap(),
                if name.is_empty() { "Unknown" } else { &name },
                i,
            );
        }
        if s.reference_index {
            for raw in &r.references {
                let key = reference_index_key(raw);
                let map = all.get_mut("references").unwrap();
                let g = map.entry(key.clone()).or_insert_with(|| IndexEntry {
                    name: raw.trim().to_string(),
                    local_targets: local_refs.get(&key).cloned().unwrap_or_default(),
                    ..Default::default()
                });
                g.indices.insert(i);
                g.variants.insert(raw.clone());
                g.occurrences += 1;
            }
        }
        let ads = addresses(r, s);
        if ads.is_empty() {
            for kind in ["institutions", "subdivisions", "countries"] {
                add(all.get_mut(kind).unwrap(), "Unknown", i);
            }
        }
        let mut countries = BTreeSet::new();
        for ad in ads {
            let parts: Vec<_> = ad.split(',').map(str::trim).collect();
            add(all.get_mut("institutions").unwrap(), parts[0], i);
            add(
                all.get_mut("subdivisions").unwrap(),
                &parts
                    .iter()
                    .take(parts.len().saturating_sub(2).clamp(1, 2))
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", "),
                i,
            );
            countries.insert(if !r.addresses.contains('[') && parts.len() < 4 {
                "Unknown".to_string()
            } else {
                country(&ad, s)
            });
        }
        if countries.len() > 1 {
            countries.remove("Unknown");
        }
        for name in countries {
            add(all.get_mut("countries").unwrap(), &name, i);
        }
        if s.title_word_index {
            let mut add_word = |word: String, title: bool| {
                if word.chars().count() <= s.min_word_length
                    || (s.stop_words && stops.contains(word.as_str()))
                {
                    return;
                }
                let map = all.get_mut("words").unwrap();
                add(map, &word, i);
                let g = map.get_mut(&word).unwrap();
                g.title_word |= title;
                g.keyword |= !title;
            };
            if s.title_words {
                for word in tokens(&r.title, s.split_hyphens) {
                    add_word(word, true);
                }
            }
            for (enabled, text) in [
                (s.author_keywords, &r.author_keywords),
                (s.keywords_plus, &r.keywords_plus),
            ] {
                if enabled {
                    for term in text.split(';').map(str::trim).filter(|s| !s.is_empty()) {
                        if s.split_keywords {
                            for word in tokens(term, s.split_keyword_hyphens) {
                                add_word(word, false);
                            }
                        } else {
                            add_word(term.to_uppercase(), false);
                        }
                    }
                }
            }
        }
    }
    a.indexes = all
        .into_iter()
        .map(|(kind, map)| {
            let groups = map
                .into_values()
                .map(|mut g| {
                    for &i in &g.indices {
                        let w = g.weights.get(&i).copied().unwrap_or(1);
                        g.lcs += a.lcs[i] * w;
                        g.gcs += a.records[i].gcs.unwrap_or(0) * w;
                        g.lcsx += a.metrics[i].lcsx * w;
                        g.lcr += a.lcr[i] * w;
                        g.lcst += a.metrics[i].lcst * w as f64;
                        g.gcst += a.metrics[i].gcst.unwrap_or(0.0) * w as f64;
                        g.lcsb += a.metrics[i].lcsb.max(0) * w as i64;
                        g.lcse += a.metrics[i].lcse * w as i64;
                    }
                    if g.indices.iter().all(|&i| a.metrics[i].lcsb < 0) {
                        g.lcsb = -1;
                    }
                    g
                })
                .collect();
            (kind, groups)
        })
        .collect();
}
pub fn h_index(values: impl IntoIterator<Item = u64>) -> usize {
    let mut values: Vec<_> = values.into_iter().collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    values
        .iter()
        .enumerate()
        .take_while(|(i, n)| **n >= (*i + 1) as u64)
        .count()
}
