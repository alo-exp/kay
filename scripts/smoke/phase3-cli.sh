#!/usr/bin/env bash
# Phase 3 smoke test — CLI happy-path (S-01 per 03-TEST-STRATEGY §2.5).
#
# Proves `kay tools list` enumerates the 7 Phase-3 tools with hardened
# descriptions as a post-build sanity check. Exit 0 == pass.
#
# REQs exercised: TOOL-02, TOOL-05, TOOL-06 (native tools enumerated,
# hardened descriptions observable by a downstream consumer).
#
# Budget: < 30s on a warm cargo cache.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "[phase3-cli] building kay-cli..."
cargo build -p kay-cli --quiet

OUTPUT="$(cargo run -p kay-cli --quiet -- tools list 2>/dev/null)"

EXPECTED_TOOLS=(
  "execute_commands"
  "task_complete"
  "image_read"
  "fs_read"
  "fs_write"
  "fs_search"
  "net_fetch"
)

fail=0
for tool in "${EXPECTED_TOOLS[@]}"; do
  if ! grep -qE "^${tool}\b" <<<"$OUTPUT"; then
    echo "[phase3-cli] FAIL: expected tool '$tool' missing from output"
    fail=1
  fi
done

# Count = exactly 7 tools
line_count="$(grep -cE '^[a-z_]+\t' <<<"$OUTPUT" || true)"
if [ "$line_count" -ne 7 ]; then
  echo "[phase3-cli] FAIL: expected 7 tools, got $line_count"
  fail=1
fi

# Hardened descriptions: image_read must mention caps; net_fetch must mention truncation
if ! grep -q "image_read.*cap" <<<"$OUTPUT"; then
  echo "[phase3-cli] FAIL: image_read description missing cap reminder"
  fail=1
fi
if ! grep -qiE "net_fetch.*truncat" <<<"$OUTPUT"; then
  echo "[phase3-cli] FAIL: net_fetch description missing truncation reminder"
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  echo "--- output ---"
  echo "$OUTPUT"
  echo "--- end ---"
  exit 1
fi

echo "[phase3-cli] PASS: 7 tools enumerated with hardened descriptions"
exit 0
