#!/bin/sh
set -e
cd "$(git rev-parse --show-toplevel)"
git add Cargo.lock crates/kay-verifier/src/verifier.rs crates/kay-verifier/tests/cost_ceiling.rs crates/kay-verifier/tests/event_order.rs
git commit -m "feat(verifier): W-6 GREEN — MultiPerspectiveVerifier cost ceiling + event ordering

Implement MultiPerspectiveVerifier::verify() with real critic calls:
- Pre-flight cost ceiling check BEFORE each critic (VERIFY-03)
- Post-critic check: emit VerifierDisabled + return Pass on breach
- Accumulate cost_usd from AgentEvent::Usage SSE events
- Emit AgentEvent::Verification per critic with role/verdict/reason/cost
- Benchmark mode: TestEngineer -> QAEngineer -> EndUser (3 critics)
- Interactive mode: EndUser only (1 critic)

Fix W-6 integration tests (cost_ceiling.rs + event_order.rs):
- Add Allowlist::from_models to OpenRouterProvider builder
- Include 'cost' field in SSE usage chunk via serde_json::json!
- Remove nonexistent mockito .expect_at_least() calls

All 28 kay-verifier tests pass (T1-T4, T2-02a..e, T2-03a..b).

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_DONE"
