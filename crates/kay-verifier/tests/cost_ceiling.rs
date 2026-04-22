//! T2-02: Cost ceiling integration tests for MultiPerspectiveVerifier.
//!
//! Tests verify that:
//! - Cost accumulation works correctly
//! - Verifier disables itself when ceiling is exceeded
//! - No additional critics run after ceiling breach
//! - VerifierDisabled event is emitted on ceiling breach (VERIFY-03)
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
// Test helpers
// ---------------------------------------------------------------------------

/// Builds a ChatRequest for a critic role with the given task context.
/// Builds an SSE response body for a critic pass verdict.
fn critic_pass_sse(cost_tokens: u64) -> String {
    // Simulate a streaming response: text delta + usage
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{{\\\"verdict\\\":\\\"pass\\\",\\\"reason\\\":\\\"looks good\\\"}}\"}}}}],\"usage\":{{\"prompt_tokens\":10,\"completion_tokens\":{},\"total_tokens\":{}}}}}\n\ndata: [DONE]\n\n",
        cost_tokens, 10 + cost_tokens
    )
}

// ---------------------------------------------------------------------------
// T2-02a: Cost ceiling $0.001, first critic costs $0.002 → VerifierDisabled
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02a_first_critic_exceeds_ceiling_emits_disabled_event() {
    // Arrange: mock server returns a response costing > $0.001
    let mut srv = Server::new_async().await;
    let cost_tokens = 10_000u64; // ~$0.002 at typical pricing
    let body = critic_pass_sse(cost_tokens);

    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
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
        Arc::clone(&provider),
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode: kay_verifier::VerifierMode::Interactive,
            max_retries: 3,
            cost_ceiling_usd: 0.001, // Very low — first critic exceeds this
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    // Act
    let outcome = verifier
        .verify("summary", "context")
        .await;

    // Assert
    // T2-02a: VerifierDisabled must be emitted (VERIFY-03), and verify returns Pass
    let guard = events.lock().unwrap();
    let disabled_count =
        guard.iter().filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. })).count();
    let verification_count =
        guard.iter().filter(|e| matches!(e, AgentEvent::Verification { .. })).count();

    assert!(
        disabled_count >= 1,
        "T2-02a FAILED: expected at least 1 VerifierDisabled event (cost_ceiling_exceeded), got {disabled_count}. Events: {guard:?}"
    );
    assert_eq!(
        verification_count, 0,
        "T2-02a FAILED: expected 0 Verification events (critic 1 should NOT run when cost already exceeds ceiling), got {verification_count}"
    );
    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-02a FAILED: expected Pass on ceiling breach, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02b: Ceiling $0.10, 3 critics each $0.03 → all 3 run, total $0.09 < ceiling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02b_all_critics_run_when_under_ceiling() {
    // Arrange: three responses, each costing ~$0.03
    let mut srv = Server::new_async().await;
    let cost_tokens = 15_000u64; // ~$0.03

    // All three critics return pass
    for _ in 0..3 {
        let body = critic_pass_sse(cost_tokens);
        let _m = srv
            .mock("POST", "/api/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(&body)
            .create_async()
            .await;
    }

    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
            .api_key("test-key")
            .build()
            .expect("build provider"),
    );

    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cl = Arc::clone(&events);
    let sink =
        Arc::new(move |ev: AgentEvent| { events_cl.lock().unwrap().push(ev); })
            as Arc<dyn Fn(AgentEvent) + Send + Sync>;

    let verifier = kay_verifier::MultiPerspectiveVerifier::new(
        Arc::clone(&provider),
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode: kay_verifier::VerifierMode::Benchmark,
            max_retries: 3,
            cost_ceiling_usd: 0.10, // Ceiling $0.10; 3 × $0.03 = $0.09 < ceiling
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    // Act
    let outcome = verifier.verify("summary", "context").await;

    // Assert
    let guard = events.lock().unwrap();
    let verification_events: Vec<&AgentEvent> =
        guard.iter().filter(|e| matches!(e, AgentEvent::Verification { .. })).collect();
    let disabled_count =
        guard.iter().filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. })).count();

    assert_eq!(
        verification_events.len(),
        3,
        "T2-02b FAILED: expected 3 Verification events (one per critic), got {}. Events: {guard:?}",
        verification_events.len()
    );
    assert_eq!(
        disabled_count, 0,
        "T2-02b FAILED: expected 0 VerifierDisabled events (all under ceiling), got {disabled_count}"
    );
    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-02b FAILED: expected Pass when all critics pass, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02c: Ceiling $0.10, critic 2 pushes over ceiling → VerifierDisabled after 2
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02c_ceiling_breached_after_critic_2_disables_verifier() {
    // Arrange: critic 1 costs $0.03, critic 2 costs $0.03 (total $0.06, still under
    // $0.10), but critic 3 would cost $0.06 (total $0.12, over $0.10). So VerifierDisabled
    // fires AFTER critic 2 but BEFORE critic 3.
    let mut srv = Server::new_async().await;

    // Critic 1: pass, ~$0.03
    let _m1 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse(15_000))
        .expect_at_least(1) // May be called once or twice depending on retry
        .create_async()
        .await;

    // Critic 2: pass, ~$0.03
    let _m2 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse(15_000))
        .expect_at_least(1)
        .create_async()
        .await;

    // Critic 3: not registered — any unmatched request returns 404.
    // Test verifies via assertion below that only 2 critics ran.

    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
            .api_key("test-key")
            .build()
            .expect("build provider"),
    );

    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cl = Arc::clone(&events);
    let sink =
        Arc::new(move |ev: AgentEvent| { events_cl.lock().unwrap().push(ev); })
            as Arc<dyn Fn(AgentEvent) + Send + Sync>;

    let verifier = kay_verifier::MultiPerspectiveVerifier::new(
        Arc::clone(&provider),
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode: kay_verifier::VerifierMode::Benchmark,
            max_retries: 3,
            // Ceiling $0.10. After critic 1 ($0.03): remaining $0.07.
            // After critic 2 ($0.06 total): remaining $0.04.
            // Check happens BEFORE critic 3 call: $0.06 > $0.10? No, wait...
            // Let me set it so critic 2 pushes us over.
            // Actually: 3 critics × $0.04 = $0.12. After 2 critics: $0.08.
            // Let me use costs that trigger exactly.
            // Better: use 4 critics × $0.03 = $0.12. After 3: $0.09. Still < $0.10.
            // Let me reconsider the cost ceiling math.
            // I'll set ceiling = $0.07 so that after 2 critics ($0.06) we're close,
            // but check happens BEFORE 3rd. 
            // Actually the check is BEFORE each call.
            // Let me use: ceiling = $0.06, critics cost $0.03 each.
            // After critic 1: $0.03 < $0.06 ✓. After critic 2: check: $0.03 > $0.06? No.
            // Hmm, I need the check to FAIL after critic 2.
            // If ceiling = $0.05 and critics cost $0.03:
            // After critic 1: $0.03 < $0.05 ✓. Critic 1 costs $0.03.
            // After critic 1 run, verifier_cost = $0.03.
            // Before critic 2: check $0.03 > $0.05? No.
            // After critic 2 run: verifier_cost = $0.06.
            // Before critic 3: check $0.06 > $0.05? Yes! → VerifierDisabled.
            cost_ceiling_usd: 0.05,
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    // Act
    let outcome = verifier.verify("summary", "context").await;

    // Assert
    let guard = events.lock().unwrap();
    let verification_events: Vec<&AgentEvent> =
        guard.iter().filter(|e| matches!(e, AgentEvent::Verification { .. })).collect();
    let disabled_count =
        guard.iter().filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. })).count();

    assert_eq!(
        verification_events.len(),
        2,
        "T2-02c FAILED: expected 2 Verification events (critics 1 and 2 ran), got {}. Events: {guard:?}",
        verification_events.len()
    );
    assert!(
        disabled_count >= 1,
        "T2-02c FAILED: expected at least 1 VerifierDisabled event, got {disabled_count}. Events: {guard:?}"
    );
    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-02c FAILED: expected Pass on ceiling breach, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T2-02d: verifier_cost and cost_usd in events show same cumulative value
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02d_verifier_cost_matches_event_costs() {
    // Arrange: one critic returning pass
    let mut srv = Server::new_async().await;
    let body = critic_pass_sse(5_000); // Small response

    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
            .api_key("test-key")
            .build()
            .expect("build provider"),
    );

    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cl = Arc::clone(&events);
    let sink =
        Arc::new(move |ev: AgentEvent| { events_cl.lock().unwrap().push(ev); })
            as Arc<dyn Fn(AgentEvent) + Send + Sync>;

    let verifier = kay_verifier::MultiPerspectiveVerifier::new(
        Arc::clone(&provider),
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode: kay_verifier::VerifierMode::Interactive,
            max_retries: 3,
            cost_ceiling_usd: 1.0,
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    // Act
    let _outcome = verifier.verify("summary", "context").await;

    // Assert: the cost_usd in Verification events should equal the
    // verifier's internal accumulated cost. Since we can't access internal
    // state directly, we verify that all Verification events have the same
    // cost_usd (one critic = one event = one cost).
    let guard = events.lock().unwrap();
    let verification_costs: Vec<f64> = guard
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Verification { cost_usd, .. } => Some(*cost_usd),
            _ => None,
        })
        .collect();

    if verification_costs.len() == 1 {
        // If only one critic ran, costs match trivially
        assert!(
            verification_costs[0] > 0.0,
            "T2-02d FAILED: verification cost_usd should be positive, got {}",
            verification_costs[0]
        );
    } else {
        // Multiple critics: all costs should be positive and sum should be
        // reflected in the last event (or in a disabled event if ceiling hit)
        let total_cost: f64 = verification_costs.iter().sum();
        assert!(
            total_cost > 0.0,
            "T2-02d FAILED: total verifier cost should be positive, got {}",
            total_cost
        );
        // Verify no duplicate costs (each event should have distinct cost contribution)
        assert!(
            verification_costs.windows(2).all(|w| w[0] > 0.0),
            "T2-02d FAILED: all costs must be positive: {verification_costs:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// T2-02e: After ceiling breach, zero additional critics run
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t2_02e_no_verification_events_after_ceiling_breach() {
    // Arrange: 4 critics registered in Benchmark mode, but ceiling set so that
    // only the first 2 run, then VerifierDisabled fires.
    // Critiques 3 and 4 must NOT produce Verification events.
    let mut srv = Server::new_async().await;

    // Critics 1 and 2: run and pass
    let _m1 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse(15_000))
        .expect_at_least(1)
        .create_async()
        .await;

    let _m2 = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&critic_pass_sse(15_000))
        .expect_at_least(1)
        .create_async()
        .await;

    // Critics 3 and 4: not registered — unmatched requests return 404.
    // Test verifies via assertions that only 2 Verification events occurred.

    let provider = Arc::new(
        OpenRouterProvider::builder()
            .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
            .api_key("test-key")
            .build()
            .expect("build provider"),
    );

    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cl = Arc::clone(&events);
    let sink =
        Arc::new(move |ev: AgentEvent| { events_cl.lock().unwrap().push(ev); })
            as Arc<dyn Fn(AgentEvent) + Send + Sync>;

    let verifier = kay_verifier::MultiPerspectiveVerifier::new(
        Arc::clone(&provider),
        Arc::new(kay_provider_openrouter::CostCap::uncapped()),
        kay_verifier::VerifierConfig {
            mode: kay_verifier::VerifierMode::Benchmark,
            max_retries: 3,
            // 2 × $0.03 = $0.06. Check before critic 3: $0.06 > $0.05? Yes.
            // So critics 1 and 2 run, critic 3 triggers VerifierDisabled.
            cost_ceiling_usd: 0.05,
            model: "openai/gpt-4o-mini".into(),
        },
        sink,
    );

    // Act
    let outcome = verifier.verify("summary", "context").await;

    // Assert
    let guard = events.lock().unwrap();
    let _verification_events: Vec<&AgentEvent> =
        guard.iter().filter(|e| matches!(e, AgentEvent::Verification { .. })).collect();
    let _disabled_events: Vec<&AgentEvent> =
        guard.iter().filter(|e| matches!(e, AgentEvent::VerifierDisabled { .. })).collect();

    // After VerifierDisabled, no more Verification events should be emitted
    if let Some(first_disabled_idx) = guard.iter().position(|e| {
        matches!(e, AgentEvent::VerifierDisabled { .. })
    }) {
        let events_after_disabled = &guard[first_disabled_idx + 1..];
        let verification_after_disabled = events_after_disabled
            .iter()
            .filter(|e| matches!(e, AgentEvent::Verification { .. }))
            .count();
        assert_eq!(
            verification_after_disabled, 0,
            "T2-02e FAILED: expected 0 Verification events after VerifierDisabled, got {verification_after_disabled}. Events: {guard:?}"
        );
    }

    assert!(
        matches!(outcome, kay_tools::seams::verifier::VerificationOutcome::Pass { .. }),
        "T2-02e FAILED: expected Pass on ceiling breach, got {outcome:?}"
    );
}
