#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing sandbox crates ==="
cargo test -p kay-sandbox-linux -p kay-sandbox-macos -p kay-sandbox-windows -- --test-threads=4 2>&1 | tail -50