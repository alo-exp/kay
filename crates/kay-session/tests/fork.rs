#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use kay_session::fork::fork_session;
use kay_session::index::{create_session, list_sessions};
use kay_tools::AgentEvent;
use kay_tools::events_wire::AgentEventWire;
use tempfile::TempDir;

fn make_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn fork_sets_parent_id() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_id = parent.id;
    drop(parent);

    let child = fork_session(&store, &parent_id).unwrap();
    let child_id = child.id;
    drop(child);

    let stored_parent: Option<String> = store
        .conn
        .query_row(
            "SELECT parent_id FROM sessions WHERE id = ?1",
            rusqlite::params![child_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        stored_parent,
        Some(parent_id.to_string()),
        "child parent_id must equal parent's UUID"
    );
}

#[test]
fn fork_creates_independent_jsonl() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_path = parent.jsonl_path.clone();
    let parent_id = parent.id;
    drop(parent);

    let child = fork_session(&store, &parent_id).unwrap();
    assert_ne!(
        child.jsonl_path, parent_path,
        "child must have its own transcript path"
    );
}

#[test]
fn fork_inherits_persona_and_model() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "sage", "claude-3-7", &cwd).unwrap();
    let parent_id = parent.id;
    drop(parent);

    let _child = fork_session(&store, &parent_id).unwrap();

    let (persona, model): (String, String) = {
        let mut stmt = store
            .conn
            .prepare("SELECT persona, model FROM sessions WHERE parent_id = ?1")
            .unwrap();
        stmt.query_row(rusqlite::params![parent_id.to_string()], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap()
    };
    assert_eq!(persona, "sage");
    assert_eq!(model, "claude-3-7");
}

#[test]
fn fork_child_status_is_active() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_id = parent.id;
    kay_session::index::close_session(
        &store,
        &parent_id,
        kay_session::index::SessionStatus::Complete,
    )
    .unwrap();

    let _child = fork_session(&store, &parent_id).unwrap();

    let status: String = {
        let mut stmt = store
            .conn
            .prepare("SELECT status FROM sessions WHERE parent_id = ?1")
            .unwrap();
        stmt.query_row(rusqlite::params![parent_id.to_string()], |r| r.get(0))
            .unwrap()
    };
    assert_eq!(status, "active", "forked child must have status = 'active'");
}

#[test]
fn fork_parent_deletion_sets_null() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_id = parent.id;
    drop(parent);

    let child = fork_session(&store, &parent_id).unwrap();
    let child_id = child.id;
    drop(child);

    store
        .conn
        .execute(
            "DELETE FROM sessions WHERE id = ?1",
            rusqlite::params![parent_id.to_string()],
        )
        .unwrap();

    let parent_id_col: Option<String> = store
        .conn
        .query_row(
            "SELECT parent_id FROM sessions WHERE id = ?1",
            rusqlite::params![child_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        parent_id_col.is_none(),
        "child parent_id must be NULL after parent deletion"
    );
}

#[test]
fn fork_preserves_turn_count_independence() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let parent = create_session(&store, "parent", "forge", "model", &cwd).unwrap();
    let parent_id = parent.id;
    drop(parent);

    let mut child = fork_session(&store, &parent_id).unwrap();

    for i in 0..3 {
        let ev = AgentEvent::TextDelta { content: format!("child-event-{i}") };
        let wire = AgentEventWire::from(&ev);
        child.append_event(&wire).unwrap();
    }
    drop(child);

    let parent_turns: i64 = store
        .conn
        .query_row(
            "SELECT turn_count FROM sessions WHERE id = ?1",
            rusqlite::params![parent_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        parent_turns, 0,
        "parent turn_count must be unaffected by child appends"
    );
}
