#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use tempfile::TempDir;

fn open_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn open_creates_db() {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path());
    assert!(store.is_ok());
    assert!(dir.path().join("sessions.db").exists());
}

#[test]
fn open_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let _store1 = SessionStore::open(dir.path()).unwrap();
    let store2 = SessionStore::open(dir.path());
    assert!(store2.is_ok());
}

#[test]
fn open_sets_wal_mode() {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path());
    assert!(store.is_ok(), "open with WAL pragma should succeed");
}

#[test]
fn open_schema_version_mismatch() {
    use rusqlite::Connection;
    use kay_session::error::SessionError;

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("sessions.db");

    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("
        CREATE TABLE schema_version (version INTEGER NOT NULL);
        INSERT INTO schema_version VALUES (99);
    ").unwrap();
    drop(conn);

    let result = SessionStore::open(dir.path());
    assert!(
        matches!(result, Err(SessionError::SchemaVersionMismatch { found: 99, expected: 1 })),
        "expected SchemaVersionMismatch, got: {:?}", result
    );
}

#[test]
fn sessions_table_columns() {
    use rusqlite::Connection;

    let dir = TempDir::new().unwrap();
    let _store = SessionStore::open(dir.path()).unwrap();

    let conn = Connection::open(dir.path().join("sessions.db")).unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(sessions)").unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let expected = [
        "id", "title", "persona", "model", "status", "parent_id",
        "start_time", "end_time", "turn_count", "cost_usd", "jsonl_path", "cwd",
    ];
    for col in &expected {
        assert!(columns.iter().any(|c| c == col), "missing column: {col}");
    }
}
