#!/bin/bash
# Check kay-cli compilation status
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Checking kay-cli ==="
rm -f target/debug/.cargo-lock

# Try check first (faster than build)
cargo check -p kay-cli 2>&1 | tail -50

echo ""
echo "=== Done: $(date) ==="