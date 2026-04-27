#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
export CARGO_TARGET_DIR=/tmp/kay-test

echo "=== test kay-core ==="
/tmp/kay-test/debug/kay test -p kay-core 2>&1 | tail -10

echo ""
echo "=== build kay-cli ==="
/tmp/kay-test/debug/kay build -p kay-cli 2>&1 | tail -5

echo ""
echo "=== check kay-core ==="
/tmp/kay-test/debug/kay check -p kay-core 2>&1 | tail -5

echo ""
echo "=== fmt ==="
/tmp/kay-test/debug/kay fmt 2>&1 | tail -5

echo ""
echo "DONE"
