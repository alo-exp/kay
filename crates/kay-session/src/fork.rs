use crate::error::SessionError;
use crate::index::Session;
use crate::store::SessionStore;
use crate::transcript::TranscriptWriter;
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

/// Fork a session: create a child session with `parent_id` = parent's UUID.
///
/// The child inherits `persona`, `model`, and `cwd` from the parent.
/// It starts with a fresh empty transcript and `status = "active"`.
/// `parent_id` satisfies SESS-04 (reserved for Phase 10 multi-agent
/// orchestration); FK is ON DELETE SET NULL.
pub fn fork_session(store: &SessionStore, parent_id: &Uuid) -> Result<Session, SessionError> {
    let (persona, model, cwd_str): (String, String, String) = store
        .conn
        .query_row(
            "SELECT persona, model, cwd FROM sessions WHERE id = ?1",
            rusqlite::params![parent_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| SessionError::SessionNotFound { id: parent_id.to_string() })?;

    let cwd = PathBuf::from(&cwd_str);
    let child_id = Uuid::new_v4();
    let session_dir = store.session_dir(&child_id);
    std::fs::create_dir_all(&session_dir)?;

    let jsonl_path = session_dir.join("transcript.jsonl");
    let start_time = Utc::now().to_rfc3339();

    store.conn.execute(
        "INSERT INTO sessions
         (id, title, persona, model, status, parent_id, start_time, turn_count,
          cost_usd, jsonl_path, cwd)
         VALUES (?1, '', ?2, ?3, 'active', ?4, ?5, 0, 0.0, ?6, ?7)",
        rusqlite::params![
            child_id.to_string(),
            &persona,
            &model,
            parent_id.to_string(),
            &start_time,
            jsonl_path.to_str().unwrap_or(""),
            &cwd_str,
        ],
    )?;

    let transcript = TranscriptWriter::open(&jsonl_path, &child_id.to_string())?;

    Ok(Session { id: child_id, jsonl_path, transcript, cwd, turn_count: 0 })
}
