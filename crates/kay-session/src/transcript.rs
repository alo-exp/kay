use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use kay_tools::events_wire::AgentEventWire;
use crate::error::SessionError;

/// Append-only JSONL transcript writer.
///
/// Each `append_event` call writes one compact JSON object terminated by `\n`
/// using `AgentEventWire`'s Display impl. The file handle is kept open in
/// append mode for the session lifetime. I/O errors map to
/// `SessionError::TranscriptDeleted` (DL-9).
pub struct TranscriptWriter {
    file: File,
    path: PathBuf,
    session_id: String,
    pub(crate) line_count: u64,
}

impl TranscriptWriter {
    /// Open or create a transcript file in append mode.
    pub fn open(path: &Path, session_id: &str) -> Result<Self, SessionError> {
        unimplemented!("W-2 GREEN: open transcript file in append mode")
    }

    /// Resume a transcript: truncate to last complete \n-terminated line,
    /// count existing complete lines, reopen in append mode.
    pub fn resume(path: &Path, session_id: &str) -> Result<Self, SessionError> {
        unimplemented!("W-2 GREEN: truncate partial line + count existing lines")
    }

    /// Append one AgentEventWire as a single JSONL line.
    ///
    /// Uses `write!(self.file, "{wire}")` — NOT `writeln!` — because
    /// AgentEventWire's Display impl already appends `\n` (schema invariant).
    pub fn append_event(&mut self, wire: &AgentEventWire<'_>) -> Result<(), SessionError> {
        unimplemented!("W-2 GREEN: write JSONL line to append-mode file")
    }

    /// Number of complete lines (= event count) currently in the transcript.
    pub fn line_count(&self) -> u64 {
        self.line_count
    }
}

/// Truncate the file at `path` to the last complete \n-terminated line.
/// Returns the count of complete lines after truncation.
pub fn truncate_to_last_newline(path: &Path) -> Result<u64, std::io::Error> {
    unimplemented!("W-2 GREEN: scan backward for last \\n and set_len")
}
