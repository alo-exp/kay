//! Provider trait + input types (PROV-01).
//!
//! Mirrors `kay-provider-openrouter/provider.rs`. The concrete
//! `MiniMaxProvider` impl lives in a submodule.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

use crate::error::ProviderError;
use crate::event::AgentEvent;

/// Stream of typed agent events returned by `Provider::chat`.
pub type AgentEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'a>>;

/// A chat completion request. Minimal shape: model + messages + tools.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSchema>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// A single conversation message.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
}

/// Tool schema.
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// The provider HAL contract.
#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError>;

    async fn models(&self) -> Result<Vec<String>, ProviderError>;
}
