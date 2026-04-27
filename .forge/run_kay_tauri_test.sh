#!/bin/bash
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Testing kay-tauri commands ==="
cargo test -p kay-tauri --test commands 2>&1

echo "=== Done ==="