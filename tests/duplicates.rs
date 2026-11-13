use histcite_core::{
    analyze,
    collection::{Collection, fields_to_wos},
    parse_wos,
};
use serde_json::Value;

#[test]
fn original_duplicate_decisions_and_gcs_match_all_captured_probes() {
    for raw in [
        include_str!("fixtures/original/duplicates.json"),
        include_str!("fixtures/original/duplicates-extra.json"),
        include_str!(
            "fixtures/original/duplicates-normalization.json"
        ),
    ] {
        let probes: Vec<Value> = serde_json::from_str(raw).unwrap();
        for probe in probes {
            let input: Vec<std::collections::BTreeMap<String, Vec<String>>> =
                serde_json::from_value(probe["input"].clone()).unwrap();
            let text: String = input.iter().map(fields_to_wos).collect();
            let records = parse_wos(&text, "probe").unwrap();
            let a = analyze(records.clone());
            let c = Collection::from_text(&text, "probe").unwrap();
            let expected = probe["original"].as_array().unwrap();
            assert_eq!(
                c.records.len(),
                expected.len(),
                "{} keys: {:?}",
                probe["case"],
                records
                    .iter()
                    .map(histcite_core::duplicates::key)
                    .collect::<Vec<_>>()
            );
            assert_eq!(a.records.len(), expected.len(), "{} CLI", probe["case"]);
            c.validate().unwrap();
            if expected.len() == 1 {
                let tc = expected[0]["TC"][0]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok());
                assert_eq!(c.records[0].gcs, tc, "{} GCS", probe["case"]);
                assert_eq!(c.records[0].title, records[0].title);
                assert_eq!(c.records[0].references, records[0].references);
                assert_eq!(c.records[0].doi, records[0].doi);
                assert_eq!(c.merge_log[0].incoming.fields, records[1].fields);
                let saved = serde_json::to_string(&c).unwrap();
                let reopened = Collection::from_text(&saved, "saved.hcm").unwrap();
                assert_eq!(reopened.merge_log.len(), 1);
                assert_eq!(reopened.analyze().duplicate_records, 1);
            }
        }
    }
}

#[test]
fn merge_remaps_tags_graphs_and_preserves_conflicting_source_ut() {
    let text = |id: &str, page: u32, tc: u32| {
        format!(
            "PT J\nUT {id}\nAU Smith, A\nTI Paper\nPY 2020\nVL 1\nBP {page}\nTC {tc}\nNR 0\nER\n"
        )
    };
    let mut a = Collection::from_text(&text("first", 1, 2), "first").unwrap();
    let mut b = Collection::from_text(&text("second", 1, 7), "second").unwrap();
    b.tags.insert(
        "READ".into(),
        histcite_core::collection::Tag {
            description: "".into(),
            records: ["second".into()].into(),
        },
    );
    b.graphs.push(histcite_core::collection::SavedGraph {
        id: "g".into(),
        title: "G".into(),
        description: "".into(),
        records: vec!["second".into()],
        settings: Value::Null,
    });
    a.merge(b);
    assert_eq!(a.records.len(), 1);
    assert_eq!(a.records[0].gcs, Some(7));
    assert!(a.tags["READ"].records.contains("first"));
    assert_eq!(a.graphs[0].records, vec!["first"]);
    a.merge(Collection::from_text(&text("first", 2, 1), "third").unwrap());
    a.validate().unwrap();
    assert_eq!(a.records.len(), 2);
    assert_ne!(a.records[0].id, a.records[1].id);
    let exported = parse_wos(&a.hci(None), "roundtrip").unwrap();
    assert!(exported.iter().all(|r| r.fields["UT"] == ["first"]));
    let second = a.records[1].id.clone();
    a.edit_record(Some(&second), &text("first", 3, 4)).unwrap();
    assert_eq!(a.records[1].id, second);
    a.replace_items("authors", &["Smith A".into()].into(), "Jones, B")
        .unwrap();
    a.validate().unwrap();
    assert_eq!(a.records[1].id, second);
}

#[test]
fn tags_remain_attached_to_the_right_record_when_source_ids_conflict() {
    let text = "PT J\nUT shared\nAU Smith, A\nPY 2020\nBP 1\nNR 0\nTI First\nER\nPT J\nUT shared\nAU Smith, A\nPY 2020\nBP 2\nNR 0\nTI Second\nTG ONLYSECOND\nER\n";
    let c = Collection::from_text(text, "conflicting-ids").unwrap();
    assert_eq!(
        c.tags["ONLYSECOND"].records,
        [c.records[1].id.clone()].into()
    );
    let reopened = Collection::from_text(&c.hci(None), "roundtrip").unwrap();
    assert_eq!(
        reopened.tags["ONLYSECOND"].records,
        [reopened.records[1].id.clone()].into()
    );
}

#[test]
fn chained_imports_keep_a_complete_audit_and_the_final_survivor() {
    let record = |id: &str, gcs: u32| {
        format!("PT J\nUT {id}\nAU Smith, A\nPY 2020\nBP 1\nNR 0\nTI {id}\nTC {gcs}\nER\n")
    };
    let mut first = Collection::from_text(&record("first", 3), "first").unwrap();
    let second =
        Collection::from_text(&(record("second", 9) + &record("third", 7)), "second").unwrap();
    first.merge(second);
    assert_eq!(first.records.len(), 1);
    assert_eq!(first.records[0].gcs, Some(9));
    assert_eq!(first.records[0].title, "first");
    assert_eq!(first.duplicate_records, 2);
    assert_eq!(first.merge_log.len(), 2);
    assert!(first.merge_log.iter().all(|entry| entry.kept_id == "first"));
    first.consolidate();
    assert_eq!(first.merge_log.len(), 2);
    assert_eq!(first.analyze().duplicate_records, 2);
    first
        .edit_record(Some("first"), &record("renamed", 9))
        .unwrap();
    assert!(
        first
            .merge_log
            .iter()
            .all(|entry| entry.kept_id == "renamed")
    );
}
