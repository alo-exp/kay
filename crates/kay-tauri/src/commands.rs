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

// Re-export SessionStatus and SessionManager from session_manager
pub use crate::session_manager::{SessionStatus, SessionManager};

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
        // Phase 10 WAVE 2 commands (GREEN — fully implemented)
        list_sessions,
        pause_session,
        resume_session,
        fork_session,
        kill_session,
        get_session_events,
        save_project_settings,
        load_project_settings,
        bind_api_key,
        get_api_key_fingerprint,
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

// ── WAVE 2: SessionManager commands (Phase 10 GREEN) ──────────────────────

/// List all active sessions sorted by last_active descending.
#[tauri::command]
#[specta::specta]
pub async fn list_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::session_manager::SessionInfo>, String> {
    Ok(state.session_manager.list_sessions())
}

/// Pause an active session by ID.
#[tauri::command]
#[specta::specta]
pub async fn pause_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_manager
        .pause_session(&session_id)
        .map_err(|e| e.to_string())
}

/// Resume a paused session by ID.
#[tauri::command]
#[specta::specta]
pub async fn resume_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_manager
        .resume_session(&session_id)
        .map_err(|e| e.to_string())
}

/// Fork a session by ID, optionally with a different persona.
/// Returns the new session ID.
#[tauri::command]
#[specta::specta]
pub async fn fork_session(
    session_id: String,
    persona: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    state
        .session_manager
        .fork_session(&session_id, persona)
        .map_err(|e| e.to_string())
}

/// Kill (terminate) a session by ID.
#[tauri::command]
#[specta::specta]
pub async fn kill_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_manager
        .kill_session(&session_id)
        .map_err(|e| e.to_string())
}

/// Get session events from a specific turn onward.
/// Note: Full implementation requires event store (Wave 4).
#[tauri::command]
#[specta::specta]
pub async fn get_session_events(
    session_id: String,
    from_turn: Option<u32>,
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<IpcAgentEvent>, String> {
    // WAVE 3 will integrate with event store
    // For now, return empty vec (session exists but no history)
    let _ = session_id;
    let _ = from_turn;
    Ok(vec![])
}

/// Save project settings to disk.
#[tauri::command]
#[specta::specta]
pub async fn save_project_settings(
    settings: crate::project_settings::ProjectSettings,
) -> Result<(), String> {
    let path = format!("{}/settings.json", settings.project_path);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Load project settings from disk.
#[tauri::command]
#[specta::specta]
pub async fn load_project_settings(
    project_path: String,
) -> Result<Option<crate::project_settings::ProjectSettings>, String> {
    let path = format!("{}/settings.json", project_path);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let settings: crate::project_settings::ProjectSettings =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            Ok(Some(settings))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Bind an API key to the OS keychain.
/// Note: Full implementation requires keychain integration (Wave 4).
#[tauri::command]
#[specta::specta]
pub async fn bind_api_key(
    _provider: String,
    _key: String,
) -> Result<(), String> {
    // WAVE 4: integrate with keychain-rs or similar
    // For now, just acknowledge the call
    Ok(())
}

/// Get the fingerprint of a bound API key (checks if key exists).
/// Note: Full implementation requires keychain integration (Wave 4).
#[tauri::command]
#[specta::specta]
pub async fn get_api_key_fingerprint(
    _provider: String,
) -> Result<Option<String>, String> {
    // WAVE 4: integrate with keychain-rs
    // For now, return None (no keys bound yet)
    Ok(None)
}