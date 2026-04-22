//! T2-03: Event ordering integration tests for MultiPerspectiveVerifier.
//!
//! Tests verify that:
//! - Verification events are emitted in the correct critic order (VERIFY-04)
//! - In Benchmark mode: test_engineer → qa_engineer → end_user
//! - VerifierDisabled follows Verification events when ceiling is breached

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use kay_provider_openrouter::{Allowlist, CostCap, OpenRouterProvider};
use kay_tools::events::AgentEvent;
use kay_tools::seams::verifier::TaskVerifier;
use mockito::Server;

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
) -> (Arc<Mutex<Vec<AgentEvent>>>, kay_verifier::MultiPerspectiveVerifier) {
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
// T2-03a: Benchmark mode event order: TestEngineer → QAEngineer → EndUser
// Ceiling $1.00, critics cost $0.01 each — all three run.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_03a_benchmark_mode_emits_events_in_role_order() {
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
        build_verifier(&srv.url(), kay_verifier::VerifierMode::Benchmark, 1.0);
    let outcome = verifier.verify("summary", "context").await;

    let guard = events.lock().unwrap();
    let verification_events: Vec<&AgentEvent> = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .collect();

    assert_eq!(
        verification_events.len(),
        3,
        "T2-03a: expected 3 Verification events (Benchmark mode), got {}. Events: {guard:?}",
        verification_events.len()
    );

    let roles: Vec<String> = verification_events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Verification { critic_role, .. } => Some(critic_role.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        roles[0], "test_engineer",
        "T2-03a: first event should be test_engineer. Events: {guard:?}"
    );
    assert_eq!(
        roles[1], "qa_engineer",
        "T2-03a: second event should be qa_engineer. Events: {guard:?}"
    );
    assert_eq!(
        roles[2], "end_user",
        "T2-03a: third event should be end_user. Events: {guard:?}"
    );

    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-03a: expected Pass (all critics passed), got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-03b: VerifierDisabled follows all Verification events (ordering).
// Benchmark, ceiling $0.05, critics at $0.03 each.
// After critic 2: accumulated $0.06 > $0.05 → VerifierDisabled.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_03b_verifier_disabled_follows_all_verification_events() {
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

    let last_verification_pos =
        guard.iter().rposition(|e| matches!(e, AgentEvent::Verification { .. }));
    let first_disabled_pos =
        guard.iter().position(|e| matches!(e, AgentEvent::VerifierDisabled { .. }));

    assert!(
        first_disabled_pos.is_some(),
        "T2-03b: expected VerifierDisabled (ceiling $0.05, critics cost $0.03 each). \
         Events: {guard:?}"
    );

    if let (Some(last_v), Some(first_d)) = (last_verification_pos, first_disabled_pos) {
        assert!(
            last_v < first_d,
            "T2-03b: Verification events must precede VerifierDisabled. \
             Last Verification at {last_v}, first VerifierDisabled at {first_d}. \
             Events: {guard:?}"
        );
    }

    if let Some(AgentEvent::VerifierDisabled { reason, cost_usd }) =
        guard.iter().find(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
    {
        assert_eq!(
            reason.as_str(),
            "cost_ceiling_exceeded",
            "T2-03b: VerifierDisabled reason must be 'cost_ceiling_exceeded', got '{reason}'"
        );
        assert!(*cost_usd >= 0.0, "T2-03b: cost_usd must be non-negative, got {cost_usd}");
    }

    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-03b: expected Pass on ceiling breach, got {outcome:?}"
    );
}
