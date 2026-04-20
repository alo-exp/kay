//! Typed error taxonomy for kay-provider-openrouter (PROV-08).
//!
//! Phase 3 Wave 2 (plan 03-03) relocation: the canonical definitions now live
//! in the `kay-provider-errors` crate so `kay-tools::events::AgentEvent` can
//! hold `ProviderError` without creating a `kay-tools <-> kay-provider-openrouter`
//! dependency cycle. This module re-exports them for backward-compatibility —
//! all existing `kay_provider_openrouter::error::{...}` and top-level
//! `kay_provider_openrouter::{ProviderError, RetryReason, AuthErrorKind}`
//! call-sites continue to work unchanged.

pub use kay_provider_errors::{AuthErrorKind, ProviderError, RetryReason};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexport_provider_error_smoke() {
        let e = ProviderError::ModelNotAllowlisted {
            requested: "openai/gpt-5.4".into(),
            allowed: vec!["anthropic/claude-sonnet-4.6".into()],
        };
        let s = e.to_string();
        assert!(s.contains("openai/gpt-5.4"));
        assert!(s.contains("anthropic/claude-sonnet-4.6"));
    }

    #[test]
    fn reexport_auth_error_kind_smoke() {
        let rendered = format!("{:?}", AuthErrorKind::Invalid);
        assert_eq!(rendered, "Invalid");
    }

    #[test]
    fn reexport_retry_reason_smoke() {
        assert_eq!(RetryReason::RateLimited, RetryReason::RateLimited);
    }
}
