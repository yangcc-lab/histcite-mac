# Compatibility and validation

HistCite Mac is an independent implementation of HistCite 12.03.17 workflows. It does not include the original HistCite executable, source code or manual. Its analysis core is written in Rust; the desktop shell uses Tauri and the system web view. Graphs use the open-source Graphviz 2.2.1 engine compiled to offline WebAssembly.

## What has been compared

The following checks were completed before the first public release:

| Area | Scope | Result |
| --- | --- | --- |
| Reference research corpus | 1,217 records; 62,116 reference entries | 954 directed citation links and each record's LCS/LCR/GCS matched |
| Analytical lists | Ten indexes; 1,217 record numbers | Group membership/counts and comparable totals matched |
| Citation rules | 19 synthetic cases | Captured reference results matched |
| Reference grouping | 25 paired probes; 40 groups | Captured reference results matched |
| Duplicate merging | 191 synthetic pairs | Identity decisions and retained GCS matched |
| Graph ordering/layout | 28 settings and operation-history cases | Node/rank order and PostScript geometry matched, including merged curves |
| Interaction paths | Menus, hints, three node shapes, keyboard and touch | Browser integration checks passed |
| Native Mac app | Import, editing, saving, graphs and selected pointer interactions | Tested separately from browser IPC mocks |

These results apply to the tested inputs and settings. They are not a guarantee of equivalence for every possible dataset or operation sequence. The reference research corpus and private capture environment are not redistributed. Public synthetic fixtures make the citation, reference-grouping and duplicate checks reproducible with `cargo test`.

## Details that affect results

- **Input:** use WoS tagged plain text with Full Record and Cited References. Missing reference lists are reported as incomplete data, not treated as evidence of no citations. Excel and native Scopus exports are not direct input formats.
- **Duplicate identity:** the default follows the observed original bibliographic rule, which can merge records with different UTs or DOIs. The retained record keeps the first bibliographic fields and reference list; GCS is the maximum. The merge history preserves incoming fields. Existing `.hcm` collections can be consolidated explicitly with **Tools → Merge duplicates** and undone.
- **Graph history:** the original layout order depends on prior graph operations. New saved graphs preserve the ordering state; old graphs without this state start from an empty history.
- **Spacing:** original HistCite ignores its X/Y distance controls in the captured DOT output. HistCite layout reproduces this; **Custom spacing** applies the requested distances.
- **Reference labels:** a group can display a different representative spelling while preserving its identity and occurrence count.
- **Browser integration:** WoS links open a browser and can use a configurable URL template. The original Internet Explorer integration is not reproduced.
- **Exports:** CSV has consistent quoting, explicit metrics and preserved original fields. It is not byte-identical to the legacy CSV. HTML is a self-contained interactive folder; display limits do not delete its underlying data.
- **Interaction rendering:** native macOS tooltips and controls can differ visually from Windows/Internet Explorer. The tested hover and keyboard paths do not cover every original interaction.

## Platforms

The public DMG contains Apple Silicon and Intel executables and requires macOS 13 or later. Native checks were performed on Apple Silicon; Intel builds are additionally checked under Rosetta where available. Testing has not covered every Mac model or macOS version.

The core library and CLI can be tested independently of the Mac desktop. The app runs offline after importing files; external DOI/WoS links require a browser and network access when selected.
