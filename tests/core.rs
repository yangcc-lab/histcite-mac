use histcite_core::{MatchingMode, analyze, analyze_with_mode, parse_wos};
use std::collections::BTreeSet;

fn record(id: &str, author: &str, year: u32, page: &str, doi: &str, refs: &[&str]) -> String {
    let mut text = format!(
        "PT J\nAU {author}\nTI Synthetic paper {id}\nSO TEST JOURNAL\nJ9 TEST JOURNAL\nPY {year}\nVL 1\nBP {page}\nDI {doi}\nNR {}\nTC 8\nUT {id}\n",
        refs.len()
    );
    for (i, r) in refs.iter().enumerate() {
        text.push_str(&format!("{}{r}\n", if i == 0 { "CR " } else { "   " }));
    }
    text.push_str("ER\n");
    text
}

#[test]
fn original_behavior_probe_snapshot() {
    let analysis =
        analyze(parse_wos(include_str!("fixtures/legacy-rule-probes.txt"), "probe").unwrap());
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/original/citation-rules.json"
    ))
    .unwrap();
    let actual: BTreeSet<_> = analysis
        .edges
        .iter()
        .map(|e| {
            (
                &analysis.records[e.citing].id,
                &analysis.records[e.cited].id,
            )
        })
        .collect();
    let expected: BTreeSet<_> = original["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e[0].as_str().unwrap().to_string(),
                e[1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        actual
            .into_iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect::<BTreeSet<_>>(),
        expected
    );
    for item in original["records"].as_array().unwrap() {
        let i = analysis
            .records
            .iter()
            .position(|r| r.id == item["id"].as_str().unwrap())
            .unwrap();
        assert_eq!(analysis.lcs[i], item["lcs"].as_u64().unwrap());
        assert_eq!(analysis.lcr[i], item["lcr"].as_u64().unwrap());
    }
}

#[test]
fn bom_crlf_and_continuation_lines_preserve_authors_and_title() {
    let text = "\u{feff}FN Clarivate Analytics Web of Science\r\nPT J\r\nAU Chen, A\r\n   Wu, B\r\nTI A long\r\n   title\r\nPY 2020\r\nUT x\r\nNR 0\r\nER\r\nEF\r\n";
    let records = parse_wos(text, "x").unwrap();
    assert_eq!(records[0].title, "A long title");
    assert_eq!(records[0].authors, vec!["Chen, A", "Wu, B"]);
}

#[test]
fn truncated_records_fail_loudly() {
    assert!(
        parse_wos("PT J\nTI Lost record\nUT x\n", "bad")
            .unwrap_err()
            .contains("缺少 ER")
    );
    assert!(parse_wos("PT J\nUT x\nPT J\nUT y\nER\n", "bad").is_err());
}

#[test]
fn invalid_numeric_fields_are_not_silently_zero() {
    assert!(parse_wos("PT J\nUT x\nTC unavailable\nER\n", "bad").is_err());
}

#[test]
fn duplicate_files_and_repeated_references_do_not_inflate_scores() {
    let target = record("b", "Smith, AB", 2020, "101", "", &[]);
    let citing = record(
        "a",
        "Jones, C",
        2021,
        "102",
        "",
        &[
            "Smith AB, 2020, TEST JOURNAL, V1, P101",
            "Smith A, 2020, TEST JOURNAL, V1, P101",
        ],
    );
    let text = format!("{target}{citing}{target}{citing}");
    let a = analyze(parse_wos(&text, "duplicates").unwrap());
    assert_eq!(a.records.len(), 2);
    assert_eq!(a.duplicate_records, 2);
    assert_eq!(a.edges.len(), 1);
    assert_eq!(a.lcs, vec![1, 0]);
    assert_eq!(a.lcr, vec![0, 1]);
}

#[test]
fn lowercase_page_tag_and_supplement_page_match() {
    let text = record("b", "Weeks, M", 2016, "56S", "", &[])
        + &record(
            "a",
            "Jones, A",
            2020,
            "2",
            "",
            &["Weeks M, 2016, TEST JOURNAL, v1, p56S"],
        );
    let a = analyze(parse_wos(&text, "supplement").unwrap());
    assert_eq!(a.edges.len(), 1);
}

#[test]
fn missing_reference_lists_mark_local_scores_incomplete() {
    let text = "PT J\nUT x\nNR 12\nTC 0\nER\n";
    let a = analyze(parse_wos(text, "missing").unwrap());
    assert!(!a.references_complete);
    assert!(!a.warnings.is_empty());
    assert_eq!(a.records[0].gcs, Some(0));
}

#[test]
fn missing_gcs_is_not_zero() {
    let a = parse_wos("PT J\nUT x\nNR 0\nER\n", "x").unwrap();
    assert_eq!(a[0].gcs, None);
}

#[test]
fn doi_mode_is_distinct_from_legacy_mode() {
    let text = record("b", "Smith, AB", 2020, "101", "10.99999/test", &[])
        + &record(
            "a",
            "Jones, A",
            2021,
            "102",
            "",
            &["Smith AB, 2019, TEST JOURNAL, V1, P101, DOI 10.99999/TEST"],
        );
    let records = parse_wos(&text, "doi").unwrap();
    assert!(analyze(records.clone()).edges.is_empty());
    let a = analyze_with_mode(records, MatchingMode::Doi);
    assert_eq!(a.edges.len(), 1);
    assert_eq!(a.edges[0].method, "doi");
}

#[test]
fn distinct_dois_require_review_even_if_only_one_is_in_collection() {
    let text = record("b", "Smith, AB", 2020, "101", "10.99999/test", &[])
        + &record(
            "a",
            "Jones, A",
            2021,
            "102",
            "",
            &["Smith AB, 2020, TEST JOURNAL, V1, P101, DOI [10.99999/test, 10.99999/other]"],
        );
    let a = analyze_with_mode(parse_wos(&text, "ambiguous").unwrap(), MatchingMode::Doi);
    assert!(a.edges.is_empty());
    assert_eq!(a.ambiguities.len(), 1);
}

#[test]
fn duplicate_doi_candidates_are_not_arbitrarily_selected() {
    let text = record("b", "Smith, AB", 2020, "101", "10.99999/test", &[])
        + &record("c", "Jones, A", 2020, "200", "10.99999/test", &[])
        + &record(
            "a",
            "Liu, C",
            2021,
            "102",
            "",
            &["Smith AB, 2020, TEST JOURNAL, V1, P101, DOI 10.99999/test"],
        );
    let a = analyze_with_mode(
        parse_wos(&text, "duplicate-doi").unwrap(),
        MatchingMode::Doi,
    );
    assert!(a.edges.is_empty());
    assert_eq!(a.ambiguities[0].candidates.len(), 2);
}
