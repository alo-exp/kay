//! Tauri IPC command handlers.
//!
//! Three commands — `start_session`, `stop_session`, `get_session_status` —
//! are the complete Phase 9 IPC surface. Phase 10 adds settings, model picker,
//! and OS keychain binding.
//!
//! ## Specta v2 setup
//!
//! The specta `Builder` is constructed here (same crate as `#[tauri::command]`
//! functions, so `__cmd__` macro visibility works).  `main.rs` calls
//! `specta_builder()` to get the builder and call `.export()` / `.invoke_handler()`.

use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tauri_specta::Builder;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_tools::AgentEvent;

use crate::agent_loop::run_agent_loop;
use crate::flush::flush_task;
use crate::ipc_event::IpcAgentEvent;
use crate::state::AppState;

// ── Specta builder (same crate as #[tauri::command] so __cmd__ macros resolve) ──

/// Constructs the tauri-specta `Builder` for Phase 9 IPC commands.
/// Call this from `main.rs` to export TypeScript bindings and register handlers.
///
/// ## Why here instead of main.rs?
///
/// `#[tauri::command]` generates `macro_rules! __cmd__<name>` inside this module.
/// `collect_commands!` needs to reference those macros, which only works when
/// both are in the same crate.
pub fn specta_builder<R: tauri::Runtime>() -> Builder<R> {
    Builder::<R>::new().commands(tauri_specta::collect_commands![
        start_session,
        stop_session,
        get_session_status,
    ])
}

// ── Commands (annotated with #[tauri::command] + #[specta::specta]) ─────────

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
        true => Ok(SessionStatus::Running),
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