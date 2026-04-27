#!/bin/sh
set -e

echo "=== M12 Verification: Running all test suites ==="
echo ""

# Step 1: kay-core inline unit tests
echo "Step 1: cargo test -p kay-core"
cargo test -p kay-core 2>&1 | tail -20
echo "---"

# Step 2: kay-context inline unit tests
echo "Step 2: cargo test -p kay-context"
cargo test -p kay-context 2>&1 | tail -20
echo "---"

# Step 3: kay-tools integration tests
echo "Step 3: cargo test -p kay-tools"
cargo test -p kay-tools 2>&1 | tail -30
echo "---"

# Step 4: kay-verifier integration tests
echo "Step 4: cargo test -p kay-verifier"
cargo test -p kay-verifier 2>&1 | tail -20
echo "---"

# Step 5: kay-session integration tests
echo "Step 5: cargo test -p kay-session"
cargo test -p kay-session 2>&1 | tail -20
echo "---"

# Step 6: kay-sandbox-linux tests (skip on macOS)
if [ "$(uname -s)" = "Linux" ]; then
    echo "Step 6: cargo test -p kay-sandbox-linux"
    cargo test -p kay-sandbox-linux 2>&1 | tail -20
    echo "---"
else
    echo "Step 6: SKIPPED (not on Linux)"
fi

# Step 7: kay-sandbox-macos tests (skip on non-macOS)
if [ "$(uname -s)" = "Darwin" ]; then
    echo "Step 7: cargo test -p kay-sandbox-macos"
    cargo test -p kay-sandbox-macos 2>&1 | tail -20
    echo "---"
else
    echo "Step 7: SKIPPED (not on macOS)"
fi

# Step 8: kay-sandbox-windows tests (skip on non-Windows)
if [ "$(uname -s)" = "Windows" ] || echo "$OSTYPE" | grep -q "msys\|cygwin"; then
    echo "Step 8: cargo test -p kay-sandbox-windows"
    cargo test -p kay-sandbox-windows 2>&1 | tail -20
    echo "---"
else
    echo "Step 8: SKIPPED (not on Windows)"
fi

# Step 9: kay-provider-openrouter tests
echo "Step 9: cargo test -p kay-provider-openrouter"
cargo test -p kay-provider-openrouter 2>&1 | tail -20
echo "---"

# Step 10: kay-cli tests (including live smoke if key available)
echo "Step 10: cargo test -p kay-cli"
cargo test -p kay-cli 2>&1 | tail -30
echo "---"

echo "=== M12 Verification Complete ==="