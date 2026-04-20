//! SHELL-04 integration: timeout cascade SIGTERM → 2s grace → SIGKILL → reap.
#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support/mod.rs"]
mod support;

use std::time::{Duration, Instant};

use kay_tools::{AgentEvent, ExecuteCommandsTool, Tool, ToolError, ToolOutputChunk};
use serde_json::json;
use support::{EventLog, make_ctx};

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(windows))]
async fn timeout_sigterm_then_sigkill() {
    let tool =
        ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_millis(500));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let start = Instant::now();
    let err = tool
        .invoke(json!({"command": "sleep 30"}), &ctx, "call_t")
        .await
        .expect_err("must time out");
    let elapsed = start.elapsed();

    assert!(
        matches!(err, ToolError::Timeout { .. }),
        "expected Timeout, got {err:?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "cascade must complete within 5s; took {elapsed:?}"
    );

    let events = log.drain();
    let saw_closed_none = events.iter().any(|ev| {
        matches!(
            ev,
            AgentEvent::ToolOutput {
                chunk: ToolOutputChunk::Closed {
                    exit_code: None,
                    marker_detected: false,
                },
                ..
            }
        )
    });
    assert!(
        saw_closed_none,
        "Closed{{None,false}} must be emitted on timeout; events={events:?}"
    );
}
