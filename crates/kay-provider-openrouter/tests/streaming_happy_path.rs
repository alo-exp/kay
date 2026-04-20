//! Integration test: OpenRouterProvider end-to-end happy path.
//!
//! Wires a `mockito` OpenRouter-flavored SSE cassette into
//! `OpenRouterProvider::chat` and asserts the emitted `AgentEvent`
//! sequence matches the expected shape for PROV-01 / AC-03 / AC-04:
//!   - `TextDelta { "Hello" }`
//!   - `TextDelta { " world" }`
//!   - `Usage { prompt=10, completion=2, cost=0.000015 }`
//!
//! This closes plan 02-08's HTTP-side proof (allowlist gate tests
//! stopped at the pre-flight boundary).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider,
};

use crate::common::mock_server::MockServer;

fn launch_allowlist() -> Allowlist {
    let path = format!(
        "{}/tests/fixtures/config/allowlist.json",
        env!("CARGO_MANIFEST_DIR")
    );
    Allowlist::from_path(&path).expect("load fixture")
}

#[tokio::test]
async fn happy_path_emits_text_deltas_and_usage() {
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("happy_path");
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-happy-path")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let req = ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "hi".into(),
            tool_call_id: None,
        }],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    };

    let mut stream = provider.chat(req).await.expect("chat returns stream");

    let mut text_chunks: Vec<String> = Vec::new();
    let mut usage_seen: Option<(u64, u64, f64)> = None;
    let mut unexpected: Vec<String> = Vec::new();

    while let Some(ev) = stream.next().await {
        let ev = ev.expect("stream event err-free");
        match ev {
            AgentEvent::TextDelta { content } => text_chunks.push(content),
            AgentEvent::Usage { prompt_tokens, completion_tokens, cost_usd } => {
                usage_seen = Some((prompt_tokens, completion_tokens, cost_usd));
            }
            other => {
                unexpected.push(format!("{other:?}"));
            }
        }
    }

    assert_eq!(
        text_chunks,
        vec!["Hello".to_string(), " world".to_string()],
        "delta-granular TextDelta stream"
    );
    assert_eq!(
        usage_seen,
        Some((10u64, 2u64, 0.000015f64)),
        "usage emitted with cost"
    );
    assert!(
        unexpected.is_empty(),
        "unexpected event types seen: {unexpected:?}"
    );
}

#[tokio::test]
async fn non_allowlisted_model_rejected_before_http_call() {
    // No mock arm registered — if the allowlist gate leaks through, mockito
    // will either 404 or hang. A missing-mock failure is still a failure,
    // but the intended assertion is the pre-flight short-circuit.
    let srv = MockServer::new().await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-allowlist-gate")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let req = ChatRequest {
        model: "mistral/mixtral-8x22b".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "should never reach http".into(),
            tool_call_id: None,
        }],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    };

    match provider.chat(req).await {
        Ok(_) => panic!("expected ModelNotAllowlisted, got Ok stream"),
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("ModelNotAllowlisted"),
                "expected ModelNotAllowlisted, got {msg}"
            );
        }
    }
}
