#!/bin/bash
set -e

WORKSPACE="/Users/shafqat/Documents/Projects/opencode/vs-others"
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

echo "========================================="
echo "Verifying 9 Wave 2 batch 2 crates"
echo "========================================="
echo ""

for crate in "${CRATES[@]}"; do
    echo "[CHECK] $crate..."
    if cd "$WORKSPACE" && cargo check -p "$crate" --tests 2>&1; then
        echo "[PASS]  $crate"
    else
        echo "[FAIL]  $crate - fixing test file..."
        # Fix the test file by examining and correcting the issue
        TEST_FILE="$WORKSPACE/crates/$crate/tests/$crate.rs"
        if [ -f "$TEST_FILE" ]; then
            echo "       Found test file at: $TEST_FILE"
            # Run cargo check again to see detailed errors
            cargo check -p "$crate" --tests 2>&1 | head -100
        else
            echo "       No test file found at: $TEST_FILE"
        fi
    fi
    echo ""
done

echo "========================================="
echo "Verification complete"
echo "========================================="
