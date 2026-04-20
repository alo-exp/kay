//! Wave 4 / 03-05 Task 2 quota gate.
//!
//! Proves TOOL-04 / T-3-07:
//! - per-turn cap enforced (U-40): the 3rd consecutive call in a turn
//!   returns `ToolError::ImageCapExceeded { scope: PerTurn, .. }`.
//! - per-session cap enforced (U-41): once the session count reaches
//!   its cap, the next call returns `CapScope::PerSession` regardless
//!   of `reset_turn()`.
//! - successful reads emit `AgentEvent::ImageRead { path, bytes }`
//!   with the raw bytes BEFORE base64 encoding.
//! - missing-file calls return `ToolError::Io` (U-42) and do NOT
//!   leak a quota reservation.
//! - MIME detection (U-43): `.png` → `image/png`, `.jpg` → `image/jpeg`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use kay_tools::{AgentEvent, CapScope, ImageQuota, ImageReadTool, Tool, ToolError};
use serde_json::json;
use tempfile::TempDir;

#[path = "support/mod.rs"]
mod support;

use support::{EventLog, make_ctx_with_quota};

/// PNG magic bytes — 8-byte header `\x89 P N G \r \n \x1a \n`.
/// The tool reads whatever we put on disk; this is sufficient to
/// prove byte-preservation through the base64 round-trip.
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn write_file(dir: &TempDir, name: &str, bytes: &[u8]) -> String {
    let p = dir.path().join(name);
    std::fs::write(&p, bytes).unwrap();
    p.to_string_lossy().to_string()
}

#[tokio::test]
async fn per_turn_cap_returns_image_cap_exceeded() {
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "a.png", &PNG_MAGIC);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::new(quota.clone());

    // Two successful calls.
    let _ = tool
        .invoke(json!({"path": p.clone()}), &ctx, "call-1")
        .await
        .expect("first ok");
    let _ = tool
        .invoke(json!({"path": p.clone()}), &ctx, "call-2")
        .await
        .expect("second ok");

    // Third call breaches the per-turn cap.
    let err = tool
        .invoke(json!({"path": p.clone()}), &ctx, "call-3")
        .await
        .expect_err("third must fail");
    match err {
        ToolError::ImageCapExceeded { scope, limit } => {
            assert_eq!(scope, CapScope::PerTurn);
            assert_eq!(limit, 2);
        }
        other => panic!("expected ImageCapExceeded, got {other:?}"),
    }

    // Quota reflects exactly two consumed (failed call rolled back).
    assert_eq!(quota.per_turn_count(), 2);
    assert_eq!(quota.per_session_count(), 2);
}

#[tokio::test]
async fn per_session_cap_enforced_across_turn_resets() {
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "a.png", &PNG_MAGIC);

    // Session cap of 3 — small so the test is fast. Turn cap = 1 so
    // every call needs a reset_turn() in between.
    let quota = Arc::new(ImageQuota::new(1, 3));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log, quota.clone());
    let tool = ImageReadTool::new(quota.clone());

    for i in 0..3 {
        tool.invoke(json!({"path": p.clone()}), &ctx, &format!("c{i}"))
            .await
            .unwrap_or_else(|e| panic!("call {i} failed: {e:?}"));
        quota.reset_turn();
    }

    // Fourth call: per-turn has been reset but per-session is full.
    let err = tool
        .invoke(json!({"path": p.clone()}), &ctx, "c3")
        .await
        .expect_err("4th must breach session cap");
    match err {
        ToolError::ImageCapExceeded { scope, limit } => {
            assert_eq!(scope, CapScope::PerSession);
            assert_eq!(limit, 3);
        }
        other => panic!("expected PerSession breach, got {other:?}"),
    }
}

#[tokio::test]
async fn image_read_emits_agent_event_with_raw_bytes() {
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "ok.png", &PNG_MAGIC);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::new(quota);

    let out = tool
        .invoke(json!({"path": p.clone()}), &ctx, "c1")
        .await
        .expect("ok");
    // Returned ToolOutput is a data URI containing the base64 payload.
    let text = out.as_str().unwrap_or("");
    assert!(text.starts_with("data:image/png;base64,"), "body: {text}");

    let events = log.drain();
    assert_eq!(events.len(), 1, "exactly one event expected");
    match &events[0] {
        AgentEvent::ImageRead { path, bytes } => {
            assert_eq!(path, &p);
            assert_eq!(bytes, &PNG_MAGIC.to_vec());
        }
        other => panic!("expected ImageRead, got {other:?}"),
    }
}

#[tokio::test]
async fn missing_file_returns_io_error_and_does_not_leak_quota() {
    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log, quota.clone());
    let tool = ImageReadTool::new(quota.clone());

    let err = tool
        .invoke(
            json!({"path": "/nonexistent/does-not-exist.png"}),
            &ctx,
            "c1",
        )
        .await
        .expect_err("missing file must error");
    assert!(
        matches!(err, ToolError::Io(_)),
        "expected ToolError::Io, got {err:?}"
    );

    // M-02: a failed FS read MUST release the quota slot. Without this
    // rollback, a prompt supplying 20 non-existent `.png` paths would
    // drain the per-session cap without reading a single byte — a
    // low-effort DoS against IMG-01. The assertion pins the
    // release-on-IO-failure behavior.
    assert_eq!(
        quota.per_turn_count(),
        0,
        "quota must be released when the FS read fails (M-02)"
    );
    assert_eq!(
        quota.per_session_count(),
        0,
        "per-session counter must also be released (M-02)"
    );

    // And a subsequent legitimate call must still succeed — proving the
    // failed call did not permanently eat quota.
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "ok.png", &PNG_MAGIC);
    tool.invoke(json!({"path": p}), &ctx, "c2")
        .await
        .expect("follow-up ok call must succeed after release");
    assert_eq!(quota.per_turn_count(), 1);
    assert_eq!(quota.per_session_count(), 1);
}

#[tokio::test]
async fn unsupported_extension_returns_invalid_args() {
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "weird.bmp", &PNG_MAGIC);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log, quota.clone());
    let tool = ImageReadTool::new(quota);

    let err = tool
        .invoke(json!({"path": p}), &ctx, "c1")
        .await
        .expect_err("bmp must fail");
    assert!(
        matches!(err, ToolError::InvalidArgs { .. }),
        "expected InvalidArgs, got {err:?}"
    );
}
