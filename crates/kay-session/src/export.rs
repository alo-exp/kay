use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::error::SessionError;
use crate::index::Session;
use crate::store::SessionStore;

/// Manifest written alongside transcript.jsonl on export.
///
/// Schema version is always 1 in Phase 6.
/// DL-7: title is NOT included — it's untrusted user data not needed for replay.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportManifest {
    pub session_id: Uuid,
    pub kay_version: String,
    pub schema_version: u32,
    pub turn_count: i64,
    pub model: String,
    pub persona_name: String,
    pub start_time: DateTime<Utc>,
    pub export_time: DateTime<Utc>,
}

/// Export a session to `output_dir` as `transcript.jsonl` + `manifest.json`.
///
/// DL-5: directory format (not tarball); no snapshots included.
pub fn export_session(
    store: &SessionStore,
    session_id: &Uuid,
    output_dir: &Path,
) -> Result<(), SessionError> {
    unimplemented!("W-6 GREEN: copy jsonl + write manifest.json")
}

/// Import a previously exported session bundle.
///
/// Creates a new session with a fresh UUID (not the original session ID).
/// Copies the transcript JSONL to the new session directory.
pub fn import_session(
    store: &SessionStore,
    bundle_dir: &Path,
) -> Result<Session, SessionError> {
    unimplemented!("W-6 GREEN: read manifest.json + copy transcript + create_session")
}

/// Replay a transcript file by writing each JSONL line to `dest`.
///
/// DL-1: reads stored events only — no OpenRouter calls, no model context reconstruction.
pub fn replay(jsonl_path: &Path, dest: &mut dyn Write) -> Result<u64, SessionError> {
    unimplemented!("W-6 GREEN: BufRead::lines, write each to dest, return line count")
}
