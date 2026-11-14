# Security policy

Please report suspected vulnerabilities through [GitHub's private vulnerability reporting](https://github.com/yangnim21029/histcite-mac/security/advisories/new). Include the affected version, expected and actual behavior, and a minimal synthetic example when possible. Do not post private research data, credentials or exploit details in a public issue.

Security fixes are made for the latest release. Use **0.3.3 or later** for the import and file-save hardening described in the [review report](docs/security-review.md).

The [Security checks workflow](https://github.com/yangnim21029/histcite-mac/actions/workflows/security.yml) checks both Rust lockfiles, macOS dependency policy, first-party source patterns and Git history for secrets. It runs on changes and weekly. OpenSSF Scorecard reports repository practices. These checks are not security certification or an independent professional audit.

The legacy Graphviz 2.2.1 WebAssembly engine has separate, unresolved review limitations. It is **not covered by a clean Rust dependency result**. See the report for the exact scope.

The app processes collections locally. Opening DOI or WoS links uses the system browser. Releases remain ad-hoc signed and **not Apple-notarized**; scanning does not change that status.
