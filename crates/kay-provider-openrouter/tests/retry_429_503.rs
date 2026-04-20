//! Integration test: PROV-07 retry policy, end-to-end through `OpenRouterProvider::chat`.
//!
//! Covers:
//!   - 429 (Retry-After: 1) → success (200): a single `AgentEvent::Retry`
//!     frame precedes the normal TextDelta + Usage stream; attempt == 1,
//!     reason == RateLimited, delay matches the Retry-After header
//!     (overrides backon per D-09 + Pitfall 6).
//!   - 503 × 3 attempts: backon's max_times = 3 is honored; after 3
//!     attempts `chat()` surfaces `ProviderError::ServerError`.
//!
//! We cannot pause tokio time here because mockito + reqwest use the
//! tokio runtime and paused time breaks real TCP I/O. The 429 test
//! therefore waits the real 1 second; the 503 test rides backon's
//! schedule which maxes out at ~3.5s (500ms + 1s + 2s, with jitter).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider, ProviderError,
    RetryReason,
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
async fn rate_limit_429_retries_then_succeeds() {
    // Sequence: POST → 429 (Retry-After: 1) → POST → 200 (happy_path SSE).
    // Expect: one AgentEvent::Retry frame (RateLimited, attempt 1, 1000ms)
    // followed by the normal TextDelta + Usage stream.
    let mut srv = MockServer::new().await;
    let _m_429 = srv.mock_rate_limit(1).await;
    let events = MockServer::load_sse_cassette("happy_path");
    let _m_200 = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-retry-429")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let mut stream = provider.chat(base_req()).await.expect("stream");

    let mut retries: Vec<(u32, u64, RetryReason)> = Vec::new();
    let mut text_chunks: Vec<String> = Vec::new();
    let mut usage_seen = false;

    while let Some(ev) = stream.next().await {
        match ev {
            Ok(AgentEvent::Retry { attempt, delay_ms, reason }) => {
                retries.push((attempt, delay_ms, reason))
            }
            Ok(AgentEvent::TextDelta { content }) => text_chunks.push(content),
            Ok(AgentEvent::Usage { .. }) => usage_seen = true,
            Ok(_) => {}
            Err(e) => panic!("stream errored: {e:?}"),
        }
    }

    assert_eq!(retries.len(), 1, "expected exactly one Retry frame");
    assert_eq!(retries[0].0, 1, "attempt must be 1");
    assert_eq!(retries[0].2, RetryReason::RateLimited, "RateLimited reason");
    assert_eq!(
        retries[0].1, 1000,
        "Retry-After: 1 overrides backon → 1000ms"
    );
    assert_eq!(
        text_chunks,
        vec!["Hello".to_string(), " world".to_string()],
        "post-retry happy path delivered"
    );
    assert!(usage_seen, "Usage frame emitted after retry");
}

#[tokio::test]
async fn server_error_503_retries_then_exhausts() {
    // Three consecutive 503s. retry_with_emitter_using exhausts backon's
    // 3-attempt schedule and surfaces ProviderError::ServerError from the
    // outer `chat().await`. Because exhaustion short-circuits the pre_events
    // chaining path (we return the Err from chat() directly rather than
    // turning it into a stream frame), we assert on the `chat()` return,
    // not on a stream item.
    let mut srv = MockServer::new().await;
    let _m1 = srv.mock_server_error_503().await;
    let _m2 = srv.mock_server_error_503().await;
    let _m3 = srv.mock_server_error_503().await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-retry-503")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected ServerError exhaustion, got Ok"),
        Err(ProviderError::ServerError { status }) => {
            assert_eq!(status, 503, "status preserved");
        }
        Err(other) => panic!("expected ServerError, got {other:?}"),
    }
}
