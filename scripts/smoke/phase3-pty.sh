#!/usr/bin/env bash
# Phase 3 smoke test — PTY-path sanity check (S-02 per 03-TEST-STRATEGY §2.5).
#
# The full PTY execution path is exercised by the Rust integration tests
# (`pty_integration.rs`) which spawn real ttys. This smoke script's job is
# lightweight: confirm the `kay-cli` binary links the PTY code path (via
# `portable-pty`) so a shipped release can't silently regress the feature
# by dropping the dep.
#
# Budget: < 5s.
#
# REQs exercised: SHELL-02 (PTY fallback present in binary).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "[phase3-pty] building kay-cli..."
cargo build -p kay-cli --quiet

BIN="$REPO_ROOT/target/debug/kay-cli"
if [ ! -x "$BIN" ]; then
  echo "[phase3-pty] FAIL: kay-cli binary not found at $BIN"
  exit 1
fi

# Proof-of-link: the kay-tools crate depends on portable-pty per Cargo.toml,
# so the compiled artifact must contain symbols referencing the portable_pty
# crate. `strings` is POSIX-portable; `nm` is an acceptable fallback.
if command -v strings >/dev/null 2>&1; then
  # Capture to tmpfile to avoid SIGPIPE-under-pipefail flakiness on large bins.
  tmp_syms="$(mktemp)"
  strings "$BIN" > "$tmp_syms" 2>/dev/null || true
  if ! grep -qi "portable_pty" "$tmp_syms"; then
    echo "[phase3-pty] FAIL: portable_pty symbols missing from kay-cli binary"
    rm -f "$tmp_syms"
    exit 1
  fi
  rm -f "$tmp_syms"
else
  echo "[phase3-pty] WARN: strings unavailable; skipping symbol check"
fi

# Execute a no-op `kay tools list` to confirm the binary runs to completion
# without PTY-related link errors on the host.
cargo run -p kay-cli --quiet -- tools list >/dev/null 2>&1

echo "[phase3-pty] PASS: kay-cli links portable-pty and runs clean"
exit 0
