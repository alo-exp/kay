//! Display-form snapshot lock for `ToolError::ImageTooLarge` (R-2).
//!
//! Why a dedicated snapshot test for an error's Display?
//! ------------------------------------------------------
//! `ToolError::ImageTooLarge` is NOT an `AgentEvent` — it surfaces
//! from `Tool::invoke` as `Err(ToolError::ImageTooLarge { … })` and
//! is what the turn-loop sees when R-2's metadata gate rejects an
//! over-cap `image_read` call. The loop is free to render that error
//! in a few places:
//!
//!   * as a `ToolResult.content` string when the tool output is
//!     re-injected into the model context (opaque to the user, but
//!     token-stable so diff-based turn replays don't drift);
//!   * as a JSONL `error` frame on the kay-cli output stream (the
//!     stream consumers — TUI/GUI — expect a stable string shape);
//!   * as a `tracing::warn!` field for debugging.
//!
//! Freezing the Display form with `insta` gives us a single
//! committed snapshot to review when the variant's fields or format
//! string change. A silent tweak (e.g. switching `{path:?}` →
//! `{path}` or dropping the "byte cap" tail) would ripple through
//! every consumer at once and is exactly what this snapshot catches.
//!
//! Reference: `.planning/REQUIREMENTS.md` R-2,
//! `.planning/phases/05-agent-loop/05-PLAN.md` Wave 6b, T6b.4.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_tools::ToolError;

#[test]
fn snap_image_too_large_display() {
    // Deterministic fixture values so the snapshot diff is stable:
    //   path        — a realistic-looking but synthetic path
    //   actual_size — 2 MiB
    //   cap         — 1 MiB
    // The {path:?} debug-print quotes the string literal, which is
    // the exact wire-surface today and what downstream consumers
    // must not be surprised by.
    let err = ToolError::ImageTooLarge {
        path: "/tmp/evil-20gb.img".to_string(),
        actual_size: 2 * 1024 * 1024,
        // 1 MiB, written as `1024 * 1024` rather than `1 * 1024 * 1024`
        // so `clippy::identity_op` stays silent under `-D warnings`.
        cap: 1024 * 1024,
    };
    insta::assert_snapshot!(err.to_string());
}
