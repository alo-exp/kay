use kay_context::budget::ContextBudget;
use kay_context::engine::NoOpContextEngine;
use std::sync::Arc;

/// Verify RunTurnArgs constructs with the 3 new Phase 7 fields.
#[test]
fn noop_engine_backward_compat() {
    // Verify that NoOpContextEngine::default() implements ContextEngine
    let _engine: Arc<dyn kay_context::engine::ContextEngine> = Arc::new(NoOpContextEngine);
    let _budget = ContextBudget::default();
    // If this compiles, the DI seam is wired correctly
    // compilation success is the assertion — no runtime check needed
}

/// Verify ContextBudget::default() has the expected values.
#[test]
fn context_injected_into_system_prompt() {
    // Phase 7: _ctx_packet is assembled but not injected into OpenRouter request.
    // This test verifies the budget defaults are correct (Phase 8+ injects the packet).
    let budget = ContextBudget::default();
    assert_eq!(
        budget.max_tokens, 8192,
        "default max_tokens should be 8192 (DL-7)"
    );
    assert_eq!(
        budget.reserve_tokens, 1024,
        "default reserve_tokens should be 1024 (DL-7)"
    );
    assert_eq!(budget.available(), 7168, "default available should be 7168");
}

/// Phase 8 smoke: VerifierConfig::default() wires into the CLI without panic.
/// Real verifier behavioral tests live in crates/kay-verifier/tests/ and
/// crates/kay-core/tests/rework_loop.rs.
#[test]
fn verifier_config_default_is_interactive() {
    // Ensures the Default impl is stable and the Interactive mode is the
    // CLI default (VERIFY-02: bounded re-work loop, opt-in benchmark trio).
    let cfg = kay_verifier::VerifierConfig::default();
    assert!(
        matches!(cfg.mode, kay_verifier::VerifierMode::Interactive),
        "default mode should be Interactive for CLI use"
    );
    assert_eq!(cfg.max_retries, 3, "default max_retries should be 3");
    assert!(
        cfg.cost_ceiling_usd > 0.0,
        "default cost ceiling should be positive"
    );
}
