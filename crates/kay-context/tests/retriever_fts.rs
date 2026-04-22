use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use kay_context::retriever::{rrf_merge, rrf_score, apply_name_bonus};
use tempfile::TempDir;

fn make_store() -> (SymbolStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (store, dir)
}

fn insert_sym(store: &SymbolStore, name: &str, sig: &str, file: &str) {
    store.upsert_symbol(&Symbol {
        id: 0,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: file.to_string(),
        start_line: 1, end_line: 2,
        sig: sig.to_string(),
    }).unwrap();
}

#[test]
fn fts_exact_match_returns_symbol() {
    let (store, _dir) = make_store();
    insert_sym(&store, "run_loop", "fn run_loop()", "a.rs");
    let results = store.search_fts("run_loop", 10).unwrap();
    assert!(!results.is_empty(), "expected run_loop in results");
    assert!(results.iter().any(|s| s.name == "run_loop"));
}

#[test]
fn fts_no_match_returns_empty() {
    let (store, _dir) = make_store();
    insert_sym(&store, "actual_fn", "fn actual_fn()", "a.rs");
    let results = store.search_fts("zzznomatch", 10).unwrap();
    assert!(results.is_empty(), "expected empty for non-matching query");
}

#[test]
fn fts_prefix_match() {
    let (store, _dir) = make_store();
    insert_sym(&store, "run_loop", "fn run_loop()", "a.rs");
    // FTS5 prefix search
    let results = store.search_fts("run_lo*", 10).unwrap();
    assert!(!results.is_empty(), "expected prefix match");
    assert!(results.iter().any(|s| s.name == "run_loop"));
}

#[test]
fn fts_name_bonus_applied() {
    // rrf_score + apply_name_bonus arithmetic
    let base = rrf_score(0);  // 1/60
    let with_bonus = apply_name_bonus(base, "query_term", "query_term");
    let without_bonus = apply_name_bonus(base, "other_name", "query_term");
    assert!(with_bonus > without_bonus,
        "exact name match should have higher score: {} vs {}", with_bonus, without_bonus);
    assert!((with_bonus - base - 0.5).abs() < f64::EPSILON,
        "name bonus should be exactly +0.5");
}

#[test]
fn fts_ranking_order() {
    let (store, _dir) = make_store();
    // Symbol with "foo" in sig gets ranked higher than one without
    insert_sym(&store, "foo_heavy", "fn foo_heavy() foo foo foo", "a.rs");
    insert_sym(&store, "foo_light", "fn foo_light()", "b.rs");
    let results = store.search_fts("foo", 10).unwrap();
    assert!(!results.is_empty());
    // heavy has more occurrences, should rank first
    assert_eq!(results[0].name, "foo_heavy",
        "symbol with more query occurrences should rank first");
}

#[test]
fn fts_multi_word_query() {
    let (store, _dir) = make_store();
    insert_sym(&store, "run_loop_fn", "fn run_loop_fn() // run and loop", "a.rs");
    insert_sym(&store, "only_run", "fn only_run()", "b.rs");
    // FTS5 multi-word: both tokens must match
    let results = store.search_fts("run loop", 10).unwrap();
    // run_loop_fn has both "run" and "loop", only_run has only "run"
    assert!(results.iter().any(|s| s.name == "run_loop_fn"),
        "multi-token query should match symbol containing both tokens");
}
