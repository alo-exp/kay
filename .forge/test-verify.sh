#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Phase 12 Test Verification ==="
echo "Running: $(date)"
echo ""

# Critical crates first (no output, just check compilation)
echo "[1/9] kay-core (20 inline + 11 integration)..."
cargo test -p kay-core --no-fail-fast 2>&1 | tail -20
echo ""

echo "[2/9] kay-context (33 inline + 7 integration)..."
cargo test -p kay-context --no-fail-fast 2>&1 | tail -20
echo ""

echo "[3/9] kay-tools (20 integration tests)..."
cargo test -p kay-tools --no-fail-fast 2>&1 | tail -20
echo ""

echo "[4/9] kay-verifier (3 integration tests)..."
cargo test -p kay-verifier --no-fail-fast 2>&1 | tail -20
echo ""

echo "[5/9] kay-session (12 integration tests)..."
cargo test -p kay-session --no-fail-fast 2>&1 | tail -20
echo ""

echo "[6/9] kay-provider-openrouter (12 inline + 8 integration)..."
cargo test -p kay-provider-openrouter --no-fail-fast 2>&1 | tail -20
echo ""

echo "[7/9] kay-sandbox-macos..."
cargo test -p kay-sandbox-macos --no-fail-fast 2>&1 | tail -20
echo ""

echo "[8/9] kay-cli (16 integration tests)..."
cargo test -p kay-cli --no-fail-fast 2>&1 | tail -20
echo ""

echo "[9/9] kay-tui..."
cargo test -p kay-tui --no-fail-fast 2>&1 | tail -20
echo ""

echo "=== Test Verification Complete: $(date) ==="
echo "Review output above for any FAILED tests."