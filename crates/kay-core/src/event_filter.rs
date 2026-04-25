//! Event filter — decides which `AgentEvent` variants are safe to
//! re-inject into the model's message history.
//!
//! ## QG-C4 (carry-forward from Phase 4)
//!
//! `AgentEvent::SandboxViolation` MUST NEVER be re-fed to the model.
//! Doing so would teach the model the exact policy rule that blocked
//! its last action — a prompt-injection attack surface where the model
//! learns to phrase the next call to evade the sandbox rather than
//! respect it. See `.planning/phases/04-sandbox/04-QUALITY-GATES.md`
//! §QG-C4 for the full rationale and threat model.
//!
//! This module is the single choke-point enforcing that policy. It is:
//!
//! - **Security-critical.** Any regression is a CI SHIP BLOCK (the
//!   `coverage-event-filter` job enforces 100%-line + 100%-branch).
//! - **Tested variant-by-variant.** See
//!   `crates/kay-core/tests/event_filter.rs` (T2.1) for a test per
//!   `AgentEvent` variant locking the allow/deny decision — 15 cases
//!   total (13 allow + 2 deny, the latter are the two
//!   `SandboxViolation` shapes `os_error=Some(errno)` kernel-denial
//!   vs. `os_error=None` pre-flight).
//! - **Proptest-proven.** `tests/event_filter_property.rs` (T2.3 + T2.4)
//!   runs 10,000 random sequences of up to 20 `AgentEvent`s each
//!   (~100k filter calls per run) and asserts the bi-conditional
//!   invariant `for_model_context(&ev) == !matches!(ev, SandboxViolation { .. })`
//!   for every event. The property holds by construction — the
//!   filter's implementation (`!matches!(event, …)`) is literally the
//!   definition of the property — so the proptest is a regression
//!   tripwire rather than a discovery harness. Any future refactor
//!   that accidentally introduces field-dependent allow/deny behavior
//!   (e.g., "allow `SandboxViolation` when `os_error=None`") fails
//!   loudly within 10k cases.
//!
//! ## Policy choice: deny-explicit, allow-default
//!
//! `#[non_exhaustive]` on `AgentEvent` means cross-crate matches (we
//! are cross-crate to `kay-tools`) must handle unknown future
//! variants. We chose **allow-default** (safer for the normal
//! model-feedback loop) over **deny-default** (safer against unknown
//! future variants) because:
//!
//! 1. The sole deny case (`SandboxViolation`) is explicitly named,
//!    so reviewers see the filter matrix at PR time.
//! 2. Any new variant is an additive schema change that requires
//!    bumping `CONTRACT-AgentEvent.md` + adding a new snapshot in
//!    `events_wire_snapshots.rs`. The author of the new variant is
//!    forced to think about wire form — and at that point, extending
//!    `tests/event_filter.rs` with a new test is a natural next step
//!    (the test file lists one test per variant, so the pattern is
//!    visible).
//! 3. Deny-default would accidentally block benign additions like a
//!    future `Progress` or `ToolWarning` event, forcing noise in
//!    every PR that touches `AgentEvent` just to re-allow the new
//!    variant.
//!
//! The `matches!` macro yields exactly 2 branch points (match /
//! no-match for `SandboxViolation`), both covered by the unit tests —
//! so `coverage-event-filter` passes 100%-branch without relying on
//! unreachable wildcard arms.

use kay_tools::events::AgentEvent;

/// Returns `true` if `event` is safe to include in the message
/// history fed back to the model, `false` if it MUST be routed only
/// to the UI/user event stream.
///
/// # QG-C4 invariant
///
/// `for_model_context(&AgentEvent::SandboxViolation { .. })` MUST
/// return `false` for every possible field combination. This is
/// proptest-guarded in `tests/event_filter_property.rs`.
///
/// # Examples
///
/// ```
/// use kay_core::event_filter::for_model_context;
/// use kay_tools::events::AgentEvent;
///
/// let safe = AgentEvent::TextDelta { content: "hi".to_string() };
/// assert!(for_model_context(&safe));
///
/// let blocked = AgentEvent::SandboxViolation {
///     call_id: "c1".to_string(),
///     tool_name: "fs_write".to_string(),
///     resource: "/etc/passwd".to_string(),
///     policy_rule: "write-outside-project-root".to_string(),
///     os_error: Some(13),
/// };
/// assert!(!for_model_context(&blocked));
/// ```
pub fn for_model_context(event: &AgentEvent) -> bool {
    !matches!(event, AgentEvent::SandboxViolation { .. })
}

// M12-Task 6: Inline unit tests for kay-core event_filter module.
// These complement the exhaustive integration tests in tests/event_filter.rs
// with quick synchronous assertions on the QG-C4 invariant.

#[cfg(test)]
mod unit {
    use super::*;
    use kay_tools::events::AgentEvent;
    use kay_tools::seams::verifier::VerificationOutcome;

    // QG-C4: SandboxViolation MUST return false from for_model_context.
    #[test]
    fn sandbox_violation_denied_from_model_context() {
        let ev = AgentEvent::SandboxViolation {
            call_id: "c1".to_string(),
            tool_name: "fs_write".to_string(),
            resource: "/etc/passwd".to_string(),
            policy_rule: "project_root_only".to_string(),
            os_error: Some(13),
        };
        assert!(!for_model_context(&ev), "QG-C4: SandboxViolation must be blocked from model context");
    }

    #[test]
    fn sandbox_violation_without_os_error_denied() {
        // os_error=None case — pre-flight denial — also must be blocked.
        let ev = AgentEvent::SandboxViolation {
            call_id: "c2".to_string(),
            tool_name: "fs_write".to_string(),
            resource: "/etc/evil".to_string(),
            policy_rule: "project_root_only".to_string(),
            os_error: None,
        };
        assert!(!for_model_context(&ev), "QG-C4: pre-flight SandboxViolation must also be blocked");
    }

    // All other variants must be allowed.
    #[test]
    fn text_delta_allowed() {
        let ev = AgentEvent::TextDelta { content: "hello".to_string() };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn task_complete_pass_allowed() {
        let ev = AgentEvent::TaskComplete {
            call_id: "done".to_string(),
            verified: true,
            outcome: VerificationOutcome::Pass { note: "ok".to_string() },
        };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn task_complete_fail_allowed() {
        let ev = AgentEvent::TaskComplete {
            call_id: "fail".to_string(),
            verified: false,
            outcome: VerificationOutcome::Fail { reason: "assertion mismatch".to_string() },
        };
        // Even a failing task outcome is safe to show the model (it describes what went wrong).
        assert!(for_model_context(&ev));
    }

    #[test]
    fn usage_allowed() {
        let ev = AgentEvent::Usage { prompt_tokens: 100, completion_tokens: 50, cost_usd: 0.002 };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn retry_allowed() {
        use kay_provider_errors::RetryReason;
        let ev = AgentEvent::Retry { attempt: 1, delay_ms: 500, reason: RetryReason::RateLimited };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn aborted_allowed() {
        let ev = AgentEvent::Aborted { reason: "user_abort".to_string() };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn tool_call_start_allowed() {
        let ev = AgentEvent::ToolCallStart {
            id: "call_1".to_string(),
            name: "fs_read".to_string(),
        };
        assert!(for_model_context(&ev));
    }

    #[test]
    fn context_truncated_allowed() {
        let ev = AgentEvent::ContextTruncated { dropped_symbols: 512, budget_tokens: 8192 };
        assert!(for_model_context(&ev));
    }
}
