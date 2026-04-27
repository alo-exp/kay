#!/bin/bash
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Testing forge_stream ==="
cargo test -p forge_stream 2>&1

echo "=== Testing forge_walker ==="
cargo test -p forge_walker 2>&1

echo "=== Testing forge_template ==="
cargo test -p forge_template 2>&1

echo "=== Testing kay-tauri ==="
cargo test -p kay-tauri --test commands 2>&1

echo "=== ALL DONE ==="