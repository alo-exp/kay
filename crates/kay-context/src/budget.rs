use crate::store::Symbol;

#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub reserve_tokens: usize,
}

impl ContextBudget {
    pub fn new(max_tokens: usize, reserve_tokens: usize) -> Self {
        Self { max_tokens, reserve_tokens }
    }

    pub fn available(&self) -> usize {
        self.max_tokens.saturating_sub(self.reserve_tokens)
    }

    /// Assemble a ContextPacket from symbols, respecting the token budget.
    /// Symbols are taken in order until the budget is exhausted (tail-drop).
    /// When available == 0, returns an empty packet with dropped_symbols = 0
    /// (no budget was ever open, so nothing was "dropped").
    /// `schemas` are passed through as-is (hardening happens in engine.rs).
    pub fn assemble(&self, symbols: Vec<Symbol>, schemas: &[serde_json::Value]) -> ContextPacket {
        let available = self.available();

        // Zero budget: no capacity was ever open, nothing is counted as dropped.
        if available == 0 {
            return ContextPacket {
                symbols: Vec::new(),
                dropped_symbols: 0,
                budget_tokens: 0,
                hardened_schemas: schemas.to_vec(),
            };
        }

        let mut included = Vec::new();
        let mut used_tokens = 0usize;
        let total = symbols.len();

        for sym in symbols {
            let cost = estimate_tokens(&sym.name, &sym.sig);
            if used_tokens + cost <= available {
                used_tokens += cost;
                included.push(sym);
            }
        }

        let dropped = total - included.len();
        ContextPacket {
            symbols: included,
            dropped_symbols: dropped,
            budget_tokens: available,
            hardened_schemas: schemas.to_vec(),
        }
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::new(8192, 1024)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContextPacket {
    pub symbols: Vec<Symbol>,
    pub dropped_symbols: usize,
    pub budget_tokens: usize,
    pub hardened_schemas: Vec<serde_json::Value>,
}

impl ContextPacket {
    pub fn truncated(&self) -> bool {
        self.dropped_symbols > 0
    }
}

pub fn estimate_tokens(name: &str, sig: &str) -> usize {
    (name.chars().count() + sig.chars().count() + 10) / 4
}

// M12-Task 6: Inline unit tests for kay-context budget module.
// Complements the integration tests in tests/budget.rs with quick
// synchronous assertions on ContextBudget and estimate_tokens.

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn context_budget_default_values() {
        let b = ContextBudget::default();
        assert_eq!(b.max_tokens, 8192, "default max_tokens must be 8192");
        assert_eq!(
            b.reserve_tokens, 1024,
            "default reserve_tokens must be 1024"
        );
    }

    #[test]
    fn context_budget_available_is_max_minus_reserve() {
        let b = ContextBudget::new(8192, 1024);
        assert_eq!(b.available(), 7168);
    }

    #[test]
    fn context_budget_available_saturates_on_zero_reserve() {
        let b = ContextBudget::new(8192, 8192);
        assert_eq!(b.available(), 0);
    }

    #[test]
    fn context_budget_available_never_negative() {
        let b = ContextBudget::new(100, 200);
        assert_eq!(b.available(), 0, "saturating_sub must prevent negative");
    }

    #[test]
    fn context_packet_truncated_when_dropped_symbols() {
        let packet = ContextPacket {
            symbols: vec![],
            dropped_symbols: 5,
            budget_tokens: 8192,
            hardened_schemas: vec![],
        };
        assert!(packet.truncated());
    }

    #[test]
    fn context_packet_not_truncated_when_no_dropped() {
        let packet = ContextPacket {
            symbols: vec![],
            dropped_symbols: 0,
            budget_tokens: 8192,
            hardened_schemas: vec![],
        };
        assert!(!packet.truncated());
    }

    #[test]
    fn estimate_tokens_formula() {
        // (name_len + sig_len + 10) / 4
        assert_eq!(estimate_tokens("", ""), 2); // (0 + 0 + 10) / 4 = 2
        assert_eq!(estimate_tokens("foo", ""), 3); // (3 + 0 + 10) / 4 = 3
        assert_eq!(estimate_tokens("", "()"), 3); // (0 + 2 + 10) / 4 = 3
        assert_eq!(estimate_tokens("fn", "i32"), 3); // (2 + 3 + 10) / 4 = 15 / 4 = 3 (integer div)
    }

    #[test]
    fn context_budget_assemble_with_zero_budget() {
        let b = ContextBudget::new(100, 200);
        let syms = vec![];
        let packet = b.assemble(syms.clone(), &[]);
        assert!(packet.symbols.is_empty());
        assert_eq!(packet.dropped_symbols, 0, "zero budget: nothing is dropped");
        assert_eq!(packet.budget_tokens, 0);
    }

    #[test]
    fn context_budget_assemble_preserves_schema() {
        use crate::store::Symbol;
        let b = ContextBudget::new(8192, 0);
        let schema = serde_json::json!({"type": "object"});
        let sym = Symbol {
            id: 1,
            name: "foo".to_string(),
            sig: "()".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 10,
            kind: crate::store::SymbolKind::Function,
        };
        let packet = b.assemble(vec![sym], &[schema.clone()]);
        assert_eq!(packet.hardened_schemas, vec![schema]);
    }
}
