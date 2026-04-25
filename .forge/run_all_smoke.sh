#!/bin/sh
set -e
echo "=== kay-core ==="
cargo test -p kay-core 2>&1 | tail -5
echo "=== kay-tools ==="
cargo test -p kay-tools 2>&1 | tail -5
echo "=== kay-context ==="
cargo test -p kay-context 2>&1 | tail -5
echo "=== kay-verifier ==="
cargo test -p kay-verifier 2>&1 | tail -5
echo "=== kay-session ==="
cargo test -p kay-session 2>&1 | tail -5
echo "=== kay-sandbox-linux ==="
cargo test -p kay-sandbox-linux 2>&1 | tail -5
echo "=== kay-sandbox-macos ==="
cargo test -p kay-sandbox-macos 2>&1 | tail -5
echo "=== kay-provider-openrouter ==="
cargo test -p kay-provider-openrouter 2>&1 | tail -5
echo "=== kay-provider-errors ==="
cargo test -p kay-provider-errors 2>&1 | tail -5
echo "=== kay-tauri ==="
cargo test -p kay-tauri 2>&1 | tail -5
echo "=== kay-tui ==="
cargo test -p kay-tui 2>&1 | tail -5
echo "ALL_SMOKE_DONE"
