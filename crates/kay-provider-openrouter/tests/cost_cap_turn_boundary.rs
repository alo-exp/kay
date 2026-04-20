//! Integration test: PROV-06 / D-10 cost-cap turn-boundary enforcement.
//!
//! Covers:
//!   1. A turn that completes normally accumulates cost into the shared
//!      `CostCap`. A subsequent `chat()` call on the SAME provider instance
//!      — when the accumulated spend exceeds the cap — is rejected at the
//!      pre-flight `cost_cap.check()?` gate with `CostCapExceeded`. Critical:
//!      the FIRST turn completes in full — we never abort mid-response
//!      (Pitfall 3).
//!   2. `max_usd(0.0)` on the builder is rejected at build-time with a
//!      `ProviderError::Stream(_)` carrying a clear message.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider, ProviderError,
};

use crate::common::mock_server::MockServer;

fn launch_allowlist() -> Allowlist {
    let path = format!(
        "{}/tests/fixtures/config/allowlist.json",
        env!("CARGO_MANIFEST_DIR")
    );
    Allowlist::from_path(&path).expect("load fixture")
}

fn base_req() -> ChatRequest {
    ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "hi".into(),
            tool_call_id: None,
        }],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    }
}

#[tokio::test]
async fn turn_n_completes_even_if_it_crosses_cap_then_n_plus_1_rejected() {
    // Cap: $0.000010 (10 micro-dollars).
    // Happy-path cassette emits usage.cost = 0.000015 — well over the cap.
    // Expected behavior (D-10 + Pitfall 3):
    //   Turn 1: cost_cap.check() passes (spent=0 < cap=10µ) → turn runs in
    //           full → translator's Usage branch accumulates 15µ → spent now 15µ.
    //   Turn 2: cost_cap.check() sees spent=15µ > cap=10µ → returns
    //           CostCapExceeded BEFORE any HTTP is made.
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("happy_path");
    // Mock fires on turn 1. Turn 2 must NOT hit the mock (pre-flight reject).
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-cost-cap")
        .allowlist(launch_allowlist())
        .max_usd(0.000010)
        .build()
        .expect("build provider");

    // ---- Turn 1: should complete in full ----
    let mut stream1 = provider.chat(base_req()).await.expect("turn 1 stream");
    let mut text_chunks: Vec<String> = Vec::new();
    let mut usage_cost: Option<f64> = None;
    while let Some(ev) = stream1.next().await {
        let ev = ev.expect("turn 1 err-free");
        match ev {
            AgentEvent::TextDelta { content } => text_chunks.push(content),
            AgentEvent::Usage { cost_usd, .. } => usage_cost = Some(cost_usd),
            _ => {}
        }
    }
    assert_eq!(
        text_chunks,
        vec!["Hello".to_string(), " world".to_string()],
        "turn 1 completed in full — never abort mid-response (Pitfall 3)"
    );
    assert_eq!(
        usage_cost,
        Some(0.000015),
        "usage accumulated into cost_cap"
    );
    // Confirm accumulator sees the spend.
    let cap = provider.cost_cap();
    assert!(
        (cap.spent_usd() - 0.000015).abs() < 1e-12,
        "cost_cap accumulated 0.000015; got {}",
        cap.spent_usd()
    );
    assert_eq!(cap.cap_usd(), Some(0.000010));

    // ---- Turn 2: must be rejected pre-flight ----
    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected CostCapExceeded on turn 2, got Ok stream"),
        Err(ProviderError::CostCapExceeded { cap_usd, spent_usd }) => {
            assert_eq!(cap_usd, 0.000010);
            assert!(
                (spent_usd - 0.000015).abs() < 1e-12,
                "spent_usd carried verbatim; got {spent_usd}"
            );
        }
        Err(other) => panic!("expected CostCapExceeded, got {other:?}"),
    }
}

#[tokio::test]
async fn max_usd_zero_rejected_at_builder_time() {
    // Zero is not a positive finite number → CostCap::with_cap rejects
    // → builder propagates ProviderError::Stream with clear message.
    let srv = MockServer::new().await;
    let result = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-cost-cap-zero")
        .allowlist(launch_allowlist())
        .max_usd(0.0)
        .build();
    match result {
        Ok(_) => panic!("build must reject max_usd(0.0)"),
        Err(ProviderError::Stream(msg)) => {
            assert!(
                msg.contains("--max-usd") || msg.contains("positive"),
                "error message should guide the user: got {msg:?}"
            );
        }
        Err(other) => panic!("expected ProviderError::Stream, got {other:?}"),
    }
}

#[tokio::test]
async fn max_usd_negative_rejected_at_builder_time() {
    let srv = MockServer::new().await;
    let result = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-cost-cap-neg")
        .allowlist(launch_allowlist())
        .max_usd(-1.0)
        .build();
    match result {
        Ok(_) => panic!("build must reject max_usd(-1.0)"),
        Err(ProviderError::Stream(_)) => {}
        Err(other) => panic!("expected ProviderError::Stream, got {other:?}"),
    }
}
