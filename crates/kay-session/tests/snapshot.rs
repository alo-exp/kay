#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::index::create_session;
use kay_session::snapshot::{SessConfig, record_snapshot, rewind};
use kay_session::{SessionError, SessionStore};
use tempfile::TempDir;

fn make_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn record_snapshot_writes_file() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let original_path = cwd.join("test_file.txt");
    std::fs::write(&original_path, b"original content").unwrap();

    let config = SessConfig::default();
    record_snapshot(
        &store,
        &id,
        &cwd,
        1,
        &original_path,
        b"snapshot content",
        &config,
    )
    .unwrap();

    let snap_rows: Vec<String> = {
        let mut stmt = store
            .conn
            .prepare("SELECT snapshot_path FROM snapshots WHERE session_id = ?1")
            .unwrap();
        stmt.query_map(rusqlite::params![id.to_string()], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };
    assert_eq!(snap_rows.len(), 1, "one snapshot row must exist");
    let snap_path = std::path::PathBuf::from(&snap_rows[0]);
    assert!(snap_path.exists(), "snapshot file must exist on disk");
    assert_eq!(std::fs::read(&snap_path).unwrap(), b"snapshot content");
}

#[test]
fn snapshot_path_preserves_subdirectory() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let src_dir = cwd.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let original_path = src_dir.join("main.rs");
    std::fs::write(&original_path, b"fn main() {}").unwrap();

    let config = SessConfig::default();
    record_snapshot(
        &store,
        &id,
        &cwd,
        1,
        &original_path,
        b"fn main() {}",
        &config,
    )
    .unwrap();

    let snap_path: String = store
        .conn
        .query_row(
            "SELECT snapshot_path FROM snapshots WHERE session_id = ?1",
            rusqlite::params![id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        snap_path.contains("src") && snap_path.ends_with("main.rs"),
        "snapshot path must preserve subdirectory: {snap_path}"
    );
}

#[test]
fn snapshot_byte_cap_no_eviction_below_cap() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let content = vec![0u8; 1_048_576];
    let config = SessConfig::default();

    for i in 0..10 {
        let original = cwd.join(format!("file_{i}.dat"));
        std::fs::write(&original, &content).unwrap();
        record_snapshot(&store, &id, &cwd, i as u64, &original, &content, &config).unwrap();
    }

    let count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM snapshots WHERE session_id = ?1",
            rusqlite::params![id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 10,
        "no eviction should occur below 100 MiB cap (10 MiB written)"
    );
}

#[test]
fn snapshot_byte_cap_triggers_lru_eviction() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let config = SessConfig { snapshot_max_bytes: 5_242_880 }; // 5 MiB
    let content = vec![42u8; 3_145_728]; // 3 MiB each

    for i in 0..3 {
        let original = cwd.join(format!("large_{i}.dat"));
        std::fs::write(&original, &content).unwrap();
        record_snapshot(&store, &id, &cwd, i as u64, &original, &content, &config).unwrap();
    }

    let total_bytes: i64 = store
        .conn
        .query_row(
            "SELECT COALESCE(SUM(byte_size), 0) FROM snapshots WHERE session_id = ?1",
            rusqlite::params![id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        total_bytes <= 5_242_880,
        "total snapshot bytes must be <= cap after eviction: {total_bytes}"
    );
}

#[test]
fn snapshot_hash_matches_original() {
    use sha2::{Digest, Sha256};

    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let content = b"integrity check content";
    let expected_hash = hex::encode(Sha256::digest(content));

    let original = cwd.join("integrity.txt");
    std::fs::write(&original, content).unwrap();
    let config = SessConfig::default();
    record_snapshot(&store, &id, &cwd, 1, &original, content, &config).unwrap();

    let stored_hash: String = store
        .conn
        .query_row(
            "SELECT sha256 FROM snapshots WHERE session_id = ?1",
            rusqlite::params![id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        stored_hash, expected_hash,
        "stored sha256 must match computed hash"
    );
}

#[test]
fn rewind_restores_most_recent_snapshot() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;
    let config = SessConfig::default();

    let original = cwd.join("rewind_test.txt");
    std::fs::write(&original, b"original").unwrap();

    record_snapshot(&store, &id, &cwd, 1, &original, b"turn1 content", &config).unwrap();

    std::fs::write(&original, b"after turn 1").unwrap();
    record_snapshot(&store, &id, &cwd, 2, &original, b"turn2 content", &config).unwrap();

    std::fs::write(&original, b"corrupted").unwrap();

    let restored = rewind(&store, &id).unwrap();
    assert!(
        !restored.is_empty(),
        "rewind must restore at least one file"
    );
    assert_eq!(
        std::fs::read(&original).unwrap(),
        b"turn2 content",
        "most recent snapshot (turn 2) must be restored"
    );
}

#[test]
fn rewind_no_snapshot_returns_err() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "title", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let result = rewind(&store, &id);
    assert!(
        matches!(result, Err(SessionError::NoSnapshotsAvailable { .. })),
        "expected NoSnapshotsAvailable, got: {:?}",
        result
    );
}

#[test]
fn snapshot_cap_default_is_100mib() {
    let config = SessConfig::default();
    assert_eq!(
        config.snapshot_max_bytes, 104_857_600,
        "default cap must be 100 MiB"
    );
}
