use crate::store::Symbol;

pub struct Retriever;

impl Retriever {
    pub fn rrf_score(rank: usize) -> f64 {
        1.0 / (60.0 + rank as f64)
    }
}

pub fn rrf_merge(_fts: Vec<Symbol>, _ann: Vec<Symbol>, _query: &str) -> Vec<Symbol> {
    todo!("W-3/W-4 implementation")
}
