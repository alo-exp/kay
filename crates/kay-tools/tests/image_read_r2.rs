//! R-2 regression — `image_read` enforces a `max_image_bytes` cap via a
//! metadata-size check BEFORE the file's raw bytes are read into memory.
//!
//! Why the cap matters
//! -------------------
//! Without it, a prompt-injected `image_read {"path": "/tmp/20GB.img"}`
//! call allocates 20 GiB into `Vec<u8>` via `tokio::fs::read`, blows the
//! process memory ceiling, and kills the agent. Metadata-first check
//! rejects the call with `ToolError::ImageTooLarge { path, actual_size,
//! cap }` before any byte is read, and rolls the quota reservation back
//! (consistent with the `ImageCapExceeded` / `Io` handling already in
//! place).
//!
//! These five tests lock:
//!   1. under-cap read succeeds and emits the expected
//!      `AgentEvent::ImageRead` frame;
//!   2. over-cap read returns `ToolError::ImageTooLarge` AND skips the
//!      `ImageRead` event emission (proves metadata-first ordering);
//!   3. at-cap-boundary (exactly `max_image_bytes`) succeeds — cap is
//!      inclusive;
//!   4. over-cap call does not emit an `ImageRead` event (bytes never
//!      read into memory) — the "metadata-checked-before-read"
//!      observable guarantee;
//!   5. the default cap for `ImageReadTool::new(_)` is exactly 20 MiB.
//!
//! Reference: `.planning/REQUIREMENTS.md` R-2,
//! `.planning/phases/05-agent-loop/05-PLAN.md` Wave 6b.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use kay_tools::{
    AgentEvent, ImageQuota, ImageReadTool, Tool, ToolError, DEFAULT_MAX_IMAGE_BYTES,
};
use serde_json::json;
use tempfile::TempDir;

#[path = "support/mod.rs"]
mod support;

use support::{make_ctx_with_quota, EventLog};

/// PNG magic bytes (8 bytes). image_read only inspects the extension
/// for MIME detection, so any payload works — this just keeps the
/// fixture recognizable.
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn write_file(dir: &TempDir, name: &str, bytes: &[u8]) -> String {
    let p = dir.path().join(name);
    std::fs::write(&p, bytes).unwrap();
    p.to_string_lossy().to_string()
}

/// Build a test payload of `len` bytes starting with PNG magic. The
/// rest is zero padding — we never decode it, so content doesn't
/// matter.
fn payload_of_size(len: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(len);
    buf.extend_from_slice(&PNG_MAGIC);
    if len > buf.len() {
        buf.resize(len, 0u8);
    } else {
        buf.truncate(len);
    }
    buf
}

#[tokio::test]
async fn image_read_under_cap_succeeds() {
    // 100-byte file with a 1 KiB cap — well under.
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "small.png", &payload_of_size(100));

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let out = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-under")
        .await
        .expect("under-cap read must succeed");
    let s = out.as_str().unwrap_or("");
    assert!(
        s.starts_with("data:image/png;base64,"),
        "expected a base64 data URI, got: {s}"
    );
    let events = log.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::ImageRead { .. })),
        "under-cap read must emit AgentEvent::ImageRead"
    );
}

#[tokio::test]
async fn image_read_over_cap_returns_image_too_large() {
    // 2 KiB file with a 1 KiB cap — over.
    let dir = TempDir::new().unwrap();
    let large = payload_of_size(2 * 1024);
    let p = write_file(&dir, "big.png", &large);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let err = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-over")
        .await
        .expect_err("over-cap read must fail with ImageTooLarge");
    match err {
        ToolError::ImageTooLarge { path, actual_size, cap } => {
            assert_eq!(path, p, "path field must echo the requested path");
            assert_eq!(actual_size, 2 * 1024, "actual_size must be the file's real length");
            assert_eq!(cap, 1024, "cap must be the tool's configured max_image_bytes");
        }
        other => panic!("expected ImageTooLarge, got {other:?}"),
    }
    // Quota must be fully released on over-cap rejection — this failed
    // call should NOT count toward per-turn / per-session caps.
    assert_eq!(quota.per_turn_count(), 0, "per_turn must be zero after over-cap rejection");
    assert_eq!(quota.per_session_count(), 0, "per_session must be zero after over-cap rejection");
}

#[tokio::test]
async fn image_read_at_cap_boundary_succeeds() {
    // Exactly 1024 bytes with cap=1024 — boundary MUST succeed (cap is
    // inclusive). Without this test a strict `>= cap` check would
    // silently regress file-size boundary users.
    let dir = TempDir::new().unwrap();
    let exact = payload_of_size(1024);
    let p = write_file(&dir, "exact.png", &exact);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let _ = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-exact")
        .await
        .expect("exact-cap boundary must succeed (cap is inclusive)");
}

#[tokio::test]
async fn image_read_metadata_checked_before_read() {
    // Proves "metadata-first" ordering. If over-cap was only detected
    // AFTER `tokio::fs::read` completed, an `AgentEvent::ImageRead`
    // would still leak to the event log because the raw bytes would
    // have been read. Post-R-2 the metadata check gates the read, so
    // the event is NEVER emitted for an over-cap file.
    let dir = TempDir::new().unwrap();
    let large = payload_of_size(2 * 1024);
    let p = write_file(&dir, "toolarge.png", &large);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let _ = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-metadata")
        .await
        .expect_err("must reject");

    let events = log.drain();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::ImageRead { .. })),
        "over-cap read MUST NOT emit AgentEvent::ImageRead — \
         raw bytes must not have been read (pre-read metadata check)"
    );
}

#[test]
fn image_read_default_cap_20mib() {
    // The default cap, applied by `ImageReadTool::new(_)`, must be
    // exactly 20 MiB = 20 * 1024 * 1024 bytes. This is the public
    // contract exposed as `DEFAULT_MAX_IMAGE_BYTES`.
    assert_eq!(
        DEFAULT_MAX_IMAGE_BYTES,
        20 * 1024 * 1024,
        "DEFAULT_MAX_IMAGE_BYTES must be 20 MiB"
    );
    let quota = Arc::new(ImageQuota::new(2, 20));
    let tool = ImageReadTool::new(quota);
    assert_eq!(
        tool.max_image_bytes(),
        DEFAULT_MAX_IMAGE_BYTES,
        "ImageReadTool::new must apply the 20 MiB default cap"
    );
}
