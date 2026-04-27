#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Running kay-tauri commands tests ==="
cargo test -p kay-tauri --test commands 2>&1
echo "DONE"