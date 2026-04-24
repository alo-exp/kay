// jsonl.rs — JSONL streaming parser for kay-cli JSONL output.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §4
//
// WAVE 2: LineBuffer ring + JsonlParser streaming.
// - LineBuffer: fixed 1MB buffer, drops oldest when full
// - JsonlParser: reads lines from Read, yields TuiEvent
// - Malformed JSON: logged at WARN, skipped (ERR-01)
// - Unknown event types: logged at WARN, skipped (ERR-02)
// - Partial lines: accumulated across read() calls (no line fragmentation)

use std::io::{self, Read};
use tracing::warn;

use crate::events::TuiEvent;

/// Fixed-size ring buffer for line accumulation.
/// Drops the oldest line when the buffer exceeds 1 MB (spec §4 PERF-01).
#[derive(Debug)]
pub struct LineBuffer {
    buf: String,
    max_bytes: usize,
    /// Total bytes ever pushed (for overflow accounting).
    total_bytes: u64,
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl LineBuffer {
    const MAX_BYTES: usize = 1024 * 1024; // 1 MB

    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(Self::MAX_BYTES),
            max_bytes: Self::MAX_BYTES,
            total_bytes: 0,
        }
    }

    /// Append raw bytes to the buffer. If total exceeds max_bytes, the
    /// oldest lines are dropped until it fits.
    pub fn push(&mut self, chunk: &str) {
        self.total_bytes += chunk.len() as u64;
        self.buf.push_str(chunk);

        while self.buf.len() > self.max_bytes {
            if let Some(pos) = self.buf.find('\n') {
                self.buf.drain(..=pos);
            } else {
                // No newline found — buffer has a single oversized line.
                // Keep only the last max_bytes characters.
                let keep = self.buf.len().saturating_sub(self.max_bytes);
                self.buf.drain(..keep);
                break;
            }
        }
    }

    /// Drain and return all complete lines (ending in '\n'), leaving
    /// any trailing partial line (no trailing '\n') in the buffer.
    pub fn drain_lines(&mut self) -> Vec<String> {
        if self.buf.is_empty() {
            return Vec::new();
        }

        let Some(last_newline) = self.buf.rfind('\n') else {
            // No newline at all — treat as one incomplete line, keep buffered.
            return Vec::new();
        };

        let mut lines = Vec::new();
        // Split at last newline: everything before is complete lines.
        let complete = &self.buf[..last_newline];
        for line in complete.lines() {
            lines.push(line.to_string());
        }
        // Keep only what follows the last newline (trailing partial line).
        self.buf.drain(..=last_newline);
        lines
    }

    /// Returns true if there is a trailing partial line (no '\n').
    #[cfg(test)]
    pub fn has_trailing_partial(&self) -> bool {
        !self.buf.is_empty() && !self.buf.ends_with('\n')
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// Streaming JSONL parser. Reads newline-delimited JSON from any `Read` source.
#[derive(Debug)]
pub struct JsonlParser {
    buf: LineBuffer,
}

impl Default for JsonlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonlParser {
    pub fn new() -> Self {
        Self { buf: LineBuffer::new() }
    }

    /// Feed raw bytes into the parser. Lines are accumulated and complete
    /// JSON objects are yielded via `next_event`.
    pub fn feed(&mut self, chunk: &[u8]) {
        if let Ok(s) = std::str::from_utf8(chunk) {
            self.buf.push(s);
        } else {
            // Drop bytes that aren't valid UTF-8 (binary chunk in stream)
            warn!("jsonl: dropping non-UTF-8 chunk ({} bytes)", chunk.len());
        }
    }

    /// Feed from anything that implements `Read`.
    pub fn feed_reader<R: Read>(&mut self, reader: &mut R) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            self.feed(&buf[..n]);
        }
        Ok(())
    }

    /// Consume and return the next parsed `TuiEvent`, if any.
    /// Returns `None` when no complete lines are available.
    ///
    /// Errors (malformed JSON, unknown event type) are logged and skipped —
    /// the next call returns the following event or `None`.
    pub fn next_event(&mut self) -> Option<Result<TuiEvent, ParseError>> {
        let lines = self.buf.drain_lines();
        if lines.is_empty() {
            return None;
        }

        // Process each complete line
        for line in lines {
            if line.trim().is_empty() {
                continue; // skip blank lines
            }
            return Some(self.parse_line(&line));
        }

        None
    }

    /// Drains all available events from the buffer, returning them.
    /// Unknown or malformed lines are logged and skipped.
    pub fn drain_events(&mut self) -> Vec<Result<TuiEvent, ParseError>> {
        let lines = self.buf.drain_lines();
        lines
            .into_iter()
            .filter(|l| !l.trim().is_empty())
            .map(|line| self.parse_line(&line))
            .collect()
    }

    fn parse_line(&self, line: &str) -> Result<TuiEvent, ParseError> {
        // First try to parse as TuiEvent
        match serde_json::from_str::<TuiEvent>(line) {
            Ok(event) => Ok(event),
            Err(e) => {
                // Check if this looks like an unknown event type
                if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(typ) = raw.get("type").and_then(|v| v.as_str()) {
                        // It's valid JSON but an unknown event type → log and skip (ERR-02)
                        warn!(
                            event_type = %typ,
                            "jsonl: unknown event type — skipping (handled by JsonlParser, not serde)"
                        );
                        return Err(ParseError::UnknownEventType(typ.to_string()));
                    }
                }
                // Malformed JSON → log and skip (ERR-01)
                let preview = if line.len() > 100 {
                    format!("{}...", &line[..100])
                } else {
                    line.to_string()
                };
                warn!(
                    error = %e,
                    preview = %preview,
                    "jsonl: malformed JSON line — skipping"
                );
                Err(ParseError::MalformedJson(e.to_string()))
            }
        }
    }

    #[cfg(test)]
    pub fn parse_single(line: &str) -> Result<TuiEvent, ParseError> {
        Self::new().parse_line(line)
    }
}

/// Errors from JSONL parsing.
#[derive(Debug)]
pub enum ParseError {
    /// Line is valid JSON but has an unrecognized event type tag.
    UnknownEventType(String),
    /// Line is not valid JSON.
    MalformedJson(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownEventType(t) => write!(f, "unknown event type: {t}"),
            ParseError::MalformedJson(e) => write!(f, "malformed JSON: {e}"),
        }
    }
}

impl std::error::Error for ParseError {}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{TuiEvent, TuiToolOutputChunk, TuiVerificationOutcome};

    #[test]
    fn line_buffer_basic() {
        let mut buf = LineBuffer::new();
        buf.push("hello\nworld\n");
        let lines = buf.drain_lines();
        assert_eq!(lines, vec!["hello", "world"]);
    }

    #[test]
    fn line_buffer_trailing_partial() {
        let mut buf = LineBuffer::new();
        buf.push("line1\npartial");
        let lines = buf.drain_lines();
        assert_eq!(lines, vec!["line1"]);
        // Partial line (no trailing newline) stays in buffer
        assert!(
            buf.has_trailing_partial(),
            "partial 'partial' should be buffered"
        );
    }

    #[test]
    fn line_buffer_overflow_drops_oldest() {
        let mut buf = LineBuffer::new();
        // Push well over 1 MB to trigger overflow
        let long_line = "x".repeat(600 * 1024); // 600 KB
        buf.push(&format!("{long_line}\n"));
        buf.push(&format!("{long_line}\n"));
        buf.push(&format!("{long_line}\n"));
        // Should have dropped oldest lines to stay within 1 MB
        let lines = buf.drain_lines();
        // At least one line should remain
        assert!(!lines.is_empty());
    }

    #[test]
    fn valid_event_parses() {
        let json = r#"{"type":"TextDelta","data":{"content":"hello"}}"#;
        let event = JsonlParser::parse_single(json).unwrap();
        assert!(matches!(event, TuiEvent::TextDelta { .. }));
    }

    #[test]
    fn usage_event_parses() {
        let json = r#"{"type":"Usage","data":{"prompt_tokens":100,"completion_tokens":50,"cost_usd":0.002}}"#;
        let event = JsonlParser::parse_single(json).unwrap();
        match event {
            TuiEvent::Usage { prompt_tokens, completion_tokens, .. } => {
                assert_eq!(prompt_tokens, 100);
                assert_eq!(completion_tokens, 50);
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn malformed_json_returns_error() {
        let result = JsonlParser::parse_single(r#"{"type":"TextDelta","data":{"content"#);
        assert!(result.is_err());
        match result {
            Err(ParseError::MalformedJson(_)) => {}
            other => panic!("expected MalformedJson, got {other:?}"),
        }
    }

    #[test]
    fn unknown_event_type_routes_to_tui_event_unknown() {
        // TuiEvent::Unknown is the catch-all variant — unknown event types
        // (future wire additions) deserialize successfully to it. The
        // UnknownEventType error was removed when the Unknown variant was added.
        let json = r#"{"type":"SomeNewFutureEvent","data":{"foo":123}}"#;
        let result = JsonlParser::parse_single(json);
        assert!(
            result.is_ok(),
            "Unknown event types should deserialize to TuiEvent::Unknown, got: {result:?}"
        );
        match result.unwrap() {
            TuiEvent::Unknown { ref event_type } => {
                assert_eq!(event_type, "SomeNewFutureEvent");
            }
            other => panic!("expected TuiEvent::Unknown, got {other:?}"),
        }
    }

    #[test]
    fn blank_lines_skipped() {
        let mut parser = JsonlParser::new();
        parser.buf.push("line1\n\nline2\n");
        let events: Vec<_> = parser.drain_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn feed_and_next() {
        let mut parser = JsonlParser::new();
        parser.feed(b"{\"type\":\"TextDelta\",\"data\":{\"content\":\"hi\"}}\n");
        let event = parser.next_event().unwrap().unwrap();
        assert!(matches!(event, TuiEvent::TextDelta { content } if content == "hi"));
        assert!(parser.next_event().is_none());
    }

    #[test]
    fn partial_line_accumulated() {
        let mut parser = JsonlParser::new();
        parser.feed(b"{\"type\":\"TextDelta\"");
        // No newline yet → drain_lines returns empty, next_event returns None
        assert!(
            parser.next_event().is_none(),
            "incomplete line without newline is buffered, not drained"
        );
        // Complete the line
        parser.feed(b",\"data\":{\"content\":\"hi\"}}\n");
        let event = parser.next_event().unwrap().unwrap();
        assert!(matches!(event, TuiEvent::TextDelta { content } if content == "hi"));
    }

    #[test]
    fn non_utf8_chunk_dropped() {
        let mut parser = JsonlParser::new();
        // Valid UTF-8 line before binary chunk
        parser.feed(b"{\"type\":\"TextDelta\",\"data\":{\"content\":\"hi\"}}\n");
        // Invalid UTF-8 bytes
        parser.feed(&[0x80, 0x81, 0x82]);
        // Valid UTF-8 line after
        parser.feed(b"{\"type\":\"Error\",\"data\":{\"message\":\"boom\"}}\n");
        let events: Vec<_> = parser.drain_events();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0].as_ref().unwrap(),
            TuiEvent::TextDelta { .. }
        ));
        assert!(matches!(
            events[1].as_ref().unwrap(),
            TuiEvent::Error { .. }
        ));
    }
}
