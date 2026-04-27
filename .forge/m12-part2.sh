#!/bin/sh
set -e

# Step 5: kay-tools
echo "=== Step 5: cargo test -p kay-tools ==="
cargo test -p kay-tools 2>&1 | tail -15

# Step 6: kay-sandbox-macos
echo "=== Step 6: cargo test -p kay-sandbox-macos ==="
cargo test -p kay-sandbox-macos 2>&1 | tail -15

# Step 7: kay-provider-openrouter
echo "=== Step 7: cargo test -p kay-provider-openrouter ==="
cargo test -p kay-provider-openrouter 2>&1 | tail -15

echo "=== M12 Part 2 Complete ==="