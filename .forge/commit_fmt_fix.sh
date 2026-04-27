#!/bin/sh
set -e
git add \
  crates/kay-verifier/src/critic.rs \
  crates/kay-verifier/src/verifier.rs \
  crates/kay-verifier/tests/cost_ceiling.rs \
  crates/kay-verifier/tests/event_order.rs \
  crates/kay-cli/tests/context_smoke.rs

git commit -m "fix(ci): apply stable rustfmt to Phase 8 verifier files

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
