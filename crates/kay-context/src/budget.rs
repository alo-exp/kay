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

pub fn estimate_tokens(name: &str, sig: &str) -> usize {
    (name.chars().count() + sig.chars().count() + 10) / 4
}
