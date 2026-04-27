#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Kay Test Suite (nextest) ==="
echo "Started: $(date)"
echo ""

# Batch 1: Core crates
echo "[1/6] kay-core..."
cargo nextest run -p kay-core 2>&1 | tail -30
echo "---"

echo "[2/6] kay-context..."
cargo nextest run -p kay-context 2>&1 | tail -30
echo "---"

echo "[3/6] kay-tools..."
cargo nextest run -p kay-tools 2>&1 | tail -30
echo "---"

echo "[4/6] kay-verifier..."
cargo nextest run -p kay-verifier 2>&1 | tail -30
echo "---"

echo "[5/6] kay-session..."
cargo nextest run -p kay-session 2>&1 | tail -30
echo "---"

echo "[6/6] kay-provider-openrouter..."
cargo nextest run -p kay-provider-openrouter 2>&1 | tail -30
echo "---"

echo "=== Complete: $(date) ==="