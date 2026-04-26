//! MiniMax native API provider implementation (Phase 12).
//!
//! Handles MiniMax's JSON-over-SSE streaming format and translates
//! chunks into standard `AgentEvent` frames.

use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::auth::{resolve_api_key, ConfigAuthSource};
use crate::client::MiniMaxClient;
use crate::error::ProviderError;
use crate::event::AgentEvent;
use crate::provider::{AgentEventStream, ChatRequest, Provider};

/// MiniMax provider built from the builder pattern.
#[derive(Debug)]
pub struct MiniMaxProvider {
    client: MiniMaxClient,
    allowlist: Arc<Vec<String>>,
}

impl MiniMaxProvider {
    /// Builder for configuring the provider.
    #[must_use]
    pub fn builder() -> MiniMaxProviderBuilder {
        MiniMaxProviderBuilder::default()
    }
}

/// Builder for `MiniMaxProvider`.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MiniMaxProviderBuilder {
    /// MiniMax API endpoint.
    endpoint: Option<String>,
    /// API key source.
    auth: Option<ConfigAuthSource>,
    /// Allowlist of permitted models.
    allowlist: Vec<String>,
}

impl Default for MiniMaxProviderBuilder {
    fn default() -> Self {
        Self {
            // Default MiniMax endpoint (native v2 API)
            endpoint: Some("https://api.minimax.io/v1/text/chatcompletion_v2".to_string()),
            auth: None,
            allowlist: vec!["MiniMax-M2.1".to_string()],
        }
    }
}

impl MiniMaxProviderBuilder {
    /// Set a custom endpoint (useful for testing with mock servers).
    pub fn endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Set the API key directly (bypasses env/config resolution).
    pub fn api_key(mut self, api_key: String) -> Self {
        self.auth = Some(ConfigAuthSource::new(Some(api_key)));
        self
    }

    /// Set model allowlist.
    pub fn allowlist(mut self, allowlist: Vec<String>) -> Self {
        self.allowlist = allowlist;
        self
    }

    /// Build the provider. Fails if endpoint is missing or API key cannot
    /// be resolved.
    pub fn build(self) -> Result<MiniMaxProvider, ProviderError> {
        let endpoint = self.endpoint.unwrap_or_else(|| {
            "https://api.minimax.io/v1/text/chatcompletion_v2".to_string()
        });

        let api_key = resolve_api_key(self.auth.as_ref())?;

        let client = MiniMaxClient::try_new(endpoint, api_key)?;

        Ok(MiniMaxProvider {
            client,
            allowlist: Arc::new(self.allowlist),
        })
    }
}

#[async_trait]
impl Provider for MiniMaxProvider {
    /// Stream chat completion events from MiniMax.
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError> {
        // Build the JSON body with streaming enabled
        let body = self.build_request_body(&request)?;

        // Make the streaming request
        let resp = self.client.stream_chat(body).await?;

        // Build the event stream
        let stream = MiniMaxEventStream::new(resp);

        Ok(Box::pin(stream))
    }

    /// List allowlisted models.
    async fn models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(self.allowlist.iter().cloned().collect())
    }
}

impl MiniMaxProvider {
    /// Build the JSON request body for MiniMax API.
    fn build_request_body(&self, request: &ChatRequest) -> Result<Bytes, ProviderError> {
        // Build messages array
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                let mut map = serde_json::Map::new();
                map.insert(
                    "role".to_string(),
                    serde_json::Value::String(msg.role.clone()),
                );
                map.insert(
                    "content".to_string(),
                    serde_json::Value::String(msg.content.clone()),
                );
                serde_json::Value::Object(map)
            })
            .collect();

        // Build the body
        let mut body_map = serde_json::Map::new();
        body_map.insert(
            "model".to_string(),
            serde_json::Value::String(request.model.clone()),
        );
        body_map.insert("messages".to_string(), serde_json::Value::Array(messages));
        body_map.insert("stream".to_string(), serde_json::Value::Bool(true));

        if let Some(max_tokens) = request.max_tokens {
            body_map.insert(
                "max_tokens".to_string(),
                serde_json::Value::Number(max_tokens.into()),
            );
        }
        if let Some(temp) = request.temperature {
            body_map.insert(
                "temperature".to_string(),
                serde_json::Number::from_f64(temp as f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::Null),
            );
        }

        // Serialize to bytes
        let body =
            serde_json::to_vec(&body_map).map_err(|e| ProviderError::Stream(e.to_string()))?;

        Ok(Bytes::from(body))
    }
}

/// A stream that yields `AgentEvent`s from MiniMax's JSON-over-SSE response.
pub struct MiniMaxEventStream {
    inner: Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'static>>,
}

impl MiniMaxEventStream {
    /// Create a new stream from a streaming HTTP response.
    pub fn new(resp: reqwest::Response) -> Self {
        use futures::StreamExt;

        // Get the bytes stream from the HTTP response
        let bytes_stream = resp.bytes_stream();

        // Parse SSE lines and translate to AgentEvents
        let stream = SseEventStream::new(bytes_stream);

        let inner = Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'static>>;

        MiniMaxEventStream { inner }
    }
}

impl Stream for MiniMaxEventStream {
    type Item = Result<AgentEvent, ProviderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Internal stream that parses SSE lines and emits AgentEvents.
struct SseEventStream<S> {
    inner: S,
    buffer: Vec<u8>,
    done: bool,
}

impl<S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin> SseEventStream<S> {
    fn new(inner: S) -> Self {
        SseEventStream {
            inner,
            buffer: Vec::new(),
            done: false,
        }
    }
}

impl<S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin> Stream for SseEventStream<S> {
    type Item = Result<AgentEvent, ProviderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        // If we're done, return None
        if self.done {
            return Poll::Ready(None);
        }

        // Process buffer for complete lines
        loop {
            if let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
                let line = self.buffer.drain(..=pos).collect::<Vec<u8>>();
                // Parse the line
                if let Ok(line_str) = String::from_utf8(line) {
                    let line = line_str.trim();
                    if line.starts_with("data:") {
                        let data = line.strip_prefix("data:").unwrap_or("").trim();
                        // Translate to AgentEvent
                        match crate::translator::MiniMaxTranslator::translate(data) {
                            Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                            Ok(None) => {
                                // Skip this line, continue processing
                            }
                            Err(e) => return Poll::Ready(Some(Err(e))),
                        }
                    }
                }
                continue;
            }
            break;
        }

        // Poll for more data
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                self.buffer.extend_from_slice(&chunk);
                // Recursively process
                self.poll_next(cx)
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(ProviderError::Stream(e.to_string()))))
            }
            Poll::Ready(None) => {
                self.done = true;
                // Emit TaskComplete
                Poll::Ready(Some(Ok(AgentEvent::TaskComplete {
                    call_id: uuid::Uuid::new_v4().to_string(),
                    verified: true,
                    outcome: kay_tools::VerificationOutcome::Pass {
                        note: "stream ended".into(),
                    },
                })))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
