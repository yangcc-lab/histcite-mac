#!/bin/bash
set -euo pipefail

cd "$SRC/histcite-mac"
target="$ARCHITECTURE-unknown-linux-gnu"
cargo fetch --locked --manifest-path fuzz/Cargo.toml --target "$target"
lock_checksum=$(sha256sum fuzz/Cargo.lock)
CARGO_NET_OFFLINE=true cargo fuzz build -O --debug-assertions \
  --target "$target" --target-dir "$WORK/fuzz-target" wos_import
printf '%s\n' "$lock_checksum" | sha256sum --check
cp "$WORK/fuzz-target/$target/release/wos_import" "$OUT/"
zip -qj "$OUT/wos_import_seed_corpus.zip" examples/demo-wos.txt
