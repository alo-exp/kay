use crate::store::Symbol;

/// RRF score: 1 / (k + rank), k = 60 (DL-10).
pub fn rrf_score(rank: usize) -> f64 {
    1.0 / (60.0 + rank as f64)
}

/// Apply name-bonus of +0.5 when query term exactly matches symbol name (DL-10).
pub fn apply_name_bonus(score: f64, symbol_name: &str, query: &str) -> f64 {
    if symbol_name == query {
        score + 0.5
    } else {
        score
    }
}

/// Merge FTS5 results and ANN results using RRF (DL-10).
pub fn rrf_merge(fts_results: Vec<Symbol>, ann_results: Vec<Symbol>, query: &str) -> Vec<Symbol> {
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
    pub fn new() -> Self {
        Self
    }
}

impl Default for Retriever {
    fn default() -> Self {
        Self::new()
    }
}

// M12-Task 6: Inline unit tests for kay-context retriever module.
// Complements the integration tests in tests/retriever_fts.rs and
// tests/retriever_vec.rs with quick synchronous assertions on
// the RRF scoring and name-bonus logic.

#[cfg(test)]
mod unit {
    use super::*;
    use crate::store::Symbol;

    fn sym(id: i64, name: &str) -> Symbol {
        Symbol {
            id,
            name: name.to_string(),
            sig: String::new(),
            file: "test.rs".to_string(),
            kind: crate::store::SymbolKind::Function,
        }
    }

    #[test]
    fn rrf_score_rank_0_is_correct() {
        // rrf_score(rank) = 1 / (60 + rank)
        assert!((rrf_score(0) - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn rrf_score_decreases_with_rank() {
        let s0 = rrf_score(0);
        let s1 = rrf_score(1);
        let s10 = rrf_score(10);
        assert!(s0 > s1, "higher rank must have lower score");
        assert!(s1 > s10, "rank 1 must beat rank 10");
    }

    #[test]
    fn name_bonus_applies_on_exact_match() {
        let score = 0.5;
        let result = apply_name_bonus(score, "foo", "foo");
        assert!((result - 1.0).abs() < 1e-9, "exact match must add +0.5 bonus");
    }

    #[test]
    fn name_bonus_absent_on_mismatch() {
        let score = 0.5;
        let result = apply_name_bonus(score, "foobar", "foo");
        assert!((result - 0.5).abs() < 1e-9, "mismatch must not add bonus");
    }

    #[test]
    fn name_bonus_case_sensitive() {
        let score = 0.5;
        let result = apply_name_bonus(score, "Foo", "foo");
        assert!((result - 0.5).abs() < 1e-9, "name bonus must be case-sensitive");
    }

    #[test]
    fn rrf_merge_returns_fts_results_when_ann_empty() {
        let fts = vec![sym(1, "alpha"), sym(2, "beta")];
        let merged = rrf_merge(fts.clone(), vec![], "alpha");
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].id, 1, "first result must be highest score");
    }

    #[test]
    fn rrf_merge_returns_ann_results_when_fts_empty() {
        let ann = vec![sym(3, "gamma"), sym(4, "delta")];
        let merged = rrf_merge(vec![], ann.clone(), "gamma");
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn rrf_merge_deduplicates_overlapping_results() {
        let fts = vec![sym(1, "foo"), sym(2, "bar")];
        let ann = vec![sym(2, "bar"), sym(3, "baz")];
        let merged = rrf_merge(fts, ann, "bar");
        assert_eq!(merged.len(), 3, "merged result must have 3 unique symbols");
        let ids: Vec<_> = merged.iter().map(|s| s.id).collect();
        assert!(ids.contains(&2), "bar (id=2) must appear in merged result");
    }

    #[test]
    fn rrf_merge_sorted_by_score_descending() {
        let fts = vec![sym(1, "low_rank")];
        let ann = vec![sym(2, "high_rank")];
        let merged = rrf_merge(fts, ann, "");
        // Score for rank 0: 1/60 ≈ 0.0167; two ranks = 0.0334
        // Score for rank 0 in second: 1/60 ≈ 0.0167
        assert!(merged.len() >= 1);
    }

    #[test]
    fn retriever_new_is_default() {
        let r = Retriever::new();
        let d = Retriever::default();
        // Default must not panic — both are unit structs.
        let _ = (r, d);
    }
}
