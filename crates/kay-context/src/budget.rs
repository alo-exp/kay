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
