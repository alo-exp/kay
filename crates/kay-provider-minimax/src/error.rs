//! Typed error taxonomy for kay-provider-minimax (PROV-08).
//!
//! Aligns with `kay-provider-openrouter` error shapes so agent code
//! can match on `ProviderError` without knowing which provider is wired.

pub use kay_provider_errors::{AuthErrorKind, ProviderError};
