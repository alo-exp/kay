// M12-Task 5 RED: kay-session integration tests for session index operations.
//
// Verifies create_session, list_sessions, close_session, resume_session,
// mark_session_lost round-trip correctly with temp directories.

use kay_session::{create_session, list_sessions, close_session, mark_session_lost,
                  resume_session, SessionStore, SessionStatus};

#[test]
fn create_session_inserts_row_and_opens_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let cwd = std::path::PathBuf::from("/tmp");

    let session = create_session(&store, "Test Session", "forge", "minimax/MiniMax-M2.7", &cwd).unwrap();

    assert!(!session.id.is_nil());
    assert_eq!(session.turn_count, 0);
    assert!(session.jsonl_path.exists());
    drop(session);
    drop(store);
}

#[test]
fn list_sessions_returns_created_session() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let cwd = std::path::PathBuf::from("/tmp");

    let session = create_session(&store, "List Test", "forge", "minimax/MiniMax-M2.7", &cwd).unwrap();
    drop(session);

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, "List Test");
    assert_eq!(sessions[0].status, "active");
    drop(store);
}

#[test]
fn close_session_updates_status() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let cwd = std::path::PathBuf::from("/tmp");

    let session = create_session(&store, "Close Test", "forge", "minimax/MiniMax-M2.7", &cwd).unwrap();
    close_session(&store, &session.id, SessionStatus::Complete).unwrap();
    drop(session);

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].status, "complete");
    drop(store);
}

#[test]
fn resume_session_reopens_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let cwd = std::path::PathBuf::from("/tmp");

    let session = create_session(&store, "Resume Test", "forge", "minimax/MiniMax-M2.7", &cwd).unwrap();
    let id = session.id;
    drop(session);

    let resumed = resume_session(&store, &id).unwrap();
    assert_eq!(resumed.id, id);
    assert_eq!(resumed.turn_count, 0);
    drop(resumed);
    drop(store);
}

#[test]
fn resume_nonexistent_session_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let bogus = uuid::Uuid::new_v4();

    let result = resume_session(&store, &bogus);
    assert!(result.is_err());
    drop(store);
}

#[test]
fn mark_session_lost_updates_status() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let cwd = std::path::PathBuf::from("/tmp");

    let session = create_session(&store, "Lost Test", "forge", "minimax/MiniMax-M2.7", &cwd).unwrap();
    mark_session_lost(&store, &session.id).unwrap();
    drop(session);

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions[0].status, "lost");
    drop(store);
}
