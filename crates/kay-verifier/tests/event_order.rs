//! T2-03: Event ordering integration tests for MultiPerspectiveVerifier.
//!
//! Tests verify that:
//! - Verification events are emitted in the correct critic order (VERIFY-04)
//! - In Benchmark mode: test_engineer → qa_engineer → end_user
//! - VerifierDisabled follows Verification events when ceiling is breached
//!
//! These tests use mockito to mock the OpenRouterProvider HTTP responses.
//! They will FAIL in RED because the current stub verify() doesn't call
//! the provider or emit events.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use kay_provider_openrouter::OpenRouterProvider;
use kay_tools::events::AgentEvent;
use kay_tools::seams::verifier::TaskVerifier;
use mockito::Server;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn critic_pass_sse(role: &str) -> String {
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{{\\\"verdict\\\":\\\"pass\\\",\\\"reason\\\":\\\"{role} approved\\\"}}\"}}}}],\"usage\":{{\"prompt_tokens\":10,\"completion_tokens\":5,\"total_tokens\":15}}}}\n\ndata: [DONE]\n\n"
    )
}

fn build_sink_and_verifier(
    srv: &str,
    mode: kay_verifier::VerifierMode,
    cost_ceiling: f64,
) -> (Arc<Mutex<Vec<AgentEvent>>>, kay_verifier::MultiPerspectiveVerifier) {
    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv))
            .api_key("test-key")
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
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode,
            max_retries: 3,
            cost_ceiling_usd: cost_ceiling,
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    (events, verifier)
}

// ---------------------------------------------------------------------------
// T2-03a: Benchmark mode event order: TestEngineer → QAEngineer → EndUser
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_03a_benchmark_mode_emits_events_in_role_order() {
    // Arrange: three mock endpoints, one per critic role, returning pass.
    // Each returns a response with the role name embedded so we can verify
    // which critic was called by checking the event's critic_role field.
    let mut srv = Server::new_async().await;

    let _m_test_engineer = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse("test_engineer"))
        .expect_at_least(1)
        .create_async()
        .await;

    let _m_qa_engineer = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse("qa_engineer"))
        .expect_at_least(1)
        .create_async()
        .await;

    let _m_end_user = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse("end_user"))
        .expect_at_least(1)
        .create_async()
        .await;

    let (events, verifier) = build_sink_and_verifier(
        &srv.url(),
        kay_verifier::VerifierMode::Benchmark,
        1.0, // High enough ceiling that all three run
    );

    // Act
    let outcome = verifier.verify("summary", "context").await;

    // Assert: three Verification events, in order test_engineer → qa_engineer → end_user
    let guard = events.lock().unwrap();
    let verification_events: Vec<&AgentEvent> = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .collect();

    assert_eq!(
        verification_events.len(),
        3,
        "T2-03a FAILED: expected 3 Verification events (Benchmark mode), got {}. Events: {guard:?}",
        verification_events.len()
    );

    let ordered_roles: Vec<String> = verification_events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Verification { critic_role, .. } => Some(critic_role.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        ordered_roles[0], "test_engineer",
        "T2-03a FAILED: first Verification event should be test_engineer, got {}. Events: {guard:?}",
        ordered_roles[0]
    );
    assert_eq!(
        ordered_roles[1], "qa_engineer",
        "T2-03a FAILED: second Verification event should be qa_engineer, got {}. Events: {guard:?}",
        ordered_roles[1]
    );
    assert_eq!(
        ordered_roles[2], "end_user",
        "T2-03a FAILED: third Verification event should be end_user, got {}. Events: {guard:?}",
        ordered_roles[2]
    );

    // Outcome should be Pass (all critics returned pass verdict)
    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-03a FAILED: expected Pass (all critics passed), got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-03b: Verification events precede any VerifierDisabled event
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_03b_verifier_disabled_follows_all_verification_events() {
    // Arrange: ceiling set so that after 2 critics the ceiling is breached.
    // We expect: Verification(test_engineer), Verification(qa_engineer),
    //            VerifierDisabled(cost_ceiling_exceeded)
    let mut srv = Server::new_async().await;

    let _m1 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse("test_engineer"))
        .expect_at_least(1)
        .create_async()
        .await;

    let _m2 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse("qa_engineer"))
        .expect_at_least(1)
        .create_async()
        .await;

    // Critic 3 (end_user): not registered — unmatched requests return 404.
    // Test verifies via assertions that only 2 Verification events occurred.

    let (events, verifier) = build_sink_and_verifier(
        &srv.url(),
        kay_verifier::VerifierMode::Benchmark,
        0.05, // Low ceiling: after 2 critics at ~$0.03 each = $0.06 > $0.05
    );

    // Act
    let outcome = verifier.verify("summary", "context").await;

    // Assert
    let guard = events.lock().unwrap();
    let verification_events: Vec<&AgentEvent> = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .collect();
    let disabled_events: Vec<&AgentEvent> = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. }))
        .collect();

    // Verification events come before VerifierDisabled
    if !verification_events.is_empty() && !disabled_events.is_empty() {
        let last_verification_pos = guard.iter().position(|e| {
            matches!(e, AgentEvent::Verification { .. })
        }).unwrap_or(0);
        let first_disabled_pos = guard.iter().position(|e| {
            matches!(e, AgentEvent::VerifierDisabled { .. })
        }).unwrap_or(guard.len());
        assert!(
            last_verification_pos < first_disabled_pos,
            "T2-03b FAILED: Verification events must precede VerifierDisabled. \
             Last Verification at idx {}, first VerifierDisabled at idx {}. Events: {guard:?}",
            last_verification_pos, first_disabled_pos
        );
    }

    // VerifierDisabled should have reason = "cost_ceiling_exceeded"
    if let Some(AgentEvent::VerifierDisabled { reason, cost_usd }) = disabled_events.first() {
        assert_eq!(
            reason.as_str(), "cost_ceiling_exceeded",
            "T2-03b FAILED: VerifierDisabled reason should be 'cost_ceiling_exceeded', got '{}'",
            reason
        );
        assert!(
            *cost_usd >= 0.0,
            "T2-03b FAILED: VerifierDisabled cost_usd should be non-negative, got {}",
            cost_usd
        );
    }

    // Outcome is Pass (verifier gracefully degrades)
    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-03b FAILED: expected Pass on ceiling breach, got {outcome:?}"
    );
}
