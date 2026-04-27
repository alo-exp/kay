#!/bin/sh
set -e
# Check all features of tauri-specta
cargo info tauri-specta 2>&1
echo "==="
# Check if specta-typescript is a dep of tauri-specta (without features)
cargo tree -p tauri-specta 2>&1 | grep specta-typescript
