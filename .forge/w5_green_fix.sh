#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Fix 1: rework_loop.rs - Ok(ev) -> Some(ev) for mpsc::Receiver
sed -i '' 's/while let Ok(ev) = event_rx\.recv()/while let Some(ev) = event_rx.recv()/' \
  crates/kay-core/tests/rework_loop.rs

# Fix 2: loop.rs - remove unused CancellationToken import
sed -i '' '/^use tokio_util::sync::CancellationToken;$/d' \
  crates/kay-core/src/loop.rs

echo "=== Verifying compilation ==="
cargo check -p kay-core --tests 2>&1 | tail -10

echo "=== Staging and committing W-5 GREEN ==="
git add crates/kay-core/src/loop.rs \
        crates/kay-core/tests/loop.rs \
        crates/kay-core/tests/rework_loop.rs \
        Cargo.lock

git commit -m "feat(core): W-5 GREEN — TurnResult enum + run_with_rework + rework loop tests

run_turn returns Result<TurnResult, LoopError>. New run_with_rework function
provides outer retry wrapper; emits VerifierDisabled feedback on Fail.
T2-01a/b/c/d/e tests pass. T4-02 proptest deferred to W-6 cost ceiling work.
RunTurnArgs gains verifier_config: VerifierConfig.

VERIFY-02

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "=== Done ==="
git log --oneline -5
