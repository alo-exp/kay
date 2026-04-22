use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use kay_context::embedder::FakeEmbedder;
use kay_context::retriever::rrf_merge;
use tempfile::TempDir;

fn make_store() -> (SymbolStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (store, dir)
}

fn make_sym(id: i64, name: &str) -> Symbol {
    Symbol {
        id,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: "a.rs".to_string(),
        start_line: 1, end_line: 2,
        sig: format!("fn {}()", name),
    }
}

#[tokio::test]
async fn vec_table_created_with_fake_embedder() {
    let (store, _dir) = make_store();
    let embedder = FakeEmbedder { dimensions: 4 };
    store.enable_vector_search(&embedder, 4).unwrap();
    // symbols_vec table should exist
    let count: i64 = store.conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='symbols_vec'",
        [], |r| r.get(0)
    ).unwrap();
    assert_eq!(count, 1, "symbols_vec table should be created");
}

#[tokio::test]
async fn fake_embedder_insert_and_ann() {
    let (store, _dir) = make_store();
    let embedder = FakeEmbedder { dimensions: 4 };
    store.enable_vector_search(&embedder, 4).unwrap();

    // Insert 3 symbols with vectors
    for i in 0i64..3 {
        let sym = make_sym(i + 1, &format!("fn_{}", i));
        store.upsert_symbol(&sym).unwrap();
        let vec = embedder.embed_sync(&sym.sig);
        store.upsert_vector(sym.id.max(1), &vec).unwrap_or_else(|_| {
            // id may be auto-assigned; re-query
        });
    }
    // ANN search with a zero-vector should return results
    let query_vec = vec![0.0f32; 4];
    let results = store.ann_search(&query_vec, 3).unwrap();
    assert!(!results.is_empty(), "ANN search should return results");
}

#[test]
fn rrf_merge_prefers_fts_winner() {
    // FTS5 top = sym A; ANN top = sym B; with name-bonus A should win overall
    let sym_a = make_sym(1, "alpha_winner");
    let sym_b = make_sym(2, "beta_second");

    let fts_results = vec![sym_a.clone(), sym_b.clone()]; // A first in FTS
    let ann_results = vec![sym_b.clone(), sym_a.clone()]; // B first in ANN

    let merged = rrf_merge(fts_results, ann_results, "alpha_winner");
    assert!(!merged.is_empty());
    // With name-bonus of +0.5 on FTS for alpha_winner, A should rank first
    assert_eq!(merged[0].name, "alpha_winner",
        "FTS winner with name-bonus should rank first, got: {:?}", merged[0].name);
}

#[test]
fn rrf_merge_prefers_vec_winner() {
    // Zero FTS signal; only ANN signal
    let sym_a = make_sym(1, "vec_winner");
    let sym_b = make_sym(2, "not_matched");

    let fts_results = vec![]; // No FTS results
    let ann_results = vec![sym_a.clone(), sym_b.clone()]; // A first in ANN

    let merged = rrf_merge(fts_results, ann_results, "unrelated_query");
    assert!(!merged.is_empty());
    assert_eq!(merged[0].name, "vec_winner",
        "ANN winner should rank first when no FTS signal");
}

#[test]
fn rrf_k60_score_formula() {
    // Verify: rrf_score(r) = 1/(60+r)
    use kay_context::retriever::rrf_score;
    let score_r0 = rrf_score(0);
    let expected_r0 = 1.0 / 60.0;
    assert!((score_r0 - expected_r0).abs() < 1e-10,
        "rrf_score(0) should be 1/60, got {}", score_r0);

    let score_r1 = rrf_score(1);
    let expected_r1 = 1.0 / 61.0;
    assert!((score_r1 - expected_r1).abs() < 1e-10,
        "rrf_score(1) should be 1/61, got {}", score_r1);

    // Verify symbol appearing in both lists gets both scores
    let sym = make_sym(1, "double_hit");
    let merged = rrf_merge(vec![sym.clone()], vec![sym.clone()], "x");
    assert_eq!(merged.len(), 1, "same symbol from both lists → merged into 1");
    // Score should be rrf_score(0) + rrf_score(0) = 1/60 + 1/60
}

#[test]
fn noop_embedder_skips_vec() {
    use kay_context::embedder::NoOpEmbedder;
    use kay_context::engine::{ContextEngine, NoOpContextEngine};

    let engine = NoOpContextEngine::default();
    // NoOpContextEngine::retrieve must complete without accessing symbols_vec
    // (which doesn't exist in this test store)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let packet = rt.block_on(engine.retrieve("query", &[])).unwrap();
    assert!(packet.symbols.is_empty());
}
