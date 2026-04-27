#!/bin/sh
set -e
cd "$(git rev-parse --show-toplevel)"
git add crates/kay-cli/src/run.rs
git add crates/kay-cli/Cargo.toml
git add crates/kay-cli/tests/context_smoke.rs
git rm --cached crates/kay-cli/tests/context_e2e.rs 2>/dev/null || true
git add crates/kay-context/src/store.rs
git commit -m "feat(cli): W-7 GREEN — wire verifier_config + Phase 8 backlog items

- run.rs: add verifier_config: Default::default() to RunTurnArgs (fixes
  missing field compile error introduced by W-5 RunTurnArgs expansion)
- Cargo.toml: add kay-verifier as dev-dependency for context_smoke.rs
- context_e2e.rs → context_smoke.rs: rename per Phase 999.6 backlog;
  add Phase 8 smoke test verifying VerifierConfig::default() is Interactive
- store.rs: SymbolKind::from_kind_str unknown arm emits tracing::warn!
  per Phase 999.7 backlog (silent fallback was hiding schema drift)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "W7_COMMIT_DONE"
git log --oneline -3
