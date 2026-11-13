# Developer guide

The [Mac download](https://github.com/yangnim21029/histcite-mac/releases/latest) is the recommended installation for researchers. This guide is for contributors and source builds.

## Build the Mac app

Use macOS 13 or later, Xcode Command Line Tools, a current stable Rust toolchain (edition 2024), and Python 3.9 or later. The UI has no npm build step. The offline layout engine is already included.

```sh
git clone https://github.com/yangnim21029/histcite-mac.git
cd histcite-mac
python3 scripts/build_macos.py
open "dist/HistCite Mac.app"
```

The script builds for your Mac's architecture and ad-hoc signs the app. To produce the universal download:

```sh
python3 scripts/build_macos.py --universal
python3 scripts/package_dmg.py
```

This installs both Rust targets, builds each architecture, combines them with `lipo`, verifies the signature, and creates a DMG with an Applications shortcut and sample collection. The checksum is written to `dist/SHA256SUMS.txt`. Apple notarization is not performed by these scripts.

After dependency changes, regenerate bundled Rust license notices with `python3 scripts/package_rust_notices.py`. This reads the locked macOS dependency graph and retrieves missing upstream notices. Regular builds use the checked-in notices and do not need that step.

## Run the tests

The Rust core and its fixtures also run on Linux:

```sh
cargo test --locked
```

All 29 tests use synthetic fixtures checked into `tests/fixtures`; no research data or original HistCite installation is required.

Browser checks use real Rust output from the synthetic demo collection:

```sh
python3 -m venv .runtime/venv
source .runtime/venv/bin/activate
pip install -r tests/requirements.txt
python -m playwright install chromium
python scripts/check_interactions.py
python scripts/check_demo.py
```

On Linux, `python -m playwright install --with-deps chromium` also installs required system libraries. Reports and screenshots go into `.runtime/`. `check_interactions.py` explicitly mocks desktop IPC to exercise menus, hints, keyboard and pointer behavior. `check_demo.py` checks the offline HTML presentation with actual popup windows and downloads; it does not mock IPC. Neither substitutes for native macOS dialog tests.

For an existing analysis fixture, use `check_interactions.py --fixture path/to/analysis.json --output path/to/report`. Set `CHROME` to choose a Chrome executable; otherwise a local Chrome installation or Playwright's Chromium is used.

## Command line

```sh
cargo run --release -- --output analysis.json examples/demo-wos.txt
```

Multiple input files can follow the output filename. The default `histcite` matching mode follows the observed original behavior. `--mode doi` before `--output` selects an experimental DOI matching mode; its output should not be described as HistCite-compatible.

## Code map

| Location | Purpose |
| --- | --- |
| `src/lib.rs`, `src/analytics.rs` | WoS parsing, citation matching, scores and analytical indexes |
| `src/duplicates.rs`, `src/collection.rs` | Duplicate rules, editable collections and persistence |
| `desktop/` | Tauri 2 windows, dialogs, local files and exports |
| `ui/` | Tables, detail windows, menus and Graph Maker |
| `ui/graph-layout.js` | Original DOT conventions and ordering state |
| `ui/graphviz.js` | Bundled Graphviz 2.2.1 WebAssembly engine |
| `tests/fixtures/` | Synthetic cases and captured expected results |

The existing application identifier is retained to preserve saved workspaces across updates. User collections and recovery files are never bundled in releases.

## Rebuild the layout engine (optional)

The complete upstream source, portable configuration, sorting adapter and licenses are in `ui/Graphviz-source.zip`. They are included in every app and HTML export. Only engine development needs Emscripten 4.0.10:

```sh
mkdir -p .runtime/graphviz-source vendor
unzip -o ui/Graphviz-source.zip -d .runtime/graphviz-source
tar -xzf .runtime/graphviz-source/graphviz-2.2.1.tar.gz -C vendor
EMCC=/path/to/emsdk/upstream/emscripten/emcc python3 scripts/build_graphviz.py
python3 scripts/package_graphviz_source.py
```

`vendor/graphviz-2.2.1/` is an ignored working directory. Keep modifications to the portable build under its existing CPL 1.0 terms, and retain the BSD-3-Clause notice on the sorting adapter. See [third-party notices](../ui/THIRD-PARTY.txt).
