//! Phase 5 Wave 2 T2.3 RED — proptest over 10k random `AgentEvent`
//! sequences asserting `for_model_context` NEVER leaks a
//! `SandboxViolation` into the model's message history.
//!
//! ## QG-C4 (carry-forward from Phase 4)
//!
//! `AgentEvent::SandboxViolation` MUST NEVER be re-fed to the model.
//! The unit tests in `tests/event_filter.rs` (T2.1) lock one happy-path
//! fixture per variant — this file probes the invariant with 10k
//! random field combinations so that a future refactor cannot
//! accidentally introduce field-dependent behavior (e.g., "allow
//! SandboxViolation when os_error is None" regression, or partial
//! matching on a field).
//!
//! ## Property statement
//!
//! For any `AgentEvent` drawn from the full product space of
//! variants + fields:
//!
//!   `for_model_context(&ev)  ==  !matches!(ev, SandboxViolation { .. })`
//!
//! This is a bi-conditional — BOTH directions matter:
//!
//! 1. **Leak direction (security-critical):** if `ev` is a
//!    `SandboxViolation`, the filter MUST return `false`. A
//!    regression here defeats QG-C4 — the sandbox is effectively
//!    disabled because the model learns the policy rule that blocked
//!    it and phrases the next call to evade.
//! 2. **Block direction (liveness):** if `ev` is NOT a
//!    `SandboxViolation`, the filter MUST return `true`. A
//!    regression here breaks the normal model-feedback loop
//!    (TextDelta / ToolOutput / TaskComplete / etc. must round-trip
//!    back to the model or the loop hangs).
//!
//! ## Why a proxy `EventSpec` instead of a direct `Arbitrary for AgentEvent`
//!
//! `AgentEvent` does NOT implement `Clone` — `ProviderError` carries
//! `reqwest::Error` and `serde_json::Error`, neither of which is
//! `Clone`. Proptest's shrinking requires generated values to be
//! `Clone + Debug`, so we generate a `Clone`-able `EventSpec` proxy
//! and convert to `AgentEvent` inside the test body. The proxy
//! mirrors every variant and field of `AgentEvent` one-to-one; for
//! `ProviderError` we enumerate only the 9 constructible variants
//! (skipping `Network(reqwest::Error)` and `Serialization(serde_json::Error)`
//! which have no public constructors — the filter does not inspect
//! `ProviderError` anyway, so variant coverage inside `AgentEvent::Error`
//! is what matters).
//!
//! ## Expected RED state (T2.3)
//!
//! This file was authored after `event_filter::for_model_context`
//! already exists (T2.2 GREEN). Per PLAN.md §3 T2.4:
//!
//!   > "Property holds by construction (filter matches on variant
//!   >  discriminant). If proptest fails, iterate until green."
//!
//! So the first run of this proptest is expected to be GREEN. The
//! RED-labeled commit here captures the property spec; T2.4 promotes
//! the filter's module-level doc to reference this file as the
//! proven-safe proptest guardrail.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_arguments)]

use std::time::Duration;

use kay_core::event_filter::for_model_context;
use kay_provider_errors::{AuthErrorKind, ProviderError, RetryReason};
use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;
use proptest::prelude::*;

// ---------------------------------------------------------------------
// Clone-safe proxies (see module doc §"Why a proxy EventSpec")
// ---------------------------------------------------------------------

/// Proxy for the 9 `Clone`-constructible `ProviderError` variants.
/// `Network(reqwest::Error)` and `Serialization(serde_json::Error)` are
/// intentionally absent — they have no public constructors. The filter
/// treats all `Error` variants identically (variant-level allow), so
/// this does not undermine the property.
#[derive(Debug, Clone)]
enum ProviderErrorSpec {
    Http { status: u16, body: String },
    RateLimited { retry_ms: Option<u64> },
    ServerError { status: u16 },
    Auth { reason: AuthErrorKind },
    ModelNotAllowlisted { requested: String, allowed: Vec<String> },
    CostCapExceeded { cap_usd: f64, spent_usd: f64 },
    ToolCallMalformed { id: String, error: String },
    Stream(String),
    Canceled,
}

impl ProviderErrorSpec {
    fn into_error(self) -> ProviderError {
        match self {
            Self::Http { status, body } => ProviderError::Http { status, body },
            Self::RateLimited { retry_ms } => ProviderError::RateLimited {
                retry_after: retry_ms.map(Duration::from_millis),
            },
            Self::ServerError { status } => ProviderError::ServerError { status },
            Self::Auth { reason } => ProviderError::Auth { reason },
            Self::ModelNotAllowlisted { requested, allowed } => {
                ProviderError::ModelNotAllowlisted { requested, allowed }
            }
            Self::CostCapExceeded { cap_usd, spent_usd } => {
                ProviderError::CostCapExceeded { cap_usd, spent_usd }
            }
            Self::ToolCallMalformed { id, error } => {
                ProviderError::ToolCallMalformed { id, error }
            }
            Self::Stream(s) => ProviderError::Stream(s),
            Self::Canceled => ProviderError::Canceled,
        }
    }
}

/// One-to-one Clone proxy for every `AgentEvent` variant. Field types
/// are chosen to already be `Clone` (`serde_json::Value`, `RetryReason`,
/// `VerificationOutcome`, `ToolOutputChunk` all derive Clone; strings
/// and primitives are Clone by default).
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
enum EventSpec {
    TextDelta(String),
    ToolCallStart(String, String),
    ToolCallDelta(String, String),
    ToolCallComplete(String, String, serde_json::Value),
    ToolCallMalformed(String, String, String),
    Usage(u64, u64, f64),
    Retry(u32, u64, RetryReason),
    Error(ProviderErrorSpec),
    ToolOutput(String, ToolOutputChunk),
    TaskComplete(String, bool, VerificationOutcome),
    ImageRead(String, Vec<u8>),
    SandboxViolation {
        call_id: String,
        tool_name: String,
        resource: String,
        policy_rule: String,
        os_error: Option<i32>,
    },
    Paused,
    Aborted(String),
}

impl EventSpec {
    /// Materialize the proxy into the real non-Clone `AgentEvent`.
    fn into_event(self) -> AgentEvent {
        match self {
            Self::TextDelta(content) => AgentEvent::TextDelta { content },
            Self::ToolCallStart(id, name) => AgentEvent::ToolCallStart { id, name },
            Self::ToolCallDelta(id, arguments_delta) => {
                AgentEvent::ToolCallDelta { id, arguments_delta }
            }
            Self::ToolCallComplete(id, name, arguments) => {
                AgentEvent::ToolCallComplete { id, name, arguments }
            }
            Self::ToolCallMalformed(id, raw, error) => {
                AgentEvent::ToolCallMalformed { id, raw, error }
            }
            Self::Usage(p, c, cost) => AgentEvent::Usage {
                prompt_tokens: p,
                completion_tokens: c,
                cost_usd: cost,
            },
            Self::Retry(attempt, delay_ms, reason) => AgentEvent::Retry {
                attempt,
                delay_ms,
                reason,
            },
            Self::Error(spec) => AgentEvent::Error {
                error: spec.into_error(),
            },
            Self::ToolOutput(call_id, chunk) => AgentEvent::ToolOutput { call_id, chunk },
            Self::TaskComplete(call_id, verified, outcome) => AgentEvent::TaskComplete {
                call_id,
                verified,
                outcome,
            },
            Self::ImageRead(path, bytes) => AgentEvent::ImageRead { path, bytes },
            Self::SandboxViolation {
                call_id,
                tool_name,
                resource,
                policy_rule,
                os_error,
            } => AgentEvent::SandboxViolation {
                call_id,
                tool_name,
                resource,
                policy_rule,
                os_error,
            },
            Self::Paused => AgentEvent::Paused,
            Self::Aborted(reason) => AgentEvent::Aborted { reason },
        }
    }

    /// Mirrors `matches!(event, AgentEvent::SandboxViolation { .. })`
    /// without having to materialize the event. Used to precompute the
    /// expected filter decision before the conversion + filter call.
    fn is_sandbox_violation(&self) -> bool {
        matches!(self, Self::SandboxViolation { .. })
    }
}

// ---------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------

fn arb_retry_reason() -> impl Strategy<Value = RetryReason> {
    prop_oneof![
        Just(RetryReason::RateLimited),
        Just(RetryReason::ServerError),
    ]
}

fn arb_auth_kind() -> impl Strategy<Value = AuthErrorKind> {
    prop_oneof![
        Just(AuthErrorKind::Missing),
        Just(AuthErrorKind::Invalid),
        Just(AuthErrorKind::Expired),
    ]
}

fn arb_provider_error_spec() -> impl Strategy<Value = ProviderErrorSpec> {
    prop_oneof![
        (any::<u16>(), ".*")
            .prop_map(|(status, body)| ProviderErrorSpec::Http { status, body }),
        proptest::option::of(0u64..=60_000u64)
            .prop_map(|retry_ms| ProviderErrorSpec::RateLimited { retry_ms }),
        (500u16..=599u16).prop_map(|status| ProviderErrorSpec::ServerError { status }),
        arb_auth_kind().prop_map(|reason| ProviderErrorSpec::Auth { reason }),
        (".*", proptest::collection::vec(".*", 0..=4))
            .prop_map(|(requested, allowed)| ProviderErrorSpec::ModelNotAllowlisted {
                requested,
                allowed,
            }),
        (0.0f64..1_000.0, 0.0f64..1_000.0)
            .prop_map(|(cap_usd, spent_usd)| ProviderErrorSpec::CostCapExceeded {
                cap_usd,
                spent_usd,
            }),
        (".*", ".*").prop_map(|(id, error)| ProviderErrorSpec::ToolCallMalformed { id, error }),
        ".*".prop_map(ProviderErrorSpec::Stream),
        Just(ProviderErrorSpec::Canceled),
    ]
}

fn arb_tool_output_chunk() -> impl Strategy<Value = ToolOutputChunk> {
    prop_oneof![
        ".*".prop_map(ToolOutputChunk::Stdout),
        ".*".prop_map(ToolOutputChunk::Stderr),
        (proptest::option::of(any::<i32>()), any::<bool>()).prop_map(
            |(exit_code, marker_detected)| ToolOutputChunk::Closed {
                exit_code,
                marker_detected,
            }
        ),
    ]
}

fn arb_verification_outcome() -> impl Strategy<Value = VerificationOutcome> {
    prop_oneof![
        ".*".prop_map(|reason| VerificationOutcome::Pending { reason }),
        ".*".prop_map(|note| VerificationOutcome::Pass { note }),
        ".*".prop_map(|reason| VerificationOutcome::Fail { reason }),
    ]
}

/// Simple Clone-able `serde_json::Value` strategy — scalar + empty
/// container leaves are sufficient since the filter never inspects
/// `ToolCallComplete.arguments` (it only switches on variant).
fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i64>().prop_map(|n| serde_json::Value::Number(n.into())),
        ".*".prop_map(serde_json::Value::String),
    ]
}

fn arb_event_spec() -> impl Strategy<Value = EventSpec> {
    prop_oneof![
        ".*".prop_map(EventSpec::TextDelta),
        (".*", ".*").prop_map(|(id, name)| EventSpec::ToolCallStart(id, name)),
        (".*", ".*").prop_map(|(id, delta)| EventSpec::ToolCallDelta(id, delta)),
        (".*", ".*", arb_json_value())
            .prop_map(|(id, name, args)| EventSpec::ToolCallComplete(id, name, args)),
        (".*", ".*", ".*").prop_map(|(id, raw, err)| EventSpec::ToolCallMalformed(id, raw, err)),
        (0u64..100_000, 0u64..100_000, 0.0f64..1.0)
            .prop_map(|(p, c, cost)| EventSpec::Usage(p, c, cost)),
        (0u32..100, 0u64..60_000, arb_retry_reason())
            .prop_map(|(attempt, delay, reason)| EventSpec::Retry(attempt, delay, reason)),
        arb_provider_error_spec().prop_map(EventSpec::Error),
        (".*", arb_tool_output_chunk())
            .prop_map(|(call_id, chunk)| EventSpec::ToolOutput(call_id, chunk)),
        (".*", any::<bool>(), arb_verification_outcome()).prop_map(
            |(call_id, verified, outcome)| EventSpec::TaskComplete(call_id, verified, outcome)
        ),
        (".*", proptest::collection::vec(any::<u8>(), 0..=16))
            .prop_map(|(path, bytes)| EventSpec::ImageRead(path, bytes)),
        (
            ".*",
            ".*",
            ".*",
            ".*",
            proptest::option::of(any::<i32>()),
        )
            .prop_map(
                |(call_id, tool_name, resource, policy_rule, os_error)| {
                    EventSpec::SandboxViolation {
                        call_id,
                        tool_name,
                        resource,
                        policy_rule,
                        os_error,
                    }
                }
            ),
        Just(EventSpec::Paused),
        ".*".prop_map(EventSpec::Aborted),
    ]
}

// ---------------------------------------------------------------------
// Property — 10_000 random sequences
// ---------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// QG-C4 LOAD-BEARING PROPERTY.
    ///
    /// For any sequence of up to 20 random `AgentEvent`s, the filter's
    /// decision MUST equal `!matches!(ev, SandboxViolation { .. })` for
    /// every event in the sequence. If this property breaks, the
    /// sandbox enforcement is defeated at the agent-loop level.
    #[test]
    fn model_context_filter_random_sequences_never_leak_sandbox_violation(
        specs in proptest::collection::vec(arb_event_spec(), 1..=20usize)
    ) {
        for spec in specs {
            let expect_sandbox = spec.is_sandbox_violation();
            let event = spec.into_event();
            let allowed = for_model_context(&event);

            if expect_sandbox {
                prop_assert!(
                    !allowed,
                    "QG-C4 LEAK: SandboxViolation was admitted into model context: {event:?}"
                );
            } else {
                prop_assert!(
                    allowed,
                    "LIVENESS REGRESSION: non-SandboxViolation event was blocked: {event:?}"
                );
            }
        }
    }
}
