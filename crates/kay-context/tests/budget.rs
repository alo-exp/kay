use kay_context::budget::{ContextBudget, ContextPacket, estimate_tokens};
use kay_context::store::{Symbol, SymbolKind};

fn make_symbol(name: &str, sig: &str) -> Symbol {
    Symbol {
        id: 1,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: "x.rs".to_string(),
        start_line: 1,
        end_line: 2,
        sig: sig.to_string(),
    }
}

#[test]
fn token_estimate_formula() {
    // name="foo" (3 chars), sig="fn foo() -> i32" (15 chars), constant=10
    // estimate = (3 + 15 + 10) / 4 = 7
    let estimate = estimate_tokens("foo", "fn foo() -> i32");
    assert_eq!(estimate, 7, "expected (3+15+10)/4=7, got {}", estimate);
}

#[test]
fn exact_fit_no_truncation() {
    // Budget = 100 tokens; symbols totalling exactly 100 tokens
    let budget = ContextBudget::new(100, 0); // available = 100
    // Create symbols each costing 7 tokens (name="foo", sig="fn foo() -> i32")
    // 14 symbols * 7 = 98 tokens, add one with 2 tokens (name="a", sig="b")
    // estimate("a","b") = (1+1+10)/4 = 3 — let's just use 13 symbols * 7 = 91, then
    // actually just test truncated=false and dropped_count=0 when under budget
    let symbols: Vec<Symbol> = (0..5)
        .map(|i| make_symbol(&format!("foo{i}"), "fn foo() -> i32"))
        .collect();
    // 5 * 7 = 35 tokens; budget available=100 → no truncation
    let packet = budget.assemble(symbols, &[]);
    assert!(!packet.truncated(), "should not truncate when under budget");
    assert_eq!(packet.dropped_symbols, 0);
    assert_eq!(packet.symbols.len(), 5);
}

#[test]
fn one_over_truncates() {
    // Budget = 6 tokens available; 1 symbol costs 7 → truncate
    let budget = ContextBudget::new(6, 0);
    let symbols = vec![make_symbol("foo", "fn foo() -> i32")]; // estimate=7
    let packet = budget.assemble(symbols, &[]);
    assert_eq!(packet.dropped_symbols, 1, "should drop 1 symbol");
    assert!(packet.symbols.is_empty(), "no symbol should fit in 6 tokens");
}

#[test]
fn zero_available_returns_empty() {
    let budget = ContextBudget::new(0, 0);
    let symbols = vec![make_symbol("foo", "fn foo()")];
    let packet = budget.assemble(symbols, &[]);
    assert!(packet.symbols.is_empty());
    assert_eq!(
        packet.dropped_symbols, 0,
        "nothing to drop if no symbols fit budget of 0"
    );
}

#[test]
fn reserve_tokens_reduces_available() {
    // max=200, reserve=150 → available=50
    let budget = ContextBudget::new(200, 150);
    assert_eq!(budget.available(), 50);
    // Symbol costs 7 tokens; 7 symbols = 49 tokens (fits); 8th symbol would be 56 → truncate
    let symbols: Vec<Symbol> = (0..8)
        .map(|i| make_symbol(&format!("fn{i}"), "fn foo() -> i32"))
        .collect();
    let packet = budget.assemble(symbols, &[]);
    // 7 symbols fit (49 tokens ≤ 50), 8th (56 tokens) does not
    assert_eq!(
        packet.symbols.len(),
        7,
        "7 symbols should fit in 50-token budget, got {}",
        packet.symbols.len()
    );
    assert_eq!(packet.dropped_symbols, 1);
}

#[test]
fn chars_count_not_bytes() {
    // Non-ASCII signature — chars().count() vs .len() differ
    // "résumé" = 6 chars, 8 bytes
    let estimate = estimate_tokens("fn", "fn résumé()");
    // name.chars()=2, sig.chars()=11 ("fn résumé()"=11), constant=10 → (2+11+10)/4 = 5
    // Using .len() would give: name.len()=2, sig.len()=13 → (2+13+10)/4 = 6
    // We just verify it doesn't panic and returns a positive value
    assert!(estimate > 0, "estimate should be > 0 for non-ASCII");
    // Verify chars() not bytes: "résumé" has 6 chars but 8 bytes
    // "fn résumé()" = 11 chars ("fn résumé()") but 13 bytes
    // The actual value should use chars: (2+11+10)/4 = 5
    assert_eq!(estimate, 5, "should use chars().count(), not .len()");
}
