#!/bin/sh
set -e
git stash push -m "temp: Cargo.lock" Cargo.lock || true
git checkout main
git pull --ff-only origin main
git add .planning/STATE.md
git commit -m "chore(state): advance to Phase 9 — Phase 8 merged (PR #17, b21897a2)

MultiPerspectiveVerifier (3 KIRA critics), run_with_rework loop, cost
ceiling + VerifierDisabled event, kay-verifier crate. All CI checks green.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin main
echo "STATE_PUSHED_OK"
