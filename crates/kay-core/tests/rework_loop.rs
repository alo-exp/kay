//! W-5 RED: Integration tests for the re-work loop.
//!
//! T2-01a through T2-01f: tests for bounded retry loop behavior.
//! T4-02: proptest for retry counter invariant.
//!
//! These tests use todo!() stubs that will compile but fail at runtime.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use kay_core::control::{ControlMsg, control_channel};
use kay_core::r#loop::{RunTurnArgs, run_with_rework, TurnResult};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, ServicesHandle, ToolCallContext,
    ToolRegistry, VerificationOutcome,
};
use kay_verifier::{VerifierConfig, VerifierMode};

/// CycleVerifier: toggles between Fail (first N invocations) and Pass.
/// Used to test retry behavior.
struct CycleVerifier {
    fail_count: usize,
    call_count: Arc<Mutex<usize>>,
}

impl CycleVerifier {
    fn new(fail_count: usize) -> Self {
        Self {
            fail_count,
            call_count: Arc::new(Mutex::new(0)),
        }
    }
    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl kay_tools::seams::verifier::TaskVerifier for CycleVerifier {
    async fn verify(
        &self,
        _task_summary: &str,
        _task_context: &str,
    ) -> VerificationOutcome {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current <= self.fail_count {
            VerificationOutcome::Fail {
                reason: format!("CycleVerifier: failing call {}/{}", current, self.fail_count),
            }
        } else {
            VerificationOutcome::Pass {
                note: "CycleVerifier: passing after fail threshold".into(),
            }
        }
    }
}

/// NullVerifier: always passes (for tests that don't need verification to fail).
struct NullVerifier;

#[async_trait::async_trait]
impl kay_tools::seams::verifier::TaskVerifier for NullVerifier {
    async fn verify(&self, _: &str, _: &str) -> VerificationOutcome {
        VerificationOutcome::Pass {
            note: "NullVerifier: always passing".into(),
        }
    }
}

/// FailingVerifier: always fails.
struct FailingVerifier;

#[async_trait::async_trait]
impl kay_tools::seams::verifier::TaskVerifier for FailingVerifier {
    async fn verify(&self, _: &str, _: &str) -> VerificationOutcome {
        VerificationOutcome::Fail {
            reason: "FailingVerifier: always failing".into(),
        }
    }
}

struct NullServices;

#[async_trait]
impl ServicesHandle for NullServices {
    async fn fs_read(&self, _: FSRead) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_write(&self, _: FSWrite) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_search(&self, _: FSSearch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn net_fetch(&self, _: NetFetch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
}

/// Build minimal RunTurnArgs for testing. Uses CycleVerifier with configurable fail count.
fn build_test_args(
    model_rx: tokio::sync::mpsc::Receiver<Result<AgentEvent, ProviderError>>,
    verifier: Arc<dyn kay_tools::seams::verifier::TaskVerifier>,
    max_retries: u8,
) -> RunTurnArgs {
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, _) = tokio::sync::mpsc::channel::<AgentEvent>(32);
    let registry = Arc::new(ToolRegistry::new());
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(NoOpSandbox),
        verifier,
        0,
        Arc::new(Mutex::new(String::new())),
    );
    let persona = Persona::load("forge").expect("bundled forge persona loads");

    RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
        verifier_config: VerifierConfig {
            mode: VerifierMode::Interactive,
            max_retries: max_retries.into(),
            cost_ceiling_usd: 1.0,
            model: "openai/gpt-4o-mini".into(),
        },
    }
}

// ───────────────────────────────────────────────────────────────────────
// T2-01a: Verifier returns PASS on first call → TurnResult::Verified
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_pass_terminates_loop() {
    todo!("W-5 RED: test_pass_terminates_loop not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01b: Verifier FAIL once then PASS → Verified after 1 retry
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fail_retry_then_pass() {
    todo!("W-5 RED: test_fail_retry_then_pass not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01c: Verifier FAIL × max_retries → VerifierDisabled emitted
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_max_retries_exhausted() {
    todo!("W-5 RED: test_max_retries_exhausted not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01d: FAIL × (max_retries - 1) then PASS → Verified after retries
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fail_before_max_retries_then_pass() {
    todo!("W-5 RED: test_fail_before_max_retries_then_pass not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01e: Feedback message contains "Verification failed:" + reason
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fail_injects_feedback() {
    todo!("W-5 RED: test_fail_injects_feedback not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01f: max_retries = 0 → no retries; single FAIL → VerificationFailed
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_zero_max_retries_immediate_failure() {
    todo!("W-5 RED: test_zero_max_retries_immediate_failure not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01: Verifier disabled mode skips re-work
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_verifier_disabled_skips_rework() {
    todo!("W-5 RED: test_verifier_disabled_skips_rework not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01: Cost ceiling stops re-work before max_retries exhausted
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cost_ceiling_stops_rework() {
    todo!("W-5 RED: test_cost_ceiling_stops_rework not yet implemented");
}

// ───────────────────────────────────────────────────────────────────────
// T4-02: Proptest — rework_count never exceeds max_retries
// ───────────────────────────────────────────────────────────────────────

proptest::proptest! {
    #[test]
    fn rework_count_never_exceeds_max_retries(max in 0u8..=10u8) {
        // W-5 RED: proptest will be fully implemented in GREEN phase
        // when run_with_rework is working. For now, just verify
        // the test compiles.
        let _ = max;
    }
}
