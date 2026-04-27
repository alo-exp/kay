#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing kay-tools ==="
cargo test -p kay-tools -- --test-threads=4 2>&1 | tail -50