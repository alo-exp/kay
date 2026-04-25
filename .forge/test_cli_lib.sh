#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing kay-cli lib ==="
cargo test -p kay-cli --lib -- --test-threads=4 2>&1 | tail -50