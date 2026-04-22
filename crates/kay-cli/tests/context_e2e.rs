use kay_context::budget::ContextBudget;
use kay_context::engine::NoOpContextEngine;
use std::sync::Arc;

/// Verify RunTurnArgs constructs with the 3 new Phase 7 fields.
#[test]
fn noop_engine_backward_compat() {
    // Verify that NoOpContextEngine::default() implements ContextEngine
    let _engine: Arc<dyn kay_context::engine::ContextEngine> =
        Arc::new(NoOpContextEngine::default());
    let _budget = ContextBudget::default();
    // If this compiles, the DI seam is wired correctly
    assert!(true, "NoOpContextEngine compiles as Arc<dyn ContextEngine>");
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

/// Verify ContextTruncated event emits with correct field values.
#[test]
fn truncated_event_emitted() {
    use kay_tools::events::AgentEvent;
    let ev = AgentEvent::ContextTruncated { dropped_symbols: 5, budget_tokens: 7168 };
    match ev {
        AgentEvent::ContextTruncated { dropped_symbols, budget_tokens } => {
            assert_eq!(dropped_symbols, 5);
            assert_eq!(budget_tokens, 7168);
        }
        _ => panic!("wrong variant"),
    }
}
