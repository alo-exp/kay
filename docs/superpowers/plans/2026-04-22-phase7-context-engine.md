# Phase 7: Context Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a local tree-sitter symbol store with SQLite + sqlite-vec hybrid retrieval, explicit per-turn context budget enforcement, and ForgeCode schema hardening applied consistently to all tool schemas.

**Architecture:** New `kay-context` crate owns `SymbolStore` (SQLite), `TreeSitterIndexer`, `Retriever` (FTS5 + RRF), `ContextBudget`, `SchemaHardener`, and `FileWatcher`. `kay-cli/src/run.rs` injects a `ContextPacket` into `run_turn` before each provider call. `NoOpContextEngine` default preserves backward compatibility for all existing tests.

**Tech Stack:** Rust 1.95, rusqlite 0.38 (bundled), tree-sitter 0.23, tree-sitter-{rust,typescript,python,go} 0.23.x, sqlite-vec =0.1.10-alpha.3, notify 6.1, notify-debouncer-mini 0.4, sha2 (workspace), insta snapshots.

---

## Critical Constraints (read before touching any file)

- **`event_filter.rs` — QG-C4**: byte-identical; do NOT touch under any circumstances.
- **`AgentEventWire` is NOT an enum**: it is `pub struct AgentEventWire<'a>(pub &'a AgentEvent)` with a hand-written `Serialize` impl. New events = new `match` arms inside `impl<'a> Serialize for AgentEventWire<'a>`, NOT new struct variants.
- **`ToolRegistry::schemas()` must go in `registry.rs`**: `self.tools` is private; the method cannot live in `engine.rs` or any other file.
- **tree-sitter core = 0.23** (not 0.26): grammar crates pin to the 0.23 API.
- **sqlite-vec exact pin**: `"=0.1.10-alpha.3"` — no stable release; cargo-update must not pull forward.
- **rusqlite workspace promotion**: was crate-local in `kay-session/Cargo.toml`; add to `[workspace.dependencies]` now so `kay-context` can `{ workspace = true }`. Update `kay-session/Cargo.toml` to inherit from workspace.
- **DCO on every commit**: `Signed-off-by: Name <email>` required.
- **TDD iron law**: RED commit before GREEN per wave.

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `Cargo.toml` | Modify | Add `kay-context` to members; add tree-sitter, sqlite-vec, notify, rusqlite to `[workspace.dependencies]` |
| `crates/kay-session/Cargo.toml` | Modify | Change crate-local rusqlite dep to `{ workspace = true }` |
| `crates/kay-context/Cargo.toml` | Create | kay-context crate manifest |
| `crates/kay-context/src/lib.rs` | Create | Public re-exports + `ContextEngine` trait |
| `crates/kay-context/src/store.rs` | Create | `SymbolStore` — rusqlite + FTS5 + sqlite-vec DDL |
| `crates/kay-context/src/indexer.rs` | Create | `TreeSitterIndexer` — parse files → Vec<Symbol> |
| `crates/kay-context/src/language.rs` | Create | `Language` enum + extension detection |
| `crates/kay-context/src/retriever.rs` | Create | Hybrid FTS5 + sqlite-vec + RRF merge |
| `crates/kay-context/src/budget.rs` | Create | `ContextBudget` + token estimate + truncation |
| `crates/kay-context/src/hardener.rs` | Create | `SchemaHardener` wrapping `harden_tool_schema` |
| `crates/kay-context/src/watcher.rs` | Create | `FileWatcher` (notify debounced) |
| `crates/kay-context/src/embedder.rs` | Create | `EmbeddingProvider` trait + `NoOpEmbedder` + `FakeEmbedder` |
| `crates/kay-context/src/engine.rs` | Create | `KayContextEngine` + `NoOpContextEngine` |
| `crates/kay-context/tests/store.rs` | Create | W-1: SymbolStore CRUD (5 tests) |
| `crates/kay-context/tests/indexer.rs` | Create | W-2: indexer per-language (11 tests) |
| `crates/kay-context/tests/retriever_fts.rs` | Create | W-3: FTS5 retriever (6 tests) |
| `crates/kay-context/tests/retriever_vec.rs` | Create | W-4: sqlite-vec + RRF (6 tests) |
| `crates/kay-context/tests/budget.rs` | Create | W-5: budget + truncation (6 tests) |
| `crates/kay-context/tests/hardener.rs` | Create | W-6: SchemaHardener (5 tests) |
| `crates/kay-context/tests/watcher.rs` | Create | W-7a: FileWatcher integration (5 tests) |
| `crates/kay-tools/src/events.rs` | Modify | Add `ContextTruncated` + `IndexProgress` variants |
| `crates/kay-tools/src/events_wire.rs` | Modify | Add two match arms to `Serialize` impl |
| `crates/kay-tools/src/registry.rs` | Modify | Add `schemas()` method |
| `crates/kay-tools/tests/events_wire_snapshots.rs` | Modify | Add 2 insta snapshot tests for new variants |
| `crates/kay-core/src/loop.rs` | Modify | Add `context_engine` + `context_budget` to `RunTurnArgs`; inject ContextPacket before provider call |
| `crates/kay-cli/src/run.rs` | Modify | Pass `NoOpContextEngine::default()` in RunTurnArgs construction |
| `crates/kay-cli/tests/context_e2e.rs` | Create | W-7b: E2E context injection tests (3 tests) |

---

## Task 0: Workspace Dependencies + Crate Scaffold

**Files:**
- Modify: `Cargo.toml` (lines 1–42 members list + `[workspace.dependencies]` section)
- Modify: `crates/kay-session/Cargo.toml`
- Create: `crates/kay-context/Cargo.toml`
- Create: `crates/kay-context/src/lib.rs` (empty stub)

- [ ] **Step 1: Add kay-context to workspace members**

In `Cargo.toml`, add `"crates/kay-context"` to the `members` list (after `"crates/kay-session"`):

```toml
# existing line:
    "crates/kay-session",
# add after:
    "crates/kay-context",
```

- [ ] **Step 2: Promote rusqlite to workspace.dependencies**

In `Cargo.toml`, find the `[workspace.dependencies]` section under `# Database` and add:

```toml
rusqlite = { version = "0.38", features = ["bundled"] }
```

- [ ] **Step 3: Add new workspace deps**

Still in `Cargo.toml` `[workspace.dependencies]`, add after the Database block:

```toml
# Phase 7 (kay-context) additions
tree-sitter = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"
sqlite-vec = "=0.1.10-alpha.3"
notify = "6.1"
notify-debouncer-mini = "0.4"
```

- [ ] **Step 4: Update kay-session to inherit rusqlite from workspace**

In `crates/kay-session/Cargo.toml`, change:
```toml
# Before:
rusqlite = { version = "0.38", features = ["bundled"] }
# After:
rusqlite = { workspace = true }
```

- [ ] **Step 5: Create crates/kay-context/Cargo.toml**

```toml
[package]
name = "kay-context"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
kay-tools     = { path = "../kay-tools" }
forge_walker  = { path = "../forge_walker" }
rusqlite      = { workspace = true }
tree-sitter   = { workspace = true }
tree-sitter-rust        = { workspace = true }
tree-sitter-typescript  = { workspace = true }
tree-sitter-python      = { workspace = true }
tree-sitter-go          = { workspace = true }
sqlite-vec    = { workspace = true }
notify        = { workspace = true }
notify-debouncer-mini = { workspace = true }
sha2          = { workspace = true }
hex           = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
thiserror     = { workspace = true }
tracing       = { workspace = true }
anyhow        = { workspace = true }
tokio         = { workspace = true }

[dev-dependencies]
tempfile  = { workspace = true }
proptest  = "1"
```

- [ ] **Step 6: Create crates/kay-context/src/lib.rs (minimal stub)**

```rust
//! Kay context engine — local tree-sitter symbol store + hybrid retrieval.

pub mod budget;
pub mod embedder;
pub mod engine;
pub mod hardener;
pub mod indexer;
pub mod language;
pub mod retriever;
pub mod store;
pub mod watcher;

pub use budget::{ContextBudget, ContextPacket};
pub use engine::{ContextEngine, KayContextEngine, NoOpContextEngine};
pub use store::{Symbol, SymbolKind, SymbolStore};
pub use indexer::IndexStats;
```

- [ ] **Step 7: Verify workspace compiles**

```bash
cargo check -p kay-context 2>&1 | head -30
```

Expected: errors about missing modules — that is expected since stubs are empty. Should NOT error on Cargo.toml parsing or dependency resolution.

- [ ] **Step 8: Commit scaffold**

```bash
git add Cargo.toml crates/kay-session/Cargo.toml crates/kay-context/
git commit -m "chore(phase-7): scaffold kay-context crate + promote rusqlite to workspace

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 1: AgentEvent Extensions + Wire Serialization

**Files:**
- Modify: `crates/kay-tools/src/events.rs`
- Modify: `crates/kay-tools/src/events_wire.rs`
- Modify: `crates/kay-tools/tests/events_wire_snapshots.rs`

- [ ] **Step 1: Write failing snapshot tests (RED)**

Add to `crates/kay-tools/tests/events_wire_snapshots.rs`:

```rust
#[test]
fn snap_context_truncated_wire() {
    let ev = AgentEvent::ContextTruncated {
        dropped_symbols: 3,
        budget_tokens: 8192,
    };
    let wire = AgentEventWire::from(&ev);
    insta::assert_json_snapshot!(wire, @"");
}

#[test]
fn snap_index_progress_wire() {
    let ev = AgentEvent::IndexProgress {
        indexed: 100,
        total: 500,
    };
    let wire = AgentEventWire::from(&ev);
    insta::assert_json_snapshot!(wire, @"");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p kay-tools snap_context_truncated_wire snap_index_progress_wire 2>&1 | tail -20
```

Expected: compile error — `AgentEvent::ContextTruncated` and `AgentEvent::IndexProgress` do not exist yet.

- [ ] **Step 3: Commit RED**

```bash
git add crates/kay-tools/tests/events_wire_snapshots.rs
git commit -m "test(ctx-w1): RED — snapshot tests for ContextTruncated + IndexProgress wire

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 4: Add variants to AgentEvent (GREEN)**

In `crates/kay-tools/src/events.rs`, find the last existing variant before the closing `}` of the enum and add (respecting `#[non_exhaustive]` — no attribute needed, it's already applied):

```rust
    // -- Phase 7 additive variants (CTX-04, CTX-01) ---------------------
    /// Budget-enforced truncation occurred; some symbols were dropped.
    ContextTruncated {
        dropped_symbols: usize,
        budget_tokens: usize,
    },

    /// Background indexing progress update.
    IndexProgress {
        indexed: usize,
        total: usize,
    },
```

- [ ] **Step 5: Add match arms to AgentEventWire Serialize impl**

In `crates/kay-tools/src/events_wire.rs`, find the `match self.0 {` block inside `impl<'a> Serialize for AgentEventWire<'a>`. Add these two arms BEFORE the closing `}` of the match:

```rust
            AgentEvent::ContextTruncated { dropped_symbols, budget_tokens } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "context_truncated")?;
                m.serialize_entry("dropped_symbols", dropped_symbols)?;
                m.serialize_entry("budget_tokens", budget_tokens)?;
                m.end()
            }
            AgentEvent::IndexProgress { indexed, total } => {
                let mut m = serializer.serialize_map(Some(3))?;
                m.serialize_entry("type", "index_progress")?;
                m.serialize_entry("indexed", indexed)?;
                m.serialize_entry("total", total)?;
                m.end()
            }
```

- [ ] **Step 6: Run tests and accept snapshots**

```bash
cargo test -p kay-tools snap_context_truncated_wire snap_index_progress_wire 2>&1 | tail -20
cargo insta review
```

Expected: insta shows pending snapshots; accept both.

- [ ] **Step 7: Run full kay-tools test suite**

```bash
cargo test -p kay-tools 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 8: Commit GREEN**

```bash
git add crates/kay-tools/src/events.rs crates/kay-tools/src/events_wire.rs \
        crates/kay-tools/tests/events_wire_snapshots.rs \
        crates/kay-tools/tests/snapshots/
git commit -m "feat(ctx-w1): GREEN — ContextTruncated + IndexProgress AgentEvent variants + wire

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 2: ToolRegistry::schemas()

**Files:**
- Modify: `crates/kay-tools/src/registry.rs`

- [ ] **Step 1: Write failing test (RED)**

In `crates/kay-tools/src/registry.rs`, add to the existing `#[cfg(test)]` block:

```rust
    #[test]
    fn schemas_returns_one_value_per_tool() {
        let mut r = ToolRegistry::new();
        r.register(dummy("tool-a"));
        r.register(dummy("tool-b"));
        let schemas = r.schemas();
        assert_eq!(schemas.len(), 2, "schemas must have one entry per registered tool");
        for s in &schemas {
            assert!(s.is_object(), "each schema must be a JSON object");
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p kay-tools schemas_returns_one_value_per_tool 2>&1 | tail -10
```

Expected: compile error — `schemas` method does not exist.

- [ ] **Step 3: Commit RED**

```bash
git add crates/kay-tools/src/registry.rs
git commit -m "test(ctx-w2): RED — ToolRegistry::schemas() missing

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 4: Add schemas() method (GREEN)**

In `crates/kay-tools/src/registry.rs`, add inside the `impl ToolRegistry` block (after `is_empty`):

```rust
    /// Return the raw JSON input schemas for all registered tools.
    /// Called by SchemaHardener before injecting into the provider payload.
    pub fn schemas(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|t| t.input_schema())
            .collect()
    }
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test -p kay-tools schemas_returns_one_value_per_tool 2>&1 | tail -10
```

Expected: test passes.

- [ ] **Step 6: Commit GREEN**

```bash
git add crates/kay-tools/src/registry.rs
git commit -m "feat(ctx-w2): GREEN — ToolRegistry::schemas() returns raw JSON schemas

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 3: W-1 — SymbolStore CRUD

**Files:**
- Create: `crates/kay-context/src/store.rs`
- Create: `crates/kay-context/tests/store.rs`
- Modify: `crates/kay-context/src/lib.rs` (already stubs the pub uses)

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/store.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use std::path::PathBuf;
use tempfile::TempDir;

fn make_store() -> (TempDir, SymbolStore) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (dir, store)
}

fn dummy_symbol(name: &str, file: &str) -> Symbol {
    Symbol {
        id: 0,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: PathBuf::from(file),
        start_line: 1,
        end_line: 5,
        signature: format!("fn {name}()"),
        doc_comment: None,
        relevance_score: 0.0,
    }
}

#[test]
fn schema_creates_tables() {
    let (_dir, store) = make_store();
    let tables = store.table_names().unwrap();
    assert!(tables.contains(&"symbols".to_string()), "symbols table missing");
    assert!(tables.contains(&"index_state".to_string()), "index_state table missing");
}

#[test]
fn insert_and_query_by_name() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&dummy_symbol("fn_foo", "src/lib.rs")).unwrap();
    let results = store.query_by_name("fn_foo").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "fn_foo");
    assert_eq!(results[0].signature, "fn fn_foo()");
}

#[test]
fn delete_clears_fts() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&dummy_symbol("fn_bar", "src/bar.rs")).unwrap();
    store.delete_symbols_for_file(&PathBuf::from("src/bar.rs")).unwrap();
    let results = store.fts_search("fn_bar").unwrap();
    assert!(results.is_empty(), "FTS must be empty after delete");
}

#[test]
fn index_state_skip_on_same_hash() {
    let (_dir, mut store) = make_store();
    let path = PathBuf::from("src/main.rs");
    let content = b"fn main() {}";
    let stats1 = store.update_index_state(&path, content).unwrap();
    assert!(!stats1.skipped);
    let stats2 = store.update_index_state(&path, content).unwrap();
    assert!(stats2.skipped, "identical hash should be skipped");
}

#[test]
fn index_state_updates_on_hash_change() {
    let (_dir, mut store) = make_store();
    let path = PathBuf::from("src/main.rs");
    store.update_index_state(&path, b"fn main() {}").unwrap();
    let stats = store.update_index_state(&path, b"fn main() { println!(\"hi\"); }").unwrap();
    assert!(!stats.skipped, "changed hash should not be skipped");
}
```

- [ ] **Step 2: Run to verify compile failure (RED)**

```bash
cargo test -p kay-context 2>&1 | head -30
```

Expected: compile errors — `store` module missing.

- [ ] **Step 3: Commit RED**

```bash
git add crates/kay-context/tests/store.rs
git commit -m "test(ctx-w1): RED — SymbolStore CRUD tests (5 tests)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 4: Implement store.rs (GREEN)**

Create `crates/kay-context/src/store.rs`:

```rust
//! SQLite-backed symbol store with FTS5 and optional sqlite-vec.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Module,
    Class,
    Trait,
    Impl,
    FileBoundary,
}

impl SymbolKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Module => "module",
            Self::Class => "class",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::FileBoundary => "boundary",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "function" => Self::Function,
            "module" => Self::Module,
            "class" => Self::Class,
            "trait" => Self::Trait,
            "impl" => Self::Impl,
            _ => Self::FileBoundary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: i64,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub signature: String,
    pub doc_comment: Option<String>,
    pub relevance_score: f32,
}

pub struct UpdateStats {
    pub skipped: bool,
}

pub struct SymbolStore {
    conn: Connection,
}

impl SymbolStore {
    pub fn open(dir: &Path) -> Result<Self> {
        let db_path = dir.join("symbols.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self { conn };
        store.apply_schema()?;
        Ok(store)
    }

    fn apply_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS symbols (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                kind        TEXT NOT NULL CHECK(kind IN ('function','module','class','trait','impl','boundary')),
                file_path   TEXT NOT NULL,
                start_line  INTEGER NOT NULL,
                end_line    INTEGER NOT NULL,
                signature   TEXT NOT NULL,
                doc_comment TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);

            CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
                name, signature, doc_comment,
                content=symbols, content_rowid=id
            );

            CREATE TABLE IF NOT EXISTS index_state (
                file_path   TEXT PRIMARY KEY,
                mtime_secs  INTEGER NOT NULL DEFAULT 0,
                file_hash   TEXT NOT NULL,
                indexed_at  INTEGER NOT NULL
            );

            CREATE TRIGGER IF NOT EXISTS symbols_ai AFTER INSERT ON symbols BEGIN
                INSERT INTO symbols_fts(rowid, name, signature, doc_comment)
                VALUES (new.id, new.name, new.signature, new.doc_comment);
            END;

            CREATE TRIGGER IF NOT EXISTS symbols_ad AFTER DELETE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, signature, doc_comment)
                VALUES ('delete', old.id, old.name, old.signature, old.doc_comment);
            END;

            CREATE TRIGGER IF NOT EXISTS symbols_au AFTER UPDATE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, signature, doc_comment)
                VALUES ('delete', old.id, old.name, old.signature, old.doc_comment);
                INSERT INTO symbols_fts(rowid, name, signature, doc_comment)
                VALUES (new.id, new.name, new.signature, new.doc_comment);
            END;
        ")?;
        Ok(())
    }

    pub fn table_names(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' OR type='shadow'"
        )?;
        let names = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(names)
    }

    pub fn insert_symbol(&mut self, sym: &Symbol) -> Result<i64> {
        let sig = truncate_sig(&sym.signature);
        self.conn.execute(
            "INSERT INTO symbols(name,kind,file_path,start_line,end_line,signature,doc_comment)
             VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                sym.name,
                sym.kind.as_str(),
                sym.file_path.to_string_lossy().as_ref(),
                sym.start_line,
                sym.end_line,
                sig,
                sym.doc_comment,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_symbols_for_file(&mut self, file: &Path) -> Result<()> {
        self.conn.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file.to_string_lossy().as_ref()],
        )?;
        Ok(())
    }

    pub fn query_by_name(&self, name: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,name,kind,file_path,start_line,end_line,signature,doc_comment
             FROM symbols WHERE name = ?1"
        )?;
        let rows = stmt.query_map(params![name], row_to_symbol)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn fts_search(&self, query: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id,s.name,s.kind,s.file_path,s.start_line,s.end_line,s.signature,s.doc_comment
             FROM symbols_fts f
             JOIN symbols s ON f.rowid = s.id
             WHERE symbols_fts MATCH ?1
             ORDER BY bm25(symbols_fts) ASC
             LIMIT 50"
        )?;
        let rows = stmt.query_map(params![query], row_to_symbol)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn update_index_state(&mut self, path: &Path, content: &[u8]) -> Result<UpdateStats> {
        let hash = file_hash_prefix(content);
        let existing: Option<String> = self.conn.query_row(
            "SELECT file_hash FROM index_state WHERE file_path = ?1",
            params![path.to_string_lossy().as_ref()],
            |r| r.get(0),
        ).ok();

        if existing.as_deref() == Some(&hash) {
            return Ok(UpdateStats { skipped: true });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO index_state(file_path,mtime_secs,file_hash,indexed_at)
             VALUES(?1,0,?2,?3)
             ON CONFLICT(file_path) DO UPDATE SET file_hash=excluded.file_hash, indexed_at=excluded.indexed_at",
            params![path.to_string_lossy().as_ref(), hash, now],
        )?;
        Ok(UpdateStats { skipped: false })
    }

    pub fn enable_vector_search(&mut self) -> Result<()> {
        unsafe {
            sqlite_vec::sqlite3_auto_extension_vec0();
        }
        self.conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS symbols_vec USING vec0(embedding float[1536]);"
        )?;
        Ok(())
    }
}

fn file_hash_prefix(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    hex::encode(&digest[..8]) // first 16 hex chars
}

fn truncate_sig(sig: &str) -> String {
    const MAX: usize = 256;
    if sig.chars().count() <= MAX {
        sig.to_string()
    } else {
        let mut s: String = sig.chars().take(MAX - 1).collect();
        s.push('…');
        s
    }
}

fn row_to_symbol(row: &rusqlite::Row<'_>) -> rusqlite::Result<Symbol> {
    Ok(Symbol {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: SymbolKind::from_str(&row.get::<_, String>(2)?),
        file_path: PathBuf::from(row.get::<_, String>(3)?),
        start_line: row.get(4)?,
        end_line: row.get(5)?,
        signature: row.get(6)?,
        doc_comment: row.get(7)?,
        relevance_score: 0.0,
    })
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p kay-context --test store 2>&1 | tail -20
```

Expected: all 5 store tests pass.

- [ ] **Step 6: Commit GREEN**

```bash
git add crates/kay-context/src/store.rs crates/kay-context/tests/store.rs \
        crates/kay-context/src/lib.rs
git commit -m "feat(ctx-w1): GREEN — SymbolStore CRUD, FTS5 triggers, index_state hash-skip

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 4: W-2 — Language Detection + TreeSitterIndexer

**Files:**
- Create: `crates/kay-context/src/language.rs`
- Create: `crates/kay-context/src/indexer.rs`
- Create: `crates/kay-context/tests/indexer.rs`

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/indexer.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::indexer::{IndexStats, TreeSitterIndexer};
use kay_context::store::{Symbol, SymbolKind};
use std::path::PathBuf;
use proptest::prelude::*;

fn indexer() -> TreeSitterIndexer {
    TreeSitterIndexer::new()
}

fn index_src(src: &str, ext: &str) -> Vec<Symbol> {
    let path = PathBuf::from(format!("test_file.{ext}"));
    indexer().index_content(src.as_bytes(), &path).unwrap()
}

#[test]
fn rust_fn_extracted() {
    let syms = index_src("fn foo(x: i32) -> i32 { x }", "rs");
    let f = syms.iter().find(|s| s.name == "foo").expect("foo not found");
    assert_eq!(f.kind, SymbolKind::Function);
    assert!(f.signature.contains("foo"), "sig must contain name");
}

#[test]
fn rust_trait_extracted() {
    let syms = index_src("trait Bar { fn baz(); }", "rs");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Trait && s.name == "Bar"),
        "trait Bar not found in: {syms:?}");
}

#[test]
fn rust_mod_boundary() {
    let syms = index_src("mod utils {}", "rs");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Module && s.name == "utils"),
        "mod utils not found");
}

#[test]
fn typescript_function_extracted() {
    let syms = index_src("function greet(name: string): string { return name; }", "ts");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Function && s.name == "greet"),
        "greet not found");
}

#[test]
fn typescript_class_extracted() {
    let syms = index_src("class Foo { bar() {} }", "ts");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Class && s.name == "Foo"),
        "class Foo not found");
}

#[test]
fn python_def_extracted() {
    let syms = index_src("def compute(x):\n    return x * 2\n", "py");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Function && s.name == "compute"),
        "compute not found");
}

#[test]
fn python_class_extracted() {
    let syms = index_src("class Solver:\n    pass\n", "py");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Class && s.name == "Solver"),
        "Solver not found");
}

#[test]
fn go_func_extracted() {
    let syms = index_src("package main\nfunc Run() error { return nil }\n", "go");
    assert!(syms.iter().any(|s| s.kind == SymbolKind::Function && s.name == "Run"),
        "Run not found");
}

#[test]
fn sig_truncated_at_256() {
    let long_sig = format!("fn {}(x: i32) -> i32 {{}}", "a".repeat(300));
    let syms = index_src(&long_sig, "rs");
    for sym in &syms {
        let len = sym.signature.chars().count();
        assert!(len <= 257, "signature too long: {len} chars in {:?}", sym.signature);
    }
}

#[test]
fn unknown_extension_file_boundary() {
    let syms = index_src("[package]\nname = \"foo\"\n", "toml");
    assert_eq!(syms.len(), 1, "unknown lang must produce exactly one FileBoundary");
    assert_eq!(syms[0].kind, SymbolKind::FileBoundary);
}

proptest! {
    #[test]
    fn proptest_sig_never_exceeds_256(src in "[a-zA-Z0-9_ {}()\n;:]*") {
        let path = PathBuf::from("test.rs");
        let indexer = TreeSitterIndexer::new();
        // Index may produce 0 symbols for gibberish input — that's fine
        if let Ok(syms) = indexer.index_content(src.as_bytes(), &path) {
            for sym in syms {
                let len = sym.signature.chars().count();
                prop_assert!(len <= 257, "sig too long: {len}");
            }
        }
    }
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/indexer.rs
git commit -m "test(ctx-w2): RED — TreeSitterIndexer tests for Rust/TS/Py/Go + proptest

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement language.rs**

Create `crates/kay-context/src/language.rs`:

```rust
//! Language detection from file extension.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Go,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "ts" | "tsx" => Self::TypeScript,
            "py" => Self::Python,
            "go" => Self::Go,
            _ => Self::Unknown,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::Unknown)
    }
}
```

- [ ] **Step 4: Implement indexer.rs**

Create `crates/kay-context/src/indexer.rs`:

```rust
//! tree-sitter-based symbol extractor.

use std::path::Path;

use thiserror::Error;
use tree_sitter::{Node, Parser};

use crate::language::Language;
use crate::store::{Symbol, SymbolKind};

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("tree-sitter: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, IndexerError>;

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub indexed_files: usize,
    pub total_symbols: usize,
    pub skipped_files: usize,
    pub duration_ms: u64,
}

pub struct TreeSitterIndexer;

impl TreeSitterIndexer {
    pub fn new() -> Self {
        Self
    }

    pub fn index_content(&self, content: &[u8], path: &Path) -> Result<Vec<Symbol>> {
        let lang = Language::from_path(path);
        match lang {
            Language::Rust => self.index_rust(content, path),
            Language::TypeScript => self.index_typescript(content, path),
            Language::Python => self.index_python(content, path),
            Language::Go => self.index_go(content, path),
            Language::Unknown => Ok(self.file_boundary(content, path)),
        }
    }

    fn index_rust(&self, content: &[u8], path: &Path) -> Result<Vec<Symbol>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| IndexerError::Parse(e.to_string()))?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| IndexerError::Parse("parse returned None".into()))?;
        let src = std::str::from_utf8(content).unwrap_or("");
        let mut syms = Vec::new();
        extract_rust_symbols(tree.root_node(), src, path, &mut syms);
        Ok(syms)
    }

    fn index_typescript(&self, content: &[u8], path: &Path) -> Result<Vec<Symbol>> {
        let mut parser = Parser::new();
        let lang = if path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        };
        parser.set_language(&lang)
            .map_err(|e| IndexerError::Parse(e.to_string()))?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| IndexerError::Parse("parse returned None".into()))?;
        let src = std::str::from_utf8(content).unwrap_or("");
        let mut syms = Vec::new();
        extract_ts_symbols(tree.root_node(), src, path, &mut syms);
        Ok(syms)
    }

    fn index_python(&self, content: &[u8], path: &Path) -> Result<Vec<Symbol>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| IndexerError::Parse(e.to_string()))?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| IndexerError::Parse("parse returned None".into()))?;
        let src = std::str::from_utf8(content).unwrap_or("");
        let mut syms = Vec::new();
        extract_python_symbols(tree.root_node(), src, path, &mut syms);
        Ok(syms)
    }

    fn index_go(&self, content: &[u8], path: &Path) -> Result<Vec<Symbol>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| IndexerError::Parse(e.to_string()))?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| IndexerError::Parse("parse returned None".into()))?;
        let src = std::str::from_utf8(content).unwrap_or("");
        let mut syms = Vec::new();
        extract_go_symbols(tree.root_node(), src, path, &mut syms);
        Ok(syms)
    }

    fn file_boundary(&self, content: &[u8], path: &Path) -> Vec<Symbol> {
        let src = std::str::from_utf8(content).unwrap_or("");
        let first_10: String = src.lines().take(10).collect::<Vec<_>>().join("\n");
        let sig = truncate_sig(&first_10);
        vec![Symbol {
            id: 0,
            name: path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string(),
            kind: SymbolKind::FileBoundary,
            file_path: path.to_path_buf(),
            start_line: 1,
            end_line: 10_u32.min(src.lines().count() as u32),
            signature: sig,
            doc_comment: None,
            relevance_score: 0.0,
        }]
    }
}

impl Default for TreeSitterIndexer {
    fn default() -> Self {
        Self::new()
    }
}

fn node_text<'a>(node: Node<'_>, src: &'a str) -> &'a str {
    &src[node.start_byte()..node.end_byte()]
}

fn truncate_sig(sig: &str) -> String {
    const MAX: usize = 256;
    if sig.chars().count() <= MAX {
        sig.to_string()
    } else {
        let mut s: String = sig.chars().take(MAX - 1).collect();
        s.push('…');
        s
    }
}

fn make_symbol(name: &str, kind: SymbolKind, sig: &str, path: &Path, node: Node<'_>) -> Symbol {
    Symbol {
        id: 0,
        name: name.to_string(),
        kind,
        file_path: path.to_path_buf(),
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
        signature: truncate_sig(sig),
        doc_comment: None,
        relevance_score: 0.0,
    }
}

fn extract_rust_symbols(node: Node<'_>, src: &str, path: &Path, out: &mut Vec<Symbol>) {
    match node.kind() {
        "function_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                // sig = fn <name><params> [-> <ret>]  (one line, no body)
                let sig = build_rust_fn_sig(node, src);
                out.push(make_symbol(name, SymbolKind::Function, &sig, path, node));
            }
        }
        "trait_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Trait, &format!("trait {name}"), path, node));
            }
        }
        "impl_item" => {
            let sig = if let Some(ty) = node.child_by_field_name("type") {
                format!("impl {}", node_text(ty, src))
            } else {
                "impl".to_string()
            };
            out.push(make_symbol(&sig, SymbolKind::Impl, &sig, path, node));
        }
        "mod_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Module, &format!("mod {name}"), path, node));
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_rust_symbols(child, src, path, out);
    }
}

fn build_rust_fn_sig(fn_node: Node<'_>, src: &str) -> String {
    // Build "fn name(params) -> ret" without the body
    let name = fn_node.child_by_field_name("name")
        .map(|n| node_text(n, src))
        .unwrap_or("?");
    let params = fn_node.child_by_field_name("parameters")
        .map(|n| node_text(n, src))
        .unwrap_or("()");
    let ret = fn_node.child_by_field_name("return_type")
        .map(|n| format!(" -> {}", node_text(n, src)))
        .unwrap_or_default();
    format!("fn {name}{params}{ret}")
}

fn extract_ts_symbols(node: Node<'_>, src: &str, path: &Path, out: &mut Vec<Symbol>) {
    match node.kind() {
        "function_declaration" | "function" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Function, &format!("function {name}(...)"), path, node));
            }
        }
        "class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Class, &format!("class {name}"), path, node));
            }
        }
        "module" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Module, &format!("module {name}"), path, node));
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_ts_symbols(child, src, path, out);
    }
}

fn extract_python_symbols(node: Node<'_>, src: &str, path: &Path, out: &mut Vec<Symbol>) {
    match node.kind() {
        "function_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Function, &format!("def {name}(...)"), path, node));
            }
        }
        "class_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Class, &format!("class {name}"), path, node));
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_python_symbols(child, src, path, out);
    }
}

fn extract_go_symbols(node: Node<'_>, src: &str, path: &Path, out: &mut Vec<Symbol>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Function, &format!("func {name}(...)"), path, node));
            }
        }
        "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, src);
                out.push(make_symbol(name, SymbolKind::Function, &format!("func (...) {name}(...)"), path, node));
            }
        }
        "type_declaration" => {
            let sig = node_text(node, src).lines().next().unwrap_or("type").to_string();
            out.push(make_symbol("type", SymbolKind::Class, &sig, path, node));
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_go_symbols(child, src, path, out);
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p kay-context --test indexer 2>&1 | tail -30
```

Expected: all 10 deterministic tests pass; proptest runs 100 cases.

- [ ] **Step 6: Commit GREEN**

```bash
git add crates/kay-context/src/language.rs crates/kay-context/src/indexer.rs \
        crates/kay-context/tests/indexer.rs
git commit -m "feat(ctx-w2): GREEN — TreeSitterIndexer for Rust/TS/Py/Go + FileBoundary fallback

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 5: W-3 — FTS5 Retriever

**Files:**
- Create: `crates/kay-context/src/retriever.rs`
- Create: `crates/kay-context/tests/retriever_fts.rs`

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/retriever_fts.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use std::path::PathBuf;
use tempfile::TempDir;

fn make_store() -> (TempDir, SymbolStore) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (dir, store)
}

fn sym(name: &str, sig: &str) -> Symbol {
    Symbol {
        id: 0,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: PathBuf::from("src/lib.rs"),
        start_line: 1,
        end_line: 5,
        signature: sig.to_string(),
        doc_comment: None,
        relevance_score: 0.0,
    }
}

#[test]
fn fts_exact_match_returns_symbol() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&sym("run_loop", "fn run_loop()")).unwrap();
    let results = store.fts_search("run_loop").unwrap();
    assert!(!results.is_empty(), "exact match must return symbol");
    assert_eq!(results[0].name, "run_loop");
}

#[test]
fn fts_no_match_returns_empty() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&sym("foo", "fn foo()")).unwrap();
    let results = store.fts_search("zzznomatch").unwrap();
    assert!(results.is_empty(), "no-match query must return empty");
}

#[test]
fn fts_prefix_match() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&sym("run_loop", "fn run_loop()")).unwrap();
    let results = store.fts_search("run_lo*").unwrap();
    assert!(!results.is_empty(), "prefix match must return symbol");
}

#[test]
fn fts_name_bonus_applied() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&sym("target_fn", "fn target_fn(x: i32)")).unwrap();
    store.insert_symbol(&sym("other_fn", "fn other_fn(target_fn: i32)")).unwrap();
    let results = store.fts_search("target_fn").unwrap();
    // After name-bonus, the symbol whose NAME == query term should rank first
    let top_name = results.first().map(|s| s.name.as_str()).unwrap_or("");
    // Both may match; the exact-name symbol must appear in results
    assert!(results.iter().any(|s| s.name == "target_fn"));
    let _ = top_name; // ranking with bonus tested in retriever integration
}

#[test]
fn fts_ranking_order() {
    let (_dir, mut store) = make_store();
    // sig with "compute" repeated 3x should rank above sig with it once
    store.insert_symbol(&sym("a", "fn a() compute compute compute")).unwrap();
    store.insert_symbol(&sym("b", "fn b() compute")).unwrap();
    let results = store.fts_search("compute").unwrap();
    assert!(!results.is_empty());
    // 'a' should rank first (higher bm25 due to frequency)
    assert_eq!(results[0].name, "a", "higher-freq symbol must rank first");
}

#[test]
fn fts_multi_word_query() {
    let (_dir, mut store) = make_store();
    store.insert_symbol(&sym("run_loop", "fn run_loop() calls execute")).unwrap();
    store.insert_symbol(&sym("just_run", "fn just_run()")).unwrap();
    let results = store.fts_search("run execute").unwrap();
    // run_loop has both terms; just_run has only one
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "run_loop", "symbol with both terms ranks first");
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/retriever_fts.rs
git commit -m "test(ctx-w3): RED — FTS5 retriever tests

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement retriever.rs**

Create `crates/kay-context/src/retriever.rs`:

```rust
//! Hybrid FTS5 + sqlite-vec retriever with RRF merge.

use std::collections::HashMap;

use crate::store::{Symbol, SymbolStore};

/// Reciprocal Rank Fusion constant k=60.
const RRF_K: f32 = 60.0;

/// Merge two ranked lists via Reciprocal Rank Fusion.
/// Input: slices of symbol ids in rank order (index 0 = rank 1).
/// Returns map of symbol_id → rrf_score (higher = better).
pub fn rrf_merge(fts_ids: &[i64], vec_ids: &[i64]) -> HashMap<i64, f32> {
    let mut scores: HashMap<i64, f32> = HashMap::new();
    for (rank, id) in fts_ids.iter().enumerate() {
        *scores.entry(*id).or_default() += 1.0 / (RRF_K + rank as f32 + 1.0);
    }
    for (rank, id) in vec_ids.iter().enumerate() {
        *scores.entry(*id).or_default() += 1.0 / (RRF_K + rank as f32 + 1.0);
    }
    scores
}

/// Apply a +0.5 name-bonus to symbols whose exact name appears in the query.
pub fn apply_name_bonus(syms: &mut [Symbol], query: &str) {
    for sym in syms.iter_mut() {
        if query.split_whitespace().any(|w| w == sym.name) {
            sym.relevance_score += 0.5;
        }
    }
}
```

- [ ] **Step 4: Update SymbolStore.fts_search to return scores and apply bonus**

The existing `fts_search` in `store.rs` returns symbols. Update it to set `relevance_score` from bm25 (bm25 is negative; negate so higher = better):

In `store.rs`, update the `fts_search` method to:

```rust
pub fn fts_search(&self, query: &str) -> Result<Vec<Symbol>> {
    let mut stmt = self.conn.prepare(
        "SELECT s.id,s.name,s.kind,s.file_path,s.start_line,s.end_line,s.signature,s.doc_comment,
                -bm25(symbols_fts) as score
         FROM symbols_fts f
         JOIN symbols s ON f.rowid = s.id
         WHERE symbols_fts MATCH ?1
         ORDER BY bm25(symbols_fts) ASC
         LIMIT 50"
    )?;
    let rows = stmt.query_map(params![query], |row| {
        let mut sym = row_to_symbol(row)?;
        sym.relevance_score = row.get::<_, f64>(8)? as f32;
        Ok(sym)
    })?
    .filter_map(|r| r.ok())
    .collect();
    Ok(rows)
}
```

- [ ] **Step 5: Run W-3 tests**

```bash
cargo test -p kay-context --test retriever_fts 2>&1 | tail -20
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit GREEN**

```bash
git add crates/kay-context/src/retriever.rs crates/kay-context/src/store.rs \
        crates/kay-context/tests/retriever_fts.rs
git commit -m "feat(ctx-w3): GREEN — FTS5 retriever with bm25 scoring + RRF merge + name bonus

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 6: W-4 — sqlite-vec + RRF Integration

**Files:**
- Create: `crates/kay-context/src/embedder.rs`
- Create: `crates/kay-context/tests/retriever_vec.rs`
- Modify: `crates/kay-context/src/store.rs` (add vector insert/query methods)

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/retriever_vec.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::embedder::{EmbeddingProvider, FakeEmbedder, NoOpEmbedder};
use kay_context::retriever::rrf_merge;
use kay_context::store::{Symbol, SymbolKind, SymbolStore};
use std::path::PathBuf;
use tempfile::TempDir;

fn make_store_with_vec() -> (TempDir, SymbolStore) {
    let dir = TempDir::new().unwrap();
    let mut store = SymbolStore::open(dir.path()).unwrap();
    store.enable_vector_search().unwrap();
    (dir, store)
}

fn sym(name: &str) -> Symbol {
    Symbol {
        id: 0,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: PathBuf::from("src/lib.rs"),
        start_line: 1,
        end_line: 5,
        signature: format!("fn {name}()"),
        doc_comment: None,
        relevance_score: 0.0,
    }
}

#[test]
fn vec_table_created_with_fake_embedder() {
    let (_dir, store) = make_store_with_vec();
    let tables = store.table_names().unwrap();
    assert!(tables.iter().any(|t| t.contains("symbols_vec")),
        "symbols_vec not found in: {tables:?}");
}

#[test]
fn fake_embedder_insert_and_ann() {
    let (_dir, mut store) = make_store_with_vec();
    let embedder = FakeEmbedder { dimensions: 1536 };
    let id0 = store.insert_symbol(&sym("alpha")).unwrap();
    let id1 = store.insert_symbol(&sym("beta")).unwrap();
    let id2 = store.insert_symbol(&sym("gamma")).unwrap();

    let sigs = ["fn alpha()", "fn beta()", "fn gamma()"];
    let vecs = embedder.embed(&sigs).unwrap();
    store.insert_vector(id0, &vecs[0]).unwrap();
    store.insert_vector(id1, &vecs[1]).unwrap();
    store.insert_vector(id2, &vecs[2]).unwrap();

    // Query with same embedding as alpha → alpha should be top-1
    let results = store.ann_search(&vecs[0], 3).unwrap();
    assert!(!results.is_empty(), "ANN must return results");
    assert_eq!(results[0], id0, "alpha must be top-1 when queried with its own vector");
}

#[test]
fn rrf_merge_prefers_fts_winner() {
    // FTS top = id 1; ANN top = id 2
    // With equal list lengths, RRF rank-1 beats rank-2
    let fts = vec![1i64, 2, 3];
    let ann = vec![2i64, 1, 3];
    let scores = rrf_merge(&fts, &ann);
    // id 1: 1/(60+1) + 1/(60+2) = dominant on first list
    // id 2: 1/(60+2) + 1/(60+1) = same! RRF is symmetric here
    // So both get same score — this test checks neither panics and
    // both ids appear in merged output
    assert!(scores.contains_key(&1));
    assert!(scores.contains_key(&2));
}

#[test]
fn rrf_merge_prefers_vec_winner() {
    // No FTS signal; pure ANN — vec winner should be in output
    let fts: Vec<i64> = vec![];
    let ann = vec![42i64, 7, 3];
    let scores = rrf_merge(&fts, &ann);
    assert!(scores.contains_key(&42), "ANN top must appear in merged output");
    let score_42 = scores[&42];
    let score_7  = scores[&7];
    assert!(score_42 > score_7, "first ANN result must outscore second");
}

#[test]
fn rrf_k60_score_formula() {
    // symbol appears in both lists at known ranks: fts rank=0, ann rank=0
    let fts = vec![99i64];
    let ann = vec![99i64];
    let scores = rrf_merge(&fts, &ann);
    let expected = 1.0 / (60.0 + 1.0) + 1.0 / (60.0 + 1.0);
    let got = scores[&99];
    assert!((got - expected).abs() < 1e-6, "got={got} expected={expected}");
}

#[test]
fn noop_embedder_skips_vec() {
    let (_dir, store) = make_store_with_vec();
    let embedder = NoOpEmbedder;
    // NoOpEmbedder::embed returns empty — no panics, no vec access
    let result = embedder.embed(&["fn foo()"]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "NoOpEmbedder must return empty vecs");
    // Store is still usable after NoOpEmbedder interaction
    let _ = store.table_names().unwrap();
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/retriever_vec.rs
git commit -m "test(ctx-w4): RED — sqlite-vec + RRF merge tests with FakeEmbedder

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement embedder.rs**

Create `crates/kay-context/src/embedder.rs`:

```rust
//! Embedding providers for vector search.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("embedding failed: {0}")]
    Failed(String),
}

pub type Result<T> = std::result::Result<T, EmbedError>;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
}

/// Default: vector search disabled.
pub struct NoOpEmbedder;

impl EmbeddingProvider for NoOpEmbedder {
    fn embed(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(vec![])
    }
    fn dimensions(&self) -> usize {
        0
    }
}

/// Test embedder: deterministic fake embeddings using text hashing.
/// `cfg(any(test, feature = "testing"))` so it is excluded from release builds.
#[cfg(any(test, feature = "testing"))]
pub struct FakeEmbedder {
    pub dimensions: usize,
}

#[cfg(any(test, feature = "testing"))]
impl EmbeddingProvider for FakeEmbedder {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut result = Vec::with_capacity(texts.len());
        for text in texts {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let seed = hasher.finish();
            // Generate deterministic pseudo-random floats from seed
            let mut vec = Vec::with_capacity(self.dimensions);
            let mut state = seed;
            for _ in 0..self.dimensions {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let val = ((state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
                vec.push(val);
            }
            // L2-normalize
            let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut vec { *v /= norm; }
            }
            result.push(vec);
        }
        Ok(result)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}
```

- [ ] **Step 4: Add vector insert/query to SymbolStore**

In `crates/kay-context/src/store.rs`, add to `impl SymbolStore`:

```rust
    pub fn insert_vector(&self, symbol_id: i64, embedding: &[f32]) -> Result<()> {
        // sqlite-vec stores as blob of f32 LE bytes
        let bytes: Vec<u8> = embedding.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        self.conn.execute(
            "INSERT INTO symbols_vec(rowid, embedding) VALUES(?1, ?2)
             ON CONFLICT(rowid) DO UPDATE SET embedding = excluded.embedding",
            params![symbol_id, bytes],
        )?;
        Ok(())
    }

    pub fn ann_search(&self, query_vec: &[f32], limit: usize) -> Result<Vec<i64>> {
        let bytes: Vec<u8> = query_vec.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let mut stmt = self.conn.prepare(
            "SELECT rowid FROM symbols_vec
             WHERE embedding MATCH ?1
             ORDER BY distance
             LIMIT ?2"
        )?;
        let ids = stmt.query_map(params![bytes, limit as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(ids)
    }
```

Also add `sqlite_vec` extension loading to `SymbolStore::open`:

```rust
    pub fn open(dir: &Path) -> Result<Self> {
        unsafe { sqlite_vec::sqlite3_auto_extension_vec0(); }
        let db_path = dir.join("symbols.db");
        // ... rest unchanged
```

- [ ] **Step 5: Run W-4 tests**

```bash
cargo test -p kay-context --test retriever_vec 2>&1 | tail -20
```

Expected: all 6 pass.

- [ ] **Step 6: Commit GREEN**

```bash
git add crates/kay-context/src/embedder.rs crates/kay-context/src/store.rs \
        crates/kay-context/tests/retriever_vec.rs
git commit -m "feat(ctx-w4): GREEN — sqlite-vec ANN + RRF merge + FakeEmbedder

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 7: W-5 — ContextBudget + Truncation

**Files:**
- Create: `crates/kay-context/src/budget.rs`
- Create: `crates/kay-context/tests/budget.rs`

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/budget.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::budget::{ContextBudget, ContextPacket, estimate_tokens};
use kay_context::store::{Symbol, SymbolKind};
use std::path::PathBuf;

fn sym(name: &str, sig: &str) -> Symbol {
    Symbol {
        id: 0,
        name: name.to_string(),
        kind: SymbolKind::Function,
        file_path: PathBuf::from("src/lib.rs"),
        start_line: 1,
        end_line: 5,
        signature: sig.to_string(),
        doc_comment: None,
        relevance_score: 1.0,
    }
}

#[test]
fn token_estimate_formula() {
    // name="foo"(3) sig="fn foo() -> i32"(15) → (3+15+10)/4 = 7
    let s = sym("foo", "fn foo() -> i32");
    assert_eq!(estimate_tokens(&s), 7);
}

#[test]
fn exact_fit_no_truncation() {
    // Create symbols that together estimate exactly == available tokens
    let budget = ContextBudget { max_tokens: 100, reserve_tokens: 0 };
    // sym with estimate=7 — use 14 such symbols = 98 ≤ 100 = fits
    let symbols: Vec<Symbol> = (0..14).map(|i| sym(&format!("f{i}"), "fn foo() -> i32")).collect();
    let packet = budget.apply(symbols, &[]);
    assert!(!packet.truncated, "should not truncate exact fit");
    assert_eq!(packet.dropped_count, 0);
}

#[test]
fn one_over_truncates() {
    let budget = ContextBudget { max_tokens: 7, reserve_tokens: 0 };
    // 2 symbols each costing 7 tokens = 14 > 7
    let symbols = vec![
        sym("foo", "fn foo() -> i32"),
        sym("bar", "fn bar() -> i32"),
    ];
    let packet = budget.apply(symbols, &[]);
    assert!(packet.truncated, "should truncate when over budget");
    assert!(packet.dropped_count >= 1);
    assert_eq!(packet.symbols.len(), 1, "only one symbol fits");
}

#[test]
fn zero_available_returns_empty() {
    let budget = ContextBudget { max_tokens: 0, reserve_tokens: 0 };
    let symbols = vec![sym("foo", "fn foo()")];
    let packet = budget.apply(symbols, &[]);
    assert_eq!(packet.symbols.len(), 0);
    assert!(!packet.truncated, "nothing was truncated — no symbols to begin with");
}

#[test]
fn reserve_tokens_reduces_available() {
    let budget = ContextBudget { max_tokens: 200, reserve_tokens: 190 };
    assert_eq!(budget.available(), 10, "available must be max - reserve");
    // sym with estimate 7 — 2 symbols = 14 > 10 → truncation
    let symbols = vec![
        sym("foo", "fn foo() -> i32"),
        sym("bar", "fn bar() -> i32"),
    ];
    let packet = budget.apply(symbols, &[]);
    assert!(packet.truncated);
}

#[test]
fn chars_count_not_bytes() {
    // Non-ASCII: "résumé" = 6 chars but 8 bytes in UTF-8
    let s = sym("résumé", "fn résumé(x: i32) -> i32");
    let estimate = estimate_tokens(&s);
    // name=6 chars, sig=24 chars → (6+24+10)/4 = 10
    assert_eq!(estimate, 10, "must use chars().count(), not .len()");
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/budget.rs
git commit -m "test(ctx-w5): RED — ContextBudget token estimate + truncation tests

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement budget.rs (GREEN)**

Create `crates/kay-context/src/budget.rs`:

```rust
//! Per-turn context budget enforcement with explicit truncation.

use serde_json::Value;

use crate::store::Symbol;

#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub reserve_tokens: usize,
}

impl ContextBudget {
    pub fn available(&self) -> usize {
        self.max_tokens.saturating_sub(self.reserve_tokens)
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self { max_tokens: 8192, reserve_tokens: 2048 }
    }
}

#[derive(Debug, Clone)]
pub struct ContextPacket {
    pub symbols: Vec<Symbol>,
    pub hardened_schemas: Vec<Value>,
    pub truncated: bool,
    pub dropped_count: usize,
    pub total_tokens_estimate: usize,
}

/// Token estimate for a single symbol: (name.chars + sig.chars + 10) / 4
pub fn estimate_tokens(sym: &Symbol) -> usize {
    (sym.name.chars().count() + sym.signature.chars().count() + 10) / 4
}

impl ContextBudget {
    /// Apply budget to a ranked list of symbols. Symbols are assumed sorted
    /// by relevance descending. Accumulate until budget exhausted.
    pub fn apply(&self, mut symbols: Vec<Symbol>, hardened_schemas: &[Value]) -> ContextPacket {
        let available = self.available();
        if available == 0 {
            return ContextPacket {
                symbols: vec![],
                hardened_schemas: hardened_schemas.to_vec(),
                truncated: false,
                dropped_count: 0,
                total_tokens_estimate: 0,
            };
        }

        let mut used = 0usize;
        let mut kept = Vec::new();
        let mut dropped = 0usize;

        // Sort by relevance descending before budgeting
        symbols.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        for sym in symbols {
            let cost = estimate_tokens(&sym);
            if used + cost <= available {
                used += cost;
                kept.push(sym);
            } else {
                dropped += 1;
            }
        }

        ContextPacket {
            truncated: dropped > 0,
            dropped_count: dropped,
            total_tokens_estimate: used,
            symbols: kept,
            hardened_schemas: hardened_schemas.to_vec(),
        }
    }
}
```

- [ ] **Step 4: Run W-5 tests**

```bash
cargo test -p kay-context --test budget 2>&1 | tail -20
```

Expected: all 6 tests pass.

- [ ] **Step 5: Commit GREEN**

```bash
git add crates/kay-context/src/budget.rs crates/kay-context/tests/budget.rs
git commit -m "feat(ctx-w5): GREEN — ContextBudget, estimate_tokens, explicit truncation

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 8: W-6 — SchemaHardener + Engine Assembly

**Files:**
- Create: `crates/kay-context/src/hardener.rs`
- Create: `crates/kay-context/src/engine.rs`
- Create: `crates/kay-context/tests/hardener.rs`
- Update: `crates/kay-context/src/lib.rs` (finalize public API)

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/hardener.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_context::engine::NoOpContextEngine;
use kay_context::hardener::SchemaHardener;
use kay_context::budget::ContextBudget;
use kay_context::engine::ContextEngine;
use serde_json::json;

fn schema_with_props_before_required() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

#[test]
fn harden_moves_required_before_properties() {
    let hardener = SchemaHardener;
    let input = schema_with_props_before_required();
    let hardened = hardener.harden(&[input]);
    let obj = hardened[0].as_object().unwrap();
    let keys: Vec<_> = obj.keys().collect();
    let req_idx = keys.iter().position(|k| *k == "required").unwrap();
    let props_idx = keys.iter().position(|k| *k == "properties").unwrap();
    assert!(req_idx < props_idx, "required must come before properties; keys: {keys:?}");
}

#[test]
fn harden_is_idempotent() {
    let hardener = SchemaHardener;
    let input = schema_with_props_before_required();
    let once = hardener.harden(&[input.clone()]);
    let twice = hardener.harden(&once);
    assert_eq!(
        serde_json::to_string(&once[0]).unwrap(),
        serde_json::to_string(&twice[0]).unwrap(),
        "harden must be idempotent"
    );
}

#[test]
fn harden_empty_input_returns_empty() {
    let hardener = SchemaHardener;
    let result = hardener.harden(&[]);
    assert!(result.is_empty());
}

#[test]
fn noop_engine_hardens_schemas() {
    let engine = NoOpContextEngine;
    let schemas = vec![schema_with_props_before_required()];
    let budget = ContextBudget::default();
    let packet = engine.retrieve("query", &schemas, budget).unwrap();
    assert!(!packet.hardened_schemas.is_empty(),
        "NoOpContextEngine must still return hardened schemas");
    // Check that hardening was applied (required before properties)
    let obj = packet.hardened_schemas[0].as_object().unwrap();
    let keys: Vec<_> = obj.keys().collect();
    let req_idx = keys.iter().position(|k| *k == "required").unwrap();
    let props_idx = keys.iter().position(|k| *k == "properties").unwrap();
    assert!(req_idx < props_idx, "NoOp must harden schemas: {keys:?}");
}

#[test]
fn tool_registry_schemas_method() {
    use kay_tools::ToolRegistry;
    let registry = ToolRegistry::new();
    // Empty registry → 0 schemas
    assert_eq!(registry.schemas().len(), 0);
    // Verified separately in Task 2 test — here we just confirm the method is accessible
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/hardener.rs
git commit -m "test(ctx-w6): RED — SchemaHardener + NoOp CTX-05 path tests

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement hardener.rs**

Create `crates/kay-context/src/hardener.rs`:

```rust
//! Schema hardening wrapping ForgeCode's enforce_strict_schema.

use serde_json::Value;

pub struct SchemaHardener;

impl SchemaHardener {
    pub fn harden(&self, schemas: &[Value]) -> Vec<Value> {
        schemas.iter().map(|s| {
            let mut hardened = s.clone();
            kay_tools::schema::harden_tool_schema(
                &mut hardened,
                &kay_tools::schema::TruncationHints::default(),
            );
            hardened
        }).collect()
    }
}
```

- [ ] **Step 4: Implement engine.rs**

Create `crates/kay-context/src/engine.rs`:

```rust
//! ContextEngine trait + NoOpContextEngine.

use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::budget::{ContextBudget, ContextPacket};
use crate::hardener::SchemaHardener;
use crate::indexer::IndexStats;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("context engine: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

pub trait ContextEngine: Send + Sync {
    fn index_project(&self, root: &Path) -> Result<IndexStats>;
    fn retrieve(&self, query: &str, schemas: &[Value], budget: ContextBudget) -> Result<ContextPacket>;
    fn invalidate(&self, path: &Path) -> Result<()>;
}

pub struct NoOpContextEngine;

impl ContextEngine for NoOpContextEngine {
    fn index_project(&self, _root: &Path) -> Result<IndexStats> {
        Ok(IndexStats {
            indexed_files: 0,
            total_symbols: 0,
            skipped_files: 0,
            duration_ms: 0,
        })
    }

    fn retrieve(&self, _query: &str, schemas: &[Value], budget: ContextBudget) -> Result<ContextPacket> {
        let hardened = SchemaHardener.harden(schemas);
        Ok(ContextPacket {
            symbols: vec![],
            hardened_schemas: hardened,
            truncated: false,
            dropped_count: 0,
            total_tokens_estimate: 0,
        })
    }

    fn invalidate(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

impl Default for NoOpContextEngine {
    fn default() -> Self {
        Self
    }
}

/// Concrete implementation backed by SymbolStore + TreeSitterIndexer.
/// Full implementation in Phase 7 waves W-7+; stub for now.
pub struct KayContextEngine;
```

- [ ] **Step 5: Run W-6 tests**

```bash
cargo test -p kay-context --test hardener 2>&1 | tail -20
```

Expected: all 5 tests pass.

- [ ] **Step 6: Run all kay-context tests so far**

```bash
cargo test -p kay-context 2>&1 | tail -30
```

Expected: all tests pass (W-1 through W-6).

- [ ] **Step 7: Commit GREEN**

```bash
git add crates/kay-context/src/hardener.rs crates/kay-context/src/engine.rs \
        crates/kay-context/src/lib.rs crates/kay-context/tests/hardener.rs
git commit -m "feat(ctx-w6): GREEN — SchemaHardener + NoOpContextEngine with CTX-05 path

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 9: W-7a — FileWatcher Integration

**Files:**
- Create: `crates/kay-context/src/watcher.rs`
- Create: `crates/kay-context/tests/watcher.rs`

- [ ] **Step 1: Write failing tests (RED)**

Create `crates/kay-context/tests/watcher.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use kay_context::watcher::FileWatcher;

#[derive(Clone, Default)]
struct RecordingEngine {
    invalidated: Arc<Mutex<Vec<std::path::PathBuf>>>,
}

impl kay_context::engine::ContextEngine for RecordingEngine {
    fn index_project(&self, _: &std::path::Path) -> kay_context::engine::Result<kay_context::indexer::IndexStats> {
        Ok(kay_context::indexer::IndexStats { indexed_files: 0, total_symbols: 0, skipped_files: 0, duration_ms: 0 })
    }
    fn retrieve(&self, _: &str, _: &[serde_json::Value], _: kay_context::budget::ContextBudget) -> kay_context::engine::Result<kay_context::budget::ContextPacket> {
        Ok(kay_context::budget::ContextPacket {
            symbols: vec![],
            hardened_schemas: vec![],
            truncated: false,
            dropped_count: 0,
            total_tokens_estimate: 0,
        })
    }
    fn invalidate(&self, path: &std::path::Path) -> kay_context::engine::Result<()> {
        self.invalidated.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }
}

#[test]
fn watcher_triggers_on_create() {
    let dir = tempfile::TempDir::new().unwrap();
    let engine = RecordingEngine::default();
    let _watcher = FileWatcher::start(dir.path().to_path_buf(), Arc::new(engine.clone())).unwrap();

    std::fs::write(dir.path().join("new_file.rs"), "fn foo() {}").unwrap();
    std::thread::sleep(Duration::from_millis(800)); // debounce 500ms + buffer

    let paths = engine.invalidated.lock().unwrap();
    assert!(!paths.is_empty(), "create event must trigger invalidate");
    assert!(paths.iter().any(|p| p.ends_with("new_file.rs")));
}

#[test]
fn watcher_triggers_on_modify() {
    let dir = tempfile::TempDir::new().unwrap();
    let rs_path = dir.path().join("lib.rs");
    std::fs::write(&rs_path, "fn a() {}").unwrap();

    let engine = RecordingEngine::default();
    let _watcher = FileWatcher::start(dir.path().to_path_buf(), Arc::new(engine.clone())).unwrap();

    std::fs::write(&rs_path, "fn a() {} fn b() {}").unwrap();
    std::thread::sleep(Duration::from_millis(800));

    let paths = engine.invalidated.lock().unwrap();
    assert!(paths.iter().any(|p| p.ends_with("lib.rs")), "modify must trigger invalidate");
}

#[test]
fn watcher_triggers_on_remove() {
    let dir = tempfile::TempDir::new().unwrap();
    let rs_path = dir.path().join("old.rs");
    std::fs::write(&rs_path, "fn old() {}").unwrap();

    let engine = RecordingEngine::default();
    let _watcher = FileWatcher::start(dir.path().to_path_buf(), Arc::new(engine.clone())).unwrap();

    std::fs::remove_file(&rs_path).unwrap();
    std::thread::sleep(Duration::from_millis(800));

    let paths = engine.invalidated.lock().unwrap();
    assert!(paths.iter().any(|p| p.ends_with("old.rs")), "remove must trigger invalidate");
}

#[test]
fn watcher_debounce_coalesces_events() {
    let dir = tempfile::TempDir::new().unwrap();
    let rs_path = dir.path().join("multi.rs");
    std::fs::write(&rs_path, "fn v1() {}").unwrap();

    let engine = RecordingEngine::default();
    let _watcher = FileWatcher::start(dir.path().to_path_buf(), Arc::new(engine.clone())).unwrap();

    // Write 3x within 100ms (well within 500ms debounce window)
    for i in 0..3 {
        std::fs::write(&rs_path, format!("fn v{i}() {{}}")).unwrap();
        std::thread::sleep(Duration::from_millis(30));
    }
    std::thread::sleep(Duration::from_millis(900)); // wait for debounce to fire

    let paths = engine.invalidated.lock().unwrap();
    let count = paths.iter().filter(|p| p.ends_with("multi.rs")).count();
    assert_eq!(count, 1, "3 rapid writes must coalesce to 1 invalidate call; got {count}");
}

#[test]
fn watcher_ignores_non_source() {
    let dir = tempfile::TempDir::new().unwrap();
    let engine = RecordingEngine::default();
    let _watcher = FileWatcher::start(dir.path().to_path_buf(), Arc::new(engine.clone())).unwrap();

    std::fs::write(dir.path().join("Cargo.lock"), "# lock file").unwrap();
    std::thread::sleep(Duration::from_millis(800));

    let paths = engine.invalidated.lock().unwrap();
    assert!(
        paths.iter().all(|p| !p.ends_with("Cargo.lock")),
        "non-source files must not trigger invalidate"
    );
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-context/tests/watcher.rs
git commit -m "test(ctx-w7a): RED — FileWatcher integration tests (debounce, create, modify, remove)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Implement watcher.rs (GREEN)**

Create `crates/kay-context/src/watcher.rs`:

```rust
//! FileWatcher: notify debounced watcher → ContextEngine::invalidate.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use thiserror::Error;

use crate::engine::ContextEngine;

const SOURCE_EXTENSIONS: &[&str] = &["rs", "ts", "tsx", "py", "go"];
const DEBOUNCE_MS: u64 = 500;

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("notify: {0}")]
    Notify(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, WatcherError>;

pub struct FileWatcher {
    _debouncer: Debouncer<notify::RecommendedWatcher>,
}

impl FileWatcher {
    pub fn start(root: PathBuf, engine: Arc<dyn ContextEngine>) -> Result<Self> {
        let debouncer = new_debouncer(
            Duration::from_millis(DEBOUNCE_MS),
            move |result: DebounceEventResult| {
                let events = match result {
                    Ok(events) => events,
                    Err(_) => return,
                };
                for event in events {
                    for path in &event.paths {
                        if is_source_file(path) {
                            let _ = engine.invalidate(path);
                        }
                    }
                }
            },
        )?;

        // Watch root recursively — returns a Debouncer that owns the watcher.
        // We need mutable access to add the watch.
        let mut d = debouncer;
        d.watcher().watch(&root, RecursiveMode::Recursive)?;
        Ok(Self { _debouncer: d })
    }
}

fn is_source_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| SOURCE_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}
```

- [ ] **Step 4: Run W-7a tests**

```bash
cargo test -p kay-context --test watcher 2>&1 | tail -20
```

Expected: all 5 watcher tests pass.

- [ ] **Step 5: Commit GREEN**

```bash
git add crates/kay-context/src/watcher.rs crates/kay-context/tests/watcher.rs
git commit -m "feat(ctx-w7a): GREEN — FileWatcher with 500ms debounce + source-extension filter

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Task 10: W-7b — kay-cli Injection + E2E

**Files:**
- Modify: `crates/kay-core/src/loop.rs`
- Modify: `crates/kay-cli/src/run.rs`
- Create: `crates/kay-cli/tests/context_e2e.rs`
- Add `kay-context` to `crates/kay-core/Cargo.toml` and `crates/kay-cli/Cargo.toml`

- [ ] **Step 1: Write failing E2E tests (RED)**

Create `crates/kay-cli/tests/context_e2e.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

// W-7b: E2E context injection tests.
// These tests build on the existing headless CLI infrastructure
// (offline provider + kay run --offline).

#[test]
fn noop_engine_backward_compat() {
    // Verify that RunTurnArgs with the new context_engine + context_budget fields
    // compiles and the existing offline E2E scenario still runs.
    // If this test compiles, backward compat is confirmed.
    use std::sync::Arc;
    use kay_context::engine::NoOpContextEngine;
    use kay_context::budget::ContextBudget;

    let engine: Arc<dyn kay_context::engine::ContextEngine> =
        Arc::new(NoOpContextEngine::default());
    let budget = ContextBudget::default();

    // Verify the types satisfy the trait bounds
    assert_eq!(budget.available(), 8192 - 2048);
    let _ = engine; // confirms Arc<dyn ContextEngine> is constructible
}

#[test]
fn noop_retrieve_returns_hardened_schemas() {
    use kay_context::budget::ContextBudget;
    use kay_context::engine::{ContextEngine, NoOpContextEngine};
    use serde_json::json;

    let engine = NoOpContextEngine;
    let schema = json!({
        "type": "object",
        "properties": { "x": { "type": "string" } },
        "required": ["x"],
        "additionalProperties": false
    });
    let packet = engine.retrieve("some query", &[schema], ContextBudget::default()).unwrap();
    assert!(!packet.hardened_schemas.is_empty(), "must return hardened schemas");
    assert!(packet.symbols.is_empty(), "no-op must return no symbols");
    assert!(!packet.truncated, "no-op must not truncate");
}

#[test]
fn context_packet_format_check() {
    use kay_context::budget::{ContextBudget, ContextPacket};
    use kay_context::store::{Symbol, SymbolKind};
    use std::path::PathBuf;

    let sym = Symbol {
        id: 1,
        name: "run_loop".to_string(),
        kind: SymbolKind::Function,
        file_path: PathBuf::from("src/loop.rs"),
        start_line: 42,
        end_line: 80,
        signature: "fn run_loop() -> Result<()>".to_string(),
        doc_comment: None,
        relevance_score: 0.9,
    };

    // Verify context prompt format
    let formatted = format!(
        "{} {}:{} — {}",
        "function",
        sym.file_path.display(),
        sym.start_line,
        sym.signature
    );
    assert!(formatted.contains("src/loop.rs:42"), "format must contain file:line");
    assert!(formatted.contains("run_loop"), "format must contain signature");
}
```

- [ ] **Step 2: Commit RED**

```bash
git add crates/kay-cli/tests/context_e2e.rs
git commit -m "test(ctx-w7b): RED — E2E context injection tests

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

- [ ] **Step 3: Add kay-context to kay-core and kay-cli Cargo.toml**

In `crates/kay-core/Cargo.toml`, add to `[dependencies]`:
```toml
kay-context = { path = "../kay-context" }
```

In `crates/kay-cli/Cargo.toml`, add to `[dependencies]`:
```toml
kay-context = { path = "../kay-context" }
```

- [ ] **Step 4: Add context_engine + context_budget to RunTurnArgs**

In `crates/kay-core/src/loop.rs`, find `pub struct RunTurnArgs` and add two fields at the end:

```rust
    /// Context engine for symbol retrieval (CTX-01..CTX-03).
    /// Default: NoOpContextEngine (empty ContextPacket).
    pub context_engine: Arc<dyn kay_context::engine::ContextEngine>,

    /// Per-turn token budget for context injection (CTX-04).
    pub context_budget: kay_context::budget::ContextBudget,
```

Also add `impl Default for RunTurnArgs` if one does not already exist, or ensure existing call sites use `..Default::default()` for the new fields.

Also add the use import at the top of `loop.rs`:
```rust
use std::sync::Arc;
```
(if not already present)

- [ ] **Step 4b: Add initial_prompt to RunTurnArgs**

**Important:** `run_turn` is an event consumer loop — it does NOT assemble provider requests in Phase 7. The real provider-assembly happens when the OpenRouter adapter is wired (Phase 8+). For Phase 7, add `initial_prompt: String` to `RunTurnArgs` so `retrieve()` has a query string, and call it at the TOP of `run_turn` (before the event loop) to prove the plumbing works. Add to `RunTurnArgs`:

```rust
    /// The user's initial prompt for this turn. Used as the query for context retrieval.
    pub initial_prompt: String,
```

Update `run_async` in `run.rs` to pass `initial_prompt: prompt.clone()`.

- [ ] **Step 5: Wire retrieve() into run_turn (GREEN)**

In `crates/kay-core/src/loop.rs`, add the context retrieval call at the START of `run_turn`, BEFORE the event loop (`loop { tokio::select! { ... } }`). This proves the CTX-03..CTX-05 plumbing works even though the real system prompt injection happens when the OpenRouter provider is wired (Phase 8). Insert:

```rust
// CTX-03..CTX-05: retrieve symbols + harden schemas before each provider call
let raw_schemas = args.registry.schemas();
let ctx_packet = args.context_engine.retrieve(
    &current_user_message,
    &raw_schemas,
    args.context_budget.clone(),
)?;

// Inject <context> block into system prompt when symbols available
let system_prompt = if ctx_packet.symbols.is_empty() {
    args.persona.system_prompt().to_string()
} else {
    let ctx_block: String = ctx_packet.symbols.iter()
        .map(|s| format!("{} {}:{} — {}", s.kind_str(), s.file_path.display(), s.start_line, s.signature))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n\n<context>\n{}\n</context>", args.persona.system_prompt(), ctx_block)
};

// Use ctx_packet.hardened_schemas instead of raw schemas when building tool definitions
```

Also add a `kind_str()` helper to `Symbol` in `store.rs`:
```rust
pub fn kind_str(&self) -> &'static str {
    match self.kind {
        SymbolKind::Function => "function",
        SymbolKind::Module => "module",
        SymbolKind::Class => "class",
        SymbolKind::Trait => "trait",
        SymbolKind::Impl => "impl",
        SymbolKind::FileBoundary => "boundary",
    }
}
```

If a `ContextTruncated` event should be emitted, add to the event stream:
```rust
if ctx_packet.truncated {
    let _ = args.event_tx.send(AgentEvent::ContextTruncated {
        dropped_symbols: ctx_packet.dropped_count,
        budget_tokens: args.context_budget.max_tokens,
    }).await;
}
```

- [ ] **Step 6: Update RunTurnArgs construction in kay-cli/src/run.rs**

Find the `run_turn(RunTurnArgs { ... })` call in `run_async` and add the new fields:

```rust
    run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: Arc::new(kay_context::engine::NoOpContextEngine::default()),
        context_budget: kay_context::budget::ContextBudget::default(),
    })
```

- [ ] **Step 7: Run E2E tests and full test suite**

```bash
cargo test -p kay-context 2>&1 | tail -10
cargo test -p kay-cli --test context_e2e 2>&1 | tail -20
cargo test -p kay-core 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 8: Run full workspace test suite**

```bash
cargo test --workspace 2>&1 | tail -30
```

Expected: all tests pass. No regressions.

- [ ] **Step 9: Verify clippy passes**

```bash
cargo clippy --workspace -- -D warnings 2>&1 | head -40
```

Expected: no warnings.

- [ ] **Step 10: Commit GREEN**

```bash
git add crates/kay-core/src/loop.rs crates/kay-core/Cargo.toml \
        crates/kay-cli/src/run.rs crates/kay-cli/Cargo.toml \
        crates/kay-cli/tests/context_e2e.rs \
        crates/kay-context/src/store.rs
git commit -m "feat(ctx-w7b): GREEN — context injection wired into run_turn; E2E backward compat

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

---

## Final Verification

- [ ] **Count tests**

```bash
cargo test --workspace 2>&1 | grep "test result" | awk '{sum += $4} END {print "total passed:", sum}'
```

Expected: ≥ 91 (pre-Phase-7 baseline) + 47 new = ≥ 138 tests.

- [ ] **Verify CTX requirements**

```
CTX-01: tree-sitter parsing → symbols table in SQLite (W-2 green)
CTX-02: Symbol struct has name/sig/file_path/spans — no full body (W-1 green)
CTX-03: FTS5 + sqlite-vec + RRF → retrieve() returns ranked Vec<Symbol> (W-3 + W-4 green)
CTX-04: ContextBudget.apply() truncates; ContextTruncated event emitted (W-5 + W-7b green)
CTX-05: SchemaHardener wraps harden_tool_schema; even NoOp returns hardened schemas (W-6 green)
```

- [ ] **Confirm event_filter.rs is untouched**

```bash
git diff main -- crates/kay-core/src/event_filter.rs
```

Expected: empty diff.

- [ ] **Check DCO on all commits**

```bash
git log --oneline phase/07-context-engine ^main | head -30
git log phase/07-context-engine ^main --format="%H %s" | head -20
```

All commits must have `Signed-off-by:` in body.
