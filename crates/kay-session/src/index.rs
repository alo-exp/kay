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
///
/// Generates a UUID v4, creates a per-session subdirectory under `store.sessions_dir`,
/// inserts the row with parameterized SQL (TM-05: no string interpolation), and
/// opens a `TranscriptWriter` in append mode.
pub fn create_session(
    store: &SessionStore,
    title: &str,
    persona: &str,
    model: &str,
    cwd: &std::path::Path,
) -> Result<Session, SessionError> {
    let id = Uuid::new_v4();
    let session_dir = store.session_dir(&id);
    std::fs::create_dir_all(&session_dir)?;

    let jsonl_path = session_dir.join("transcript.jsonl");
    let start_time = Utc::now().to_rfc3339();
    let cwd_str = cwd.to_string_lossy();

    store.conn.execute(
        "INSERT INTO sessions (id, title, persona, model, status, start_time, jsonl_path, cwd)
         VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7)",
        rusqlite::params![
            id.to_string(),
            title,
            persona,
            model,
            start_time,
            jsonl_path.to_string_lossy().as_ref(),
            cwd_str.as_ref(),
        ],
    )?;

    let transcript = TranscriptWriter::open(&jsonl_path, &id.to_string())?;

    Ok(Session {
        id,
        jsonl_path,
        transcript,
        cwd: cwd.to_path_buf(),
        turn_count: 0,
    })
}

/// List sessions ordered by start_time DESC, limited to `limit` rows.
pub fn list_sessions(
    store: &SessionStore,
    limit: usize,
) -> Result<Vec<SessionSummary>, SessionError> {
    let mut stmt = store.conn.prepare(
        "SELECT id, title, status, start_time, turn_count, cost_usd
         FROM sessions
         ORDER BY start_time DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
        let id_str: String = row.get(0)?;
        let title: String = row.get(1)?;
        let status: String = row.get(2)?;
        let start_time_str: String = row.get(3)?;
        let turn_count: i64 = row.get(4)?;
        let cost_usd: f64 = row.get(5)?;
        Ok((id_str, title, status, start_time_str, turn_count, cost_usd))
    })?;

    let mut summaries = Vec::new();
    for row in rows {
        let (id_str, title, status, start_time_str, turn_count, cost_usd) = row?;
        let id = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })?;
        let start_time = DateTime::parse_from_rfc3339(&start_time_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        summaries.push(SessionSummary {
            id,
            title,
            status,
            start_time,
            turn_count,
            cost_usd,
        });
    }

    Ok(summaries)
}

/// Close a session: set status = given status + end_time = now.
pub fn close_session(
    store: &SessionStore,
    id: &Uuid,
    status: SessionStatus,
) -> Result<(), SessionError> {
    let end_time = Utc::now().to_rfc3339();
    store.conn.execute(
        "UPDATE sessions SET status = ?1, end_time = ?2 WHERE id = ?3",
        rusqlite::params![status.as_str(), end_time, id.to_string()],
    )?;
    Ok(())
}

/// Resume an existing session by ID.
///
/// Reads jsonl_path and cwd from the DB, flips status to 'active', then calls
/// `TranscriptWriter::resume` which applies last-line crash recovery. The
/// returned `turn_count` comes from the actual file (not the stale DB value).
pub fn resume_session(
    store: &SessionStore,
    id: &Uuid,
) -> Result<Session, SessionError> {
    let row = store.conn.query_row(
        "SELECT jsonl_path, cwd FROM sessions WHERE id = ?1",
        rusqlite::params![id.to_string()],
        |row| {
            let jsonl_path: String = row.get(0)?;
            let cwd: String = row.get(1)?;
            Ok((jsonl_path, cwd))
        },
    );

    let (jsonl_path_str, cwd_str) = match row {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(SessionError::SessionNotFound { id: id.to_string() });
        }
        Err(e) => return Err(e.into()),
    };

    store.conn.execute(
        "UPDATE sessions SET status = 'active' WHERE id = ?1",
        rusqlite::params![id.to_string()],
    )?;

    let jsonl_path = PathBuf::from(&jsonl_path_str);
    let transcript = TranscriptWriter::resume(&jsonl_path, &id.to_string())?;
    let turn_count = transcript.line_count();

    Ok(Session {
        id: *id,
        jsonl_path,
        transcript,
        cwd: PathBuf::from(cwd_str),
        turn_count,
    })
}

/// Mark a session as lost in SQLite (DL-9: response to TranscriptDeleted).
pub fn mark_session_lost(store: &SessionStore, id: &Uuid) -> Result<(), SessionError> {
    store.conn.execute(
        "UPDATE sessions SET status = 'lost' WHERE id = ?1",
        rusqlite::params![id.to_string()],
    )?;
    Ok(())
}
