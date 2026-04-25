#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing kay-tui ==="
cargo test -p kay-tui -- --test-threads=4 2>&1 | tail -50