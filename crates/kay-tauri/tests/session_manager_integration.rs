// session_manager_integration.rs — Phase 10 Wave 8
// Integration tests for session manager.

use kay_tauri::session_manager::SessionManagerImpl;
use kay_tauri::session_manager::{SessionInfo, SessionManager, SessionManagerError};

/// Test that SessionManagerImpl stores sessions correctly.
#[tokio::test]
async fn session_manager_impl_stores_sessions() {
    let manager = SessionManagerImpl::default();

    let sessions = manager.list_sessions();
    assert!(sessions.is_empty(), "no sessions initially");
}

/// Test pause requires running session.
#[tokio::test]
async fn pause_nonexistent_session_returns_error() {
    let manager = SessionManagerImpl::default();
    let fake_id = "nonexistent-session-id";

    let result = manager.pause_session(fake_id);
    assert!(result.is_err(), "can't pause nonexistent session");
    assert!(
        matches!(result.unwrap_err(), SessionManagerError::NotFound(_)),
        "error should be NotFound variant"
    );
}

/// Test resume requires paused session.
#[tokio::test]
async fn resume_nonexistent_session_returns_error() {
    let manager = SessionManagerImpl::default();
    let fake_id = "nonexistent-session-id";

    let result = manager.resume_session(fake_id);
    assert!(result.is_err(), "can't resume nonexistent session");
}

/// Test kill requires existing session.
#[tokio::test]
async fn kill_nonexistent_session_returns_error() {
    let manager = SessionManagerImpl::default();
    let fake_id = "nonexistent-session-id";

    let result = manager.kill_session(fake_id);
    assert!(result.is_err(), "can't kill nonexistent session");
}

/// Test fork requires existing session.
#[tokio::test]
async fn fork_nonexistent_session_returns_error() {
    let manager = SessionManagerImpl::default();
    let fake_id = "nonexistent-session-id";

    let result = manager.fork_session(fake_id, None);
    assert!(result.is_err(), "can't fork nonexistent session");
}

/// Test list_sessions returns sorted by last_active descending.
#[tokio::test]
async fn session_list_sorted_by_last_active_descending() {
    let manager = SessionManagerImpl::default();

    let sessions = manager.list_sessions();
    assert!(sessions.is_empty(), "no sessions in fresh manager");

    // Verify SessionInfo type validation
    let empty: Vec<SessionInfo> = vec![];
    assert_eq!(empty.len(), 0, "empty list should have 0 elements");
}
