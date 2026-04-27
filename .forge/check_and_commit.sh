#!/bin/sh
set -e
cd "$(dirname "$0")/.."

echo "=== cargo check kay-core tests ==="
cargo check -p kay-core --tests 2>&1

echo "=== git status ==="
git status --short

echo "=== Committing W-5 test fixes ==="
git add crates/kay-core/tests/loop_property.rs crates/kay-core/tests/loop_pause_tool_call_buffered.rs
git commit -m "fix(kay-core): update test type annotations for TurnResult return type

loop_property.rs and loop_pause_tool_call_buffered.rs used
Result<(), LoopError> in their harness signatures; run_turn now
returns Result<TurnResult, LoopError> after W-5 GREEN.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "=== Done ==="
