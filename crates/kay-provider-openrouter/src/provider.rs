//! Provider trait + input types (PROV-01).
//!
//! The trait is object-safe via `#[async_trait]`. Returned streams are
//! `Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send>>`;
//! the lifetime is tied to `&self` so the stream may borrow provider state.
//!
//! Concrete `OpenRouterProvider` impl lives in a submodule added by plan 02-08.
//! Plan 02-06 only defines the contract.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

use crate::error::ProviderError;
use crate::event::AgentEvent;

/// Stream of typed agent events returned by `Provider::chat`.
/// Lifetime is bound to `&self`; consumers that need a `'static` stream
/// must clone Arc-wrapped state into the stream closure.
pub type AgentEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'a>>;

/// A chat completion request. Minimal shape: model + messages + tools.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Canonical model ID (e.g., `anthropic/claude-sonnet-4.6`). The
    /// allowlist gate (plan 02-07) rewrites this to `:exacto` suffix on
    /// the wire.
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSchema>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// A single conversation message. Minimal shape — Phase 3 will extend for
/// multimodal content (TOOL-04 `image_read`).
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    /// Only set when `role == "tool"`; correlates a tool response to its
    /// originating tool call.
    pub tool_call_id: Option<String>,
}

/// Tool schema emitted into the provider's native `tools` parameter.
/// `input_schema` is already hardened per PROJECT.md NN-7 when it reaches
/// this layer (Phase 3 TOOL-05 runs ForgeCode's `normalize_tool_schema`
/// transformer on emission).
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// The provider HAL contract. Object-safe so consumers can use
/// `Arc<dyn Provider>`; Phase 5's agent loop depends on this.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stream a chat completion. Non-allowlisted models fail BEFORE any HTTP
    /// call with `ProviderError::ModelNotAllowlisted`. Cost-cap-exceeded
    /// sessions fail BEFORE any HTTP call with `ProviderError::CostCapExceeded`.
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError>;

    /// List allowlisted models. Backs the Phase 10 UI model picker
    /// (UI-04) and useful for diagnostics.
    async fn models(&self) -> Result<Vec<String>, ProviderError>;
}
