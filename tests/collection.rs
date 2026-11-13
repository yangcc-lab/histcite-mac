use histcite_core::{
    analytics::{self, Settings},
    collection::{Collection, SavedGraph, Tag},
    parse_wos,
};
use std::collections::BTreeSet;
fn fixture() -> Collection {
    Collection::from_text("PT J\nAU Smith, AB\nTI Original title\nSO JOURNAL\nJ9 JOURNAL\nPY 2010\nVL 1\nBP 1\nTC 8\nNR 0\nUT target\nXX Keep unknown field\nER\nPT J\nAU Smith, AB\n   Jones, C\nTI Citing title\nSO JOURNAL\nPY 2012\nTC 4\nNR 1\nCR Smith A, 2010, JOURNAL, V1, P1\nUT citing\nER\nPT J\nAU Lee, D\nTI Latest\nPY 2020\nNR 1\nCR Smith A, 2010, JOURNAL, V1, P1\nUT latest\nER\n","fixture").unwrap()
}
#[test]
fn save_reopen_preserves_metadata_tags_graphs_and_unknown_fields() {
    let mut c = fixture();
    c.title = "A <collection>".into();
    c.description = "Multiline\ndescription".into();
    c.tags.insert(
        "READ".into(),
        Tag {
            description: "Read next".into(),
            records: BTreeSet::from(["target".into()]),
        },
    );
    c.graphs.push(SavedGraph {
        id: "g".into(),
        title: "Graph".into(),
        description: "".into(),
        records: vec!["target".into(), "citing".into()],
        settings: serde_json::json!({"direction":"forward"}),
    });
    let json = serde_json::to_string(&c).unwrap();
    let reopened = Collection::from_text(&json, "saved.hcm").unwrap();
    assert_eq!(reopened.graphs[0].settings["direction"], "forward");
    assert_eq!(reopened.tags["READ"].records.len(), 1);
    assert_eq!(reopened.records[0].fields["XX"], vec!["Keep unknown field"]);
    assert_eq!(reopened.analyze().edges.len(), 2);
    let hci = c.hci(None);
    let imported = Collection::from_text(&hci, "saved.hci").unwrap();
    assert_eq!(imported.title, c.title);
    assert_eq!(imported.description, c.description);
    assert!(imported.tags["READ"].records.contains("target"));
    assert_eq!(imported.analyze().edges.len(), 2);
}
#[test]
fn edits_rebuild_citations_and_rename_record_ids_in_tags() {
    let mut c = fixture();
    c.tags.insert(
        "T".into(),
        Tag {
            description: "".into(),
            records: BTreeSet::from(["target".into()]),
        },
    );
    c.edit_record(
        Some("target"),
        "PT J\nUT renamed\nAU Smith, AB\nTI Revised\nPY 2010\nVL 1\nBP 2\nNR 0\nER\n",
    )
    .unwrap();
    assert_eq!(c.analyze().edges.len(), 0);
    assert!(c.tags["T"].records.contains("renamed"));
    assert!(!c.tags["T"].records.contains("target"));
    let before = c.records.len();
    assert!(
        c.edit_record(None, "PT J\nUT renamed\nTI Duplicate\nER\n")
            .is_err()
    );
    assert_eq!(c.records.len(), before);
}
#[test]
fn hci_subset_recalculates_local_scores_and_preserves_reference_lists() {
    let c = fixture();
    let ids = BTreeSet::from(["citing".into()]);
    let reopened = Collection::from_text(&c.hci(Some(&ids)), "subset").unwrap();
    let a = reopened.analyze();
    assert_eq!(a.records.len(), 1);
    assert_eq!(a.records[0].references.len(), 1);
    assert_eq!(a.edges.len(), 0);
}
#[test]
fn deletion_cleans_tags_and_graph_members() {
    let mut c = fixture();
    c.tags.insert(
        "T".into(),
        Tag {
            description: "".into(),
            records: BTreeSet::from(["target".into()]),
        },
    );
    c.delete(&BTreeSet::from(["target".into()]));
    assert_eq!(c.records.len(), 2);
    assert!(c.tags["T"].records.is_empty());
    assert_eq!(c.analyze().edges.len(), 0);
}
#[test]
fn editing_variants_changes_all_occurrences_but_preserves_other_fields() {
    let mut c = fixture();
    assert_eq!(
        c.replace_items("authors", &BTreeSet::from(["Smith AB".into()]), "Brown, X")
            .unwrap(),
        2
    );
    assert_eq!(c.records[0].fields["XX"], vec!["Keep unknown field"]);
    assert_eq!(c.analyze().edges.len(), 0);
    c.replace_items(
        "references",
        &BTreeSet::from(["Smith A, 2010, JOURNAL, V1, P1".into()]),
        "Brown X, 2010, JOURNAL, V1, P1",
    )
    .unwrap();
    assert_eq!(c.analyze().edges.len(), 2);
}
#[test]
fn bibliometrics_exclude_shared_authors_and_use_collection_end() {
    let c = fixture();
    let a = c.analyze();
    assert_eq!(a.metrics[0].lcsx, 1);
    assert_eq!(a.metrics[0].lcsb, 1);
    assert_eq!(a.metrics[0].lcse, 1);
    assert_eq!(a.metrics[0].ratio.as_deref(), Some("1.00"));
    assert!((a.metrics[0].lcst - 2.0 / 11.0).abs() < 1e-12);
    assert_eq!(a.metrics[2].lcsb, -1);
    assert_eq!(analytics::h_index([8, 4, 1]), 2);
}
#[test]
fn original_hci_header_and_blank_template_are_accepted() {
    let text = "FN Thomson Reuters Web of Knowledge\nMG HistCite 12.03.17\nMT Collection\nTD READ: Description\nER\nPT \nAU \nTI \nTC \nPY \nER\nPT J\nUT real\nTI Actual\nNR 0\nTG READ\nER\nEF\n";
    let c = Collection::from_text(text, "original.hci").unwrap();
    assert_eq!(c.records.len(), 1);
    assert!(c.tags["READ"].records.contains("real"));
    assert!(parse_wos("ER\nPT J\nUT x\nER\n", "not-hci").is_err());
}
#[test]
fn json_schema_and_setting_validation_reject_invalid_input() {
    let mut c = fixture();
    c.settings.record_page_size = 0;
    assert!(c.validate().is_err());
    assert!(Collection::from_text("{\"schema_version\":99}", "new").is_err());
    let s = Settings::default();
    assert!(s.validate().is_ok());
}

#[test]
fn cited_reference_index_matches_original_controlled_pairs() {
    let c = Collection::from_text(include_str!("fixtures/reference-index-probes.txt"), "probe")
        .unwrap();
    assert_eq!(c.records.len(), 25);
    let a = c.analyze();
    let groups = &a.indexes["references"];
    let original = include_str!("fixtures/original/reference-index.csv");
    let mut compared = 0;
    for line in original.lines().skip(1) {
        let (name, count) = line.trim_start_matches('"').rsplit_once("\",").unwrap();
        let key = analytics::reference_index_key(name);
        let group = groups
            .iter()
            .find(|g| analytics::reference_index_key(&g.name) == key)
            .unwrap();
        assert_eq!(group.occurrences, count.parse::<usize>().unwrap(), "{name}");
        compared += 1;
    }
    assert_eq!(compared, 40);
    assert_eq!(groups.len(), compared);
    assert_eq!(groups.iter().map(|g| g.occurrences).sum::<usize>(), 50);
}
#[test]
fn duplicate_author_names_and_address_variants_keep_legacy_weights() {
    let c=Collection::from_text("PT J\nAU Ali, S\n   Ali, S\nTI Variants\nTC 3\nPY 2020\nC1 [Ali, S] Example Univ, Dept A, City, USA.\n   [Ali, S] EXAMPLE UNIV, Dept B, City, USA.\nNR 0\nUT variants\nER\n", "variants").unwrap();
    let a = c.analyze();
    assert_eq!(a.indexes["authors"][0].occurrences, 2);
    assert_eq!(a.indexes["authors"][0].gcs, 6);
    assert_eq!(a.indexes["institutions"][0].occurrences, 2);
    assert_eq!(a.indexes["institutions"][0].indices.len(), 1);
    assert_eq!(a.indexes["countries"][0].occurrences, 1);
}
#[test]
fn duplicate_imports_are_reported_without_invalidating_collection() {
    let first = fixture();
    let text = first.hci(None);
    let mut c = Collection::from_text(&text, "one").unwrap();
    c.merge(Collection::from_text(&text, "two").unwrap());
    c.validate().unwrap();
    assert_eq!(c.records.len(), 3);
    assert_eq!(c.analyze().duplicate_records, 3);
}
#[test]
fn reference_variant_edit_rewrites_whole_legacy_group() {
    let mut c = Collection::from_text(include_str!("fixtures/reference-index-probes.txt"), "probe")
        .unwrap();
    c.replace_items(
        "references",
        &BTreeSet::from(["Probe06 A, 2000, ONE JOURNAL, V2".into()]),
        "Corrected A, 2000, JOURNAL, V2",
    )
    .unwrap();
    assert_eq!(
        c.records[6].references,
        vec!["Corrected A, 2000, JOURNAL, V2"; 2]
    );
}

#[test]
fn address_edits_cover_semicolon_entries_and_reprint_fallback() {
    let mut c=Collection::from_text("PT J\nAU Smith, A\nTI Addresses\nUT a\nNR 0\nC1 [Smith, A; Jones, B] First Univ, City, USA; Second Univ, Dept, City, UK.\nER\nPT J\nAU Other, B\nTI Reprint\nUT b\nNR 0\nRP Second Univ, Dept, City, UK.\nER\n", "addresses").unwrap();
    assert_eq!(
        c.replace_items(
            "institutions",
            &BTreeSet::from(["Second Univ".into()]),
            "Unified Univ"
        )
        .unwrap(),
        2
    );
    assert!(c.records[0].fields["C1"][0].starts_with("[Smith, A; Jones, B] First Univ"));
    assert!(c.records[0].fields["C1"][0].contains("Unified Univ, Dept, City, UK."));
    assert!(c.records[1].fields["RP"][0].starts_with("Unified Univ"));
}

#[test]
fn original_multiple_tags_roundtrip_as_space_delimited_names() {
    let c = Collection::from_text(
        include_str!("fixtures/original/multi-tag.hci"),
        "original",
    )
    .unwrap();
    assert_eq!(c.records.len(), 25);
    assert_eq!(c.tags["READ"].records.len(), 25);
    assert_eq!(c.tags["SECOND"].records.len(), 25);
    let text = c.hci(None);
    assert!(text.contains("TG READ SECOND\n"));
    let again = Collection::from_text(&text, "roundtrip").unwrap();
    assert_eq!(again.tags["SECOND"].records.len(), 25);
}
