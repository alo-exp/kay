use kay_context::indexer::TreeSitterIndexer;
use kay_context::language::Language;
use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use std::path::Path;
use tempfile::TempDir;

fn make_store() -> (SymbolStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (store, dir)
}

fn write_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[tokio::test]
async fn rust_fn_extracted() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "a.rs", "fn foo(x: i32) -> i32 { x + 1 }\n");
    let indexer = TreeSitterIndexer::new();
    let stats = indexer.index_file(&src, &store).await.unwrap();
    assert!(stats.symbols >= 1);
    let results = store.search_fts("foo", 10).unwrap();
    assert!(
        results
            .iter()
            .any(|s| s.name == "foo" && s.kind == SymbolKind::Function),
        "expected fn foo, got: {:?}",
        results
    );
}

#[tokio::test]
async fn rust_trait_extracted() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "b.rs", "trait Bar {\n    fn baz();\n}\n");
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("Bar", 10).unwrap();
    assert!(
        results
            .iter()
            .any(|s| s.name == "Bar" && s.kind == SymbolKind::Trait),
        "expected trait Bar, got: {:?}",
        results
    );
}

#[tokio::test]
async fn rust_mod_boundary() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "c.rs", "mod utils {}\n");
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("utils", 10).unwrap();
    assert!(
        results
            .iter()
            .any(|s| s.name == "utils" && s.kind == SymbolKind::Module),
        "expected mod utils, got: {:?}",
        results
    );
}

#[tokio::test]
async fn typescript_function_extracted() {
    let (store, dir) = make_store();
    let src = write_file(
        dir.path(),
        "x.ts",
        "function greet(name: string): string { return name; }\n",
    );
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("greet", 10).unwrap();
    assert!(!results.is_empty(), "expected greet function, got nothing");
}

#[tokio::test]
async fn typescript_class_extracted() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "y.ts", "class Foo {}\n");
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("Foo", 10).unwrap();
    assert!(!results.is_empty(), "expected class Foo, got nothing");
}

#[tokio::test]
async fn python_def_extracted() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "p.py", "def compute(x):\n    return x * 2\n");
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("compute", 10).unwrap();
    assert!(!results.is_empty(), "expected def compute, got nothing");
}

#[tokio::test]
async fn python_class_extracted() {
    let (store, dir) = make_store();
    let src = write_file(dir.path(), "q.py", "class Solver:\n    pass\n");
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("Solver", 10).unwrap();
    assert!(!results.is_empty(), "expected class Solver, got nothing");
}

#[tokio::test]
async fn go_func_extracted() {
    let (store, dir) = make_store();
    let src = write_file(
        dir.path(),
        "g.go",
        "package main\nfunc Run(ctx interface{}) error { return nil }\n",
    );
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("Run", 10).unwrap();
    assert!(!results.is_empty(), "expected func Run, got nothing");
}

#[tokio::test]
async fn sig_truncated_at_256() {
    let (store, dir) = make_store();
    // Generate a function with a very long signature (>256 chars)
    let long_params = "a: i32, ".repeat(40); // 320 chars of params
    let src_code = format!("fn long_fn({}) -> i32 {{ 0 }}\n", long_params);
    let src = write_file(dir.path(), "long.rs", &src_code);
    let indexer = TreeSitterIndexer::new();
    indexer.index_file(&src, &store).await.unwrap();
    let results = store.search_fts("long_fn", 10).unwrap();
    assert!(!results.is_empty(), "expected long_fn");
    let sig = &results[0].sig;
    assert!(
        sig.chars().count() <= 257,
        "sig too long: {} chars",
        sig.chars().count()
    );
    if sig.chars().count() == 257 {
        assert!(sig.ends_with('…'), "truncated sig should end with ellipsis");
    }
}

#[tokio::test]
async fn unknown_extension_file_boundary() {
    let (store, dir) = make_store();
    let src = write_file(
        dir.path(),
        "config.toml",
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n[dependencies]\n",
    );
    let indexer = TreeSitterIndexer::new();
    let stats = indexer.index_file(&src, &store).await.unwrap();
    assert_eq!(
        stats.symbols, 1,
        "should produce exactly 1 FileBoundary symbol"
    );
    let results = store.search_fts("package", 10).unwrap();
    assert!(
        !results.is_empty(),
        "FileBoundary symbol should contain first 10 lines"
    );
    assert_eq!(results[0].kind, SymbolKind::FileBoundary);
}

proptest::proptest! {
    #[test]
    fn proptest_sig_never_exceeds_256(input in "[a-z_]{1,50}") {
        // Verify truncate_sig invariant directly
        let long_sig: String = input.repeat(10);
        let truncated = kay_context::indexer::truncate_sig(&long_sig);
        assert!(truncated.chars().count() <= 257,
            "sig must be ≤257 chars, got {} chars", truncated.chars().count());
    }
}
