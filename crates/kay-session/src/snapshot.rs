use std::path::{Path, PathBuf};
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
/// Validates that `original_path` is within `session_cwd` (DL-6 + QG-C6).
/// Writes bytes to `session_dir/snapshots/<turn>/<rel_path>`.
/// Records the snapshot in the `snapshots` table.
/// Enforces `config.snapshot_max_bytes` by LRU eviction of oldest-turn snapshots.
pub fn record_snapshot(
    store: &SessionStore,
    session_id: &Uuid,
    session_cwd: &Path,
    turn: u64,
    original_path: &Path,
    content: &[u8],
    config: &SessConfig,
) -> Result<(), SessionError> {
    unimplemented!("W-4 GREEN: path traversal guard + write + DB insert + LRU eviction")
}

/// Restore the most recent snapshot for a session.
///
/// Queries snapshots table for MAX(turn) per original_path, restores bytes
/// to original path. Returns Err(NoSnapshotsAvailable) if no snapshots exist.
pub fn rewind(
    store: &SessionStore,
    session_id: &Uuid,
) -> Result<Vec<PathBuf>, SessionError> {
    unimplemented!("W-4 GREEN: SELECT MAX(turn) snapshots, restore files")
}
