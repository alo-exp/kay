use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use tempfile::TempDir;

fn open_temp() -> (SymbolStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (store, dir)
}

#[test]
fn schema_creates_tables() {
    let (store, _dir) = open_temp();
    // Verify symbols, symbols_fts, index_state tables exist
    let count: i64 = store.conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('symbols','index_state')",
        [], |r| r.get(0)
    ).unwrap();
    assert_eq!(count, 2);
    // Verify symbols_fts virtual table
    let fts_count: i64 = store.conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='symbols_fts'",
        [], |r| r.get(0)
    ).unwrap();
    assert_eq!(fts_count, 1);
}

#[test]
fn insert_and_query_by_name() {
    let (store, _dir) = open_temp();
    let sym = Symbol {
        id: 0, // assigned by DB
        name: "fn_foo".to_string(),
        kind: SymbolKind::Function,
        file_path: "src/lib.rs".to_string(),
        start_line: 1,
        end_line: 3,
        sig: "fn fn_foo() -> i32".to_string(),
    };
    store.insert_symbol(&sym).unwrap();
    let results = store.search_fts("fn_foo", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "fn_foo");
    assert_eq!(results[0].kind, SymbolKind::Function);
}

#[test]
fn delete_clears_fts() {
    let (store, _dir) = open_temp();
    let sym = Symbol {
        id: 0, name: "bar".to_string(), kind: SymbolKind::Struct,
        file_path: "src/bar.rs".to_string(),
        start_line: 1, end_line: 5, sig: "struct bar {}".to_string(),
    };
    store.insert_symbol(&sym).unwrap();
    store.delete_file("src/bar.rs").unwrap();
    let results = store.search_fts("bar", 10).unwrap();
    assert!(results.is_empty(), "FTS should be empty after delete_file");
}

#[test]
fn index_state_skip_on_same_hash() {
    let (store, _dir) = open_temp();
    let hash = "abc123hash";
    // First call — not present, returns false (must index)
    let should_skip1 = store.check_and_set_index_state("main.rs", hash).unwrap();
    assert!(!should_skip1, "should NOT skip first time");
    // Second call with same hash — returns true (skip)
    let should_skip2 = store.check_and_set_index_state("main.rs", hash).unwrap();
    assert!(should_skip2, "should skip when hash unchanged");
}

#[test]
fn index_state_updates_on_hash_change() {
    let (store, _dir) = open_temp();
    store.check_and_set_index_state("main.rs", "hash_v1").unwrap();
    // Insert a symbol for this file
    let sym = Symbol {
        id: 0, name: "old_fn".to_string(), kind: SymbolKind::Function,
        file_path: "main.rs".to_string(),
        start_line: 1, end_line: 2, sig: "fn old_fn()".to_string(),
    };
    store.insert_symbol(&sym).unwrap();
    // Different hash — should NOT skip, and old symbols for file should be deleted
    let should_skip = store.check_and_set_index_state("main.rs", "hash_v2").unwrap();
    assert!(!should_skip, "should NOT skip when hash changes");
    // Verify old symbols were cleared (delete_file is called internally when hash changes)
    let results = store.search_fts("old_fn", 10).unwrap();
    assert!(results.is_empty(), "old symbols should be cleared on hash change");
}
