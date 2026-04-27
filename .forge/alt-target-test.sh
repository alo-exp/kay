#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Use alternative target dir to avoid fileproviderd lock on default target/
TARGET_DIR="/tmp/kay-target-test"
mkdir -p "$TARGET_DIR"
export CARGO_TARGET_DIR="$TARGET_DIR"

echo "Using target: $CARGO_TARGET_DIR"
echo "Date: $(date)"

# Run full workspace test with nextest
cargo nextest run --workspace --no-fail-fast 2>&1

echo "=== DONE: $(date) ==="