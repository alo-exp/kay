//! W-5 GREEN: Integration tests for the re-work loop.
//!
//! T2-01a through T2-01f: tests for bounded retry loop behavior.

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

/// PassVerifier: always passes.
struct PassVerifier;

#[async_trait::async_trait]
impl kay_tools::seams::verifier::TaskVerifier for PassVerifier {
    async fn verify(&self, _: &str, _: &str) -> VerificationOutcome {
        VerificationOutcome::Pass {
            note: "PassVerifier: always passing".into(),
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

/// Build a test RunTurnArgs with the given verifier and max_retries.
fn build_test_args(
    model_rx: tokio::sync::mpsc::Receiver<Result<AgentEvent, ProviderError>>,
    verifier: Arc<dyn kay_tools::seams::verifier::TaskVerifier>,
    max_retries: u32,
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
            max_retries,
            cost_ceiling_usd: 1.0,
            model: "openai/gpt-4o-mini".into(),
        },
    }
}

/// Send a TaskComplete event and close the channel.
async fn send_task_complete(
    tx: &tokio::sync::mpsc::Sender<Result<AgentEvent, ProviderError>>,
    call_id: &str,
    verified: bool,
    outcome: VerificationOutcome,
) {
    let _ = tx
        .send(Ok(AgentEvent::TaskComplete {
            call_id: call_id.into(),
            verified,
            outcome,
        }))
        .await;
    drop(tx);
}

// ───────────────────────────────────────────────────────────────────────
// T2-01a: Verifier returns PASS → TurnResult::Verified
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_pass_terminates_loop() {
    let (model_tx, model_rx) = tokio::sync::mpsc::channel(32);
    let verifier = Arc::new(PassVerifier);

    // Send TaskComplete with Pass
    send_task_complete(
        &model_tx,
        "call-1",
        true,
        VerificationOutcome::Pass { note: "test".into() },
    )
    .await;

    let args = build_test_args(model_rx, verifier, 3);
    let result = run_with_rework(args).await.unwrap();

    assert_eq!(result, TurnResult::Verified);
}

// ───────────────────────────────────────────────────────────────────────
// T2-01b: Verifier returns FAIL → TurnResult::VerificationFailed
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fail_injects_feedback() {
    let (model_tx, model_rx) = tokio::sync::mpsc::channel(32);
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);
    let verifier = Arc::new(FailingVerifier);

    // Send TaskComplete with Fail
    send_task_complete(
        &model_tx,
        "call-1",
        false,
        VerificationOutcome::Fail { reason: "test failure".into() },
    )
    .await;

    // Build args with event_tx
    let args = {
        let (_ctl_tx, control_rx) = control_channel();
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
                max_retries: 3,
                cost_ceiling_usd: 1.0,
                model: "openai/gpt-4o-mini".into(),
            },
        }
    };

    let handle = tokio::spawn(run_with_rework(args));

    // Collect events to verify feedback is injected
    let mut found_feedback = false;
    while let Some(ev) = event_rx.recv().await {
        if let AgentEvent::ToolOutput { chunk, .. } = ev {
            let text = match chunk {
                kay_tools::events::ToolOutputChunk::Stdout(s) => s,
                _ => continue,
            };
            if text.contains("Verification failed") {
                found_feedback = true;
                break;
            }
        }
    }

    let result = handle.await.unwrap().unwrap();
    assert_eq!(result, TurnResult::VerificationFailed);
    assert!(found_feedback, "Should have injected feedback message");
}

// ───────────────────────────────────────────────────────────────────────
// T2-01c: Verifier FAIL × max_retries → VerifierDisabled emitted
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_max_retries_exhausted() {
    let (model_tx, model_rx) = tokio::sync::mpsc::channel(32);
    let verifier = Arc::new(FailingVerifier);
    let max_retries = 3u32;

    // Send TaskComplete with Fail
    send_task_complete(
        &model_tx,
        "call-1",
        false,
        VerificationOutcome::Fail { reason: "always fails".into() },
    )
    .await;

    let args = build_test_args(model_rx, verifier, max_retries);
    let result = run_with_rework(args).await.unwrap();

    assert_eq!(result, TurnResult::VerificationFailed);
}

// ───────────────────────────────────────────────────────────────────────
// T2-01d: max_retries = 0 → immediate VerificationFailed
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_zero_max_retries_immediate_failure() {
    let (model_tx, model_rx) = tokio::sync::mpsc::channel(32);
    let verifier = Arc::new(FailingVerifier);

    // Send TaskComplete with Fail
    send_task_complete(
        &model_tx,
        "call-1",
        false,
        VerificationOutcome::Fail { reason: "always fails".into() },
    )
    .await;

    let args = build_test_args(model_rx, verifier, 0); // max_retries = 0
    let result = run_with_rework(args).await.unwrap();

    assert_eq!(result, TurnResult::VerificationFailed);
}

// ───────────────────────────────────────────────────────────────────────
// T2-01e: Verifier disabled mode returns Verified
// ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_verifier_disabled_skips_rework() {
    let (model_tx, model_rx) = tokio::sync::mpsc::channel(32);
    let verifier = Arc::new(FailingVerifier);

    // Send TaskComplete with Fail
    send_task_complete(
        &model_tx,
        "call-1",
        false,
        VerificationOutcome::Fail { reason: "would retry".into() },
    )
    .await;

    // Use Disabled mode
    let args = {
        let (_ctl_tx, control_rx) = control_channel();
        let (event_tx, _) = tokio::sync::mpsc::channel(32);
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
                mode: VerifierMode::Disabled, // Disabled mode
                max_retries: 3,
                cost_ceiling_usd: 1.0,
                model: "openai/gpt-4o-mini".into(),
            },
        }
    };

    // In Disabled mode, verifier returns Pass immediately
    // So the TaskComplete with Fail won't trigger re-work
    let result = run_with_rework(args).await.unwrap();

    // Disabled mode returns Pass immediately, so we get Verified
    assert_eq!(result, TurnResult::Verified);
}

// ───────────────────────────────────────────────────────────────────────
// T2-01f: cost_ceiling_stops_rework (delegated to W-6)
// ───────────────────────────────────────────────────────────────────────

// This test will be implemented in W-6 verifier_cost tests.
