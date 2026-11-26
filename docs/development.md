# Developer guide

The [Mac download](https://github.com/yangcc-lab/histcite-mac/releases/latest) is the recommended installation for researchers. This guide is for contributors and source builds.

## Contribute a fix or improvement

- If you found a problem, use the [reporting guide](../README.md#report-a-problem-or-contribute) to describe the app version, steps and expected result in a GitHub issue. Corrections to the [tutorial](citation-analysis.md) are welcome too.
- For a citation matching change, read [how a reference becomes a citation link](citation-matching.md) and look at the [existing test fixtures](../tests/fixtures/README.md). Use a small fictional example to show the expected behavior, then run `cargo test --locked`.
- In a pull request, link the issue if there is one and explain how you checked the fix. When writing a release note, describe the affected workflow so researchers can tell whether the change matters to their work.

## Build the Mac app

Use macOS 13 or later, Xcode Command Line Tools, Rust 1.99.1 or later (edition 2024), and Python 3.9 or later. The UI has no npm build step. The offline layout engine is already included.

```sh
git clone https://github.com/yangcc-lab/histcite-mac.git
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

## Build and verify release downloads

The [Attested macOS build](../.github/workflows/release-build.yml) builds a Universal DMG on a GitHub-hosted runner. A version tag that matches the app version and is on main history creates a release build. A manual run on main is useful for testing the pipeline.

For a release, download the `attested-macos-build` artifact from the successful run for the version tag. Use the DMG, `SHA256SUMS.txt`, the practice dataset, and the matching `.intoto.jsonl` file together when uploading assets to a GitHub release draft.

Verify the downloaded file locally with GitHub CLI. Substitute the release tag you are checking.

```sh
tag=vX.Y.Z
dmg="HistCite-Mac-${tag#v}-universal.dmg"
commit=$(gh api "repos/yangcc-lab/histcite-mac/commits/$tag" --jq .sha)
shasum -a 256 -c SHA256SUMS.txt
gh attestation verify "$dmg" --bundle "$dmg.intoto.jsonl" \
  --repo yangcc-lab/histcite-mac \
  --signer-workflow yangcc-lab/histcite-mac/.github/workflows/release-build.yml \
  --source-ref "refs/tags/$tag" --source-digest "$commit" --deny-self-hosted-runners
```

The verification reports the source commit and workflow that produced the file.

## Run the tests

Rust 1.99.1 is the tested support floor, not a claim that earlier compilers cannot work. CI tests the core on that version and current stable. On macOS, run `cargo test --manifest-path desktop/Cargo.toml --locked` for background scheduling tests as well.

The Rust core and its fixtures also run on Linux:

```sh
cargo test --locked
```

The Rust tests use synthetic fixtures and temporary files; no research data or original HistCite installation is required. They include import limits, recovery round trips, private file permissions and source-file protection.

For security scans, see the [review and reproducible commands](security-review.md). Scanner checks also run weekly in GitHub Actions.

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
