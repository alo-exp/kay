#!/bin/sh
set -e
# Check what tauri-specta's features actually include
cargo tree -p tauri-specta -e features 2>&1 | head -20
echo "==="
# Check if kay-tauri exposes specta_typescript via any re-export
grep -rn "specta_typescript\|specta::typescript" crates/kay-tauri/src/ 2>&1
