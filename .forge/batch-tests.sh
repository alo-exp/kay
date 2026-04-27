#!/bin/bash
# Per-crate test batch script for remaining crates
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Testing kay-cli (may take a few minutes) ==="
cargo test -p kay-cli 2>&1 | tail -100
echo ""
echo "=== Testing kay-provider-openrouter ==="
cargo test -p kay-provider-openrouter 2>&1 | tail -50
echo ""
echo "=== Testing kay-sandbox-macos ==="
cargo test -p kay-sandbox-macos 2>&1 | tail -30
echo ""
echo "=== Testing kay-tui ==="
cargo test -p kay-tui 2>&1 | tail -30
echo ""
echo "=== All batch tests complete: $(date) ==="