//! Phase 5 Nyquist-audit boundary pin — R-2 `max_image_bytes` at
//! N-1 and N+1.
//!
//! Why this file exists
//! --------------------
//! The R-2 regression set in `tests/image_read_r2.rs` already locks
//! (a) an under-cap read at ~10% of the cap (100 bytes vs 1024),
//! (b) the exact-cap boundary (1024 vs 1024, "at N"), and
//! (c) an over-cap rejection at 2× the cap (2048 vs 1024, "2N").
//!
//! The Nyquist audit called out that the two WORST-CASE adversarial
//! boundaries — one byte under the cap (N-1) and one byte over the
//! cap (N+1) — are both implied by (b) + (c) but NOT directly probed.
//! An off-by-one regression in the check (e.g. changing the existing
//! `meta.len() > self.max_image_bytes` to `>= self.max_image_bytes`)
//! would:
//!
//!   * flip the N-byte boundary test (c) — which is what prevents
//!     that regression today, but
//!   * would ALSO flip a hypothetical N-1 boundary (1023 bytes would
//!     pass through either check, so it stays quiet) and break a
//!     hypothetical N+1 boundary silently (with `>= cap` a 1025-byte
//!     file also gets rejected, so no discernible signal change in
//!     the existing 2N fixture).
//!
//! In short: the *shape* of the comparison is locked by the three
//! existing tests, but the *exact byte* is not. A new engineer
//! wanting to tighten the cap by "just 1 byte" as a micro-hardening
//! measure could flip the operator and land the change with the
//! existing suite all green, because no test covers the 1-byte
//! delta. These two tests close that window.
//!
//! Two tests instead of one
//! ------------------------
//! The two boundaries probe different contract directions and the
//! failure messages should be orthogonal:
//!
//!   * N-1 MUST succeed. If this fails, the cap got tightened by
//!     one byte — a liveness regression for any legitimate image
//!     whose size is exactly the cap. "At N" and "N-1" together
//!     lock the inclusive-cap half of the contract (two points
//!     bracketing the "succeed" interval from above).
//!   * N+1 MUST fail with `ToolError::ImageTooLarge`. If this fails,
//!     the cap got loosened by one byte — a safety regression that
//!     permits a pathological payload one-byte over the intended
//!     ceiling. "2N" and "N+1" together lock the strict-rejection
//!     half (two points bracketing the "reject" interval from below).
//!
//! That's four bracket points total (N-1, N, N+1, 2N) pinning the
//! exact cap byte — the smallest number sufficient to make both
//! operator regressions (`>` → `>=` and `>` → `>`) observable.
//!
//! Reference: Phase 5 Nyquist audit — R-2 coverage gap #GAP-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use kay_tools::{AgentEvent, ImageQuota, ImageReadTool, Tool, ToolError};
use serde_json::json;
use tempfile::TempDir;

#[path = "support/mod.rs"]
mod support;

use support::{make_ctx_with_quota, EventLog};

/// PNG magic bytes (8 bytes) — keeps the fixture recognizable but
/// image_read only detects MIME from the extension so any payload
/// works.
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn write_file(dir: &TempDir, name: &str, bytes: &[u8]) -> String {
    let p = dir.path().join(name);
    std::fs::write(&p, bytes).unwrap();
    p.to_string_lossy().to_string()
}

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
async fn image_read_n_minus_1_byte_succeeds() {
    // 1023-byte file with a 1024-byte cap — one byte UNDER. MUST
    // succeed. If this fails, the cap got tightened (regression from
    // `len > cap` to `len >= cap`). The `len > cap` shape is the only
    // one that lets both N (at) AND N-1 (under) pass.
    let dir = TempDir::new().unwrap();
    let p = write_file(&dir, "nminusone.png", &payload_of_size(1023));

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let out = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-nminus1")
        .await
        .expect(
            "N-1 boundary (1023 bytes vs 1024 cap) MUST succeed — if this \
             fails, the image_read size gate got tightened by one byte",
        );
    let s = out.as_str().unwrap_or("");
    assert!(
        s.starts_with("data:image/png;base64,"),
        "under-cap read at N-1 must return a base64 data URI; got: {s}"
    );

    // Proof of bytes actually read — metadata-first path did not
    // falsely short-circuit a legitimate file.
    let events = log.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::ImageRead { .. })),
        "N-1 boundary read must emit AgentEvent::ImageRead (bytes were read)"
    );

    // Quota reservation committed on success — mirrors the at-cap
    // boundary test's implicit invariant.
    assert_eq!(
        quota.per_turn_count(),
        1,
        "per_turn must count exactly one successful read at the N-1 boundary"
    );
}

#[tokio::test]
async fn image_read_n_plus_1_byte_rejects() {
    // 1025-byte file with a 1024-byte cap — one byte OVER. MUST fail
    // with `ToolError::ImageTooLarge`. If this fails, the cap got
    // loosened (regression that permits a file one byte over the
    // ceiling). The `len > cap` shape is the only one that rejects at
    // exactly N+1 while still accepting at N.
    let dir = TempDir::new().unwrap();
    let oversize = payload_of_size(1025);
    let p = write_file(&dir, "nplusone.png", &oversize);

    let quota = Arc::new(ImageQuota::new(2, 20));
    let log = EventLog::new();
    let ctx = make_ctx_with_quota(log.clone(), quota.clone());
    let tool = ImageReadTool::with_size_cap(quota.clone(), 1024);

    let err = tool
        .invoke(json!({ "path": p.clone() }), &ctx, "call-nplus1")
        .await
        .expect_err(
            "N+1 boundary (1025 bytes vs 1024 cap) MUST reject with \
             ImageTooLarge — if this passes, the image_read size gate \
             got loosened by one byte",
        );
    match err {
        ToolError::ImageTooLarge { path, actual_size, cap } => {
            assert_eq!(path, p, "path echoed");
            assert_eq!(actual_size, 1025, "actual_size = N+1 (1025)");
            assert_eq!(cap, 1024, "cap = N (1024)");
        }
        other => panic!(
            "expected ImageTooLarge for N+1 boundary (1025 vs 1024); got {other:?}"
        ),
    }

    // Metadata-first proof — even at a one-byte overrun the raw file
    // must not be read, mirroring the invariant locked by the
    // `metadata_checked_before_read` test at 2× cap.
    let events = log.drain();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::ImageRead { .. })),
        "N+1 boundary MUST NOT emit AgentEvent::ImageRead — bytes must not \
         be read (pre-read metadata gate); got events: {events:?}"
    );

    // Quota fully released on one-byte overrun — a prompt-injected
    // flood of +1-byte paths must not drain the per-turn cap either.
    assert_eq!(
        quota.per_turn_count(),
        0,
        "per_turn must be zero after N+1 rejection (quota released on \
         over-cap)",
    );
    assert_eq!(
        quota.per_session_count(),
        0,
        "per_session must be zero after N+1 rejection",
    );
}
