#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Checking forge_embed ==="
cargo check -p forge_embed --tests 2>&1

echo "=== Checking forge_markdown_stream ==="
cargo check -p forge_markdown_stream --tests 2>&1

echo "=== Checking forge_repo ==="
cargo check -p forge_repo --tests 2>&1

echo "=== Checking forge_services ==="
cargo check -p forge_services --tests 2>&1

echo "=== Checking forge_snaps ==="
cargo check -p forge_snaps --tests 2>&1

echo "=== Checking forge_stream ==="
cargo check -p forge_stream --tests 2>&1

echo "=== Checking forge_template ==="
cargo check -p forge_template --tests 2>&1

echo "=== Checking forge_tracker ==="
cargo check -p forge_tracker --tests 2>&1

echo "=== Checking forge_walker ==="
cargo check -p forge_walker --tests 2>&1

echo "ALL_CHECKS_PASSED"
