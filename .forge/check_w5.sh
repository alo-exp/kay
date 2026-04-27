#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== GIT LOG ==="
git log --oneline -8
echo ""
echo "=== GIT STATUS ==="
git status --short
echo ""
echo "=== CARGO CHECK kay-core ==="
cargo check -p kay-core 2>&1 | tail -15
echo ""
echo "=== CARGO CHECK kay-core TESTS ==="
cargo check -p kay-core --tests 2>&1 | tail -15
