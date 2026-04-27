#!/bin/sh
set -e
cd "$(dirname "$0")/.."

cargo check -p kay-core --tests 2>&1 | grep -E "^error" && exit 1 || true

git add \
  crates/kay-core/tests/loop.rs \
  crates/kay-core/tests/loop_property.rs \
  crates/kay-core/tests/loop_dispatcher_integration.rs \
  crates/kay-core/tests/loop_sage_query_integration.rs \
  crates/kay-core/tests/loop_pause_tool_call_buffered.rs \
  crates/kay-core/tests/rework_loop.rs

git commit -m "fix(kay-core/tests): add verifier_config to RunTurnArgs in all test harnesses

W-5 GREEN added verifier_config: VerifierConfig to RunTurnArgs.
All existing test harnesses need the new field. All non-verification
tests get Disabled mode (no cost, no retries, no interference).
Also removes unused TurnResult import (loop.rs) and fixes
drop-of-reference warning (rework_loop.rs).

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "=== Committed ==="
git log --oneline -3
