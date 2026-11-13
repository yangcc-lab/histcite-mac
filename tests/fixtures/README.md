# Synthetic test fixtures

All records in this directory are artificial test inputs. They are not a research collection or a Web of Science data dump.

`original/` contains the corresponding outputs observed in HistCite 12.03.17:

- `citation-rules.json`: citation edges and per-record counts for the rule probes.
- `reference-index.csv`: grouping of paired reference variants.
- `multi-tag.hci`: round trip of two tags across 25 synthetic records.
- `duplicates*.json`: 191 pairs covering duplicate identity, normalization and citation-count selection.

These fixtures let `cargo test` check the captured behavior without installing the original program. They contain data and expected results, not HistCite program code. Larger comparisons on research datasets are summarized in [Compatibility](../../docs/compatibility.md); those datasets are not distributed.
