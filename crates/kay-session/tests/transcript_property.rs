#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::transcript::TranscriptWriter;
use kay_tools::events_wire::AgentEventWire;
use kay_tools::AgentEvent;
use proptest::prelude::*;
use tempfile::TempDir;

proptest! {
    #[test]
    fn jsonl_round_trip_proptest(contents in prop::collection::vec(0u8..=127u8, 0..100)) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("transcript.jsonl");
        let mut writer = TranscriptWriter::open(&path, "prop-session").unwrap();

        let text: String = contents.iter().map(|&b| b as char).collect();
        let event = AgentEvent::TextDelta { content: text };
        let wire = AgentEventWire::from(&event);
        writer.append_event(&wire).unwrap();
        drop(writer);

        let file_contents = std::fs::read_to_string(&path).unwrap();
        let line = file_contents.trim_end_matches('\n');
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        prop_assert!(parsed.is_ok(), "round-trip must produce valid JSON");
    }

    #[test]
    fn no_newline_in_json_body(contents in "[a-zA-Z0-9 ]{0,50}") {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("transcript.jsonl");
        let mut writer = TranscriptWriter::open(&path, "prop-session").unwrap();

        let event = AgentEvent::TextDelta { content: contents };
        let wire = AgentEventWire::from(&event);
        writer.append_event(&wire).unwrap();
        drop(writer);

        let file_contents = std::fs::read_to_string(&path).unwrap();
        let body = file_contents.trim_end_matches('\n');
        prop_assert!(!body.contains('\n'), "JSON body must not contain bare newline");
    }
}
