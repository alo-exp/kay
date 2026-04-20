//! Integration test: per-tool_call.id reassembly (PROV-01, AC-04).
//!
//! Uses the `tool_call_fragmented.jsonl` cassette, which mimics
//! OpenRouter's wire behavior where:
//!   - chunk 1 carries `id` + `name` (arguments empty),
//!   - chunks 2-3 carry only `index` + `arguments` fragments,
//!   - chunk 4 carries `finish_reason: "tool_calls"` + usage.
//!
//! Expected event sequence on the agent side:
//!   - `ToolCallStart  { id: "call_abc", name: "execute_commands" }`
//!   - `ToolCallDelta  { id: "call_abc", arguments_delta: "" }`
//!   - `ToolCallDelta  { id: "call_abc", arguments_delta: "{\"cmd\":\"" }`
//!   - `ToolCallDelta  { id: "call_abc", arguments_delta: "ls -la\"}" }`
//!   - `ToolCallComplete { id: "call_abc", name: "execute_commands",
//!                          arguments: { "cmd": "ls -la" } }`
//!   - `Usage { prompt=100, completion=25, cost=0.000375 }`
//!
//! The critical invariant: index→id mapping carries the `call_abc` id
//! forward even though chunks 2 and 3 omit `id`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider, ToolSchema,
};
use serde_json::json;

use crate::common::mock_server::MockServer;

fn launch_allowlist() -> Allowlist {
    let path = format!(
        "{}/tests/fixtures/config/allowlist.json",
        env!("CARGO_MANIFEST_DIR")
    );
    Allowlist::from_path(&path).expect("load fixture")
}

#[tokio::test]
async fn fragmented_tool_call_reassembles_under_single_id() {
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("tool_call_fragmented");
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-tool-reassembly")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let req = ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "list this directory".into(),
            tool_call_id: None,
        }],
        tools: vec![ToolSchema {
            name: "execute_commands".into(),
            description: "run a shell command and return stdout/stderr".into(),
            input_schema: json!({
                "type": "object",
                "required": ["cmd"],
                "properties": { "cmd": { "type": "string" } }
            }),
        }],
        temperature: None,
        max_tokens: None,
    };

    let mut stream = provider.chat(req).await.expect("chat returns stream");

    let mut starts: Vec<(String, String)> = Vec::new();
    let mut deltas: Vec<(String, String)> = Vec::new();
    let mut completes: Vec<(String, String, serde_json::Value)> = Vec::new();
    let mut usage: Option<(u64, u64, f64)> = None;
    let mut malformed: Vec<String> = Vec::new();
    let mut text: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some(ev) = stream.next().await {
        let ev = ev.expect("stream event err-free");
        match ev {
            AgentEvent::ToolCallStart { id, name } => starts.push((id, name)),
            AgentEvent::ToolCallDelta { id, arguments_delta } => deltas.push((id, arguments_delta)),
            AgentEvent::ToolCallComplete { id, name, arguments } => {
                completes.push((id, name, arguments))
            }
            AgentEvent::ToolCallMalformed { raw, error, .. } => {
                malformed.push(format!("raw={raw} err={error}"));
            }
            AgentEvent::Usage { prompt_tokens, completion_tokens, cost_usd } => {
                usage = Some((prompt_tokens, completion_tokens, cost_usd))
            }
            AgentEvent::TextDelta { content } => text.push(content),
            AgentEvent::Retry { .. } => {}
            AgentEvent::Error { error } => errors.push(format!("{error:?}")),
            other => errors.push(format!("unknown variant: {other:?}")),
        }
    }

    // Single start for the single tool call, carrying the upstream id.
    assert_eq!(
        starts,
        vec![("call_abc".to_string(), "execute_commands".to_string())],
        "exactly one ToolCallStart with upstream id"
    );

    // Deltas must be keyed to `call_abc` — the index→id mapping MUST carry
    // the id forward for chunks that omit it. Fragment contents preserve
    // the upstream byte sequence. The opening chunk carries an empty
    // `arguments: ""` string, which the translator elides per Pitfall 5
    // (empty-delta tolerance) — so only the two non-empty fragments are
    // emitted as `ToolCallDelta`.
    let expected_delta_contents = ["{\"cmd\":\"", "ls -la\"}"];
    assert_eq!(
        deltas.len(),
        expected_delta_contents.len(),
        "two non-empty argument deltas (empty opening elided). got: {deltas:?}"
    );
    for (i, (id, frag)) in deltas.iter().enumerate() {
        assert_eq!(id, "call_abc", "delta {i} missing upstream id");
        assert_eq!(
            frag, expected_delta_contents[i],
            "delta {i} fragment mismatch"
        );
    }

    // Exactly one ToolCallComplete with reassembled JSON arguments.
    assert_eq!(completes.len(), 1, "exactly one ToolCallComplete");
    let (cid, cname, cargs) = &completes[0];
    assert_eq!(cid, "call_abc");
    assert_eq!(cname, "execute_commands");
    assert_eq!(cargs, &json!({ "cmd": "ls -la" }));

    assert!(malformed.is_empty(), "unexpected malformed: {malformed:?}");
    assert!(text.is_empty(), "no TextDelta expected, got {text:?}");
    assert!(errors.is_empty(), "no Error expected, got {errors:?}");
    assert_eq!(
        usage,
        Some((100u64, 25u64, 0.000375f64)),
        "usage emitted on tool_calls finish"
    );
}
