//! Translates MiniMax JSON-SSE chunks into `AgentEvent` frames.
//!
//! MiniMax streaming format (each line prefixed with `data: `):
//! ```
//! data: {"id":"...","choices":[{"index":0,"delta":{"content":"Hi"}}],"object":"chat.completion.chunk"}
//! ```
//!
//! The final chunk has `"finish_reason":"length"` or `"stop"`.
//!
//! Unlike OpenRouter, MiniMax doesn't use SSE format lines. The `data: `
//! prefix is just plain text before each JSON object.

use serde::Deserialize;

use crate::error::ProviderError;
use crate::event::AgentEvent;

/// Translates raw JSON chunks into typed `AgentEvent` frames.
pub(crate) struct MiniMaxTranslator;

impl MiniMaxTranslator {
    /// Parse a single `data: {...}` line into an `AgentEvent`.
    /// Returns `None` if the chunk is a ping/heartbeat or the
    /// [DONE] sentinel (indicating end of stream).
    pub(crate) fn translate(json: &str) -> Result<Option<AgentEvent>, ProviderError> {
        // Parse the JSON (SseEventStream already stripped the "data: " prefix)
        let chunk: MiniMaxChunk = match serde_json::from_str(json) {
            Ok(c) => c,
            Err(e) => {
                // Tolerant parsing — skip malformed chunks
                tracing::debug!(e = %e, "failed to parse minimax chunk, skipping");
                return Ok(None);
            }
        };

        // Handle completion markers
        if chunk.is_done() {
            // Check for message.content in the final chunk (before returning None)
            if let Some(content) = chunk.final_message() {
                return Ok(Some(AgentEvent::TextDelta { content }));
            }
            return Ok(None); // Stream ended, caller will emit TaskComplete
        }

        // Extract text delta
        if let Some(content) = chunk.text_delta() {
            return Ok(Some(AgentEvent::TextDelta { content }));
        }

        // No text delta in this chunk — skip
        Ok(None)
    }
}

/// MiniMax streaming chunk structure.
#[derive(Debug, Deserialize)]
struct MiniMaxChunk {
    id: Option<String>,
    object: Option<String>,
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
    input_sensitive: Option<bool>,
    output_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: Option<usize>,
    finish_reason: Option<String>,
    delta: Option<Delta>,
    message: Option<Message>,  // Final message when finish_reason is set
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,  // MiniMax puts text here
    role: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: Option<u32>,
    total_characters: Option<u32>,
}

impl MiniMaxChunk {
    /// Returns `true` if this chunk signals end of stream.
    fn is_done(&self) -> bool {
        if let Some(choices) = &self.choices {
            if let Some(first) = choices.first() {
                if let Some(reason) = &first.finish_reason {
                    return reason == "stop" || reason == "length";
                }
            }
        }
        false
    }

    /// Extract the text delta from the delta field.
    fn text_delta(&self) -> Option<String> {
        // Try delta.content first (the actual response)
        if let Some(content) = self
            .choices
            .as_ref()?
            .first()?
            .delta
            .as_ref()?
            .content
            .clone()
        {
            if !content.is_empty() {
                return Some(content);
            }
        }

        // Try delta.reasoning_content (chain-of-thought, skip)
        if let Some(reasoning) = self
            .choices
            .as_ref()?
            .first()?
            .delta
            .as_ref()?
            .reasoning_content
            .clone()
        {
            if !reasoning.is_empty() {
                return Some(reasoning);
            }
        }

        None
    }

    /// Extract the final message content from the message field (in final chunk).
    fn final_message(&self) -> Option<String> {
        self.choices
            .as_ref()?
            .first()?
            .message
            .as_ref()?
            .content
            .clone()
            .filter(|c| !c.is_empty())
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_text_delta() {
        let json = r#"{"id":"abc","choices":[{"index":0,"delta":{"content":"Hello"}}],"object":"chat.completion.chunk"}"#;
        let event = MiniMaxTranslator::translate(json).unwrap();
        assert!(matches!(event, Some(AgentEvent::TextDelta { content }) if content == "Hello"));
    }

    #[test]
    fn parse_done_chunk() {
        let json = r#"{"id":"abc","choices":[{"finish_reason":"stop"}]}"#;
        let event = MiniMaxTranslator::translate(json).unwrap();
        assert!(event.is_none());
    }

    #[test]
    fn parse_invalid_json() {
        let json = "not valid json";
        let event = MiniMaxTranslator::translate(json).unwrap();
        assert!(event.is_none());
    }
}
