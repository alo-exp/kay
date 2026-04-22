use crate::store::Symbol;

/// RRF score: 1 / (k + rank), k = 60 (DL-10).
pub fn rrf_score(rank: usize) -> f64 {
    1.0 / (60.0 + rank as f64)
}

/// Apply name-bonus of +0.5 when query term exactly matches symbol name (DL-10).
pub fn apply_name_bonus(score: f64, symbol_name: &str, query: &str) -> f64 {
    if symbol_name == query { score + 0.5 } else { score }
}

/// Merge FTS5 results and ANN results using RRF (DL-10).
pub fn rrf_merge(
    fts_results: Vec<Symbol>,
    ann_results: Vec<Symbol>,
    query: &str,
) -> Vec<Symbol> {
    use std::collections::HashMap;

    // Map symbol id → cumulative RRF score + symbol
    let mut scores: HashMap<i64, (f64, Symbol)> = HashMap::new();

    for (rank, sym) in fts_results.into_iter().enumerate() {
        let s = apply_name_bonus(rrf_score(rank), &sym.name, query);
        scores.entry(sym.id).or_insert((0.0, sym.clone())).0 += s;
    }
    for (rank, sym) in ann_results.into_iter().enumerate() {
        let s = rrf_score(rank);
        scores.entry(sym.id).or_insert((0.0, sym.clone())).0 += s;
    }

    let mut merged: Vec<(f64, Symbol)> = scores.into_values().collect();
    merged.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    merged.into_iter().map(|(_, sym)| sym).collect()
}

pub struct Retriever;

impl Retriever {
    pub fn new() -> Self { Self }
}

impl Default for Retriever {
    fn default() -> Self { Self::new() }
}
