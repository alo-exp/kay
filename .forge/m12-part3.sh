#!/bin/sh
set -e

# Step 8: kay-cli (E2E + live smoke)
echo "=== Step 8: cargo test -p kay-cli ==="
cargo test -p kay-cli 2>&1 | tail -30

echo "=== M12 Part 3 Complete ==="