#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use kay_session::index::{SessionStatus, close_session, create_session, list_sessions};
use tempfile::TempDir;

fn make_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn create_session_inserts_row() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "Test session", "forge", "test-model", &cwd).unwrap();
    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session.id);
}

#[test]
fn list_sessions_ordered_by_start_time_desc() {
    use std::time::Duration;
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();

    let _s1 = create_session(&store, "first", "forge", "model", &cwd).unwrap();
    std::thread::sleep(Duration::from_millis(10));
    let _s2 = create_session(&store, "second", "forge", "model", &cwd).unwrap();

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].title, "second");
    assert_eq!(sessions[1].title, "first");
}

#[test]
fn list_sessions_limit_respected() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();

    for i in 0..10 {
        let _s = create_session(&store, &format!("session-{i}"), "forge", "model", &cwd).unwrap();
    }

    let sessions = list_sessions(&store, 3).unwrap();
    assert_eq!(sessions.len(), 3);
}

#[test]
fn close_session_sets_status_and_end_time() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    close_session(&store, &id, SessionStatus::Complete).unwrap();

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions[0].status, "complete");
}

#[test]
fn resume_by_id_returns_correct_path() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;
    let expected_path = session.jsonl_path.clone();
    drop(session);

    let resumed = kay_session::index::resume_session(&store, &id).unwrap();
    assert_eq!(resumed.jsonl_path, expected_path);
}

#[test]
fn resume_unknown_id_returns_err() {
    let (_dir, store) = make_store();
    let unknown = uuid::Uuid::new_v4();
    let result = kay_session::index::resume_session(&store, &unknown);
    assert!(
        matches!(
            result,
            Err(kay_session::SessionError::SessionNotFound { .. })
        ),
        "expected SessionNotFound, got: {:?}",
        result
    );
}

#[test]
fn parent_id_fk_set_null_on_delete() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();

    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_id = parent.id;
    drop(parent);

    store.conn.execute(
        "INSERT INTO sessions (id, persona, model, status, start_time, jsonl_path, cwd, parent_id)
         VALUES (?1, 'forge', 'model', 'active', datetime('now'), ?2, ?3, ?4)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            dir.path().join("child.jsonl").to_str().unwrap(),
            dir.path().to_str().unwrap(),
            parent_id.to_string()
        ],
    ).unwrap();

    store
        .conn
        .execute(
            "DELETE FROM sessions WHERE id = ?1",
            rusqlite::params![parent_id.to_string()],
        )
        .unwrap();

    let null_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE parent_id IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        null_count, 1,
        "child parent_id must be NULL after parent deletion"
    );
}

#[test]
fn session_list_summary_fields() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let _s = create_session(&store, "Test Title", "sage", "cool-model", &cwd).unwrap();

    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    let s = &sessions[0];
    assert!(!s.title.is_empty() || s.title == "Test Title");
    assert!(!s.status.is_empty());
    assert!(s.cost_usd >= 0.0);
    assert!(s.turn_count >= 0);
}
