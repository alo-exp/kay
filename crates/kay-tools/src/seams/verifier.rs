//! TaskVerifier DI seam (D-06). NoOp in Phase 3; Phase 8 swaps real impl.
//! VerificationOutcome is OWNED by kay-tools (B2/VAL-002) and lives
//! alongside the trait it belongs to. kay-tools does NOT depend on the
//! OpenRouter provider crate; the scaffold owns both the outcome type
//! and the trait. Per E1 four-module layout, all trait-based seams
//! (their types + impls) live in the `seams` module — events.rs owns
//! only AgentEvent.

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationOutcome {
    Pending { reason: String },
    Pass { note: String },
    Fail { reason: String },
}

#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    /// Verify a task-completion summary.
    /// `task_context`: loop-assembled summary of tool calls + outputs this turn.
    /// Empty string if unavailable (e.g., NoOpVerifier stub).
    async fn verify(&self, task_summary: &str, task_context: &str) -> VerificationOutcome;
}

pub struct NoOpVerifier;

#[async_trait::async_trait]
impl TaskVerifier for NoOpVerifier {
    async fn verify(&self, _task_summary: &str, _task_context: &str) -> VerificationOutcome {
        VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_verifier_returns_pending() {
        let v = NoOpVerifier;
        let outcome = v.verify("I finished the task", "").await;
        match outcome {
            VerificationOutcome::Pending { reason } => {
                assert!(
                    reason.contains("Phase 8"),
                    "Pending reason must mention Phase 8: {reason}"
                );
            }
            other => panic!("expected Pending, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn noop_verifier_never_returns_pass() {
        // Invariant T-3-06 (Threat #7 in 03-RESEARCH): Phase 3 NoOp MUST NOT
        // produce Pass — only Pending. Real verifier swapped in Phase 8.
        let v = NoOpVerifier;
        let outcome = v.verify("done", "").await;
        assert!(
            !matches!(outcome, VerificationOutcome::Pass { .. }),
            "NoOpVerifier must never emit Pass (Threat T-3-06)"
        );
    }

    #[tokio::test]
    async fn noop_verifier_never_returns_fail() {
        // Symmetric to the Pass invariant: Phase 3 stub must remain Pending-only.
        let v = NoOpVerifier;
        let outcome = v.verify("anything", "").await;
        assert!(
            !matches!(outcome, VerificationOutcome::Fail { .. }),
            "NoOpVerifier must never emit Fail in Phase 3"
        );
    }

    #[tokio::test]
    async fn noop_verifier_accepts_task_context_arg() {
        // Phase 8 expanded signature — will fail to compile until verifier.rs updated (W-3 RED)
        let v = NoOpVerifier;
        let outcome = v.verify("summary", "tool context string").await;
        match outcome {
            VerificationOutcome::Pending { .. } => {}
            other => panic!("expected Pending, got: {other:?}"),
        }
    }
}
