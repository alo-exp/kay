#!/bin/bash
set -e

echo "=== Testing forge_stream ==="
cd /Users/shafqat/Documents/Projects/opencode/vs-others
cargo test -p forge_stream 2>&1

echo "=== Testing forge_walker ==="
cargo test -p forge_walker 2>&1

echo "=== Testing forge_template ==="
cargo test -p forge_template 2>&1

echo "=== Testing kay-tauri ==="
cargo test -p kay-tauri 2>&1

echo "=== All tests complete ==="