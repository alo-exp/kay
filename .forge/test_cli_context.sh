#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== Testing kay-cli context_smoke ==="
cargo test -p kay-cli --test context_smoke -- --test-threads=4 2>&1 | tail -50