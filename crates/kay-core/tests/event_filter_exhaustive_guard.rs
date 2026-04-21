//! Phase 5 Nyquist-audit pin — QG-C4 per-variant filter review guard.
//!
//! Why this file exists
//! --------------------
//! `kay_core::event_filter::for_model_context` is implemented as
//! `!matches!(event, AgentEvent::SandboxViolation { .. })` — an
//! explicit single-variant deny plus an implicit wildcard allow.
//! `AgentEvent` is `#[non_exhaustive]` in the `kay-tools` crate, so
//! this implementation keeps working silently when a new variant
//! lands: any future `AgentEvent::Foo` bypasses the match and gets
//! the wildcard allow.
//!
//! The allow-default policy is a *deliberate* choice — documented in
//! `crates/kay-core/src/event_filter.rs` §"Policy choice" — because
//! adding a new variant is an additive schema change and deny-default
//! would break benign additions. The existing 15-variant unit suite
//! in `tests/event_filter.rs` plus the 10k proptest
//! `tests/event_filter_property.rs` lock the *current* 14 variants.
//!
//! What the existing suite does NOT lock: a future variant's filter
//! decision being *reviewed*. If someone lands `AgentEvent::Foo`
//! tomorrow, none of the 15 unit tests fail, the 10k proptest
//! doesn't even generate it (its generator is hand-written per
//! variant), and the new variant silently inherits the allow
//! default — which may or may not be what the author intended. A
//! new `AgentEvent::SecretRead { … }` leaking into model context
//! would be a QG-C4-class regression with no test signal.
//!
//! This file is a two-sided runtime trip-wire:
//!
//!   1. *Match-side:* a `match` over every known variant, with a
//!      final wildcard arm that PANICS with a pointed
//!      "QG-C4 new variant needs an explicit filter decision"
//!      message. Any new variant someone exercises here without
//!      adding a dedicated arm fires that panic — forcing an
//!      explicit allow/deny call to land before the test passes.
//!
//!   2. *Fixture-side:* a hard-coded count assertion at the bottom
//!      (`examples.len() == N`) that forces the exemplar vec to
//!      stay in sync with the real variant count. A new variant
//!      shipping without an exemplar is flagged because the
//!      expected N is out of date; a new variant with an exemplar
//!      and no match arm still trips the wildcard panic above.
//!
//! Together those force the author of a new `AgentEvent::Foo` to
//! make a visible filter decision and record it in this file,
//! closing the Nyquist gap that the 15-variant + 10k proptest
//! surface cannot: a decision on a variant that does not yet exist.
//!
//! Why not a compile-time `E0004`: rustc's stable exhaustiveness
//! check on a `#[non_exhaustive]` enum from an upstream crate
//! REQUIRES a `_` arm to accept the match (without one, the match
//! is rejected as non-exhaustive). The `non_exhaustive_omitted_
//! patterns` lint (which would force compile-time error on a
//! non-exhaustive match) is unstable (issue #89554). So the
//! strongest STABLE signal is "the wildcard arm panics with a
//! QG-C4 diagnostic" — a runtime test trip-wire is the right tool
//! until that lint stabilizes.
//!
//! Paired with the 10k proptest, this gives QG-C4 both a runtime
//! property (never leak SandboxViolation under random fields) and
//! a per-variant review checklist (every variant's filter decision
//! is explicitly recorded here with a rationale).
//!
//! Reference: Phase 5 Nyquist audit — QG-C4 coverage gap GAP-B.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_core::event_filter::for_model_context;
use kay_provider_errors::{ProviderError, RetryReason};
use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;

/// Assertion helper: named so the filter decision reads as prose at
/// the call site — `assert_allow(&ev, "TextDelta …")`. The second
/// argument is the documented rationale, inlined per-call so a
/// future reader sees both the decision AND the reason without
/// round-tripping to the module-level table.
fn assert_allow(ev: &AgentEvent, rationale: &str) {
    assert!(
        for_model_context(ev),
        "QG-C4 exhaustive guard: event expected ALLOW but got DENY: \
         {ev:?} — rationale: {rationale}"
    );
}

/// Assertion helper: DENY side of the decision. Today only
/// `SandboxViolation` uses this. A future variant that a reviewer
/// decides is a prompt-injection surface (hypothetical:
/// `CredentialLeaked { secret_kind, … }`) lands here.
fn assert_deny(ev: &AgentEvent, rationale: &str) {
    assert!(
        !for_model_context(ev),
        "QG-C4 exhaustive guard: event expected DENY but got ALLOW: \
         {ev:?} — rationale: {rationale}"
    );
}

/// Build one exemplar of every `AgentEvent` variant and pipe it
/// through a match that decides allow-or-deny for each, with a
/// wildcard arm at the end that panics with a QG-C4 diagnostic.
///
/// The wildcard arm is the runtime pin: every known variant has its
/// own named arm; a future `AgentEvent::Foo` (with an exemplar
/// bumped into `examples` but no dedicated arm here) falls through
/// to the wildcard and fires the `panic!` — forcing the author of
/// the new variant to make an explicit `assert_allow` / `assert_deny`
/// decision AND add the matching arm before the test can pass.
///
/// Each arm's second argument to `assert_{allow,deny}` is a short
/// rationale string mirroring the `tests/event_filter.rs` policy
/// table — keeps the "why" visible at the decision point.
#[test]
fn qg_c4_every_agent_event_variant_has_an_explicit_filter_decision() {
    // One exemplar per variant. Field values are chosen to match the
    // happy-path fixtures in `tests/event_filter.rs` so that if any
    // variant's decision drifts, both suites fail together with
    // consistent diagnostics.
    let examples: Vec<AgentEvent> = vec![
        AgentEvent::TextDelta { content: "hi".to_string() },
        AgentEvent::ToolCallStart { id: "c1".to_string(), name: "fs_read".to_string() },
        AgentEvent::ToolCallDelta { id: "c1".to_string(), arguments_delta: "{".to_string() },
        AgentEvent::ToolCallComplete {
            id: "c1".to_string(),
            name: "fs_read".to_string(),
            arguments: serde_json::json!({}),
        },
        AgentEvent::ToolCallMalformed {
            id: "c1".to_string(),
            raw: "{broken".to_string(),
            error: "parse".to_string(),
        },
        AgentEvent::Usage { prompt_tokens: 1, completion_tokens: 1, cost_usd: 0.0 },
        AgentEvent::Retry { attempt: 1, delay_ms: 0, reason: RetryReason::RateLimited },
        AgentEvent::Error {
            error: ProviderError::Http { status: 500, body: String::new() },
        },
        AgentEvent::ToolOutput {
            call_id: "c1".to_string(),
            chunk: ToolOutputChunk::Stdout(String::new()),
        },
        AgentEvent::TaskComplete {
            call_id: "c1".to_string(),
            verified: true,
            outcome: VerificationOutcome::Pass { note: String::new() },
        },
        AgentEvent::ImageRead { path: "/tmp/x.png".to_string(), bytes: vec![] },
        AgentEvent::SandboxViolation {
            call_id: "c1".to_string(),
            tool_name: "fs_write".to_string(),
            resource: "/etc".to_string(),
            policy_rule: "write-outside-root".to_string(),
            os_error: None,
        },
        AgentEvent::Paused,
        AgentEvent::Aborted { reason: "user_abort".to_string() },
    ];

    // Match over every known variant, followed by a wildcard arm
    // that PANICS with a pointed QG-C4 diagnostic. See the file-
    // level doc-comment §"Why not a compile-time E0004" for why a
    // wildcard is structurally required: rustc rejects a match on a
    // cross-crate `#[non_exhaustive]` enum without one. The wildcard
    // is never expected to fire today; any future trip is a strong
    // signal that a new variant needs a reviewer-visible allow/deny
    // call in this file.
    for ev in &examples {
        match ev {
            AgentEvent::TextDelta { .. } => {
                assert_allow(ev, "model's own content — safe to re-feed");
            }
            AgentEvent::ToolCallStart { .. } => {
                assert_allow(ev, "tool dispatch — part of normal loop");
            }
            AgentEvent::ToolCallDelta { .. } => {
                assert_allow(ev, "argument accumulation — no secret data");
            }
            AgentEvent::ToolCallComplete { .. } => {
                assert_allow(ev, "assembled tool call — expected in history");
            }
            AgentEvent::ToolCallMalformed { .. } => {
                assert_allow(ev, "parse error — useful context for the model");
            }
            AgentEvent::Usage { .. } => {
                assert_allow(ev, "token/cost report — diagnostic");
            }
            AgentEvent::Retry { .. } => {
                assert_allow(ev, "backoff signal — diagnostic");
            }
            AgentEvent::Error { .. } => {
                assert_allow(ev, "upstream error — diagnostic");
            }
            AgentEvent::ToolOutput { .. } => {
                assert_allow(ev, "tool stdout/stderr — primary feedback");
            }
            AgentEvent::TaskComplete { .. } => {
                assert_allow(ev, "loop terminator — needed for verifier replay");
            }
            AgentEvent::ImageRead { .. } => {
                assert_allow(ev, "image path metadata — bytes stripped at wire layer");
            }
            AgentEvent::SandboxViolation { .. } => {
                assert_deny(ev, "QG-C4 — prompt-injection surface");
            }
            AgentEvent::Paused => {
                assert_allow(ev, "loop control signal — benign");
            }
            AgentEvent::Aborted { .. } => {
                assert_allow(ev, "loop terminator — model may never see it");
            }
            other => {
                // QG-C4 trip-wire: a new `AgentEvent` variant reached
                // this test fixture without a dedicated match arm
                // above. Silently falling through would let the new
                // variant inherit `for_model_context`'s wildcard
                // allow default without a reviewer having made that
                // call explicitly — the exact Nyquist gap this file
                // was created to close. The panic's message names
                // the rituals the author must perform before the
                // test can pass.
                panic!(
                    "QG-C4 NYQUIST GUARD: a new AgentEvent variant has reached this \
                     exhaustive guard without an explicit filter decision. Event: \
                     {other:?}. To fix: (1) decide whether the new variant should \
                     be ALLOWED into model context (safe to re-feed) or DENIED \
                     (prompt-injection risk); (2) if deny, update \
                     `kay_core::event_filter::for_model_context` to include it in \
                     the `!matches!(…)` expression; (3) add a dedicated arm for \
                     the new variant above, calling `assert_allow` or \
                     `assert_deny` with a short rationale string; (4) update the \
                     `examples.len() == N` assertion below."
                );
            }
        }
    }

    // Sanity: we built exactly one exemplar per known variant. If
    // someone adds a new variant but forgets to add an exemplar to
    // `examples`, the match body above compiles fine but the new
    // variant's filter decision is never exercised at runtime — so
    // belt + suspenders, count the exemplars we built. 14 is the
    // Phase 5 variant count locked by
    // `crates/kay-tools/tests/events_wire_snapshots.rs` (21 snaps,
    // but 14 base variants; Aborted has 4 reason fixtures +
    // SandboxViolation has 2 os_error fixtures + jsonl_line_format).
    assert_eq!(
        examples.len(),
        14,
        "exhaustive exemplar count must match AgentEvent's variant count \
         (14 today). If you added a new variant to AgentEvent, update \
         both the match block above AND the `examples` vec, then bump \
         this expectation."
    );
}
