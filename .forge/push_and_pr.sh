#!/bin/sh
set -e
cd "$(git rev-parse --show-toplevel)"
BRANCH=$(git rev-parse --abbrev-ref HEAD)
echo "Branch: $BRANCH"
git push -u origin "$BRANCH" 2>&1
echo "PUSH_DONE"
gh pr create \
  --title "feat: Phase 8 — Multi-Perspective Verification (KIRA Critics)" \
  --body "$(cat <<'PREOF'
## Summary

Implements Phase 8: Multi-Perspective Verification (KIRA Critics) for Kay terminal coding agent.

**Requirements closed:** VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04

**Waves delivered:**
- W-0 through W-5: Crate scaffold, CriticResponse parsing, AgentEvent variants, VerifierMode/Config, MultiPerspectiveVerifier skeleton, TaskCallContext wiring, rework loop in kay-core
- W-6: Real `MultiPerspectiveVerifier::verify()` — pre-flight cost ceiling check, Usage event accumulation, Verification/VerifierDisabled event emission, Benchmark (3 critics) + Interactive (1 critic) modes
- W-7: CLI wiring (`verifier_config` in RunTurnArgs), context_e2e→context_smoke rename, SymbolKind tracing::warn! fix

**Test coverage:** 28 kay-verifier tests pass (T1 unit, T2 integration, T3 dyn-safety, T4 proptest). Cost ceiling tests (T2-02a..e) and event ordering tests (T2-03a..b) are GREEN.

## Test plan
- [ ] CI runs `cargo test -p kay-verifier` — 28 tests pass
- [ ] CI runs `cargo test -p kay-core` — rework loop tests pass  
- [ ] CI runs `cargo build -p kay-cli` — compiles with verifier_config
- [ ] No regressions in other crates

🤖 Generated with [Claude Code](https://claude.com/claude-code)
PREOF
)" \
  --base main 2>&1
echo "PR_DONE"
