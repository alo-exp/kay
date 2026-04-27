#!/bin/sh
set -e

# Step 1: kay-core
echo "=== Step 1: cargo test -p kay-core ==="
cargo test -p kay-core 2>&1 | tail -15

# Step 2: kay-context
echo "=== Step 2: cargo test -p kay-context ==="
cargo test -p kay-context 2>&1 | tail -15

# Step 3: kay-session
echo "=== Step 3: cargo test -p kay-session ==="
cargo test -p kay-session 2>&1 | tail -15

# Step 4: kay-verifier
echo "=== Step 4: cargo test -p kay-verifier ==="
cargo test -p kay-verifier 2>&1 | tail -15

echo "=== M12 Part 1 Complete ==="