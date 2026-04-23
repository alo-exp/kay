//! Tauri IPC command handlers.
//!
//! Three commands — `start_session`, `stop_session`, `get_session_status` —
//! are the complete Phase 9 IPC surface. Phase 10 adds settings, model picker,
//! and OS keychain binding.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use kay_core::control::{ControlMsg, control_channel};
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::{AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext, ToolRegistry};

use crate::flush::flush_task;
use crate::ipc_event::IpcAgentEvent;
use crate::state::AppState;

/// Start a new agent session.
///
/// Returns the session UUID on success. Events stream to `channel` via the
/// 16ms flush task. Cancel via `stop_session(session_id)`.
#[tauri::command]
#[specta::specta]
pub async fn start_session(
    prompt: String,
    persona: String,
    channel: Channel<IpcAgentEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let token = CancellationToken::new();
    state.sessions.insert(session_id.clone(), token.clone());

    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(1024);

    tokio::spawn(flush_task(event_rx, channel));
    tokio::spawn(run_agent_loop(prompt, persona, session_id.clone(), event_tx, token));

    Ok(session_id)
}

/// Cancel an active session.
#[tauri::command]
#[specta::specta]
pub async fn stop_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some((_, token)) = state.sessions.remove(&session_id) {
        token.cancel();
    }
    Ok(())
}

/// Query whether a session is still running.
#[tauri::command]
#[specta::specta]
pub async fn get_session_status(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<SessionStatus, String> {
    match state.sessions.contains_key(&session_id) {
        true  => Ok(SessionStatus::Running),
        false => Ok(SessionStatus::Complete),
    }
}

/// Phase 9 session status — Running or Complete.
/// Phase 10 will add Aborted with a reason field.
#[derive(Debug, Clone, Serialize, Type)]
pub enum SessionStatus {
    Running,
    Complete,
}

// ── Agent loop wiring ────────────────────────────────────────────────────────

/// Drive a single agent turn in a background task.
///
/// Phase 9 uses the offline provider for the agent loop (no API key management
/// UI yet — that lands in Phase 10). The IPC plumbing, flush task, React UI,
/// and memory canary are all fully exercised with the offline provider.
async fn run_agent_loop(
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
        Arc::new(NullServices),
        Arc::new(|_ev: AgentEvent| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
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

    let _ = run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: prompt,
        verifier_config: Default::default(),
    })
    .await;
}

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
