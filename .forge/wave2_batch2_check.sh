#!/bin/bash
# Wave 2 batch 2 crate check script
# Checks all 9 crates: forge_embed, forge_markdown_stream, forge_repo, forge_services,
# forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker

set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

CRATES=(
    "forge_embed"
    "forge_markdown_stream"
    "forge_repo"
    "forge_services"
    "forge_snaps"
    "forge_stream"
    "forge_template"
    "forge_tracker"
    "forge_walker"
)

RESULTS_FILE=".forge/wave2_batch2_results.txt"
> "$RESULTS_FILE"

echo "=== Wave 2 batch 2 Crate Check ===" | tee "$RESULTS_FILE"
echo ""

for crate in "${CRATES[@]}"; do
    echo "Checking $crate..." | tee -a "$RESULTS_FILE"
    if cargo check -p "$crate" --tests 2>&1 | tee -a "$RESULTS_FILE"; then
        echo "✓ $crate: PASS" | tee -a "$RESULTS_FILE"
    else
        echo "✗ $crate: FAIL" | tee -a "$RESULTS_FILE"
    fi
    echo "---" | tee -a "$RESULTS_FILE"
done

echo ""
echo "=== Summary ===" | tee -a "$RESULTS_FILE"
grep -E "^(✓|✗)" "$RESULTS_FILE" | tee -a "$RESULTS_FILE"
