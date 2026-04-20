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
    /// Verify a task-completion summary. Phase 3 NoOp returns Pending.
    /// Phase 8 signature gains &Transcript arg (03-RESEARCH §8 rec c).
    async fn verify(&self, task_summary: &str) -> VerificationOutcome;
}

pub struct NoOpVerifier;

#[async_trait::async_trait]
impl TaskVerifier for NoOpVerifier {
    async fn verify(&self, _task_summary: &str) -> VerificationOutcome {
        VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".into(),
        }
    }
}
