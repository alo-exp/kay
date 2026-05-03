//! Integration tests for MultiPerspectiveVerifier.
//!
//! These tests exercise the full verification flow with mocked OpenRouter
//! responses. They complement the unit tests in src/verifier.rs and the
//! property tests in src/critic.rs (T4-01).
//!
//! Phase 8 W-6: cost_ceiling.rs and event_order.rs live here.

#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use kay_provider_openrouter::{Allowlist, CostCap, OpenRouterProvider};
use kay_tools::{events::AgentEvent, seams::verifier::TaskVerifier};

/// Build a verifier with a mock server URL and given config.
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
// T3-01: Full turn in Benchmark mode with 3 PASS critics → TurnResult::Verified
// ---------------------------------------------------------------------------

/// T3-01: E2E test for Benchmark mode with all critics passing.
/// Uses mockito to mock the OpenRouter HTTP responses.
#[tokio::test]
async fn t3_01_benchmark_all_pass() {
    use mockito::Server;

    let mut srv = Server::new_async().await;

    // SSE response with all 3 critics passing
    let body = r#"data: {"choices":[{"delta":{"content":"{\"verdict\":\"pass\",\"reason\":\"all tests pass\"}"}}],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15,"cost":0.001}}
data: [DONE]"#;

    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) = build_verifier(
        &srv.url(),
        kay_verifier::VerifierMode::Benchmark,
        1.0, // generous ceiling
    );

    let outcome = verifier
        .verify("Fixed the CSV parser to handle quoted fields", "called tool: fs_read\ncalled tool: fs_write\n")
        .await;

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
        "T3-01: expected 3 Verification events (Benchmark mode), got {verification_count}"
    );
    assert_eq!(
        disabled_count, 0,
        "T3-01: expected 0 VerifierDisabled events (all critics under ceiling), got {disabled_count}"
    );
    assert!(
        matches!(
            outcome,
            kay_tools::seams::verifier::VerificationOutcome::Pass { .. }
        ),
        "T3-01: expected Pass when all critics pass, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// T3-02: Interactive mode — fail verdict is detected correctly
// ---------------------------------------------------------------------------

/// T3-02: Interactive mode with a critic returning FAIL verdict.
/// Verifies that the verifier correctly parses "fail" verdict and returns Fail outcome.
#[tokio::test]
async fn t3_02_interactive_fail_detected() {
    use mockito::Server;

    let mut srv = Server::new_async().await;

    // SSE response with fail verdict
    let fail_body = format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{{\\\"verdict\\\":\\\"fail\\\",\\\"reason\\\":\\\"edge case not handled\\\"}}\"}}}}],\"usage\":{{\"prompt_tokens\":10,\"completion_tokens\":5,\"total_tokens\":15,\"cost\":0.001}}}}\n\ndata: [DONE]"
    );

    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&fail_body)
        .create_async()
        .await;

    let (events, verifier) = build_verifier(
        &srv.url(),
        kay_verifier::VerifierMode::Interactive,
        1.0,
    );

    let outcome = verifier
        .verify("Implemented a feature", "called tool: fs_read\n")
        .await;

    let guard = events.lock().unwrap();

    // Count Verification events
    let verification_count = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { .. }))
        .count();

    // Interactive mode has 1 critic
    assert_eq!(
        verification_count, 1,
        "T3-02: expected 1 Verification event (Interactive = 1 critic), got {verification_count}"
    );

    // Check that the verdict was "fail"
    let fail_events = guard
        .iter()
        .filter(|e| matches!(e, AgentEvent::Verification { verdict: v, .. } if v == "fail"))
        .count();
    assert_eq!(
        fail_events, 1,
        "T3-02: expected 1 Verification event with verdict='fail', got {fail_events}"
    );

    // Verify the outcome is Fail
    assert!(
        matches!(
            outcome,
            kay_tools::seams::verifier::VerificationOutcome::Fail { .. }
        ),
        "T3-02: expected Fail for failing critic, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// Cost regression gate
// ---------------------------------------------------------------------------

/// Cost regression gate for Phase 8 W-7 (VERIFY-03).
///
/// This test measures the actual verifier cost for a fixed task summary and
/// compares it against the baseline stored in cost-baseline.json.
///
/// - On first run (null baseline): creates the baseline file and passes.
/// - On subsequent runs: fails if actual_cost > baseline * 1.3.
///
/// Run with: cargo test -p kay-verifier -- cost_regression_gate --nocapture
#[tokio::test]
async fn cost_regression_gate() {
    use mockito::Server;

    const FIXTURE_SUMMARY: &str =
        "Fixed 5-sentence task summary for cost regression. \
        The task was to implement a function that parses CSV files. \
        The implementation reads the file line by line. \
        Each line is split on commas. The result is a Vec<Vec<String>>.";

    const FIXTURE_CONTEXT: &str = "called tool: fs_read\ncalled tool: fs_write\ncalled tool: execute_commands\n";

    // Load baseline from file (or skip if null)
    let baseline_path =
        ".planning/phases/08-multi-perspective-verification/cost-baseline.json";
    let baseline: serde_json::Value = std::fs::read_to_string(baseline_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let baseline_cost = baseline
        .get("baseline_cost_usd")
        .and_then(|v| v.as_f64());

    // Run verifier with Benchmark mode + mock server
    let mut srv = Server::new_async().await;
    let body = r#"data: {"choices":[{"delta":{"content":"{\"verdict\":\"pass\",\"reason\":\"ok\"}"}}],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15,"cost":0.002}}
data: [DONE]"#;
    let _m = srv
        .mock("POST", "/api/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&body)
        .create_async()
        .await;

    let (events, verifier) = build_verifier(
        &srv.url(),
        kay_verifier::VerifierMode::Benchmark,
        10.0, // high ceiling so all 3 run
    );

    verifier.verify(FIXTURE_SUMMARY, FIXTURE_CONTEXT).await;

    // Extract total cost from events
    let guard = events.lock().unwrap();
    let actual_cost: f64 = guard
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Verification { cost_usd, .. } => Some(*cost_usd),
            _ => None,
        })
        .sum();

    if let Some(baseline_val) = baseline_cost {
        let tolerance = baseline_val * 1.30;
        assert!(
            actual_cost <= tolerance,
            "cost regression: actual {:.4} exceeds 130% of baseline {:.4}",
            actual_cost,
            baseline_val
        );
        println!(
            "cost_regression_gate: PASS — actual {:.4} <= {:.2}% of baseline {:.4}",
            actual_cost, 130, baseline_val
        );
    } else {
        // No baseline — create it
        let mut val = baseline.clone();
        val["baseline_cost_usd"] = serde_json::json!(actual_cost);
        let _ = std::fs::write(
            baseline_path,
            serde_json::to_string_pretty(&val).unwrap(),
        );
        println!(
            "cost-baseline.json created with baseline {:.4}",
            actual_cost
        );
    }
}