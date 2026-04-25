// jsonl_parser.rs — integration tests for full JSONL → TuiEvent pipeline.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md
//
// RED: These tests assert behavior from the integration of all parser components.
// They fail until the full pipeline (line buffering → JSON parsing → TuiEvent deserialization)
// is verified end-to-end.

use kay_tui::events::TuiEvent;

/// Test that a complete JSONL line with all known event types parses correctly.
#[test]
fn all_event_types_parse_from_jsonl() {
    let inputs = vec![
        // TextDelta
        r#"{"type":"TextDelta","data":{"content":"hello"}}"#,
        // ToolCallStart
        r#"{"type":"ToolCallStart","data":{"id":"t1","name":"edit_file"}}"#,
        // ToolCallDelta
        r#"{"type":"ToolCallDelta","data":{"id":"t1","arguments_delta":"foo"}}"#,
        // ToolCallComplete
        r#"{"type":"ToolCallComplete","data":{"id":"t1","name":"edit_file"}}"#,
        // Usage
        r#"{"type":"Usage","data":{"prompt_tokens":100,"completion_tokens":50,"cost_usd":0.001}}"#,
        // Error
        r#"{"type":"Error","data":{"message":"oops"}}"#,
        // Paused
        r#"{"type":"Paused","data":{}}"#,
    ];

    let mut parser = kay_tui::jsonl::JsonlParser::new();
    let mut parsed: Vec<TuiEvent> = Vec::new();

    for line in inputs {
        parser.feed(line.as_bytes());
        parser.feed(b"\n");
        while let Some(result) = parser.next_event() {
            parsed.push(result.expect("should parse valid JSONL"));
        }
    }

    // All 7 events should have been parsed
    assert_eq!(parsed.len(), 7, "should parse all 7 event types");

    // Spot-check key events (use & for move safety on Vec index)
    assert!(matches!(&parsed[0], TuiEvent::TextDelta { .. }));
    assert!(matches!(&parsed[1], TuiEvent::ToolCallStart { name, .. } if name == "edit_file"));
    assert!(matches!(
        &parsed[4],
        TuiEvent::Usage { prompt_tokens: 100, .. }
    ));
    assert!(matches!(&parsed[6], TuiEvent::Paused));
}

/// Test that JSONL parser skips blank and whitespace-only lines.
#[test]
fn blank_lines_skipped_in_jsonl() {
    // Use valid JSONL lines with blank lines between them.
    // Blank lines should be skipped, only JSON events returned.
    let mut parser = kay_tui::jsonl::JsonlParser::new();
    parser.feed(b"{\"type\":\"TextDelta\",\"data\":{\"content\":\"line1\"}}\n");
    parser.feed(b"\n"); // blank line
    parser.feed(b"\n"); // another blank line
    parser.feed(b"{\"type\":\"TextDelta\",\"data\":{\"content\":\"line2\"}}\n");

    let events: Vec<_> = parser
        .drain_events()
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(
        events.len(),
        2,
        "blank lines should be skipped, only valid JSON parsed"
    );
    assert!(matches!(&events[0], TuiEvent::TextDelta { content } if content == "line1"));
    assert!(matches!(&events[1], TuiEvent::TextDelta { content } if content == "line2"));
}

/// Test that malformed JSON lines are skipped (ERR-01 fix from spec).
#[test]
fn malformed_json_skipped_in_jsonl() {
    let input = "\
        {\"type\":\"TextDelta\",\"data\":{\"content\":\"good\"}}\n\
        this is not JSON\n\
        {\"type\":\"Usage\",\"data\":{\"prompt_tokens\":1,\"completion_tokens\":1,\"cost_usd\":0.0001}}\n\
        { broken\n";

    let mut parser = kay_tui::jsonl::JsonlParser::new();
    let mut parsed: Vec<TuiEvent> = Vec::new();

    parser.feed(input.as_bytes());
    while let Some(result) = parser.next_event() {
        // ERR-01: malformed JSON is logged and skipped; only Ok events collected.
        if let Ok(event) = result {
            parsed.push(event);
        }
    }

    assert_eq!(
        parsed.len(),
        2,
        "malformed lines should be skipped, only valid JSONL parsed"
    );
}

/// Test that partial JSONL lines are accumulated correctly.
#[test]
fn partial_jsonl_lines_accumulated() {
    let chunk1 = r#"{"type":"TextDelta","data":{"content":"hello"#;
    let chunk2 = r#"world"}}"#;
    let chunk3 = r#"

{"type":"Paused","data":{}}"#;

    let mut parser = kay_tui::jsonl::JsonlParser::new();
    let mut parsed: Vec<TuiEvent> = Vec::new();

    // Step 1: partial line, no newline — should NOT emit
    parser.feed(chunk1.as_bytes());
    let ev1 = parser.next_event();
    assert!(
        ev1.is_none(),
        "should not emit until line is complete. ev1={ev1:?}"
    );

    // Step 2: complete the TextDelta, still no newline — should NOT emit yet
    parser.feed(chunk2.as_bytes());
    let ev2 = parser.next_event();
    assert!(
        ev2.is_none(),
        "should not emit without newline. ev2={ev2:?}"
    );

    // Step 3: add newline + second event — now drain both
    parser.feed(chunk3.as_bytes());
    println!("Draining all events after chunk3...");
    let all = parser.drain_events();
    println!("Drain produced {} items: {all:?}", all.len());
    for result in all {
        if let Ok(event) = result {
            parsed.push(event);
        }
    }
    println!("Parsed {} events: {parsed:?}", parsed.len());

    assert_eq!(
        parsed.len(),
        2,
        "partial lines should be assembled then emitted. got: {parsed:?}"
    );
}
