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

// Re-export SessionStatus from session_manager for backward compatibility
pub use crate::session_manager::SessionStatus;

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
        // Phase 9 commands
        start_session,
        stop_session,
        get_session_status,
        // Phase 10 WAVE 2 commands — added in GREEN wave
        // list_sessions,
        // pause_session,
        // resume_session,
        // fork_session,
        // kill_session,
        // get_session_events,
        // save_project_settings,
        // load_project_settings,
        // bind_api_key,
        // get_api_key_fingerprint,
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
        false => Ok(SessionStatus::Completed),
    }
}

// ── WAVE 2: SessionManager commands (Phase 10) ──────────────────────────────
// Note: These are RED stubs. Real implementations added in GREEN wave.
// Commands are commented out from specta_builder to allow compilation.

/// List all active sessions sorted by last_active descending.
#[tauri::command]
#[specta::specta]
pub async fn list_sessions(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::session_manager::SessionInfo>, String> {
    todo!("WAVE 2 (RED): list_sessions returns todo!()")
}

/// Pause an active session by ID.
#[tauri::command]
#[specta::specta]
pub async fn pause_session(
    _session_id: String,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    todo!("WAVE 2 (RED): pause_session returns todo!()")
}

/// Resume a paused session by ID.
#[tauri::command]
#[specta::specta]
pub async fn resume_session(
    _session_id: String,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    todo!("WAVE 2 (RED): resume_session returns todo!()")
}

/// Fork a session by ID, optionally with a different persona.
/// Returns the new session ID.
#[tauri::command]
#[specta::specta]
pub async fn fork_session(
    _session_id: String,
    _persona: Option<String>,
    _state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    todo!("WAVE 2 (RED): fork_session returns todo!()")
}

/// Kill (terminate) a session by ID.
#[tauri::command]
#[specta::specta]
pub async fn kill_session(
    _session_id: String,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    todo!("WAVE 2 (RED): kill_session returns todo!()")
}

/// Get session events from a specific turn onward.
#[tauri::command]
#[specta::specta]
pub async fn get_session_events(
    _session_id: String,
    _from_turn: Option<u32>,
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<IpcAgentEvent>, String> {
    todo!("WAVE 2 (RED): get_session_events returns todo!()")
}

/// Save project settings to disk.
#[tauri::command]
#[specta::specta]
pub async fn save_project_settings(
    _settings: crate::project_settings::ProjectSettings,
) -> Result<(), String> {
    todo!("WAVE 2 (RED): save_project_settings returns todo!()")
}

/// Load project settings from disk.
#[tauri::command]
#[specta::specta]
pub async fn load_project_settings(
    _project_path: String,
) -> Result<Option<crate::project_settings::ProjectSettings>, String> {
    todo!("WAVE 2 (RED): load_project_settings returns todo!()")
}

/// Bind an API key to the OS keychain.
#[tauri::command]
#[specta::specta]
pub async fn bind_api_key(
    _provider: String,
    _key: String,
) -> Result<(), String> {
    todo!("WAVE 2 (RED): bind_api_key returns todo!()")
}

/// Get the fingerprint of a bound API key (checks if key exists).
#[tauri::command]
#[specta::specta]
pub async fn get_api_key_fingerprint(
    _provider: String,
) -> Result<Option<String>, String> {
    todo!("WAVE 2 (RED): get_api_key_fingerprint returns todo!()")
}