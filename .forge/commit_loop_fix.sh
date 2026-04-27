#!/bin/sh
set -e
git add \
  crates/kay-core/src/loop.rs \
  crates/kay-core/tests/loop.rs \
  crates/kay-core/tests/loop_dispatcher_integration.rs \
  crates/kay-core/tests/loop_pause_tool_call_buffered.rs \
  crates/kay-core/tests/loop_property.rs \
  crates/kay-core/tests/loop_sage_query_integration.rs \
  crates/kay-core/tests/rework_loop.rs

git commit -m "fix(ci): fix rework_loop test failures in run_with_rework

Two bugs in handle_model_event / run_with_rework:

1. When event_tx receiver is dropped (test setups without a full sink),
   event_tx.send(ev) fails and returned ExitCompleted unconditionally,
   bypassing the lifecycle gates (terminates_turn / fail_reason).
   Fix: after send failure, still honour ExitVerified / ExitVerificationFailed.

2. run_with_rework did not check verifier_config.mode == Disabled.
   test_verifier_disabled_skips_rework sends a Fail TaskComplete but
   expects Verified because the verifier is opted out.
   Fix: when verifier_disabled, treat ExitVerificationFailed as Verified.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
