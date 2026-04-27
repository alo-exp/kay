#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Compile-only check (--no-run) ==="
echo "Started: $(date)"

cargo nextest run --workspace --no-run 2>&1

echo "Completed: $(date)"