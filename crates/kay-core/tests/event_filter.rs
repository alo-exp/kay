//! Phase 5 Wave 2 T2.1 RED — per-variant allow/deny tests for
//! `kay_core::event_filter::for_model_context`.
//!
//! ## QG-C4 carry-forward (Phase 4)
//!
//! `AgentEvent::SandboxViolation` MUST NEVER be re-injected into the
//! model's message history. Doing so would teach the model to route
//! around the sandbox policy — a prompt-injection attack surface.
//!
//! This function is the single gate. These tests lock the allow/deny
//! decision for every one of the 14 AgentEvent variants. Coverage is
//! enforced at 100%-line + 100%-branch via the
//! `coverage-event-filter` CI job (T2.5) so any new variant forces a
//! coverage failure — i.e. forces the author to explicitly decide the
//! filter policy for the new variant rather than defaulting silently.
//!
//! ## Policy summary
//!
//! | Variant              | Decision | Rationale                                    |
//! |----------------------|----------|----------------------------------------------|
//! | TextDelta            | allow    | Model's own content — safe to re-feed        |
//! | ToolCallStart        | allow    | Tool dispatch — part of normal loop          |
//! | ToolCallDelta        | allow    | Argument accumulation — no secret data       |
//! | ToolCallComplete     | allow    | Assembled tool call — expected in history    |
//! | ToolCallMalformed    | allow    | Parse error is useful context for the model  |
//! | Usage                | allow    | Token/cost report — diagnostic                |
//! | Retry                | allow    | Backoff signal — diagnostic                   |
//! | Error                | allow    | Upstream error message — diagnostic           |
//! | ToolOutput           | allow    | Tool stdout/stderr is the primary feedback    |
//! | TaskComplete         | allow    | Loop terminator — needed for verifier replay  |
//! | ImageRead            | allow    | Image path metadata (bytes already stripped)  |
//! | **SandboxViolation** | **DENY** | **QG-C4 — prompt-injection surface**          |
//! | Paused               | allow    | Loop control signal — benign                  |
//! | Aborted              | allow    | Loop terminator — model may never see it      |
//!
//! ## Expected RED state (T2.1)
//!
//! `kay_core::event_filter::for_model_context` does not yet exist.
//! `cargo test -p kay-core --test event_filter` fails at compile with
//! E0433 / E0432. T2.2 GREEN adds the module + trivial impl.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_core::event_filter;
use kay_provider_errors::{ProviderError, RetryReason};
use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;

#[test]
fn filter_allows_text_delta() {
    let ev = AgentEvent::TextDelta { content: "Hello".to_string() };
    assert!(
        event_filter::for_model_context(&ev),
        "TextDelta must be re-injectable (model's own content)"
    );
}

#[test]
fn filter_allows_tool_call_start() {
    let ev = AgentEvent::ToolCallStart {
        id: "call_01".to_string(),
        name: "execute_commands".to_string(),
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ToolCallStart must be re-injectable (normal loop flow)"
    );
}

#[test]
fn filter_allows_tool_call_delta() {
    let ev = AgentEvent::ToolCallDelta {
        id: "call_01".to_string(),
        arguments_delta: "{\"cmd\":\"ls\"}".to_string(),
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ToolCallDelta must be re-injectable (argument accumulation)"
    );
}

#[test]
fn filter_allows_tool_call_complete() {
    let ev = AgentEvent::ToolCallComplete {
        id: "call_01".to_string(),
        name: "execute_commands".to_string(),
        arguments: serde_json::json!({"cmd": "ls"}),
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ToolCallComplete must be re-injectable (expected in history)"
    );
}

#[test]
fn filter_allows_tool_call_malformed() {
    let ev = AgentEvent::ToolCallMalformed {
        id: "call_02".to_string(),
        raw: "{broken".to_string(),
        error: "expected '}'".to_string(),
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ToolCallMalformed must be re-injectable (model learns from parse errors)"
    );
}

#[test]
fn filter_allows_usage() {
    let ev = AgentEvent::Usage { prompt_tokens: 100, completion_tokens: 50, cost_usd: 0.001 };
    assert!(
        event_filter::for_model_context(&ev),
        "Usage must be re-injectable (diagnostic)"
    );
}

#[test]
fn filter_allows_retry() {
    let ev = AgentEvent::Retry { attempt: 1, delay_ms: 500, reason: RetryReason::RateLimited };
    assert!(
        event_filter::for_model_context(&ev),
        "Retry must be re-injectable (backoff diagnostic)"
    );
}

#[test]
fn filter_allows_error() {
    let ev = AgentEvent::Error {
        error: ProviderError::Http { status: 503, body: "upstream overloaded".to_string() },
    };
    assert!(
        event_filter::for_model_context(&ev),
        "Error must be re-injectable (upstream diagnostic)"
    );
}

#[test]
fn filter_allows_tool_output() {
    let ev = AgentEvent::ToolOutput {
        call_id: "call_01".to_string(),
        chunk: ToolOutputChunk::Stdout("output line\n".to_string()),
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ToolOutput must be re-injectable (primary tool feedback)"
    );
}

#[test]
fn filter_allows_task_complete() {
    let ev = AgentEvent::TaskComplete {
        call_id: "call_03".to_string(),
        verified: true,
        outcome: VerificationOutcome::Pass { note: "ok".to_string() },
    };
    assert!(
        event_filter::for_model_context(&ev),
        "TaskComplete must be re-injectable (verifier replay)"
    );
}

#[test]
fn filter_allows_image_read() {
    // Note: the wire form omits raw bytes, but the runtime AgentEvent
    // carries them. The filter decision is based on variant, not bytes.
    let ev = AgentEvent::ImageRead {
        path: "/tmp/x.png".to_string(),
        bytes: vec![0x89, 0x50, 0x4e, 0x47],
    };
    assert!(
        event_filter::for_model_context(&ev),
        "ImageRead must be re-injectable (path metadata)"
    );
}

/// **QG-C4 LOAD-BEARING TEST.** Carry-forward from Phase 4 — the reason
/// the event_filter module exists. If this test regresses, the sandbox
/// enforcement is effectively defeated: the model would see the
/// violation event in its history and learn to phrase the next call
/// to avoid detection.
#[test]
fn filter_denies_sandbox_violation() {
    let ev = AgentEvent::SandboxViolation {
        call_id: "call_04".to_string(),
        tool_name: "fs_write".to_string(),
        resource: "/etc/passwd".to_string(),
        policy_rule: "write-outside-project-root".to_string(),
        os_error: Some(13),
    };
    assert!(
        !event_filter::for_model_context(&ev),
        "QG-C4 VIOLATION: SandboxViolation leaked into model context — sandbox defeated"
    );
}

#[test]
fn filter_denies_sandbox_violation_preflight() {
    // Pre-flight violation (userspace check, os_error=None) MUST also
    // be denied. The policy is variant-level, not field-level.
    let ev = AgentEvent::SandboxViolation {
        call_id: "call_05".to_string(),
        tool_name: "net_fetch".to_string(),
        resource: "http://not-allowlisted.example.com".to_string(),
        policy_rule: "net-not-allowlisted".to_string(),
        os_error: None,
    };
    assert!(
        !event_filter::for_model_context(&ev),
        "QG-C4 VIOLATION: pre-flight SandboxViolation leaked (os_error=None branch)"
    );
}

#[test]
fn filter_allows_paused() {
    let ev = AgentEvent::Paused;
    assert!(
        event_filter::for_model_context(&ev),
        "Paused must be re-injectable (benign control signal)"
    );
}

#[test]
fn filter_allows_aborted() {
    let ev = AgentEvent::Aborted { reason: "user_ctrl_c".to_string() };
    assert!(
        event_filter::for_model_context(&ev),
        "Aborted must be re-injectable (loop terminator — model won't see next turn)"
    );
}
