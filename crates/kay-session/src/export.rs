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
    let (jsonl_path, model, persona, start_time_str): (String, String, String, String) =
        store.conn.query_row(
            "SELECT jsonl_path, model, persona, start_time FROM sessions WHERE id = ?1",
            rusqlite::params![session_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| SessionError::SessionNotFound { id: session_id.to_string() })?;

    std::fs::create_dir_all(output_dir)?;

    // Count actual lines from JSONL — DB turn_count may be stale for in-memory-only appends
    let turn_count = {
        use std::io::BufRead;
        let file = std::fs::File::open(&jsonl_path)?;
        let reader = std::io::BufReader::new(file);
        reader.lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.trim().is_empty())
            .count() as i64
    };

    // Copy transcript.jsonl (DL-5: no snapshots in export)
    std::fs::copy(&jsonl_path, output_dir.join("transcript.jsonl"))?;

    let start_time = chrono::DateTime::parse_from_rfc3339(&start_time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let manifest = ExportManifest {
        session_id: *session_id,
        kay_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: 1,
        turn_count,
        model,
        persona_name: persona,
        start_time,
        export_time: Utc::now(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(output_dir.join("manifest.json"), manifest_json)?;

    Ok(())
}

/// Import a previously exported session bundle.
///
/// Creates a new session with a fresh UUID (not the original session ID).
/// Copies the transcript JSONL to the new session directory before opening
/// the writer, avoiding overwrite-while-open races.
pub fn import_session(
    store: &SessionStore,
    bundle_dir: &Path,
) -> Result<Session, SessionError> {
    let manifest_bytes = std::fs::read(bundle_dir.join("manifest.json"))?;
    let manifest: ExportManifest = serde_json::from_slice(&manifest_bytes)?;

    let new_id = Uuid::new_v4();
    let session_dir = store.session_dir(&new_id);
    std::fs::create_dir_all(&session_dir)?;
    let jsonl_path = session_dir.join("transcript.jsonl");

    // Copy transcript BEFORE opening TranscriptWriter to avoid overwrite-while-open
    std::fs::copy(bundle_dir.join("transcript.jsonl"), &jsonl_path)?;

    let start_time = Utc::now().to_rfc3339();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    store.conn.execute(
        "INSERT INTO sessions (id, title, persona, model, status, start_time, jsonl_path, cwd)
         VALUES (?1, '', ?2, ?3, 'active', ?4, ?5, ?6)",
        rusqlite::params![
            new_id.to_string(),
            &manifest.persona_name,
            &manifest.model,
            &start_time,
            jsonl_path.to_string_lossy().as_ref(),
            cwd.to_string_lossy().as_ref(),
        ],
    )?;

    // Use resume (crash recovery) since the file is already populated
    let transcript = crate::transcript::TranscriptWriter::resume(&jsonl_path, &new_id.to_string())?;
    let turn_count = transcript.line_count();

    Ok(Session { id: new_id, jsonl_path, transcript, cwd, turn_count })
}

/// Replay a transcript file by writing each JSONL line to `dest`.
///
/// DL-1: reads stored events only — no OpenRouter calls, no model context reconstruction.
pub fn replay(jsonl_path: &Path, dest: &mut dyn Write) -> Result<u64, SessionError> {
    use std::io::BufRead;
    let file = std::fs::File::open(jsonl_path)?;
    let reader = std::io::BufReader::new(file);
    let mut count = 0u64;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        writeln!(dest, "{line}").map_err(SessionError::Io)?;
        count += 1;
    }
    Ok(count)
}
