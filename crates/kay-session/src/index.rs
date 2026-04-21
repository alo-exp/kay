use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::error::SessionError;
use crate::store::SessionStore;
use crate::transcript::TranscriptWriter;

/// Status of a session row in the sessions table.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Active,
    Complete,
    Lost,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Complete => "complete",
            Self::Lost => "lost",
        }
    }
}

/// Summary row for list_sessions (lightweight — no file handle).
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub turn_count: i64,
    pub cost_usd: f64,
}

/// Live session handle — owns the open transcript file handle.
/// NOT Clone (exclusive file handle per DL-9 + T-9 canary).
#[derive(Debug)]
pub struct Session {
    pub id: Uuid,
    pub jsonl_path: PathBuf,
    pub transcript: TranscriptWriter,
    pub cwd: PathBuf,
    pub turn_count: u64,
}

impl Session {
    /// Append an event to the JSONL transcript.
    pub fn append_event(
        &mut self,
        wire: &kay_tools::events_wire::AgentEventWire<'_>,
    ) -> Result<(), SessionError> {
        self.transcript.append_event(wire)?;
        self.turn_count += 1;
        Ok(())
    }
}

/// Create a new session row and open its transcript file.
pub fn create_session(
    store: &SessionStore,
    title: &str,
    persona: &str,
    model: &str,
    cwd: &std::path::Path,
) -> Result<Session, SessionError> {
    unimplemented!("W-3 GREEN: INSERT INTO sessions + open TranscriptWriter")
}

/// List sessions ordered by start_time DESC, limited to `limit` rows.
pub fn list_sessions(
    store: &SessionStore,
    limit: usize,
) -> Result<Vec<SessionSummary>, SessionError> {
    unimplemented!("W-3 GREEN: SELECT … ORDER BY start_time DESC LIMIT ?")
}

/// Close a session: set status = given status + end_time = now.
pub fn close_session(
    store: &SessionStore,
    id: &Uuid,
    status: SessionStatus,
) -> Result<(), SessionError> {
    unimplemented!("W-3 GREEN: UPDATE sessions SET status=?, end_time=? WHERE id=?")
}

/// Resume an existing session by ID.
/// Reads jsonl_path from DB, calls TranscriptWriter::resume for last-line recovery.
pub fn resume_session(
    store: &SessionStore,
    id: &Uuid,
) -> Result<Session, SessionError> {
    unimplemented!("W-3 GREEN: SELECT jsonl_path + cwd from sessions, TranscriptWriter::resume")
}

/// Mark a session as lost in SQLite (DL-9: response to TranscriptDeleted).
pub fn mark_session_lost(store: &SessionStore, id: &Uuid) -> Result<(), SessionError> {
    unimplemented!("W-3 GREEN: UPDATE sessions SET status='lost'")
}
