#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Running workspace tests ==="
cargo test -p kay-core -p kay-tools -p kay-sandbox-macos -p kay-sandbox-linux -p kay-tauri -p kay-tui -- --test-threads=4 2>&1 | tail -80