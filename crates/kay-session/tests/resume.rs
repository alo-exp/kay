#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use kay_session::index::{SessionStatus, create_session, resume_session};
use kay_tools::AgentEvent;
use kay_tools::events_wire::AgentEventWire;
use tempfile::TempDir;

fn make_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn resume_restores_turn_count() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let mut session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    for i in 0..7 {
        let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
        let wire = AgentEventWire::from(&ev);
        session.append_event(&wire).unwrap();
    }
    drop(session);

    let resumed = resume_session(&store, &id).unwrap();
    assert_eq!(
        resumed.turn_count, 7,
        "resumed session must have turn_count = 7"
    );
}

#[test]
fn resume_appends_after_existing_events() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let mut session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;
    let path = session.jsonl_path.clone();

    for i in 0..7 {
        let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
        let wire = AgentEventWire::from(&ev);
        session.append_event(&wire).unwrap();
    }
    drop(session);

    let mut resumed = resume_session(&store, &id).unwrap();
    for i in 7..10 {
        let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
        let wire = AgentEventWire::from(&ev);
        resumed.append_event(&wire).unwrap();
    }
    drop(resumed);

    let contents = std::fs::read_to_string(&path).unwrap();
    let line_count = contents.lines().count();
    assert_eq!(
        line_count, 10,
        "must have 10 lines total after resume + 3 appends"
    );
}

#[test]
fn resume_partial_write_recovery() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;
    let path = session.jsonl_path.clone();
    drop(session);

    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b"{\"type\":\"text_delta\",\"content\":\"line1\"}\n")
            .unwrap();
        f.write_all(b"{\"type\":\"text_delta\",\"content\":\"line2\"}\n")
            .unwrap();
        f.write_all(b"{\"type\":\"text_").unwrap();
    }

    let resumed = resume_session(&store, &id).unwrap();
    assert_eq!(
        resumed.turn_count, 2,
        "partial line must be truncated; turn_count = 2"
    );
}

#[test]
fn resume_emits_session_resumed_synthetic_event() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;
    let path = session.jsonl_path.clone();
    drop(session);

    let mut resumed = resume_session(&store, &id).unwrap();
    let ev = AgentEvent::TextDelta { content: "SESSION_RESUMED_SYNTHETIC".into() };
    let wire = AgentEventWire::from(&ev);
    resumed.append_event(&wire).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.contains("SESSION_RESUMED_SYNTHETIC"),
        "resumed session must accept new events"
    );
}

#[test]
fn resume_updates_status_to_active() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    kay_session::index::close_session(&store, &id, SessionStatus::Complete).unwrap();

    let _resumed = resume_session(&store, &id).unwrap();

    let status: String = store
        .conn
        .query_row(
            "SELECT status FROM sessions WHERE id = ?1",
            rusqlite::params![id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "active", "resume must set status = 'active'");
}
