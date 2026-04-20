//! SSE → AgentEvent translator (PROV-01, PROV-05 part 1, D-04).
//!
//! Consumes a `reqwest_eventsource::EventSource` and yields
//! `Stream<Item = Result<AgentEvent, ProviderError>>`.
//!
//! Responsibilities:
//!   - Decode each `Event::Message` as an OpenRouter chat-completions
//!     streaming chunk (OpenAI-compatible wire format).
//!   - Reassemble `tool_call` fragments per `tool_call.id` via
//!     `HashMap<String, ToolCallBuilder>`. Subsequent chunks may omit `id`
//!     and use only `index` — the translator tracks the currently-open
//!     call via `index → id` association so the cassette format actually
//!     seen in production (and `fixtures/sse/tool_call_fragmented.jsonl`)
//!     round-trips correctly.
//!   - Emit ONE `ToolCallStart` per new call_id; ONE `ToolCallDelta` per
//!     non-empty non-null arguments chunk; ONE `ToolCallComplete` per
//!     terminal marker (`finish_reason == "tool_calls" | "stop"` or the
//!     `[DONE]` sentinel).
//!   - Emit `Usage` when `usage` is present on any chunk (OpenRouter sends
//!     it on the final chunk alongside the stop marker).
//!
//! NN-5 pitfalls handled:
//!   - Pitfall 5: null / empty `arguments` deltas are tolerated — the
//!     raw_arguments buffer simply stays empty if the delta contributes
//!     nothing. Never panic.
//!   - Pitfall 6: connection retry is orchestrated by `backon` in plan
//!     02-10, not by `reqwest_eventsource` — the client disables that.
//!
//! Tool-argument JSON parsing is STRICT `serde_json::from_str` in this
//! plan. Plan 02-09 wraps a tolerant second pass via `forge_json_repair`
//! and introduces the `ToolCallMalformed` emission path (today a strict
//! parse failure surfaces as `ProviderError::ToolCallMalformed` and
//! terminates the stream).
//!
//! DTO shape note (Rule-2 deviation recorded in SUMMARY.md): the plan as
//! authored pointed at `kay_core::forge_app::dto::openai::Response` / post-
//! Appendix-A `forge_app::dto::openai::Response`. That DTO is an untagged
//! enum keyed on presence of the top-level `id` field — OpenRouter SSE
//! chunks frequently omit `id`, which routes them to the `CostOnly` variant
//! (no `usage` field) and drops the usage report on the ground. This
//! translator instead uses a local, minimally-typed `SseChunk` struct that
//! matches the actual wire surface. The serialization path
//! (`build_request_body`) still honors the parity contract — only the
//! response-side decode is local. Documented as **Rule-2 sub-deviation
//! (DTO divergence)** in SUMMARY.md.

use std::collections::HashMap;

use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::Value;

use crate::error::ProviderError;
use crate::event::AgentEvent;

// ----- Local minimal DTO for SSE chunks -----

#[derive(Debug, Deserialize)]
struct SseChunk {
    #[serde(default)]
    choices: Vec<SseChoice>,
    usage: Option<SseUsage>,
}

#[derive(Debug, Deserialize)]
struct SseChoice {
    delta: SseDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SseDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<SseToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct SseToolCallDelta {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    index: Option<u32>,
    #[serde(default)]
    function: Option<SseFunctionDelta>,
}

#[derive(Debug, Default, Deserialize)]
struct SseFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SseUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    cost: Option<f64>,
}

// ----- Builder state -----

struct ToolCallBuilder {
    name: Option<String>,
    arguments_raw: String,
}

impl ToolCallBuilder {
    fn new() -> Self {
        Self {
            name: None,
            arguments_raw: String::new(),
        }
    }
}

/// Strict JSON parse for tool_call arguments_raw. Plan 02-09 adds the
/// tolerant fallback via `forge_json_repair::json_repair`. An empty buffer
/// is treated as an empty object `{}` (observed legitimate for
/// zero-argument tools).
fn parse_arguments_strict(raw: &str) -> Result<Value, serde_json::Error> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    serde_json::from_str(trimmed)
}

/// Resolve which call_id an incoming delta refers to. First chunk usually
/// carries `id` (we register `index → id`); subsequent chunks may carry
/// only `index`. If neither is present and no prior id has been seen, the
/// delta is discarded (defensive — OpenRouter always sends at least one
/// on chunk 0).
fn resolve_call_id(
    tc: &SseToolCallDelta,
    index_to_id: &mut HashMap<u32, String>,
) -> Option<String> {
    if let Some(id) = &tc.id {
        if let Some(idx) = tc.index {
            index_to_id.insert(idx, id.clone());
        }
        return Some(id.clone());
    }
    if let Some(idx) = tc.index
        && let Some(id) = index_to_id.get(&idx)
    {
        return Some(id.clone());
    }
    None
}

pub(crate) fn translate_stream(
    mut source: EventSource,
) -> impl Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'static {
    stream! {
        let mut builders: HashMap<String, ToolCallBuilder> = HashMap::new();
        let mut index_to_id: HashMap<u32, String> = HashMap::new();
        let mut terminal_seen = false;

        while let Some(next) = source.next().await {
            match next {
                Ok(Event::Open) => {
                    // Connection established; no AgentEvent to emit.
                }
                Ok(Event::Message(ev)) => {
                    let data = ev.data.trim();
                    if data.is_empty() {
                        continue;
                    }
                    // `[DONE]` sentinel per OpenRouter convention — terminal
                    // marker for the stream. Flush any remaining open
                    // builders (best effort; OpenRouter normally sends
                    // finish_reason BEFORE [DONE]).
                    if data == "[DONE]" {
                        break;
                    }

                    let chunk: SseChunk = match serde_json::from_str(data) {
                        Ok(c) => c,
                        Err(e) => {
                            yield Err(ProviderError::Stream(format!(
                                "parse chunk failed: {e}"
                            )));
                            return;
                        }
                    };

                    // Process choices[0]
                    if let Some(choice) = chunk.choices.into_iter().next() {
                        // 1) Text content
                        if let Some(content) = choice.delta.content
                            && !content.is_empty()
                        {
                            yield Ok(AgentEvent::TextDelta { content });
                        }

                        // 2) Tool-call fragments
                        if let Some(tool_calls) = choice.delta.tool_calls {
                            for tc in tool_calls {
                                let Some(call_id) = resolve_call_id(&tc, &mut index_to_id) else {
                                    continue;
                                };
                                let is_new = !builders.contains_key(&call_id);
                                let entry = builders
                                    .entry(call_id.clone())
                                    .or_insert_with(ToolCallBuilder::new);

                                let fn_delta = tc.function.unwrap_or_default();
                                // Record name on first sighting
                                if let Some(n) = fn_delta.name.clone()
                                    && entry.name.is_none()
                                {
                                    entry.name = Some(n);
                                }

                                if is_new {
                                    yield Ok(AgentEvent::ToolCallStart {
                                        id: call_id.clone(),
                                        name: entry.name.clone().unwrap_or_default(),
                                    });
                                }

                                if let Some(args_delta) = fn_delta.arguments
                                    && !args_delta.is_empty()
                                {
                                    entry.arguments_raw.push_str(&args_delta);
                                    yield Ok(AgentEvent::ToolCallDelta {
                                        id: call_id.clone(),
                                        arguments_delta: args_delta,
                                    });
                                }
                            }
                        }

                        // 3) Terminal marker: finish_reason in {"tool_calls", "stop"}
                        //    drains builders as Complete.
                        if let Some(reason) = choice.finish_reason.as_deref()
                            && (reason == "tool_calls" || reason == "stop")
                        {
                            terminal_seen = true;
                            // Drain in arbitrary order (HashMap iteration
                            // order is unspecified but our tests only check
                            // per-id counts, not inter-call ordering).
                            let drained: Vec<(String, ToolCallBuilder)> =
                                builders.drain().collect();
                            for (id, b) in drained {
                                let name = b.name.clone().unwrap_or_default();
                                match parse_arguments_strict(&b.arguments_raw) {
                                    Ok(args) => {
                                        yield Ok(AgentEvent::ToolCallComplete {
                                            id,
                                            name,
                                            arguments: args,
                                        });
                                    }
                                    Err(e) => {
                                        // Strict-parse failure terminates the
                                        // stream in plan 02-08; plan 02-09
                                        // upgrades to tolerant + emits
                                        // AgentEvent::ToolCallMalformed.
                                        yield Err(ProviderError::ToolCallMalformed {
                                            id,
                                            error: e.to_string(),
                                        });
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    // 4) Usage (OpenRouter sends on the final chunk)
                    if let Some(u) = chunk.usage {
                        yield Ok(AgentEvent::Usage {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            cost_usd: u.cost.unwrap_or(0.0),
                        });
                    }
                }
                Err(reqwest_eventsource::Error::StreamEnded) => {
                    break;
                }
                Err(err) => {
                    yield Err(map_upstream_error(err).await);
                    return;
                }
            }
        }

        // Stream ended (by break / StreamEnded / [DONE]). Flush any
        // builders that weren't drained by a finish_reason marker.
        // Unusual path — OpenRouter typically sends finish_reason first.
        if !terminal_seen && !builders.is_empty() {
            let drained: Vec<(String, ToolCallBuilder)> = builders.drain().collect();
            for (id, b) in drained {
                let name = b.name.clone().unwrap_or_default();
                match parse_arguments_strict(&b.arguments_raw) {
                    Ok(args) => {
                        yield Ok(AgentEvent::ToolCallComplete { id, name, arguments: args });
                    }
                    Err(e) => {
                        yield Err(ProviderError::ToolCallMalformed { id, error: e.to_string() });
                    }
                }
            }
        }
    }
}

async fn map_upstream_error(err: reqwest_eventsource::Error) -> ProviderError {
    // Minimal mapping for Phase 2 plan 02-08. Plan 02-10 replaces this with
    // the full taxonomy (401→Auth::Invalid, 402→Http{402}, 429 respecting
    // Retry-After → RateLimited, 5xx → ServerError).
    match err {
        reqwest_eventsource::Error::StreamEnded => {
            ProviderError::Stream("stream ended".into())
        }
        reqwest_eventsource::Error::InvalidStatusCode(status, _resp) => {
            ProviderError::Http {
                status: status.as_u16(),
                body: String::new(),
            }
        }
        reqwest_eventsource::Error::InvalidContentType(_, resp) => ProviderError::Http {
            status: resp.status().as_u16(),
            body: "invalid content type".into(),
        },
        reqwest_eventsource::Error::Transport(e) => ProviderError::Network(e),
        other => ProviderError::Stream(format!("upstream: {other}")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit {
    use super::*;

    #[test]
    fn parse_arguments_strict_empty_is_empty_object() {
        let v = parse_arguments_strict("").unwrap();
        assert_eq!(v, Value::Object(Default::default()));
    }

    #[test]
    fn parse_arguments_strict_valid_json_roundtrips() {
        let v = parse_arguments_strict(r#"{"cmd":"ls"}"#).unwrap();
        assert_eq!(v["cmd"], Value::String("ls".into()));
    }

    #[test]
    fn parse_arguments_strict_malformed_returns_err() {
        let r = parse_arguments_strict(r#"{unquoted: "x"}"#);
        assert!(r.is_err());
    }

    #[test]
    fn parse_arguments_strict_tolerates_surrounding_whitespace() {
        let v = parse_arguments_strict("  { \"k\": 1 }  ").unwrap();
        assert_eq!(v["k"], Value::Number(1.into()));
    }

    #[test]
    fn resolve_call_id_registers_and_reuses_index() {
        let mut map = HashMap::new();
        let tc1 = SseToolCallDelta {
            id: Some("call_abc".into()),
            index: Some(0),
            function: None,
        };
        assert_eq!(resolve_call_id(&tc1, &mut map), Some("call_abc".into()));

        let tc2 = SseToolCallDelta {
            id: None,
            index: Some(0),
            function: None,
        };
        assert_eq!(resolve_call_id(&tc2, &mut map), Some("call_abc".into()));
    }

    #[test]
    fn resolve_call_id_returns_none_when_no_mapping() {
        let mut map = HashMap::new();
        let tc = SseToolCallDelta {
            id: None,
            index: Some(99),
            function: None,
        };
        assert_eq!(resolve_call_id(&tc, &mut map), None);
    }
}
