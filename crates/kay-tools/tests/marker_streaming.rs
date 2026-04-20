//! SHELL-01 + SHELL-03 integration: marker closes stream; frames arrive in order.
#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support/mod.rs"]
mod support;

use std::path::PathBuf;
use std::time::Duration;

use kay_tools::{AgentEvent, ExecuteCommandsTool, Tool, ToolOutputChunk};
use serde_json::json;
use support::{EventLog, make_ctx};

#[tokio::test(flavor = "multi_thread")]
async fn marker_detected_closes_stream() {
    let tool = ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_secs(10));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let _out = tool
        .invoke(json!({"command": "echo hello"}), &ctx, "call_1")
        .await
        .unwrap();

    let events = log.drain();
    let mut saw_stdout = false;
    let mut saw_closed = false;
    for ev in &events {
        match ev {
            AgentEvent::ToolOutput { chunk: ToolOutputChunk::Stdout(s), .. }
                if s.contains("hello") =>
            {
                saw_stdout = true;
            }
            AgentEvent::ToolOutput {
                chunk: ToolOutputChunk::Closed { exit_code: Some(0), marker_detected: true },
                ..
            } => {
                saw_closed = true;
            }
            _ => {}
        }
    }
    assert!(saw_stdout, "expected Stdout(hello) frame; got {events:?}");
    assert!(
        saw_closed,
        "expected Closed{{exit:0, marker:true}}; got {events:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn streams_multiple_lines_in_order() {
    let tool = ExecuteCommandsTool::with_timeout(PathBuf::from("/tmp"), Duration::from_secs(10));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let _out = tool
        .invoke(json!({"command": "printf 'a\\nb\\nc\\n'"}), &ctx, "call_2")
        .await
        .unwrap();

    let events = log.drain();
    let stdout_payloads: Vec<String> = events
        .iter()
        .filter_map(|ev| match ev {
            AgentEvent::ToolOutput { chunk: ToolOutputChunk::Stdout(s), .. } => {
                Some(s.trim().to_string())
            }
            _ => None,
        })
        .collect();

    // The three payloads must appear in order, though the marker line never
    // surfaces (scanner consumes it).
    let joined = stdout_payloads.join(",");
    assert!(joined.contains("a"), "missing a: {joined}");
    assert!(joined.contains("b"), "missing b: {joined}");
    assert!(joined.contains("c"), "missing c: {joined}");
    let pos_a = joined.find('a').unwrap();
    let pos_b = joined.find('b').unwrap();
    let pos_c = joined.find('c').unwrap();
    assert!(pos_a < pos_b && pos_b < pos_c, "out of order: {joined}");

    let closed_ok = events.iter().any(|ev| {
        matches!(
            ev,
            AgentEvent::ToolOutput {
                chunk: ToolOutputChunk::Closed { marker_detected: true, .. },
                ..
            }
        )
    });
    assert!(
        closed_ok,
        "expected Closed{{marker:true}}; events={events:?}"
    );
}
