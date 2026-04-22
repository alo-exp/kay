---
phase: 07-context-engine
reviewed: 2026-04-22T00:00:00Z
depth: standard
files_reviewed: 31
files_reviewed_list:
  - Cargo.toml
  - crates/kay-cli/Cargo.toml
  - crates/kay-cli/src/run.rs
  - crates/kay-cli/tests/context_e2e.rs
  - crates/kay-context/Cargo.toml
  - crates/kay-context/src/budget.rs
  - crates/kay-context/src/embedder.rs
  - crates/kay-context/src/engine.rs
  - crates/kay-context/src/error.rs
  - crates/kay-context/src/hardener.rs
  - crates/kay-context/src/indexer.rs
  - crates/kay-context/src/language.rs
  - crates/kay-context/src/lib.rs
  - crates/kay-context/src/retriever.rs
  - crates/kay-context/src/store.rs
  - crates/kay-context/src/watcher.rs
  - crates/kay-context/tests/budget.rs
  - crates/kay-context/tests/hardener.rs
  - crates/kay-context/tests/indexer.rs
  - crates/kay-context/tests/retriever_fts.rs
  - crates/kay-context/tests/retriever_vec.rs
  - crates/kay-context/tests/store.rs
  - crates/kay-context/tests/watcher.rs
  - crates/kay-core/Cargo.toml
  - crates/kay-core/src/loop.rs
  - crates/kay-core/tests/loop.rs
  - crates/kay-session/Cargo.toml
  - crates/kay-tools/src/events.rs
  - crates/kay-tools/src/events_wire.rs
  - crates/kay-tools/src/registry.rs
  - crates/kay-tools/tests/events_wire_snapshots.rs
findings:
  critical: 0
  warning: 5
  info: 5
  total: 10
status: issues_found
---

# Phase 7: Code Review Report

**Reviewed:** 2026-04-22
**Depth:** standard
**Files Reviewed:** 31
**Status:** issues_found

## Summary

Phase 7 introduces `kay-context`: a 10-module Rust crate implementing tree-sitter
symbol indexing, SQLite FTS5 + sqlite-vec hybrid retrieval, per-turn context budget,
ForgeCode schema hardening delegation, and file watching. The modified files
(`kay-tools`, `kay-core`, `kay-cli`, `kay-session`) are surgical — the Phase 5
agent loop is extended with three new `RunTurnArgs` fields and `NoOpContextEngine`
is wired at the CLI level.

The overall quality is high: the deny-lint discipline is well-maintained, the
`#[non_exhaustive]` + additive-variant pattern is correctly followed, and the
DI seams are clean. No security vulnerabilities were found. Five warnings concern
correctness or reliability issues that could cause bugs in production or under
edge conditions; five info items cover code quality and maintainability.

---

## Warnings

### WR-01: `upsert_symbol` is a pure INSERT — silently duplicates symbols on re-index

**File:** `crates/kay-context/src/store.rs:148`

**Issue:** `upsert_symbol` runs a plain `INSERT` (not `INSERT OR REPLACE`). The calling path in `indexer.rs` deletes old symbols via `check_and_set_index_state` → `delete_file` before inserting fresh ones. This is correct when `check_and_set_index_state` observes a hash change, but if a caller bypasses that state-check (e.g., a future incremental re-index that calls `upsert_symbol` directly), symbols will be inserted twice. The method name `upsert_symbol` implies idempotency that the implementation does not provide. An upsert would require a unique constraint on `(name, kind, file_path, start_line)` and `INSERT OR REPLACE` semantics.

**Fix:** Either rename the method to `insert_symbol` to signal non-idempotency, or add a unique index and switch to `INSERT OR REPLACE`. For a pure-insert intent the rename is safer than an invisible collision:
```rust
pub fn insert_symbol(&self, sym: &Symbol) -> Result<(), ContextError> {
```
And update all callers (`indexer.rs` line 71).

---

### WR-02: `KayContextEngine` struct is defined but has no `ContextEngine` impl

**File:** `crates/kay-context/src/engine.rs:35`

**Issue:** `KayContextEngine` is declared with a single public `budget` field but there is no `impl ContextEngine for KayContextEngine`. It is re-exported from `lib.rs` (line 18 via `pub use engine::{ContextEngine, KayContextEngine, NoOpContextEngine}`). Any downstream crate that imports `KayContextEngine` expecting it to implement `ContextEngine` will fail at compile time with a missing-impl error. The struct is effectively dead code in its current state but its presence in the public `use` list promises an API that does not exist.

**Fix:** Either add the `impl ContextEngine for KayContextEngine` body (even a stub that delegates to `NoOpContextEngine`), or remove `KayContextEngine` from the public re-export until Phase 8 wires the real implementation:
```rust
// lib.rs: remove KayContextEngine from pub use until Phase 8
pub use engine::{ContextEngine, NoOpContextEngine};
```

---

### WR-03: `unwrap_or_default()` on embedding deserialization silently masks corruption

**File:** `crates/kay-context/src/store.rs:305`

**Issue:** In `ann_search`, the embedding JSON is deserialized with `.unwrap_or_default()` — a parse failure returns an empty `Vec<f32>`, which is then treated as a zero-vector of length 0. L2 distance against a zero-vector of length 0 equals the full magnitude of the query vector, making corrupted rows appear to have maximum distance and drop to the tail of results rather than being surfaced as errors. The error is silently eaten inside a closure inside a `filter_map`, making it invisible in logs.

**Fix:** Map the parse failure to a `ContextError::Json` and propagate it, or at minimum log it as a warning:
```rust
let embedding: Vec<f32> = match serde_json::from_str(&embedding_json) {
    Ok(v) => v,
    Err(e) => {
        tracing::warn!(symbol_id = sid, error = %e, "embedding parse failed; skipping");
        return None; // filter_map drops this row
    }
};
```

---

### WR-04: Watcher callback holds `Mutex<impl Fn()>` — potential deadlock if callback panics

**File:** `crates/kay-context/src/watcher.rs:32`

**Issue:** `FileWatcher::new` wraps the user-provided callback in `Arc<Mutex<_>>`. The `Mutex` is locked on every debounced event. If the callback panics, the `Mutex` is poisoned. Subsequent calls to `.lock()` will return `Err(PoisonError)` and the `let Ok(cb) = callback.lock()` guard (line 48) silently drops all future events without logging — the user never knows the watcher is dead. This is a reliability hazard: a one-time panic in the invalidation callback permanently silences the file watcher.

The `Mutex` is also unnecessary here: `Arc<Mutex<impl Fn()>>` wrapping a `move` closure that is called but never replaced could be replaced with `Arc<dyn Fn() + Send + Sync>` which requires no lock.

**Fix:** Replace the `Mutex` with a direct `Arc<dyn Fn() + Send + Sync>`:
```rust
let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(on_invalidate);
// In closure:
if triggered { callback(); }
```

---

### WR-05: `unwrap_or_default()` on `SystemTime::duration_since` could silently record epoch 0 for `indexed_at`

**File:** `crates/kay-context/src/store.rs:201` and `store.rs:212`

**Issue:** `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()` is used twice to compute the `indexed_at` timestamp. `unwrap_or_default()` maps `SystemTimeError` (which occurs when the system clock is set before the Unix epoch — a real scenario in containers, CI environments with broken clocks, or NTP slew) to `Duration::ZERO`, recording epoch 0 (`1970-01-01`) as the index timestamp. The `#[deny(clippy::unwrap_used)]` on `lib.rs` prevents direct `.unwrap()`, but `.unwrap_or_default()` bypasses that lint. The bug is subtle: the file will appear to have been indexed at epoch 0, which will not cause a correctness failure today but would break any future logic that uses `indexed_at` for cache invalidation (e.g., "re-index files older than N minutes").

**Fix:** Use a monotone fallback rather than silently zeroing:
```rust
let now_secs: i64 = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0); // accepted: epoch-0 is a better sentinel than panic
```
Or log the anomaly:
```rust
.unwrap_or_else(|e| {
    tracing::warn!("system clock before UNIX epoch: {e}; using 0");
    std::time::Duration::ZERO
}).as_secs()
```
This is technically the same value, but the log makes the anomaly visible. The more important fix is to add a comment acknowledging the CI-clock edge case so future maintainers do not misread the `unwrap_or_default` as a careless oversight.

---

## Info

### IN-01: `FakeEmbedder` is always compiled (no `#[cfg(test)]` or feature gate) — inflates production binary

**File:** `crates/kay-context/src/embedder.rs:20`

**Issue:** The comment on `FakeEmbedder` acknowledges it is "always compiled so integration tests can import it without feature flags." This is an intentional design decision per the review brief. However, `FakeEmbedder` adds two public items to the production API surface (`FakeEmbedder` struct and `embed_sync` method) that serve no production purpose. They will appear in rustdoc and autocomplete for any downstream crate that depends on `kay-context`. The `testing` feature flag in `Cargo.toml` exists but is not used to gate `FakeEmbedder`.

**Fix (low priority):** Either gate `FakeEmbedder` behind `#[cfg(any(test, feature = "testing"))]` (requires tests to depend on `kay-context` with `features = ["testing"]`), or add a doc comment marking it as a test double explicitly:
```rust
/// Test double — deterministic zero-vector embedder. Not for production use.
/// Compiled in all configurations so integration tests in `tests/` can
/// import without feature flags (see Phase 7 design note in embedder.rs).
pub struct FakeEmbedder {
```

---

### IN-02: `SymbolStore::conn` and `SymbolStore::db_path` are `pub` fields — breaks encapsulation

**File:** `crates/kay-context/src/store.rs:5`

**Issue:** `conn` and `db_path` are both `pub`. Tests access `store.conn` directly (e.g., `store.conn.query_row(...)` in `tests/store.rs`, `tests/retriever_vec.rs`). Exposing `rusqlite::Connection` as a public field leaks the storage layer abstraction to all callers. If `SymbolStore` ever switches to a connection pool (`r2d2`) or adds WAL-checkpoint logic, all call sites must be updated.

**Fix:** Add accessor methods and gate the raw field behind `pub(crate)`:
```rust
pub struct SymbolStore {
    pub(crate) conn: Connection,
    pub db_path: PathBuf,
}
```
Tests in `tests/` are outside the crate boundary and would need to use the `pub` accessor methods, which is the correct direction. For now at minimum `db_path` should remain `pub` and `conn` should be `pub(crate)`.

---

### IN-03: `context_e2e.rs` test is a thin compilation smoke test — no behavioral coverage

**File:** `crates/kay-cli/tests/context_e2e.rs`

**Issue:** The three tests in `context_e2e.rs` only verify that `NoOpContextEngine::default()` compiles as an `Arc<dyn ContextEngine>`, that `ContextBudget::default()` has the expected defaults, and that `ContextTruncated` can be constructed. No end-to-end behavior is exercised: the `run.rs` path that calls `context_engine.retrieve()` during a real `run_turn` invocation is not covered. This is acceptable for Phase 7 (the real engine is Phase 8+), but the file name `context_e2e.rs` overstates what the tests actually verify.

**Fix (doc only):** Rename the file to `context_smoke.rs` or add a comment at the top:
```rust
//! Phase 7 smoke tests — compilation + default-value checks only.
//! Behavioral E2E coverage lands in Phase 8 when KayContextEngine
//! is wired into run_turn.
```

---

### IN-04: `SymbolKind::from_kind_str` maps all unknown strings to `FileBoundary` — silent misclassification

**File:** `crates/kay-context/src/store.rs:45`

**Issue:** The `_` arm of `from_kind_str` returns `Self::FileBoundary`. Any typo or future `kind` value stored in the database that is not one of the eight known strings will silently become `FileBoundary`. Since this method is called when reading rows back from SQLite (e.g., in `search_fts` and `ann_search`), a database that stores a `kind` string from a future schema version will silently produce wrong `SymbolKind` values rather than a deserialization error.

**Fix:** Add a `tracing::warn!` for the unknown-kind fallback:
```rust
_ => {
    tracing::warn!(kind = s, "unknown symbol kind in database; treating as FileBoundary");
    Self::FileBoundary
}
```

---

### IN-05: `Retriever` struct has no methods except constructors — unused public type

**File:** `crates/kay-context/src/retriever.rs:38`

**Issue:** `Retriever` is a public struct with `new()` and `Default` but carries no state and implements no methods beyond construction. The actual retrieval logic (`rrf_merge`, `rrf_score`, `apply_name_bonus`) lives as free functions. `Retriever` is re-exported from `lib.rs` but serves no purpose that the free functions do not already cover. This is likely a placeholder for Phase 8 method migration, but as shipped it adds dead-weight to the public API.

**Fix:** Either add a `#[doc(hidden)]` note marking it as a Phase 8 placeholder, or remove the struct and re-export only the free functions. If the struct is intentionally reserved for Phase 8, a doc comment clarifying this prevents confusion:
```rust
/// Phase 8 placeholder — stateful retrieval methods will be added
/// once KayContextEngine wires FTS + ANN results through this type.
pub struct Retriever;
```

---

_Reviewed: 2026-04-22_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
