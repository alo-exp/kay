#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add \
  Cargo.lock \
  crates/kay-cli/src/run.rs \
  crates/kay-core/tests/loop.rs \
  crates/kay-core/tests/loop_dispatcher_integration.rs \
  crates/kay-core/tests/loop_pause_tool_call_buffered.rs \
  crates/kay-core/tests/loop_property.rs \
  crates/kay-core/tests/loop_sage_query_integration.rs \
  crates/kay-tools/src/builtins/sage_query.rs \
  crates/kay-tools/src/builtins/task_complete.rs \
  crates/kay-tools/src/runtime/context.rs \
  crates/kay-tools/tests/sage_query.rs \
  crates/kay-tools/tests/support/mod.rs \
  crates/kay-verifier/src/critic.rs \
  crates/kay-verifier/src/verifier.rs

git commit -m "feat(verifier): W-4 GREEN — ToolCallContext::task_context + task_complete snapshot

T1-06a-c: task_context accumulation + snapshot independence.
T1-08c: task_complete passes snapshot to verifier.

VERIFY-01

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "W-4 GREEN committed: $(git log --oneline -1)"
