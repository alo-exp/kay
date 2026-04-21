use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::error::SessionError;
use crate::store::SessionStore;

/// Per-session snapshot configuration.
#[derive(Debug, Clone)]
pub struct SessConfig {
    /// Maximum total snapshot bytes per session. Default: 100 MiB.
    pub snapshot_max_bytes: u64,
}

impl Default for SessConfig {
    fn default() -> Self {
        Self {
            snapshot_max_bytes: 104_857_600, // 100 MiB
        }
    }
}

/// Record a pre-edit snapshot of `original_path` within the session.
///
/// 1. Path traversal guard (DL-6 + QG-C6): canonicalize and verify within session_cwd.
/// 2. Write content to session_dir/snapshots/<turn>/<rel_path>.
/// 3. INSERT into snapshots table with SHA-256 and byte_size.
/// 4. LRU eviction: if total bytes > cap, delete oldest-turn rows + files.
pub fn record_snapshot(
    store: &SessionStore,
    session_id: &Uuid,
    session_cwd: &Path,
    turn: u64,
    original_path: &Path,
    content: &[u8],
    config: &SessConfig,
) -> Result<(), SessionError> {
    // Path traversal guard (DL-6, QG-C6)
    let canonical = original_path.canonicalize()?;
    let cwd_canonical = session_cwd.canonicalize()?;
    if !canonical.starts_with(&cwd_canonical) {
        return Err(SessionError::PathTraversalRejected {
            path: canonical,
            session_cwd: cwd_canonical,
        });
    }
    let rel_path = canonical
        .strip_prefix(&cwd_canonical)
        .map_err(|_| std::io::Error::other("path not within session cwd after starts_with check"))?;

    // Destination path: session_dir/snapshots/<turn>/<rel_path>
    let snap_dir = store
        .sessions_dir
        .join(session_id.to_string())
        .join("snapshots")
        .join(turn.to_string());
    let dest = snap_dir.join(rel_path);
    std::fs::create_dir_all(dest.parent().unwrap_or(&snap_dir))?;
    std::fs::write(&dest, content)?;

    // SHA-256 integrity
    let sha256 = hex::encode(Sha256::digest(content));

    store.conn.execute(
        "INSERT INTO snapshots (session_id, turn, original_path, snapshot_path, byte_size, sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            session_id.to_string(),
            turn as i64,
            canonical.to_string_lossy().as_ref(),
            dest.to_string_lossy().as_ref(),
            content.len() as i64,
            sha256,
        ],
    )?;

    // LRU eviction: delete oldest-turn rows+files until total <= cap
    let total: i64 = store.conn.query_row(
        "SELECT COALESCE(SUM(byte_size), 0) FROM snapshots WHERE session_id = ?1",
        rusqlite::params![session_id.to_string()],
        |r| r.get(0),
    )?;

    if total as u64 > config.snapshot_max_bytes {
        let mut stmt = store.conn.prepare(
            "SELECT id, snapshot_path, byte_size FROM snapshots
             WHERE session_id = ?1
             ORDER BY turn ASC",
        )?;
        let rows: Vec<(i64, String, i64)> = stmt
            .query_map(rusqlite::params![session_id.to_string()], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut remaining = total as u64;
        for (row_id, snap_path, byte_size) in rows {
            if remaining <= config.snapshot_max_bytes {
                break;
            }
            let _ = std::fs::remove_file(&snap_path);
            store.conn.execute(
                "DELETE FROM snapshots WHERE id = ?1",
                rusqlite::params![row_id],
            )?;
            remaining = remaining.saturating_sub(byte_size as u64);
        }
    }

    Ok(())
}

/// List original paths that would be restored by `rewind`, without restoring.
///
/// Returns the set of original_path values for the most recent snapshot of
/// each tracked file. Empty vec means no snapshots exist (not an error).
pub fn list_rewind_paths(
    store: &SessionStore,
    session_id: &Uuid,
) -> Result<Vec<PathBuf>, SessionError> {
    let mut stmt = store.conn.prepare(
        "SELECT s.original_path
         FROM snapshots s
         INNER JOIN (
             SELECT original_path, MAX(turn) AS max_turn
             FROM snapshots
             WHERE session_id = ?1
             GROUP BY original_path
         ) latest ON s.original_path = latest.original_path AND s.turn = latest.max_turn
         WHERE s.session_id = ?1",
    )?;

    let paths: Vec<PathBuf> = stmt
        .query_map(rusqlite::params![session_id.to_string()], |r| {
            Ok(PathBuf::from(r.get::<_, String>(0)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(paths)
}

/// Restore the most recent snapshot for each tracked file in a session.
///
/// For each distinct original_path in the snapshots table, selects the
/// snapshot with MAX(turn) and copies it back to the original location.
/// Returns Err(NoSnapshotsAvailable) if no snapshots exist for the session.
pub fn rewind(
    store: &SessionStore,
    session_id: &Uuid,
) -> Result<Vec<PathBuf>, SessionError> {
    // Check if any snapshots exist
    let count: i64 = store.conn.query_row(
        "SELECT COUNT(*) FROM snapshots WHERE session_id = ?1",
        rusqlite::params![session_id.to_string()],
        |r| r.get(0),
    )?;

    if count == 0 {
        return Err(SessionError::NoSnapshotsAvailable {
            session_id: session_id.to_string(),
        });
    }

    // For each distinct original_path, get the latest snapshot_path
    let mut stmt = store.conn.prepare(
        "SELECT s.original_path, s.snapshot_path
         FROM snapshots s
         INNER JOIN (
             SELECT original_path, MAX(turn) AS max_turn
             FROM snapshots
             WHERE session_id = ?1
             GROUP BY original_path
         ) latest ON s.original_path = latest.original_path AND s.turn = latest.max_turn
         WHERE s.session_id = ?1",
    )?;

    let pairs: Vec<(String, String)> = stmt
        .query_map(rusqlite::params![session_id.to_string()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut restored = Vec::new();
    for (original_path_str, snapshot_path_str) in pairs {
        let original = PathBuf::from(&original_path_str);
        let snapshot = PathBuf::from(&snapshot_path_str);
        if let Some(parent) = original.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&snapshot, &original)?;
        restored.push(original);
    }

    Ok(restored)
}
