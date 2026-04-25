// M12-Task 5 RED: kay-session integration tests for SessionStore
//
// Verifies SessionStore::open creates schema v1 and WAL mode on temp dir.

use kay_session::SessionStore;

#[test]
fn open_creates_sessions_db_in_temp_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    assert!(tmp.path().join("sessions.db").exists());
    drop(store);
}

#[test]
fn open_sets_wal_journal_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let mode: String = store.conn.query_row(
        "PRAGMA journal_mode",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(mode.to_lowercase(), "wal");
    drop(store);
}

#[test]
fn sessions_dir_returns_correct_path() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    assert_eq!(store.sessions_dir(), tmp.path());
    drop(store);
}

#[test]
fn sessions_table_exists_after_open() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let count: i64 = store.conn.query_row(
        "SELECT COUNT(*) FROM sessions",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 0);
    drop(store);
}

#[test]
fn schema_version_is_1_after_first_open() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SessionStore::open(tmp.path()).unwrap();
    let version: i64 = store.conn.query_row(
        "SELECT version FROM schema_version",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(version, 1);
    drop(store);
}

#[test]
fn second_open_does_not_recreate_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let store1 = SessionStore::open(tmp.path()).unwrap();
    drop(store1);
    // Re-open — should not panic and should preserve data
    let store2 = SessionStore::open(tmp.path()).unwrap();
    let count: i64 = store2.conn.query_row(
        "SELECT COUNT(*) FROM sessions",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 0);
    drop(store2);
}
