#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing kay-core ==="
cargo test -p kay-core -- --test-threads=4 2>&1 | tail -50