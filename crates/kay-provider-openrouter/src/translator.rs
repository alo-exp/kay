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
//! Tool-argument JSON parsing is TOLERANT as of plan 02-09: deltas flow
//! through `crate::tool_parser::parse_tool_arguments` which runs
//! `serde_json::from_str` first and falls back to
//! `forge_json_repair::json_repair` on strict failure. When BOTH passes
//! fail, the translator emits `AgentEvent::ToolCallMalformed { id, raw,
//! error }` as a DATA event (Ok variant) so the stream CONTINUES —
//! subsequent tool_calls, text content, and Usage frames still flow.
//! This supersedes the plan 02-08 behavior where a strict-parse failure
//! surfaced `ProviderError::ToolCallMalformed` and terminated the stream.
//!
//! TM-06 safety cap: each `ToolCallBuilder::arguments_raw` is capped at
//! `MAX_TOOL_ARGS_BYTES` (1 MiB). If an incoming delta would push the
//! accumulated buffer past the cap, the builder is evicted, a
//! `ToolCallMalformed` event is emitted with an empty `raw` (to avoid
//! yielding a near-1MB string back through `AgentEvent`), and subsequent
//! deltas for that call_id are silently ignored.
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
use std::sync::Arc;

use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;

use crate::cost_cap::CostCap;
use crate::error::ProviderError;
use crate::event::AgentEvent;
use crate::retry::classify_http_error;
use crate::tool_parser::{MAX_TOOL_ARGS_BYTES, ParseOutcome, parse_tool_arguments};

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
        Self { name: None, arguments_raw: String::new() }
    }
}

/// Resolve which call_id an incoming delta refers to. First chunk usually
/// carries `id` (we register `index → id`); subsequent chunks may carry
/// only `index`. If the opening chunk carries `id` but omits `index`
/// (observed in Anthropic-via-OpenRouter style), we backfill the mapping
/// under a monotonically-assigned slot so that subsequent `index`-only
/// chunks still resolve. If neither is present and no prior id has been
/// seen, the delta is discarded (defensive).
fn resolve_call_id(
    tc: &SseToolCallDelta,
    index_to_id: &mut HashMap<u32, String>,
) -> Option<String> {
    if let Some(id) = &tc.id {
        // If the provider omits `index`, synthesize one by scanning for the
        // first free slot rather than using `index_to_id.len()` — the latter
        // collides when the map has holes (e.g., existing indices {0, 2}
        // with len()==2 would synthesize idx=2, clobbering the existing
        // index-2 mapping and corrupting subsequent `index`-only lookups
        // for the displaced id). Fix for REVIEW LO-01.
        let idx = match tc.index {
            Some(i) => i,
            None => {
                let mut i: u32 = 0;
                while index_to_id.contains_key(&i) {
                    i = i.saturating_add(1);
                }
                i
            }
        };
        index_to_id.insert(idx, id.clone());
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
    cost_cap: Arc<CostCap>,
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
                                    // TM-06: enforce 1 MiB cap on accumulated
                                    // arguments_raw before appending. If this
                                    // delta would push the total past the cap,
                                    // evict the builder, emit Malformed (with
                                    // empty raw — we don't want to yield near-
                                    // 1MB strings back through AgentEvent), and
                                    // silently drop subsequent deltas for this
                                    // call_id (the builder is gone; resolve
                                    // continues to return this id via the
                                    // index_to_id map, but lookups into
                                    // `builders` will miss on the next branch).
                                    if entry.arguments_raw.len() + args_delta.len()
                                        > MAX_TOOL_ARGS_BYTES
                                    {
                                        let evicted_id = call_id.clone();
                                        builders.remove(&evicted_id);
                                        yield Ok(AgentEvent::ToolCallMalformed {
                                            id: evicted_id,
                                            raw: String::new(),
                                            error: format!(
                                                "tool_call arguments exceeded {MAX_TOOL_ARGS_BYTES} byte limit"
                                            ),
                                        });
                                        continue;
                                    }
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
                                match parse_tool_arguments(&b.arguments_raw) {
                                    ParseOutcome::Clean(args)
                                    | ParseOutcome::Repaired(args) => {
                                        yield Ok(AgentEvent::ToolCallComplete {
                                            id,
                                            name,
                                            arguments: args,
                                        });
                                    }
                                    ParseOutcome::Malformed { error } => {
                                        // Plan 02-09 upgrade: emit Malformed as
                                        // a DATA event (Ok variant), not a
                                        // terminal ProviderError. The stream
                                        // continues - subsequent tool_calls,
                                        // text, or Usage frames still flow.
                                        yield Ok(AgentEvent::ToolCallMalformed {
                                            id,
                                            raw: b.arguments_raw,
                                            error,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // 4) Usage (OpenRouter sends on the final chunk).
                    // Plan 02-10: accumulate cost into CostCap BEFORE emitting
                    // the Usage frame so a subsequent `check()` (turn boundary)
                    // sees the accumulated spend. Negative cost values are
                    // clamped to 0 inside `accumulate`.
                    if let Some(u) = chunk.usage {
                        let cost_usd = u.cost.unwrap_or(0.0);
                        cost_cap.accumulate(cost_usd);
                        yield Ok(AgentEvent::Usage {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            cost_usd,
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
                match parse_tool_arguments(&b.arguments_raw) {
                    ParseOutcome::Clean(args) | ParseOutcome::Repaired(args) => {
                        yield Ok(AgentEvent::ToolCallComplete { id, name, arguments: args });
                    }
                    ParseOutcome::Malformed { error } => {
                        yield Ok(AgentEvent::ToolCallMalformed {
                            id,
                            raw: b.arguments_raw,
                            error,
                        });
                    }
                }
            }
        }
    }
}

async fn map_upstream_error(err: reqwest_eventsource::Error) -> ProviderError {
    // Plan 02-10: full error taxonomy via `classify_http_error`.
    //   - 401        → Auth { reason: Invalid }
    //   - 429        → RateLimited { retry_after: parse_retry_after(&headers) }
    //   - 500..=599  → ServerError { status }
    //   - otherwise  → Http { status, body }
    //
    // Transport errors and premature stream-end continue to map to
    // Network / Stream respectively (D-05 / PROV-08).
    match err {
        reqwest_eventsource::Error::StreamEnded => ProviderError::Stream("stream ended".into()),
        reqwest_eventsource::Error::InvalidStatusCode(status, resp) => {
            let headers = resp.headers().clone();
            let status = status.as_u16();
            // Best-effort body read; on failure leave body empty. Body
            // content is surfaced verbatim for the `Http` variant only —
            // never for `Auth` / `RateLimited` / `ServerError` (TM-01).
            let body = resp.text().await.unwrap_or_default();
            classify_http_error(status, &headers, body)
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

    // The 3 strict-parse tests from plan 02-08
    //   (parse_arguments_strict_{empty_is_empty_object, valid_json_roundtrips,
    //    malformed_returns_err, tolerates_surrounding_whitespace})
    // are SUPERSEDED by tool_parser::unit (6 classic + 2 proptest = 8 tests)
    // in plan 02-09 Task 1. The strict-only helper is gone; parse_tool_arguments
    // now handles the strict path as Pass 1 and provides the Repaired / Malformed
    // paths on top.

    #[test]
    fn resolve_call_id_registers_and_reuses_index() {
        let mut map = HashMap::new();
        let tc1 = SseToolCallDelta { id: Some("call_abc".into()), index: Some(0), function: None };
        assert_eq!(resolve_call_id(&tc1, &mut map), Some("call_abc".into()));

        let tc2 = SseToolCallDelta { id: None, index: Some(0), function: None };
        assert_eq!(resolve_call_id(&tc2, &mut map), Some("call_abc".into()));
    }

    #[test]
    fn resolve_call_id_returns_none_when_no_mapping() {
        let mut map = HashMap::new();
        let tc = SseToolCallDelta { id: None, index: Some(99), function: None };
        assert_eq!(resolve_call_id(&tc, &mut map), None);
    }

    #[test]
    fn resolve_call_id_synthesized_slot_skips_existing_holes() {
        // REVIEW LO-01: if `index_to_id` has holes (indices 0 and 2
        // registered, gap at 1), an id-only delta should synthesize idx=1
        // (first free slot) — NOT idx=2 (which `len()`-based sizing would
        // produce and which would clobber the existing index-2 mapping).
        let mut map = HashMap::new();
        map.insert(0u32, "call_zero".to_string());
        map.insert(2u32, "call_two".to_string());

        let tc = SseToolCallDelta { id: Some("call_new".into()), index: None, function: None };
        assert_eq!(resolve_call_id(&tc, &mut map), Some("call_new".into()));

        // The gap at index 1 was filled — NOT index 2.
        assert_eq!(map.get(&1), Some(&"call_new".to_string()));
        assert_eq!(
            map.get(&2),
            Some(&"call_two".to_string()),
            "existing index-2 mapping must not be clobbered"
        );
        assert_eq!(map.get(&0), Some(&"call_zero".to_string()));
    }
}
