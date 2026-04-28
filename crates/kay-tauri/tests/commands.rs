//! Integration tests for Tauri IPC commands.
//!
//! These tests exercise the full Tauri command path (IPC + State injection).
//! They are marked #[ignore] because they require a live Tauri app context.
//! Run with:  cargo test -p kay-tauri --test commands -- --ignored
//!
//! For unit-test coverage of the underlying logic, see:
//!   - session_manager::tests (list/pause/resume/fork/kill)
//!   - flush::tests (IPC event translation)
//!   - ipc_event::tests (round-trip serialization)

use kay_tauri::commands::{get_session_status, stop_session};
use kay_tauri::session_manager::SessionStatus;
use kay_tauri::state::AppState;
use tauri::test::{mock_app, mock_builder};
use tauri::Manager;

/// Setup: builds a mock app with AppState registered.
/// The builder MUST outlive the state returned from this function.
/// We return the state as a reference from the app's state registry.
fn state_from_app(app: &tauri::App<tauri::test::MockRuntime>) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

#[tokio::test]
#[ignore = "requires full Tauri app context with registered State"]
async fn stop_nonexistent_session_returns_ok() {
    let _builder = mock_builder();
    let app = mock_app();
    let state = state_from_app(&app);

    // stop_session is idempotent — returns Ok even if the session doesn't exist.
    let result = stop_session("nonexistent-session-id".to_string(), state).await;
    assert!(result.is_ok(), "stop on unknown session should return Ok");
}

#[tokio::test]
#[ignore = "requires full Tauri app context with registered State"]
async fn get_status_of_unknown_session_returns_complete() {
    let _builder = mock_builder();
    let app = mock_app();
    let state = state_from_app(&app);

    // get_session_status checks state.sessions.contains_key().
    // Unknown sessions are not in the map, so they return Completed.
    let status = get_session_status("unknown-session-id".to_string(), state)
        .await
        .unwrap();
    assert_eq!(
        status,
        SessionStatus::Completed,
        "unknown session should report Completed"
    );
}

#[tokio::test]
#[ignore = "requires full Tauri app context with registered State"]
async fn start_and_stop_session_roundtrip() {
    let _builder = mock_builder();
    let app = mock_app();
    let state = state_from_app(&app);

    // Insert an active session directly into the sessions map.
    let session_id = "test-roundtrip-session".to_string();
    let token = tokio_util::sync::CancellationToken::new();
    state.sessions.insert(session_id.clone(), token);

    // Verify it is Running
    let status_running = get_session_status(session_id.clone(), state.clone())
        .await
        .unwrap();
    assert_eq!(status_running, SessionStatus::Running);

    // Stop it — removes the token from state.sessions.
    stop_session(session_id.clone(), state.clone()).await.unwrap();

    // get_session_status now returns Completed (not in sessions map).
    let status_done = get_session_status(session_id, state).await.unwrap();
    assert_eq!(status_done, SessionStatus::Completed);
}
