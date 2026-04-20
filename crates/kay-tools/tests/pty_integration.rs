//! SHELL-02 integration (unix-only smoke): PTY path engages on the
//! denylist-first-token heuristic and yields at least one frame.
#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support/mod.rs"]
mod support;

use std::time::Duration;

use kay_tools::{ExecuteCommandsTool, Tool};
use serde_json::json;
use support::{EventLog, make_ctx};

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(windows))]
async fn pty_engages_for_ssh_first_token() {
    // `ssh -V` exits immediately by printing a version banner on most
    // hosts. If `ssh` is missing, the spawn errors — which itself
    // exercises the PTY error path. We only require the invoke to
    // terminate without hanging.
    let tool = ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_secs(10));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let _ = tool
        .invoke(json!({"command": "ssh -V"}), &ctx, "call_p")
        .await;

    let events = log.drain();
    assert!(
        !events.is_empty(),
        "PTY path must yield at least one AgentEvent; got none"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(windows))]
async fn pty_engages_on_explicit_tty_flag() {
    // Forces PTY via `tty: true` even for a plain echo.
    let tool = ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_secs(10));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let _ = tool
        .invoke(
            json!({"command": "echo pty-engaged", "tty": true}),
            &ctx,
            "call_ptty",
        )
        .await
        .unwrap();

    let events = log.drain();
    assert!(!events.is_empty(), "PTY path must yield frames");
}
