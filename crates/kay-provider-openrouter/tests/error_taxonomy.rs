//! Integration test: PROV-08 error taxonomy.
//!
//! Asserts that HTTP errors surface as the correct typed `ProviderError`
//! variant via `retry::classify_http_error` applied inside the retry loop's
//! `open_and_probe`:
//!
//!   - 401 → `Auth { reason: AuthErrorKind::Invalid }` (terminal, no retry)
//!   - 402 → `Http { status: 402, body }` (terminal, body preserved — used
//!     by OpenRouter for "insufficient credits")
//!   - 404 → `Http { status: 404, body }` (terminal)
//!   - 502 → retried 3× then surfaces `ServerError { status: 502 }`
//!
//! The classifier is also unit-tested in `retry::unit`; this module
//! verifies the end-to-end wire path.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use kay_provider_openrouter::{
    Allowlist, AuthErrorKind, ChatRequest, Message, OpenRouterProvider, Provider, ProviderError,
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
async fn http_401_classified_as_auth_invalid() {
    let mut srv = MockServer::new().await;
    let _m = srv
        .mock_status_body(
            401,
            r#"{"error":{"code":"invalid_api_key","message":"bad key"}}"#,
        )
        .await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-401")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected Auth::Invalid"),
        Err(ProviderError::Auth { reason: AuthErrorKind::Invalid }) => {}
        Err(other) => panic!("expected Auth::Invalid, got {other:?}"),
    }
}

#[tokio::test]
async fn http_402_preserves_body_as_http_variant() {
    let mut srv = MockServer::new().await;
    let _m = srv
        .mock_status_body(
            402,
            r#"{"error":{"code":"insufficient_credits","message":"top up"}}"#,
        )
        .await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-402")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected Http{{402}}"),
        Err(ProviderError::Http { status, body }) => {
            assert_eq!(status, 402);
            assert!(
                body.contains("insufficient_credits"),
                "402 body preserved verbatim for error surfacing; got {body:?}"
            );
        }
        Err(other) => panic!("expected Http{{402}}, got {other:?}"),
    }
}

#[tokio::test]
async fn http_404_preserves_body_as_http_variant() {
    let mut srv = MockServer::new().await;
    let _m = srv
        .mock_status_body(
            404,
            r#"{"error":{"code":"not_found","message":"no such route"}}"#,
        )
        .await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-404")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected Http{{404}}"),
        Err(ProviderError::Http { status, body }) => {
            assert_eq!(status, 404);
            assert!(
                body.contains("not_found"),
                "404 body preserved verbatim; got {body:?}"
            );
        }
        Err(other) => panic!("expected Http{{404}}, got {other:?}"),
    }
}

#[tokio::test]
async fn http_502_exhausts_retries_as_server_error() {
    // 502 is retryable under `is_retryable`. Three consecutive 502s exhaust
    // backon and surface ServerError { status: 502 } (no body per D-05 —
    // ServerError carries status only, body is intentionally dropped).
    let mut srv = MockServer::new().await;
    let _m1 = srv.mock_status_body(502, "bad gateway 1").await;
    let _m2 = srv.mock_status_body(502, "bad gateway 2").await;
    let _m3 = srv.mock_status_body(502, "bad gateway 3").await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-502")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected ServerError{{502}}"),
        Err(ProviderError::ServerError { status }) => {
            assert_eq!(status, 502, "status preserved through retry loop");
        }
        Err(other) => panic!("expected ServerError{{502}}, got {other:?}"),
    }
}

#[tokio::test]
async fn transport_failure_classified_as_network_error() {
    // Point at an unreachable endpoint so reqwest's transport layer fails
    // before any HTTP response comes back. Retry fires 3× on Network per
    // `is_retryable` and then surfaces Network.
    //
    // Port 1 is commonly firewalled (tcpmux); if it happens to be open on
    // a CI box, the TLS handshake will still fail fast. Either way we get
    // a Transport error, not a real response.
    let provider = OpenRouterProvider::builder()
        .endpoint("http://127.0.0.1:1/api/v1/chat/completions")
        .api_key("test-key-transport")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    match provider.chat(base_req()).await {
        Ok(_) => panic!("expected Network error from unreachable endpoint"),
        Err(ProviderError::Network(_)) => {}
        // Transport errors can also surface as Http/Stream depending on
        // exact failure point; the test intent is that a transport-layer
        // problem never produces a bogus Auth or ServerError.
        Err(other) => {
            panic!("expected Network, got {other:?} — transport errors must not be misclassified")
        }
    }
}
