//! kay-provider-openrouter — OpenRouter provider HAL (Phase 2).
//!
//! Typed wrapper over ForgeCode's existing OpenAI-compatible provider path
//! (D-02 in .planning/phases/02-provider-hal-tolerant-json-parser/02-CONTEXT.md).
//! Re-exports:
//!   - `Provider` trait (PROV-01)
//!   - `AgentEvent` enum (PROV-01, delta-granular frames)
//!   - `ProviderError` enum (PROV-08)
//!
//! The concrete `OpenRouterProvider` impl is assembled incrementally by
//! plans 02-07 (allowlist + auth), 02-08 (streaming + translator),
//! 02-09 (tolerant JSON parser), 02-10 (retry + cost + error taxonomy).

// Crate-wide lint: forbid `.unwrap()` / `.expect()` in non-test code.
// Backs threat model TM-01 (API-key leakage via panic trace) and
// PROV-05 (never panic). Tests override with module-level attributes.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow unused until plans 02-07 through 02-10 wire everything up.
#![allow(dead_code)]

mod allowlist;
mod auth;
mod error;
mod event;
mod provider;

pub use allowlist::Allowlist;
pub use auth::{ConfigAuthSource, resolve_api_key};
// Note: ApiKey is NOT re-exported — it's crate-internal. Only used by
// plan 02-08's delegation layer via its pub(crate) as_str() accessor.
pub use error::{AuthErrorKind, ProviderError, RetryReason};
pub use event::AgentEvent;
pub use provider::{AgentEventStream, ChatRequest, Message, Provider, ToolSchema};
