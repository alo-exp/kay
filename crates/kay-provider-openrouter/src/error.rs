//! Typed error taxonomy for kay-provider-openrouter (PROV-08).
//!
//! Every variant is `#[non_exhaustive]`-compatible (the enum itself is
//! non_exhaustive; Phase 5 may add control-channel variants like `Interrupted`).
//! Variants listed in CONTEXT.md D-05.
//!
//! Threat model TM-01 (API key leakage): custom `Debug` for `AuthErrorKind`
//! never prints credential payloads. See tests in plan 02-07 (auth wiring)
//! for explicit Debug redaction assertion.

use std::time::Duration;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("rate limited; retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },

    #[error("server error HTTP {status}")]
    ServerError { status: u16 },

    #[error("authentication failed: {reason:?}")]
    Auth { reason: AuthErrorKind },

    #[error("model {requested} not allowlisted; allowed: {allowed:?}")]
    ModelNotAllowlisted {
        requested: String,
        allowed: Vec<String>,
    },

    #[error("cost cap ${cap_usd} exceeded (spent ${spent_usd})")]
    CostCapExceeded { cap_usd: f64, spent_usd: f64 },

    #[error("tool call {id} malformed: {error}")]
    ToolCallMalformed { id: String, error: String },

    #[error("serialization: {0}")]
    Serialization(#[source] serde_json::Error),

    #[error("stream: {0}")]
    Stream(String),

    #[error("canceled")]
    Canceled,
}

/// Sub-classification for `ProviderError::Auth`. Non-exhaustive per D-05; Phase 5
/// may add OAuth-related variants if the auth strategy grows.
///
/// Deliberately does NOT carry the credential material. See TM-01.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthErrorKind {
    /// No credential found (env var unset, config missing key).
    Missing,
    /// Credential rejected by provider (401 response).
    Invalid,
    /// Credential expired (provider returns `expired_key` error code).
    Expired,
}

/// Reason a provider request is being retried. Emitted on `AgentEvent::Retry`
/// so UIs can show "retrying in 2s (rate-limited)" etc.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryReason {
    /// 429 Too Many Requests (optionally respects `Retry-After`).
    RateLimited,
    /// 5xx server error; standard backon retry.
    ServerError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_impl_never_prints_credential_material() {
        // AuthErrorKind has no credential payload; this is a structural check.
        let e = AuthErrorKind::Invalid;
        let rendered = format!("{:?}", e);
        assert!(
            !rendered.contains("sk-"),
            "must not surface API-key-like strings"
        );
        assert_eq!(rendered, "Invalid");
    }

    #[test]
    fn provider_error_display_includes_context() {
        let e = ProviderError::ModelNotAllowlisted {
            requested: "openai/gpt-5.4".into(),
            allowed: vec!["anthropic/claude-sonnet-4.6".into()],
        };
        let s = e.to_string();
        assert!(s.contains("openai/gpt-5.4"));
        assert!(s.contains("anthropic/claude-sonnet-4.6"));
    }
}
