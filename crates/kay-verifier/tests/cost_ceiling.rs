//! T2-02: Cost ceiling integration tests for MultiPerspectiveVerifier.
//!
//! Tests verify that:
//! - Cost accumulates correctly from provider usage events
//! - VerifierDisabled fires before a critic when accumulated cost > ceiling
//! - VerifierDisabled reason = "cost_ceiling_exceeded" (VERIFY-03)
//! - No Verification events appear after VerifierDisabled
//!
//! These tests use mockito to mock OpenRouterProvider HTTP responses.
//! They FAIL in RED because the stub verify() returns Pass without calling
//! the provider or emitting events.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use kay_provider_openrouter::{Allowlist, CostCap, OpenRouterProvider};
use kay_tools::events::AgentEvent;
use kay_tools::seams::verifier::TaskVerifier;
use mockito::Server;

// ---------------------------------------------------------------------------
// SSE helper: returns a pass verdict with the given USD cost in the usage chunk.
// ---------------------------------------------------------------------------

fn critic_pass_sse(cost_usd: f64) -> String {
    let chunk = serde_json::json!({
        "choices": [{"delta": {"content": r#"{"verdict":"pass","reason":"looks good"}"#}}],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15,
            "cost": cost_usd
        }
    });
    format!("data: {chunk}\n\ndata: [DONE]")
}

fn build_verifier(
    server_url: &str,
    mode: kay_verifier::VerifierMode,
    cost_ceiling_usd: f64,
) -> (
    Arc<Mutex<Vec<AgentEvent>>>,
    kay_verifier::MultiPerspectiveVerifier,
) {
    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", server_url))
            .api_key("test-key")
            .allowlist(Allowlist::from_models(vec!["openai/gpt-4o-mini".into()]))
            .build()
            .expect("build provider"),
    );
    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cl = Arc::clone(&events);
    let sink = Arc::new(move |ev: AgentEvent| {
        events_cl.lock().unwrap().push(ev);
    }) as Arc<dyn Fn(AgentEvent) + Send + Sync>;
    let verifier = kay_verifier::MultiPerspectiveVerifier::new(
        provider,
        Arc::new(CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode,
            max_retries: 3,
            cost_ceiling_usd,
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );
    (events, verifier)
}

// ---------------------------------------------------------------------------
// T2-02a: Benchmark mode, ceiling $0.01, each critic costs $0.02.
// Critic 1 runs → Verification emitted, accumulated = $0.02.
// Before critic 2: $0.02 > $0.01 → VerifierDisabled fires.
// Result: 1 Verification + ≥1 VerifierDisabled.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02a_ceiling_breached_after_critic_1() {
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(0.02);
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) =
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Benchmark, 0.01);
    let outcome = verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    let verification_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .count();
    let disabled_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
        .count();

    assert_eq!(
        verification_count, 1,
        "T2-02a: expected 1 Verification event (critic 1 ran before ceiling check), \
         got {verification_count}. Events: {guard:?}"
    );
    assert!(
        disabled_count >= 1,
        "T2-02a: expected >=1 VerifierDisabled (ceiling breached after critic 1), \
         got {disabled_count}. Events: {guard:?}"
    );
    assert!(
        matches!(
            outcome,
            kay_tools::seams::verifier::VerificationOutcome::Pass { .. }
        ),
        "T2-02a: expected Pass on ceiling breach (graceful degradation), got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02b: Benchmark mode, ceiling $0.10, 3 critics at $0.02 each = $0.06 total.
// All 3 run. 3 Verification events, 0 VerifierDisabled.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02b_all_critics_run_when_under_ceiling() {
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(0.02);
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) =
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Benchmark, 0.10);
    let outcome = verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    let verification_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .count();
    let disabled_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
        .count();

    assert_eq!(
        verification_count, 3,
        "T2-02b: expected 3 Verification events (3 x $0.02 = $0.06 < $0.10 ceiling), \
         got {verification_count}. Events: {guard:?}"
    );
    assert_eq!(
        disabled_count, 0,
        "T2-02b: expected 0 VerifierDisabled events (all under ceiling), \
         got {disabled_count}. Events: {guard:?}"
    );
    assert!(
        matches!(
            outcome,
            kay_tools::seams::verifier::VerificationOutcome::Pass { .. }
        ),
        "T2-02b: expected Pass when all critics pass, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02c: Benchmark mode, ceiling $0.05, critics at $0.03 each.
// After critic 1: $0.03. After critic 2: $0.06 > $0.05.
// Before critic 3: VerifierDisabled fires. Result: 2 Verification + >=1 VerifierDisabled.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02c_ceiling_breached_after_critic_2() {
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(0.03);
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) =
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Benchmark, 0.05);
    let outcome = verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    let verification_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .count();
    let disabled_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
        .count();

    assert_eq!(
        verification_count, 2,
        "T2-02c: expected 2 Verification events (critics 1 and 2 ran), \
         got {verification_count}. Events: {guard:?}"
    );
    assert!(
        disabled_count >= 1,
        "T2-02c: expected >=1 VerifierDisabled event (ceiling breached after critic 2), \
         got {disabled_count}. Events: {guard:?}"
    );
    assert!(
        matches!(
            outcome,
            kay_tools::seams::verifier::VerificationOutcome::Pass { .. }
        ),
        "T2-02c: expected Pass on ceiling breach, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02d: Interactive mode, ceiling $1.0, 1 critic at $0.01.
// Verification event has positive cost_usd.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02d_verification_event_has_positive_cost() {
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(0.01);
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) =
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Interactive, 1.0);
    verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    let costs: Vec<f64> = guard
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Verification { cost_usd, .. } => Some(*cost_usd),
            _ => None,
        })
        .collect();

    assert_eq!(
        costs.len(),
        1,
        "T2-02d: expected 1 Verification event (Interactive = 1 critic), \
         got {}. Events: {guard:?}",
        costs.len()
    );
    assert!(
        costs[0] > 0.0,
        "T2-02d: cost_usd in Verification event should be positive, got {}",
        costs[0]
    );
}

// ---------------------------------------------------------------------------
// T2-02e: After VerifierDisabled fires, no Verification events appear.
// Benchmark, ceiling $0.05, critics at $0.03 each.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02e_no_verification_after_ceiling_breach() {
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(0.03);
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) =
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Benchmark, 0.05);
    verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    if let Some(disabled_pos) = guard
        .iter()
        .position(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
    {
        let after = &guard[disabled_pos + 1..];
        let verification_after = after
            .iter()
            .filter(|e| matches!(e, AgentEvent::Verification { .. }))
            .count();
        assert_eq!(
            verification_after, 0,
            "T2-02e: expected 0 Verification events after VerifierDisabled, \
             got {verification_after}. Events: {guard:?}"
        );
    } else {
        panic!(
            "T2-02e: expected VerifierDisabled event (ceiling breached after 2 critics \
             at $0.03 each = $0.06 > $0.05 ceiling). Events: {guard:?}"
        );
    }
}
