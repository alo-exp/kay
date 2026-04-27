#!/bin/sh
set -e
cd "$(git rev-parse --show-toplevel)"
rm -f crates/kay-cli/tests/context_e2e.rs
echo "=== kay-cli check ==="
cargo check -p kay-cli --tests 2>&1 | tail -5
echo "=== kay-context check ==="
cargo check -p kay-context 2>&1 | tail -5
echo "ALL_CHECKS_DONE"
