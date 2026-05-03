// agent_loop.rs — Drive a single agent turn in a background task.
//
// Phase 9 uses the offline provider for the agent loop (no API key management
// UI yet — that lands in Phase 10). The IPC plumbing, flush task, React UI,
// and memory canary are all fully exercised with the offline provider.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use kay_core::control::{ControlMsg, control_channel};
use kay_core::r#loop::{RunTurnArgs, run_with_rework};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_verifier::{MultiPerspectiveVerifier, VerifierConfig};
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, ServicesHandle, ToolCallContext,
    ToolRegistry,
};

// ── Public API (needed by main.rs specta builder) ───────────────────────────

pub use self::NullServices as NullServicesHandle;

/// Start the agent loop and return immediately.
/// Caller spawns this asynchronously.
pub async fn run_agent_loop(
    prompt: String,
    persona_name: String,
    _session_id: String,
    event_tx: mpsc::Sender<AgentEvent>,
    cancel: CancellationToken,
) {
    let persona = match Persona::load(&persona_name) {
        Ok(p) => p,
        Err(e) => {
            let _ = event_tx
                .send(AgentEvent::Aborted { reason: format!("persona_error: {e}") })
                .await;
            return;
        }
    };

    let (control_tx, control_rx) = control_channel();
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(256);

    let registry = Arc::new(ToolRegistry::new());
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServicesHandle),
        Arc::new(|_ev: AgentEvent| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(MultiPerspectiveVerifier::new(VerifierConfig::default())),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    // Forward cancellation to the loop's control channel.
    tokio::spawn(async move {
        cancel.cancelled().await;
        let _ = control_tx.send(ControlMsg::Abort).await;
    });

    // Offline echo provider — Phase 10 swaps in OpenRouter transport.
    tokio::spawn(offline_provider(prompt.clone(), model_tx));

    let _ = run_with_rework(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: prompt,
        verifier_config: VerifierConfig::default(),
    })
    .await;
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Echo provider — emits a single `TextDelta` then closes the stream.
/// Phase 10 replaces this with the real OpenRouter transport.
async fn offline_provider(
    prompt: String,
    model_tx: mpsc::Sender<Result<AgentEvent, ProviderError>>,
) {
    let _ = model_tx
        .send(Ok(AgentEvent::TextDelta {
            content: format!("echo: {prompt}"),
        }))
        .await;
    // model_tx drops here → run_turn sees stream close → exits
}

/// No-op services stub — mirrored from `kay-cli/src/run.rs::NullServices`.
pub struct NullServices;

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
