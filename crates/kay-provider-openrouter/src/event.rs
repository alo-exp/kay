//! Typed event stream output for kay-provider-openrouter (PROV-01).
//!
//! `AgentEvent` is the delta-granular frame type emitted by `Provider::chat`.
//! Phase 5 will mark it `#[non_exhaustive]` and freeze it as the cross-phase
//! contract (LOOP-02). For Phase 2 it already carries the annotation so Phase 3
//! can add `ToolOutput`, Phase 4 `SandboxViolation`, Phase 5 `TurnEnd`, Phase 8
//! `Verification` without breaking callers.
//!
//! Variants listed in CONTEXT.md D-06.

use serde_json::Value;

use crate::error::{ProviderError, RetryReason};

#[non_exhaustive]
#[derive(Debug)]
pub enum AgentEvent {
    /// Streaming text chunk from the model's content channel.
    TextDelta { content: String },

    /// A tool call has begun; subsequent deltas carry arguments.
    ToolCallStart { id: String, name: String },

    /// Additional arguments bytes for an in-progress tool call. Empty/null
    /// argument deltas are legal per OpenRouter variance; the accumulator
    /// in plan 02-09 handles them defensively.
    ToolCallDelta { id: String, arguments_delta: String },

    /// Tool call fully assembled with valid JSON arguments. Tool-argument
    /// schema validation is the consumer's responsibility (Phase 3 TOOL-05).
    ToolCallComplete {
        id: String,
        name: String,
        arguments: Value,
    },

    /// Tool call assembled but arguments did not parse even after
    /// `forge_json_repair` fallback. `raw` carries the original bytes for
    /// diagnosis; consumers should surface this to the user, not execute.
    ToolCallMalformed {
        id: String,
        raw: String,
        error: String,
    },

    /// Usage/cost report emitted at turn end (per OpenRouter streaming docs,
    /// usage arrives on the final chunk). Fed into the cost-cap accumulator
    /// in plan 02-10.
    Usage {
        prompt_tokens: u64,
        completion_tokens: u64,
        cost_usd: f64,
    },

    /// A retryable upstream error is being retried after `delay_ms`. Emitted
    /// BEFORE the backoff sleep so UIs can show progress.
    ///
    /// `attempt` is the 1-based index of the attempt that JUST FAILED — so
    /// `attempt == 1` means the first try errored and the wrapper is about
    /// to sleep `delay_ms` before the second try. This matches `backon`'s
    /// `max_times` semantics and is what the `retry_emission_unit` tests
    /// assert. UIs that want to render "retrying attempt N of M" should
    /// display `attempt + 1` as the upcoming attempt number.
    Retry {
        attempt: u32,
        delay_ms: u64,
        reason: RetryReason,
    },

    /// Terminal, non-retryable error. The stream ends immediately after this.
    Error { error: ProviderError },
}
