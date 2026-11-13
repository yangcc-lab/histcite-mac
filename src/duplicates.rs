//! Import identity, separate from citation matching and cited-reference grouping.
//! Rules are backed by the original application's controlled HCI exports.
use crate::Record;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRecord {
    pub kept_id: String,
    pub key: Vec<String>,
    pub previous_gcs: Option<u64>,
    /// Complete incoming record, including fields not used by the application.
    pub incoming: Record,
}

pub struct Deduplicated {
    pub records: Vec<Record>,
    pub merges: Vec<MergeRecord>,
    /// One survivor ID for each input record, in input order.
    pub canonical_ids: Vec<String>,
}

pub fn key(r: &Record) -> Vec<String> {
    static AUTHOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([\p{L}\p{N}_ ]+)[ .]([A-Z]+)$").unwrap());
    let raw = r
        .authors
        .first()
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .replace([',', '-', '\''], "")
        .to_uppercase();
    let (surname, initial) = if let Some(c) = AUTHOR.captures(&raw) {
        (c[1].to_string(), c[2].chars().next().unwrap().to_string())
    } else {
        (raw, String::new())
    };
    let surname: String = surname
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '\''))
        .take(8)
        .collect();
    let article = r.article_number.trim().to_uppercase();
    let page = r.page.trim().to_uppercase();
    vec![
        surname,
        initial,
        r.year.map(|y| y.to_string()).unwrap_or_default(),
        r.volume.trim().to_uppercase(),
        r.issue.trim().to_uppercase(),
        if article.is_empty() {
            "BP".into()
        } else {
            "AR".into()
        },
        if article.is_empty() {
            page.trim_start_matches('0').to_string()
        } else {
            article
        },
    ]
}

pub fn deduplicate(input: Vec<Record>) -> Deduplicated {
    let mut records: Vec<Record> = Vec::new();
    let mut index = BTreeMap::new();
    // Reserve all supplied IDs so generated handles cannot steal a later UT.
    let mut reserved: BTreeSet<String> = input.iter().map(|r| r.id.clone()).collect();
    let mut used = BTreeSet::new();
    let mut merges = Vec::new();
    let mut canonical_ids = Vec::new();
    for mut r in input {
        let k = key(&r);
        if let Some(&i) = index.get(&k) {
            let kept: &mut Record = &mut records[i];
            merges.push(MergeRecord {
                kept_id: kept.id.clone(),
                key: k,
                previous_gcs: kept.gcs,
                incoming: r.clone(),
            });
            kept.gcs = kept.gcs.max(r.gcs);
            if let Some(gcs) = kept.gcs {
                kept.fields.insert("TC".into(), vec![gcs.to_string()]);
            }
            canonical_ids.push(kept.id.clone());
        } else {
            if !used.insert(r.id.clone()) {
                // Internal handle only: the source UT remains intact in fields.
                let mut suffix = 2;
                let base = r.id.clone();
                loop {
                    let candidate = format!("{base}#histcite-{suffix}");
                    if reserved.insert(candidate.clone()) {
                        r.id = candidate;
                        used.insert(r.id.clone());
                        break;
                    }
                    suffix += 1;
                }
            }
            canonical_ids.push(r.id.clone());
            index.insert(k, records.len());
            records.push(r);
        }
    }
    Deduplicated {
        records,
        merges,
        canonical_ids,
    }
}
