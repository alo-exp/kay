#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::transcript::TranscriptWriter;
use kay_tools::AgentEvent;
use kay_tools::events_wire::AgentEventWire;
use tempfile::TempDir;

fn make_wire_event() -> AgentEvent {
    AgentEvent::TextDelta { content: "hello wave 2".into() }
}

#[test]
fn append_writes_valid_json_newline() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("transcript.jsonl");
    let mut writer = TranscriptWriter::open(&path, "test-session-id").unwrap();

    let event = make_wire_event();
    let wire = AgentEventWire::from(&event);
    writer.append_event(&wire).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.ends_with('\n'),
        "line must end with exactly one newline"
    );
    let trimmed = contents.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "must be single-line JSON");
    let parsed: serde_json::Value = serde_json::from_str(trimmed).unwrap();
    assert!(parsed.is_object(), "must be a JSON object");
}

#[test]
fn append_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("transcript.jsonl");
    let mut writer = TranscriptWriter::open(&path, "test-session-id").unwrap();

    let event = AgentEvent::TextDelta { content: "round-trip test".into() };
    let wire = AgentEventWire::from(&event);
    writer.append_event(&wire).unwrap();
    drop(writer);

    let contents = std::fs::read_to_string(&path).unwrap();
    let line = contents.trim_end_matches('\n');
    let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
    assert!(
        parsed.get("type").is_some(),
        "wire schema must have 'type' field"
    );
}

#[test]
fn last_line_truncation() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("transcript.jsonl");

    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b"{\"type\":\"text_delta\",\"content\":\"complete\"}\n")
            .unwrap();
        f.write_all(b"{\"type\":\"text_delta\",\"content\":\"partial-no-newline\"}")
            .unwrap();
    }

    let writer = TranscriptWriter::resume(&path, "test-session-id").unwrap();
    assert_eq!(writer.line_count(), 1, "only the complete line survives");

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        !contents.contains("partial-no-newline"),
        "partial line must be removed"
    );
}

#[test]
fn last_line_empty_file_ok() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("transcript.jsonl");

    std::fs::File::create(&path).unwrap();

    let writer = TranscriptWriter::resume(&path, "test-session-id").unwrap();
    assert_eq!(writer.line_count(), 0, "empty file = 0 turns");
}

#[test]
fn line_count_matches_turn_count() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("transcript.jsonl");
    let mut writer = TranscriptWriter::open(&path, "test-session-id").unwrap();

    let n = 7usize;
    for i in 0..n {
        let event = AgentEvent::TextDelta { content: format!("event-{i}") };
        let wire = AgentEventWire::from(&event);
        writer.append_event(&wire).unwrap();
    }

    assert_eq!(
        writer.line_count(),
        n as u64,
        "line_count must match append call count"
    );

    let contents = std::fs::read_to_string(&path).unwrap();
    let actual_lines = contents.lines().count();
    assert_eq!(actual_lines, n, "file must have exactly N lines");
}
