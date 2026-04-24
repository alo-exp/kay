#!/usr/bin/env bash
# Verify that committed bindings.ts matches what tauri-specta would generate.
# Exits 1 if out of sync — run `cargo test -p kay-tauri --test gen_bindings` to regenerate.
set -euo pipefail

COMMITTED="crates/kay-tauri/ui/src/bindings.ts"

if [ ! -f "$COMMITTED" ]; then
  echo "ERROR: $COMMITTED not found. Run: cargo test -p kay-tauri --test gen_bindings"
  exit 1
fi

cp "$COMMITTED" /tmp/bindings-committed.ts

cargo test -p kay-tauri --test gen_bindings export_tauri_bindings 2>/dev/null

if ! diff -q "$COMMITTED" /tmp/bindings-committed.ts > /dev/null; then
  echo "ERROR: bindings.ts is out of sync with Rust types."
  echo "Run: cargo test -p kay-tauri --test gen_bindings && git add $COMMITTED"
  exit 1
fi

echo "bindings.ts: OK (in sync)"
