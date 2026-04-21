use std::path::PathBuf;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session not found: {id}")]
    SessionNotFound { id: String },

    #[error("schema version mismatch: found {found}, expected {expected}")]
    SchemaVersionMismatch { found: u32, expected: u32 },

    #[error("path traversal rejected: {path:?} is outside session cwd {session_cwd:?}")]
    PathTraversalRejected { path: PathBuf, session_cwd: PathBuf },

    #[error("transcript deleted for session {session_id}: {path:?}")]
    TranscriptDeleted { session_id: String, path: PathBuf },

    #[error("snapshot cap exceeded: cap is {cap} bytes")]
    SnapshotCapExceeded { cap: u64 },

    #[error("no snapshots available for session {session_id}")]
    NoSnapshotsAvailable { session_id: String },

    #[error("destructive operation requires --force confirmation")]
    ConfirmationRequired,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
