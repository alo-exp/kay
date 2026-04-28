//! kay-provider-minimax — MiniMax native API provider (Phase 12).
//!
//! MiniMax uses a JSON-over-SSE streaming format (unlike OpenRouter's
//! `text/event-stream`). Each SSE "event" is a JSON object prefixed
//! with `data: ` (not wrapped in SSE format lines like OpenRouter).
//!
//! This provider wraps the MiniMax native endpoint:
//!   POST https://api.minimax.io/v1/text/chatcompletion_v2
//!   Body: `{"model": "MiniMax-M2.1", "messages": [...], "stream": true}`
//!
//! Streaming response format (each chunk on its own line):
//! ```ignore
//! data: {"id":"...","choices":[{"index":0,"delta":{"content":"Hi"}}],"..."}
//! data: {"id":"...","choices":[{"index":0,"delta":{"content":" there"}}],...}
//! ```
//!
//! The final chunk has `"finish_reason":"length"` in choices[0].

// Crate-wide lint: forbid `.unwrap()` / `.expect()` in non-test code.
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod auth;
mod client;
mod error;
mod event;
mod minimax_provider;
mod provider;
mod translator;

pub use auth::{ConfigAuthSource, resolve_api_key};
pub use error::{AuthErrorKind, ProviderError};
pub use event::AgentEvent;
pub use minimax_provider::{MiniMaxProvider, MiniMaxProviderBuilder};
pub use provider::{AgentEventStream, ChatRequest, Message, Provider, ToolSchema};
