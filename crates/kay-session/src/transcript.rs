use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use kay_tools::events_wire::AgentEventWire;
use crate::error::SessionError;

/// Append-only JSONL transcript writer for a single session.
///
/// Holds an open file handle in append mode for the session's lifetime.
/// On `append_event`, uses `AgentEventWire`'s Display impl which emits
/// one compact JSON object followed by exactly one `\n` (schema invariant).
///
/// I/O errors from `append_event` map to `SessionError::TranscriptDeleted`
/// per DL-9 (caller marks session "lost" and exits 1).
pub struct TranscriptWriter {
    file: File,
    path: PathBuf,
    session_id: String,
    pub(crate) line_count: u64,
}

impl TranscriptWriter {
    /// Open or create a transcript file in append mode.
    /// Used when creating a new session (zero initial lines).
    pub fn open(path: &Path, session_id: &str) -> Result<Self, SessionError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            file,
            path: path.to_path_buf(),
            session_id: session_id.to_string(),
            line_count: 0,
        })
    }

    /// Resume a transcript: truncate to last complete \n-terminated line,
    /// count existing complete lines, then reopen in append mode.
    ///
    /// Handles the case where the last write was partial (crashed mid-line)
    /// by truncating to the last complete line boundary.
    pub fn resume(path: &Path, session_id: &str) -> Result<Self, SessionError> {
        let line_count = if path.exists() {
            truncate_to_last_newline(path)?
        } else {
            0
        };

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
            session_id: session_id.to_string(),
            line_count,
        })
    }

    /// Append one AgentEventWire as a single JSONL line.
    ///
    /// CRITICAL: uses `write!(self.file, "{wire}")` — NOT `writeln!` —
    /// because `AgentEventWire`'s Display impl already appends `\n`.
    /// Double-newline would break the single-line-per-event schema invariant.
    ///
    /// I/O errors map to `SessionError::TranscriptDeleted` (DL-9).
    pub fn append_event(&mut self, wire: &AgentEventWire<'_>) -> Result<(), SessionError> {
        write!(self.file, "{wire}").map_err(|_| SessionError::TranscriptDeleted {
            session_id: self.session_id.clone(),
            path: self.path.clone(),
        })?;
        self.line_count += 1;
        Ok(())
    }

    /// Number of complete lines (= event count) currently in the transcript.
    pub fn line_count(&self) -> u64 {
        self.line_count
    }

    /// Path to the transcript file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Truncate the file at `path` to the last complete \n-terminated line.
///
/// Returns the count of complete lines after truncation.
/// An empty file truncates to 0 bytes and returns 0 lines (no error).
pub fn truncate_to_last_newline(path: &Path) -> Result<u64, std::io::Error> {
    let content = std::fs::read(path)?;

    let truncate_at = content
        .iter()
        .rposition(|&b| b == b'\n')
        .map(|i| i + 1)
        .unwrap_or(0);

    let file = OpenOptions::new().write(true).open(path)?;
    file.set_len(truncate_at as u64)?;

    let line_count = content[..truncate_at]
        .iter()
        .filter(|&&b| b == b'\n')
        .count() as u64;

    Ok(line_count)
}
