//! Wire-schema snapshot lock for `AgentEvent` → `AgentEventWire` JSONL serialization.
//!
//! Phase 5 LOOP-02 / CLI-05 — every variant gets an `insta` snapshot that
//! freezes the on-the-wire JSON form so downstream consumers (kay-cli JSONL
//! stream, Phase 9 Tauri GUI, Phase 9.5 TUI) can rely on the schema NOT
//! drifting silently.
//!
//! Any intentional schema change requires:
//! 1. Updating the corresponding `.snap` file by running `cargo insta review`
//! 2. Bumping the schema version note in `.planning/CONTRACT-AgentEvent.md`
//! 3. Bumping the event-schema version in `kay-cli --version` if user-facing
//!
//! Each test constructs an `AgentEvent`, wraps it in `AgentEventWire`, and
//! snapshots the `serde_json::to_value(&wire)` result so deterministic
//! fixture payloads produce deterministic JSON. Strings with runtime-random
//! content (UUIDs, timestamps) must be frozen to literals here.
//!
//! RED/GREEN cadence (PLAN.md Wave 1 T1.1 → T1.2):
//! - T1.1 RED: tests reference `kay_tools::events_wire::AgentEventWire`
//!   which does NOT exist yet → compile-fail on module path.
//! - T1.2 GREEN: `events_wire.rs` module lands; tests compile; `insta`
//!   creates pending `.snap` files; accept them via `cargo insta accept`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_provider_errors::{ProviderError, RetryReason};
use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::events_wire::AgentEventWire;
use kay_tools::seams::verifier::VerificationOutcome;
use serde_json::Value;

/// Helper: serialize wire form as a `serde_json::Value` so snapshot diffs
/// show clean JSON structure rather than escaped strings.
fn wire_value(event: &AgentEvent) -> Value {
    let wire = AgentEventWire::from(event);
    serde_json::to_value(&wire).expect("AgentEventWire must serialize to JSON")
}

#[test]
fn snap_text_delta() {
    let ev = AgentEvent::TextDelta { content: "Hello, world!".to_string() };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_call_start() {
    let ev = AgentEvent::ToolCallStart {
        id: "call_01".to_string(),
        name: "execute_commands".to_string(),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_call_delta() {
    let ev = AgentEvent::ToolCallDelta {
        id: "call_01".to_string(),
        arguments_delta: "{\"cmd\":\"ls\"}".to_string(),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_call_complete() {
    let ev = AgentEvent::ToolCallComplete {
        id: "call_01".to_string(),
        name: "execute_commands".to_string(),
        arguments: serde_json::json!({"cmd": "ls -la", "cwd": "/tmp"}),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_call_malformed() {
    let ev = AgentEvent::ToolCallMalformed {
        id: "call_02".to_string(),
        raw: "{broken-json".to_string(),
        error: "expected '}' at position 12".to_string(),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_usage() {
    let ev = AgentEvent::Usage {
        prompt_tokens: 1024,
        completion_tokens: 256,
        cost_usd: 0.0025,
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_retry() {
    let ev = AgentEvent::Retry { attempt: 2, delay_ms: 1500, reason: RetryReason::RateLimited };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_error() {
    // ProviderError::Http{status, body} is the cleanest variant to snapshot —
    // it avoids the reqwest::Error (non-deterministic) + serde_json::Error
    // (non-deterministic line/col) variants. Wire form MUST NOT leak internal
    // source-error details that could vary by version.
    let ev = AgentEvent::Error {
        error: ProviderError::Http { status: 503, body: "upstream overloaded".to_string() },
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_output_stdout() {
    let ev = AgentEvent::ToolOutput {
        call_id: "call_01".to_string(),
        chunk: ToolOutputChunk::Stdout("line of output\n".to_string()),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_output_stderr() {
    let ev = AgentEvent::ToolOutput {
        call_id: "call_01".to_string(),
        chunk: ToolOutputChunk::Stderr("warning: unused variable\n".to_string()),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_tool_output_closed() {
    let ev = AgentEvent::ToolOutput {
        call_id: "call_01".to_string(),
        chunk: ToolOutputChunk::Closed { exit_code: Some(0), marker_detected: true },
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_task_complete() {
    let ev = AgentEvent::TaskComplete {
        call_id: "call_03".to_string(),
        verified: false,
        outcome: VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".to_string(),
        },
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_image_read() {
    // Wire MUST NOT inline raw image bytes — they can be multi-MiB. The wire
    // form reports `path` + `size_bytes` + an `encoding: "base64"` indicator
    // with bytes omitted (consumers that want the payload call the tool
    // again locally). See .planning/CONTRACT-AgentEvent.md §ImageRead.
    let ev = AgentEvent::ImageRead {
        path: "/tmp/screenshot.png".to_string(),
        bytes: vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a],
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_sandbox_violation() {
    let ev = AgentEvent::SandboxViolation {
        call_id: "call_04".to_string(),
        tool_name: "fs_write".to_string(),
        resource: "/etc/passwd".to_string(),
        policy_rule: "write-outside-project-root".to_string(),
        os_error: Some(13),
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_sandbox_violation_preflight() {
    // Pre-flight violation: `os_error = None` distinguishes userspace check
    // from kernel denial. QG-C4 carry-forward: filter MUST deny this variant
    // regardless of `os_error` value.
    let ev = AgentEvent::SandboxViolation {
        call_id: "call_05".to_string(),
        tool_name: "net_fetch".to_string(),
        resource: "http://not-allowlisted.example.com".to_string(),
        policy_rule: "net-not-allowlisted".to_string(),
        os_error: None,
    };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_paused() {
    // T1.5: Paused is a unit variant — wire form is `{"type":"paused"}`,
    // no payload. Emitted when ControlMsg::Pause hits the agent loop.
    // Downstream UI (Phase 9 GUI, Phase 9.5 TUI) renders a paused
    // indicator; events between Paused and Resume are buffered by the
    // loop and replayed in order.
    let ev = AgentEvent::Paused;
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_aborted_user_ctrl_c() {
    // T1.5: Ctrl-C cooperative abort after 2s grace period. Reason tag
    // is `"user_ctrl_c"` — consumers switch exhaustively on this string.
    let ev = AgentEvent::Aborted { reason: "user_ctrl_c".to_string() };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_aborted_max_turns() {
    // T1.5: Turn-budget safety net (LOOP-04). Prevents unbounded loops
    // when the model never emits `task_complete`.
    let ev = AgentEvent::Aborted { reason: "max_turns_exceeded".to_string() };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_aborted_verifier_fail() {
    // T1.5: `task_complete` returned `verified: false` and loop policy
    // says fail-fast rather than continue. Phase 8 may refine this.
    let ev = AgentEvent::Aborted { reason: "verifier_fail".to_string() };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_aborted_sandbox_violation_propagated() {
    // T1.5: SandboxViolation event count crossed the abort threshold.
    // Related to QG-C4: the loop MUST abort rather than keep re-feeding
    // the violation to the model (prompt-injection surface). This is
    // how the carry-forward guardrail terminates a runaway session.
    let ev = AgentEvent::Aborted { reason: "sandbox_violation_propagated".to_string() };
    insta::assert_json_snapshot!(wire_value(&ev));
}

// Phase 7 additions (DL-12) — snap these AFTER adding variants to events.rs
#[test]
fn snap_context_truncated_wire() {
    let ev = AgentEvent::ContextTruncated { dropped_symbols: 3, budget_tokens: 7168 };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_index_progress_wire() {
    let ev = AgentEvent::IndexProgress { indexed: 10, total: 100 };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_jsonl_line_format() {
    // T1.3/T1.4: Display impl produces a valid JSONL line — single-line
    // JSON object terminated by exactly one `\n` so stream consumers can
    // delimit events by newline without needing a full JSON streaming
    // parser. Golden format lock — schema-breaking changes must bump
    // CONTRACT-AgentEvent.md + kay-cli --version.
    let ev = AgentEvent::TextDelta { content: "one\ntwo".to_string() };
    let line = format!("{}", AgentEventWire::from(&ev));

    // Must end with exactly one newline
    assert!(
        line.ends_with('\n'),
        "Display output must terminate with '\\n': {line:?}"
    );
    assert!(
        !line.ends_with("\n\n"),
        "Display output must not double-terminate: {line:?}"
    );

    // Embedded newline in `content` must be escaped as `\n` in the JSON
    // (not a literal newline), so the outer line remains single-line.
    let trimmed = line.trim_end_matches('\n');
    assert!(
        !trimmed.contains('\n'),
        "JSON payload must be single-line (embedded newlines must be \\n-escaped): {trimmed:?}"
    );

    // Payload must parse back as valid JSON with expected schema.
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).expect("Display output must be valid JSON");
    assert_eq!(parsed["type"], "text_delta");
    assert_eq!(parsed["content"], "one\ntwo");

    // Full snapshot lock — this is the exact wire bytes consumers see.
    insta::assert_snapshot!(line);
}
