# Security review — 14 November 2026

This is an AI-assisted review and reproducible automated checks. It is **not a certification, a complete security audit, or an audit performed by Trail of Bits**. Badges link to live evidence and may change as dependencies, rules and code change.

Baseline: [`ad20431`](https://github.com/yangcc-lab/histcite-mac/commit/ad204319a7e382feb8a6e5d9421c512d414284ca). Application fixes: [`2000de4`](https://github.com/yangcc-lab/histcite-mac/commit/2000de4), version 0.3.3. Version **0.3.4** ([application changes](https://github.com/yangcc-lab/histcite-mac/commit/03327f2)) follows up with bounded background work, notification recovery and Graphviz warning triage. The checks below were rerun after remediation.

## What the badges cover

| Check | Local verification result and scope |
|---|---|
| cargo-audit 0.23.0 | Both complete lockfiles: 25 core and 431 desktop dependencies; zero vulnerability-class advisories. Two Linux-only informational advisories remain visible below. |
| cargo-deny 0.20.2 | Both crates, Apple Silicon and Intel macOS dependency graphs: advisory, license, source and wildcard policy passed, with no ignored advisories. Duplicate-version warnings remain warnings. |
| Semgrep 1.185.0 | `p/security-audit` and `p/trailofbits`; 27 first-party source/build/port files scanned, zero findings and zero parsing errors after remediation. Metrics disabled. |
| Gitleaks 8.30.1 | Full available Git history; zero detected secrets. Reports are redacted. Ignored private working files are outside this scan. |
| OpenSSF Scorecard | Repository practices, published by the official action. See the live score and individual checks; a score does not prove that the application has no vulnerabilities. |

[Scanner runs and artifacts](https://github.com/yangcc-lab/histcite-mac/actions/workflows/security.yml) · [Scorecard runs](https://github.com/yangcc-lab/histcite-mac/actions/workflows/scorecard.yml)

Source findings, scanner failures and parsing failures fail the check. Workflow artifacts retain logs and JSON for 30 days. The badge describes the latest workflow on `main`; it does not attest to an arbitrary downloaded binary. Rust tests and Mac builds run in the separate [Checks workflow](https://github.com/yangcc-lab/histcite-mac/actions/workflows/checks.yml).

## AI review coverage

The full [Trail of Bits rust-review skill](https://github.com/trailofbits/skills/tree/149d7b21f4067765b82e67029730b42d228ef4b4/plugins/rust-review), version 1.1.2, supplied the plan, review prompts, artifact validation and sequential deduplication/severity review. The user selected malicious imported data and local unprivileged attack scenarios, with all severity levels reported.

**11 of 12 planned clusters completed.** The panic/resource-exhaustion cluster was interrupted by the agent tool's own security restriction and was not retried. It remains incomplete. Targeted import tests added during remediation do not constitute completion of that entire cluster.

Completed clusters: unsafe boundaries, concurrency locking, data races, recursion, error handling, logic correctness, async runtime, static hygiene, resource handling, filesystem/path safety and pointer disclosure. First-party Rust contained no unsafe blocks or FFI definitions. This Rust review does not establish the safety of C code, platform libraries or Rust dependencies.

The workflow was adapted to Codex: generated assignments were read from files, a file ledger replaced Claude task tools, and the current Codex model performed the reviews. The upstream plan, per-pass coverage validation and sequential judges were retained; the Claude cache-primer step was disabled. This is use of an open-source skill, not a vendor endorsement.

## Findings and disposition

The pipeline retained **one medium finding and 14 low hardening/reliability observations** on the baseline. The low observations are not 14 demonstrated exploits; the upstream rubric deliberately labels real hardening gaps as low findings.

| Finding | Baseline issue | Status through 0.3.4 |
|---|---|---|
| TOCTOU-001 — medium | Saving into a shared directory with an attacker-replaceable ancestor could redirect pathname resolution after source-file validation. Requires another local actor, user interaction, knowledge of the target and winning a race. | Fixed: retain an open destination directory, compare protected source identities through it, and create/rename relative to the same descriptor. |
| CARGOLINT-001, 002 — low | Neither first-party Rust crate forbade unsafe code. | Fixed: both manifests enforce `unsafe_code = "forbid"`. |
| RESDISC-003 — low | Temporary-file cleanup errors were silently discarded. | Fixed: cleanup only follows successful temporary creation; failures are returned with the original error. |
| RESDISC-001, 002 — low | Collection-change and graph-load notifications are best-effort; delivery failures are not surfaced. | Improved in 0.3.4: log broadcast failures, return explicit graph-delivery errors, surface listener registration errors, and check the authoritative revision on focus/visibility changes. Saved-graph requests survive stale window data. Delivery is still best-effort; active windows may remain stale until refocused if an event is lost. |
| MSRV-001, 002 — low | No minimum Rust toolchain is declared. | Resolved in 0.3.4: both manifests declare the tested Rust 1.99.1 support floor. Core CI tests that version and stable; local desktop tests/builds use 1.99.1 and 1.100. Packaged-app installation is unaffected. |
| ASYNCBLOCK-001–007 — low | Several desktop commands perform analysis or synchronous filesystem work on async executor threads. | Addressed in 0.3.4: analysis, serialization, recovery and file writes run on a serial blocking lane, with at most 32 admitted operations. Queued callers await asynchronously. Revision checks stay inside transactions; native quit waits for admitted work. Dialogs run outside the lane. This is scheduling hardening, not a bound on the duration of each analysis. |

Additional coordinator findings and hardening, outside those 15 entries:

- **Recovery round trip:** deeply nested graph settings could parse as an HCM collection but exceed the parser limit once wrapped in recovery JSON. Settings now have a depth limit; accepted settings are tested through a recovery round trip.
- **File privacy:** newly saved working data, recovery files and exports use owner-only read/write permissions (`0600`). Replacing a file does not reintroduce group/other read access. Existing exports elsewhere are not retroactively changed.
- **Input bounds:** files must be regular files of at most 100 MiB, with the limit enforced while reading too. Collections are limited to 100,000 records, 2,000,000 references and 100 MiB of compact serialized data; recovery wrappers are checked before writing. Years must be 1–9999 or absent, GCS must fit an unsigned 32-bit count, and graph settings may nest at most 16 levels. These bounds do not guarantee that every accepted collection will be fast.
- **Export snapshots (0.3.4):** exports protect both the source files of the captured snapshot and those in the current workspace when a different window changes collections during the save dialog.
- **Other hardening:** replace fallible desktop `unwrap` calls with errors, reap the browser-launch helper, and compute graph year bounds once without expanding large argument lists. Citation definitions and tested graph coordinates are unchanged.

The initial first-party Semgrep scan reported four `unwrap` observations and one dynamic-URL observation. The `unwrap` sites were changed. The license-notice downloader's URL observation was reviewed as a false positive: URLs are built from fixed upstream HTTPS hosts during development, not imported research data. A single rule-specific suppression with rationale sits beside that call. No vulnerability advisory is ignored in `deny.toml`.

## Rust dependency advisories

The full desktop lockfile also contains dependencies for platforms this app does not ship:

- [`RUSTSEC-2024-0370`](https://rustsec.org/advisories/RUSTSEC-2024-0370.html): `proc-macro-error` is unmaintained.
- [`RUSTSEC-2024-0429`](https://rustsec.org/advisories/RUSTSEC-2024-0429.html): unsound iterator implementations in `glib` 0.18.5.

Both were absent from the resolved Apple Silicon and Intel dependency graphs. The full-lockfile audit keeps the warnings visible; cargo-deny's gate uses the shipped macOS targets. This does not clear a Linux desktop build. The local RustSec database was commit `4ef2df8e806a65e2f1ecc14f56f51249062b23d6`; scheduled scans retrieve current data.

## Legacy Graphviz: unresolved scope

Graphviz 2.2.1 is retained for tested HistCite layout compatibility. Generated `ui/graphviz.js` and the upstream C archive are **outside the clean first-party Semgrep gate**. Cargo scanners do not cover them.

An additional source scan covered 656 upstream files with 35 applicable rules. It reported **50 pattern warnings**: 18 string-copy, 11 concatenation, 15 tokenizer, 2 double-free and 4 formatted-output observations. There were also **90 parser-error/partial-parser entries** on legacy C constructs. These are patterns, not 50 confirmed vulnerabilities. The 0.3.4 follow-up classified all 50: 21 in files omitted from the build, 10 in disabled source sections, and 19 in enabled source sections inspected in context. See the [site-by-site disposition and its limits](graphviz-review.md). The worker now explicitly permits only SVG/PostScript. This is not a successful full C security review.

The app generates DOT from numeric identifiers, years and constrained controls, runs the engine in a fresh Web Worker, limits WebAssembly linear memory to 256 MiB, and terminates layout after 60 seconds. These boundaries reduce exposure; they do not prove the absence of C memory bugs or browser-level effects. [NVD documents historical Graphviz memory-corruption issues](https://nvd.nist.gov/vuln/detail/CVE-2020-18032); a version match alone does not establish reachability in this port.

A dedicated review of compiled C paths and controlled fuzzing remain outstanding. There was no full WebAssembly binary audit, platform penetration test or independent professional review. The badge must not be presented as clearing this engine.

## Regression evidence

- 34 Rust tests passed, including five import/filesystem tests from 0.3.3 for directory identity, protected source aliases, private permissions, import bounds and recovery nesting.
- Six additional desktop tests passed: async executor availability, cancelled caller lifetime, serialized revision checks, queue admission limits, orderly shutdown and the race between admission and execution.
- Nine browser interaction groups and the offline HTML demo passed. Interaction tests explicitly mock desktop IPC; the offline demo uses real popups/downloads without that mock.
- All 28 saved original-HistCite graph geometry/order comparisons passed. These local reference-data comparisons are compatibility evidence, not public CI or a security test.
- The full 1,217-record analysis (62,116 references, 954 citation links) was byte-equivalent as parsed JSON to the pre-fix core's output, including every record, score and analysis index. The private source records are not published.
- The desktop crate passed `cargo check --locked`; public CI also builds the Mac app.

Version 0.3.4 did not change the citation core or the bundled Graphviz binary. Notification recovery was tested with intentionally dropped notifications and explicit IPC mocks; scheduling was tested in Rust. An isolated macOS app identifier was used for native demo import, a 12-node/31-link graph, an HCM save with private permissions, and normal menu-driven quit. The final native build also passed startup from the saved HCM and normal menu-driven quit after the scheduling and export-source changes.

No research collection, original proprietary executable, signing key or private agent workspace is published with these checks. Releases remain ad-hoc signed and unnotarized.

## Repeat the checks

Install `cargo-audit`, `cargo-deny`, Semgrep and Gitleaks, then run:

```sh
python3 scripts/check_security.py all
cargo test --locked
```

Use `rust`, `source` or `secrets` instead of `all` for one group. Results go to ignored `.runtime/security/`. CI extracts the native scanner and its bundled shared libraries from the [SHA-256-pinned Semgrep wheel](../.github/requirements-semgrep.txt). The local command uses the standard Semgrep CLI. Dependency policy for the Rust packages is in [deny.toml](../deny.toml). Registry rules and advisories intentionally refresh, so later results may differ.

Application fixes and CI/report changes are separate Conventional Commits, reviewed with `git diff --cached` and `git diff --cached --check`. Raw private agent logs are not uploaded to CI.
