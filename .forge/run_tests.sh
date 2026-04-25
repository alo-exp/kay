#!/bin/sh
set -e
echo "=== Running kay-tauri tests ==="
cargo test -p kay-tauri 2>&1 | tail -15
