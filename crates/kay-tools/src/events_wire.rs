//! Wire serialization for `AgentEvent` — the frozen JSONL schema consumed
//! by `kay-cli` (Phase 5), Tauri GUI (Phase 9), and TUI (Phase 9.5).
//!
//! ## Design constraint
//!
//! `AgentEvent` (in [`crate::events`]) is intentionally NOT `Serialize` — it
//! holds `ProviderError`, which contains `reqwest::Error` and
//! `serde_json::Error` (neither `Clone` nor `Serialize`). The
//! [`AgentEventWire`] newtype here owns the custom `Serialize` impl so the
//! runtime type stays clean while the wire schema is fully deterministic.
//!
//! ## Schema stability
//!
//! Every variant's shape is locked by
//! `crates/kay-tools/tests/events_wire_snapshots.rs` via `insta`. Any
//! intentional schema change requires:
//!
//! 1. Updating the corresponding `.snap` file (`cargo insta review`)
//! 2. Bumping the schema-version note in `.planning/CONTRACT-AgentEvent.md`
//! 3. Bumping the event-schema version in `kay-cli --version`
//!
//! ## `ImageRead` policy
//!
//! Wire form reports `{path, size_bytes, encoding: "base64"}` only — raw
//! bytes are NOT inlined (they can be multi-MiB and would make the JSONL
//! stream unusable for real-time UI consumers). If a consumer needs the
//! payload it should invoke the same tool locally with the same path.
//!
//! ## `Error` policy
//!
//! Wire form reports `{kind, message}` only — source-error internals
//! (reqwest line numbers, serde_json positions) are intentionally stripped
//! to keep snapshots deterministic across crate-version bumps.
//!
//! ## Variant-tag casing
//!
//! All `type` tags use `snake_case` (matching Python/JS ecosystems).
//! `AgentEvent` variant names use `PascalCase` (matching Rust convention).

use kay_provider_errors::{ProviderError, RetryReason};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::events::{AgentEvent, ToolOutputChunk};
use crate::seams::verifier::VerificationOutcome;

/// Borrowing newtype that adds JSONL `Serialize` to `AgentEvent`.
///
/// Created via `AgentEventWire::from(&event)` so callers do not clone the
/// underlying `AgentEvent`. The Display impl (Phase 5 T1.4) emits a single
/// JSONL line with a trailing `\n` for stream consumers.
#[derive(Debug)]
pub struct AgentEventWire<'a>(pub &'a AgentEvent);

impl<'a> From<&'a AgentEvent> for AgentEventWire<'a> {
    fn from(event: &'a AgentEvent) -> Self {
        AgentEventWire(event)
    }
}

impl<'a> Serialize for AgentEventWire<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            AgentEvent::TextDelta { content } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("type", "text_delta")?;
                m.serialize_entry("content", content)?;
                m.end()
            }
            AgentEvent::ToolCallStart { id, name } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "tool_call_start")?;
                m.serialize_entry("id", id)?;
                m.serialize_entry("name", name)?;
                m.end()
            }
            AgentEvent::ToolCallDelta { id, arguments_delta } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "tool_call_delta")?;
                m.serialize_entry("id", id)?;
                m.serialize_entry("arguments_delta", arguments_delta)?;
                m.end()
            }
            AgentEvent::ToolCallComplete { id, name, arguments } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "tool_call_complete")?;
                m.serialize_entry("id", id)?;
                m.serialize_entry("name", name)?;
                m.serialize_entry("arguments", arguments)?;
                m.end()
            }
            AgentEvent::ToolCallMalformed { id, raw, error } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "tool_call_malformed")?;
                m.serialize_entry("id", id)?;
                m.serialize_entry("raw", raw)?;
                m.serialize_entry("error", error)?;
                m.end()
            }
            AgentEvent::Usage { prompt_tokens, completion_tokens, cost_usd } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "usage")?;
                m.serialize_entry("prompt_tokens", prompt_tokens)?;
                m.serialize_entry("completion_tokens", completion_tokens)?;
                m.serialize_entry("cost_usd", cost_usd)?;
                m.end()
            }
            AgentEvent::Retry { attempt, delay_ms, reason } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "retry")?;
                m.serialize_entry("attempt", attempt)?;
                m.serialize_entry("delay_ms", delay_ms)?;
                m.serialize_entry("reason", retry_reason_tag(*reason))?;
                m.end()
            }
            AgentEvent::Error { error } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "error")?;
                m.serialize_entry("kind", provider_error_kind(error))?;
                m.serialize_entry("message", &format!("{error}"))?;
                m.end()
            }
            AgentEvent::ToolOutput { call_id, chunk } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "tool_output")?;
                m.serialize_entry("call_id", call_id)?;
                m.serialize_entry("chunk", &ToolOutputChunkWire(chunk))?;
                m.end()
            }
            AgentEvent::TaskComplete { call_id, verified, outcome } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "task_complete")?;
                m.serialize_entry("call_id", call_id)?;
                m.serialize_entry("verified", verified)?;
                m.serialize_entry("outcome", &VerificationOutcomeWire(outcome))?;
                m.end()
            }
            AgentEvent::ImageRead { path, bytes } => {
                // Raw bytes deliberately omitted — see module header §ImageRead policy.
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("type", "image_read")?;
                m.serialize_entry("path", path)?;
                m.serialize_entry("size_bytes", &bytes.len())?;
                m.serialize_entry("encoding", "base64")?;
                m.end()
            }
            AgentEvent::SandboxViolation {
                call_id,
                tool_name,
                resource,
                policy_rule,
                os_error,
            } => {
                let mut m = serializer.serialize_map(Some(6))?;
                m.serialize_entry("type", "sandbox_violation")?;
                m.serialize_entry("call_id", call_id)?;
                m.serialize_entry("tool_name", tool_name)?;
                m.serialize_entry("resource", resource)?;
                m.serialize_entry("policy_rule", policy_rule)?;
                m.serialize_entry("os_error", os_error)?;
                m.end()
            }
            AgentEvent::Paused => {
                // Unit variant — `type` alone is the full payload.
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("type", "paused")?;
                m.end()
            }
            AgentEvent::Aborted { reason } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("type", "aborted")?;
                m.serialize_entry("reason", reason)?;
                m.end()
            }
        }
    }
}

/// JSONL line form: one compact JSON object terminated by a single `\n`.
///
/// This is the single point where `AgentEvent` → wire bytes. `kay-cli`,
/// Tauri GUI (Phase 9), and TUI (Phase 9.5) read these bytes directly off
/// the stream. Newline framing lets consumers delimit events with a
/// `BufRead::read_line` loop — no full JSON streaming parser required.
///
/// Invariants locked by `tests/events_wire_snapshots.rs::snap_jsonl_line_format`:
///
/// 1. Output ends with exactly one `\n` (never zero, never two)
/// 2. The JSON payload is a single physical line — any newline inside
///    string fields is escaped as `\n` by `serde_json`, so the outer
///    line stays delimiter-safe even if the user prompt contains `\n`.
/// 3. Byte-for-byte deterministic — snapshot equality enforces this.
///
/// Errors: `serde_json` serialization of `AgentEventWire` is infallible for
/// all current variants (no `Serialize` impls that return `Err`), but the
/// trait signature forces us to handle the `Result`. A genuine failure
/// here would indicate a programmer error (e.g. a newly added variant
/// whose custom impl returns `Err`), so we surface it as `fmt::Error` —
/// the `format!` / `writeln!` macros will propagate, and the bug will
/// surface immediately in tests rather than silently emitting half an
/// event.
impl<'a> std::fmt::Display for AgentEventWire<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        writeln!(f, "{json}")
    }
}

/// Stable wire tag for `RetryReason` — snake_case per schema convention.
fn retry_reason_tag(reason: RetryReason) -> &'static str {
    match reason {
        RetryReason::RateLimited => "rate_limited",
        RetryReason::ServerError => "server_error",
        _ => "unknown",
    }
}

/// Stable wire kind for `ProviderError` — must not leak `reqwest::Error`
/// / `serde_json::Error` internals (non-deterministic across versions).
fn provider_error_kind(error: &ProviderError) -> &'static str {
    match error {
        ProviderError::Network(_) => "network",
        ProviderError::Http { .. } => "http",
        ProviderError::RateLimited { .. } => "rate_limited",
        ProviderError::ServerError { .. } => "server_error",
        ProviderError::Auth { .. } => "auth",
        ProviderError::ModelNotAllowlisted { .. } => "model_not_allowlisted",
        ProviderError::CostCapExceeded { .. } => "cost_cap_exceeded",
        ProviderError::ToolCallMalformed { .. } => "tool_call_malformed",
        ProviderError::Serialization(_) => "serialization",
        ProviderError::Stream(_) => "stream",
        ProviderError::Canceled => "canceled",
        _ => "unknown",
    }
}

/// Helper newtype for `ToolOutputChunk` wire form.
struct ToolOutputChunkWire<'a>(&'a ToolOutputChunk);

impl<'a> Serialize for ToolOutputChunkWire<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            ToolOutputChunk::Stdout(data) => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("kind", "stdout")?;
                m.serialize_entry("data", data)?;
                m.end()
            }
            ToolOutputChunk::Stderr(data) => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("kind", "stderr")?;
                m.serialize_entry("data", data)?;
                m.end()
            }
            ToolOutputChunk::Closed { exit_code, marker_detected } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("kind", "closed")?;
                m.serialize_entry("exit_code", exit_code)?;
                m.serialize_entry("marker_detected", marker_detected)?;
                m.end()
            } // `ToolOutputChunk` is `#[non_exhaustive]` from the outside but we
              // own the enum here (kay-tools) so the compiler checks match
              // exhaustiveness at build-time. If a new variant lands, this
              // match fails compile — which is what we want for schema review.
        }
    }
}

/// Helper newtype for `VerificationOutcome` wire form — externally tagged
/// so the wire schema is obvious: `{"status": "pending" | "pass" | "fail", "note_or_reason": "..."}`.
///
/// Using a custom impl (not the derived one from `VerificationOutcome`)
/// ensures the wire schema doesn't drift if upstream adds fields.
struct VerificationOutcomeWire<'a>(&'a VerificationOutcome);

impl<'a> Serialize for VerificationOutcomeWire<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            VerificationOutcome::Pending { reason } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("status", "pending")?;
                m.serialize_entry("reason", reason)?;
                m.end()
            }
            VerificationOutcome::Pass { note } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("status", "pass")?;
                m.serialize_entry("note", note)?;
                m.end()
            }
            VerificationOutcome::Fail { reason } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("status", "fail")?;
                m.serialize_entry("reason", reason)?;
                m.end()
            } // `VerificationOutcome` is `#[non_exhaustive]` from the outside
              // but owned by kay-tools, so the compiler enforces match
              // exhaustiveness here. Adding a variant is a schema-change event
              // (see CONTRACT-AgentEvent.md) — we want compile failure, not a
              // silent `"unknown"` wire tag.
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn retry_reason_tag_is_snake_case() {
        assert_eq!(retry_reason_tag(RetryReason::RateLimited), "rate_limited");
        assert_eq!(retry_reason_tag(RetryReason::ServerError), "server_error");
    }

    #[test]
    fn provider_error_kind_is_stable() {
        assert_eq!(
            provider_error_kind(&ProviderError::Http { status: 500, body: String::new() }),
            "http"
        );
        assert_eq!(provider_error_kind(&ProviderError::Canceled), "canceled");
    }

    #[test]
    fn wire_never_leaks_image_bytes() {
        // QG-C4-adjacent invariant: ImageRead wire MUST NOT carry raw bytes.
        let ev = AgentEvent::ImageRead {
            path: "/tmp/x.png".to_string(),
            bytes: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        let json = serde_json::to_string(&AgentEventWire::from(&ev)).unwrap();
        assert!(
            !json.contains("DEAD") && !json.contains("\"bytes\""),
            "wire form must not inline raw bytes: {json}"
        );
        assert!(
            json.contains("\"size_bytes\":4"),
            "wire form must report size_bytes: {json}"
        );
    }
}
