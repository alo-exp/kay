# Phase 7: Context Engine — Pattern Map

**Mapped:** 2026-04-22
**Files analyzed:** 14 (10 new + 4 modified)
**Analogs found:** 14 / 14

---

## File Classification

| New / Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------------|------|-----------|----------------|---------------|
| `crates/kay-context/src/lib.rs` | module root / re-export | — | `crates/kay-session/src/lib.rs` | exact |
| `crates/kay-context/src/store.rs` | service / persistence | CRUD | `crates/kay-session/src/store.rs` | exact |
| `crates/kay-context/src/indexer.rs` | service / transform | batch | `crates/kay-session/src/transcript.rs` (writer pattern) | role-match |
| `crates/kay-context/src/language.rs` | utility / enum | transform | `crates/kay-tools/src/seams/verifier.rs` (owned enum + NoOp) | role-match |
| `crates/kay-context/src/retriever.rs` | utility / pure fn | transform | `crates/kay-tools/src/schema.rs` (pure fn, no state) | role-match |
| `crates/kay-context/src/budget.rs` | service / model | CRUD | `crates/kay-session/src/snapshot.rs` (SessConfig) | role-match |
| `crates/kay-context/src/hardener.rs` | utility / wrapper | transform | `crates/kay-tools/src/schema.rs` (`harden_tool_schema` wrapper) | exact |
| `crates/kay-context/src/watcher.rs` | service / event-driven | event-driven | `crates/kay-tools/src/seams/sandbox.rs` (trait + NoOp) | role-match |
| `crates/kay-context/src/embedder.rs` | seam / trait + NoOp | request-response | `crates/kay-tools/src/seams/verifier.rs` | exact |
| `crates/kay-context/src/engine.rs` | seam / trait + NoOp | request-response | `crates/kay-tools/src/seams/verifier.rs` | exact |
| `crates/kay-tools/src/events.rs` | enum extension | — | same file (existing variant shapes) | exact |
| `crates/kay-tools/src/events_wire.rs` | serialization extension | — | same file (existing match arms) | exact |
| `crates/kay-tools/src/registry.rs` | method addition | — | same file (`tool_definitions()` pattern) | exact |
| `crates/kay-core/src/loop.rs` | struct extension | — | same file (`RunTurnArgs` struct) | exact |

---

## Pattern Assignments

### `crates/kay-context/src/lib.rs` (module root, re-exports)

**Analog:** `crates/kay-session/src/lib.rs`

**Module-level doc + lint gate** (lines 1–4):
```rust
//! kay-context — Context engine: symbol store, indexer, retriever, budget,
//! and schema hardening (Phase 7: CTX-01..CTX-05).
//!
//! See .planning/phases/07-context-engine/07-CONTEXT.md for decisions DL-1..DL-15.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

**Module declarations** — `pub mod` for public surfaces, unqualified `mod` for internals:
```rust
pub mod budget;
pub mod embedder;
pub mod engine;
pub mod error;
pub mod hardener;
pub mod indexer;
pub mod language;
pub mod retriever;
pub mod store;
pub mod watcher;
```

**Flat re-export style** — every public type surfaced at crate root:
```rust
pub use budget::{ContextBudget, ContextPacket};
pub use embedder::{EmbeddingProvider, NoOpEmbedder};
pub use engine::{ContextEngine, KayContextEngine, NoOpContextEngine};
pub use error::ContextError;
pub use hardener::SchemaHardener;
pub use indexer::{IndexStats, TreeSitterIndexer};
pub use language::Language;
pub use retriever::Retriever;
pub use store::SymbolStore;
pub use watcher::FileWatcher;
```

**Key convention:** `kay-session/src/lib.rs` uses `pub use` for every exported name with the module path spelled out explicitly — no glob re-exports (`pub use module::*`).

---

### `crates/kay-context/src/store.rs` (service, CRUD, SQLite)

**Analog:** `crates/kay-session/src/store.rs` (lines 1–109) — the single closest analog; follow it precisely.

**Imports** (analog lines 1–3):
```rust
use crate::error::ContextError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
```

**Struct shape** — `pub conn`, `pub` path field, manual `Debug` impl hiding the connection:
```rust
pub struct SymbolStore {
    pub conn: Connection,
    pub db_path: PathBuf,
}

impl std::fmt::Debug for SymbolStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolStore")
            .field("db_path", &self.db_path)
            .finish_non_exhaustive()
    }
}
```

**`open()` pattern** (analog lines 27–43) — `create_dir_all` → `Connection::open` → `execute_batch` PRAGMAs → `init_schema`:
```rust
pub fn open(root: &Path) -> Result<Self, ContextError> {
    std::fs::create_dir_all(root)?;
    let db_path = root.join("context.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 30000;
        PRAGMA foreign_keys = ON;
    ",
    )?;

    Self::init_schema(&conn)?;

    Ok(Self { conn, db_path: db_path.clone() })
}
```

**`init_schema()` pattern** (analog lines 46–98) — `CREATE TABLE IF NOT EXISTS` + `schema_version` guard:
```rust
fn init_schema(conn: &Connection) -> Result<(), ContextError> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS schema_version ( version INTEGER NOT NULL );
        CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(...);
        CREATE TABLE IF NOT EXISTS symbols ( ... );
        ...
    ")?;

    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;

    if count == 0 {
        conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
    } else {
        let version: i64 =
            conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;
        if version != 1 {
            return Err(ContextError::SchemaVersionMismatch { found: version as u32, expected: 1 });
        }
    }

    Ok(())
}
```

**CRUD method style** — `pub fn`, `&self` or `&mut self`, `Result<T, ContextError>`:
```rust
pub fn upsert_symbol(&self, ...) -> Result<(), ContextError> { ... }
pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Symbol>, ContextError> { ... }
```

**Test isolation pattern** (from session tests):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::TempDir;
    use super::*;

    fn open_temp() -> (SymbolStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = SymbolStore::open(dir.path()).unwrap();
        (store, dir)
    }

    #[test]
    fn schema_version_is_1() {
        let (store, _dir) = open_temp();
        let v: i64 = store.conn.query_row(
            "SELECT version FROM schema_version", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(v, 1);
    }
}
```

---

### `crates/kay-context/src/error.rs` (implied by store.rs and all other modules)

**Analog:** `crates/kay-session/src/error.rs` (lines 1–36)

**Error enum pattern** — `thiserror`, `#[non_exhaustive]`, named struct variants with `#[from]` transparent wrappers at the bottom:
```rust
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ContextError {
    #[error("schema version mismatch: found {found}, expected {expected}")]
    SchemaVersionMismatch { found: u32, expected: u32 },

    #[error("symbol not found: {id}")]
    SymbolNotFound { id: i64 },

    // Transparent wrappers at the end:
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

**Key convention:** `thiserror` (NOT `anyhow`) is used for library crate errors. `anyhow` appears only in binary / integration code (`kay-cli`). `ToolError` in `kay-tools/src/error.rs` uses `anyhow::Error` as a `#[source]` field in one variant — that is the only exception, and it is in the `ExecutionFailed` variant only, not as the crate's own error type.

---

### `crates/kay-context/src/indexer.rs` (service, batch transform)

**Analog:** `crates/kay-session/src/snapshot.rs` (writer pattern) + `crates/kay-tools/src/seams/verifier.rs` (struct with impl)

**Key patterns:**
- Plain `struct TreeSitterIndexer { ... }` — no trait required at this layer
- Async method via `async fn index_file(&self, path: &Path, store: &SymbolStore) -> Result<IndexStats, ContextError>`
- `#[deny(clippy::unwrap_used)]` means use `?` propagation everywhere — no `.unwrap()` / `.expect()` in non-test code
- Associated `struct IndexStats { pub files: usize, pub symbols: usize }` with `Default` derived
- Test coverage via `#[cfg(test)]` inline module with `tempfile::TempDir`

**Signature truncation** (DL-11 — from `crates/kay-tools/src/schema.rs` truncation pattern):
```rust
// In Symbol::new() — truncate at 256 chars, append ellipsis
const SIG_MAX: usize = 256;
if sig.chars().count() > SIG_MAX {
    let truncated: String = sig.chars().take(SIG_MAX).collect();
    format!("{truncated}\u{2026}")   // U+2026 HORIZONTAL ELLIPSIS
} else {
    sig.to_string()
}
```

---

### `crates/kay-context/src/language.rs` (utility, enum)

**Analog:** `crates/kay-tools/src/seams/verifier.rs` (owned enum + NoOp pattern) and `ToolOutputChunk` in `crates/kay-tools/src/events.rs`

**Pattern:** Plain `#[non_exhaustive]` enum, no trait required, `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`:
```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
```

**Key convention:** `#[non_exhaustive]` on all public enums — established in `AgentEvent`, `ToolOutputChunk`, `VerificationOutcome`, `LoopError`, `SessionError`, `ToolError`. Phase 7 enums follow suit.

---

### `crates/kay-context/src/retriever.rs` (utility, pure functions)

**Analog:** `crates/kay-tools/src/schema.rs` — pure free functions, no struct state, tests inline.

**Pattern:** Free functions, no `impl` block, direct `Result` returns:
```rust
/// RRF merge score: 1 / (k + rank), k = 60 (DL-10).
pub fn rrf_score(rank: usize) -> f64 {
    1.0 / (60.0 + rank as f64)
}

/// Apply name-bonus of +0.5 when query term exactly matches symbol name (DL-10).
pub fn apply_name_bonus(score: f64, symbol_name: &str, query: &str) -> f64 {
    if symbol_name == query { score + 0.5 } else { score }
}

pub fn rrf_merge(
    fts_results: Vec<Symbol>,
    ann_results: Vec<Symbol>,
    query: &str,
) -> Vec<Symbol> { ... }
```

**Test pattern** (from `crates/kay-tools/src/schema.rs` lines 59–217):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn rrf_score_at_rank_0_is_one_over_sixty() {
        let score = rrf_score(0);
        assert!((score - 1.0 / 60.0).abs() < f64::EPSILON);
    }
}
```

---

### `crates/kay-context/src/budget.rs` (service, model + CRUD)

**Analog:** `crates/kay-session/src/config.rs` (config struct with `Default`) + `crates/kay-tools/src/quota.rs` (budget tracking struct)

**Pattern:** Plain struct with `new()` constructor and `Default` impl pointing at `new(8192, 1024)`:
```rust
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
```

**Token estimate function** — free function, not a method (DL-7):
```rust
/// Estimate token count: (name.chars().count() + sig.chars().count() + 10) / 4.
pub fn estimate_tokens(name: &str, sig: &str) -> usize {
    (name.chars().count() + sig.chars().count() + 10) / 4
}
```

**`ContextPacket`** — plain struct with `Default`:
```rust
#[derive(Debug, Default, Clone)]
pub struct ContextPacket {
    pub symbols: Vec<Symbol>,
    pub dropped_symbols: usize,
    pub budget_tokens: usize,
}
```

---

### `crates/kay-context/src/hardener.rs` (utility, thin wrapper)

**Analog:** `crates/kay-tools/src/schema.rs` — it wraps `harden_tool_schema()` directly.

**Key convention** (DL-14): `SchemaHardener::harden()` must call `kay_tools::harden_tool_schema()`, not re-implement it. From `schema.rs` lines 36–57, the signature is:
```rust
// In kay-tools/src/schema.rs:
pub fn harden_tool_schema(schema: &mut serde_json::Value, hints: &TruncationHints)
```

**Wrapper pattern:**
```rust
use kay_tools::{TruncationHints, harden_tool_schema};
use serde_json::Value;

pub struct SchemaHardener {
    hints: TruncationHints,
}

impl SchemaHardener {
    pub fn new(hints: TruncationHints) -> Self { Self { hints } }

    pub fn harden(&self, schema: &mut Value) {
        harden_tool_schema(schema, &self.hints);
    }

    pub fn harden_all(&self, schemas: &mut Vec<Value>) {
        for s in schemas.iter_mut() {
            self.harden(s);
        }
    }
}

impl Default for SchemaHardener {
    fn default() -> Self { Self::new(TruncationHints::default()) }
}
```

---

### `crates/kay-context/src/watcher.rs` (service, event-driven)

**Analog:** `crates/kay-tools/src/seams/sandbox.rs` (trait + NoOp struct pattern)

**Pattern:** Plain struct wrapping the `notify` debounced watcher; no trait required at this layer (the trait lives in `engine.rs`):
```rust
use std::path::Path;

pub struct FileWatcher {
    // internal: notify debounced watcher handle
}

impl FileWatcher {
    pub fn new(root: &Path) -> Result<Self, ContextError> { ... }
    pub fn stop(self) { ... }
}
```

**Key convention:** `notify = "6.1"` + `notify-debouncer-mini = "0.4"`, 500ms debounce (DL-8). Ignored paths: `*.lock`, `target/`, `.git/`, `*.tmp`, `*.swp`. Watched extensions: `.rs`, `.ts`, `.tsx`, `.py`, `.go`.

---

### `crates/kay-context/src/embedder.rs` (seam, trait + NoOp)

**Analog:** `crates/kay-tools/src/seams/verifier.rs` (lines 1–78) — the exact template.

**Pattern:** `async_trait` trait + production NoOp + `#[cfg(test)]` fake:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ContextError>;
}

pub struct NoOpEmbedder;

#[async_trait]
impl EmbeddingProvider for NoOpEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ContextError> {
        Ok(vec![])
    }
}

/// Deterministic test double — returns zeros of length `dimensions`.
/// `#[cfg(test)]` only — NOT part of the production API.
#[cfg(test)]
pub struct FakeEmbedder {
    pub dimensions: usize,
}

#[cfg(test)]
#[async_trait]
impl EmbeddingProvider for FakeEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ContextError> {
        Ok(vec![0.0f32; self.dimensions])
    }
}
```

**Key convention from `verifier.rs` lines 19–23:**
```rust
#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    async fn verify(&self, task_summary: &str) -> VerificationOutcome;
}
```
Both `Send + Sync` bounds are always present on DI-seam traits. The `#[async_trait]` crate is in `workspace.dependencies` (`async-trait = "0.1"`).

---

### `crates/kay-context/src/engine.rs` (seam, trait + NoOp + stub)

**Analog:** `crates/kay-tools/src/seams/verifier.rs` (lines 1–78) — same triple pattern: trait + NoOp + tests.

**Pattern:**
```rust
use std::sync::Arc;
use async_trait::async_trait;
use crate::budget::{ContextBudget, ContextPacket};

/// Phase 8+ implementors: if session.title is injected into the system prompt,
/// it MUST be delimited as [USER_DATA: session_title] per Phase 6 DL-7.
#[async_trait]
pub trait ContextEngine: Send + Sync {
    async fn retrieve(
        &self,
        prompt: &str,
        schemas: &[serde_json::Value],
    ) -> Result<ContextPacket, crate::error::ContextError>;
}

pub struct NoOpContextEngine;

#[async_trait]
impl ContextEngine for NoOpContextEngine {
    async fn retrieve(
        &self,
        _prompt: &str,
        _schemas: &[serde_json::Value],
    ) -> Result<ContextPacket, crate::error::ContextError> {
        Ok(ContextPacket::default())
    }
}

impl Default for NoOpContextEngine {
    fn default() -> Self { Self }
}

/// Phase 7 stub — wired in Phase 8+ with real symbol retrieval.
pub struct KayContextEngine {
    pub store: Arc<crate::store::SymbolStore>,
    pub budget: ContextBudget,
}
```

**Test pattern** (mirroring `verifier.rs` lines 38–78):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_engine_returns_empty_packet() {
        let engine = NoOpContextEngine;
        let packet = engine.retrieve("query", &[]).await.unwrap();
        assert!(packet.symbols.is_empty());
        assert_eq!(packet.dropped_symbols, 0);
    }
}
```

---

### `crates/kay-tools/src/events.rs` — Adding 2 new `AgentEvent` variants (DL-12)

**Analog:** Existing variants in the same file, specifically the Phase 5 additive block (lines 128–151).

**Insertion point:** After the `Aborted` variant (line 151), before the closing `}`. Add a new phase comment block:

```rust
    // -- Phase 7 additive variants (DL-12 — context engine signals) ----------
    /// Emitted by ContextBudget when context assembly drops symbols to fit
    /// the token budget. Consumers may surface this as a "context truncated"
    /// warning in the UI. `dropped_symbols` is the count of symbols that
    /// did not fit; `budget_tokens` is the configured available-token ceiling.
    ContextTruncated {
        dropped_symbols: usize,
        budget_tokens: usize,
    },

    /// Emitted by TreeSitterIndexer during incremental re-index. `indexed`
    /// is the count of files processed so far; `total` is the total file
    /// count in the watch scope. Consumers may use this to render a progress
    /// bar. Final emission has `indexed == total`.
    IndexProgress {
        indexed: usize,
        total: usize,
    },
```

**Key conventions from existing variants:**
- Field names use `snake_case` (matching all existing fields: `call_id`, `tool_name`, `dropped_symbols`)
- Field types: `usize` for counts, `String` for identifiers, `bool` for flags — no exotic types in leaf-variant fields
- Doc comment before each variant explains: what emits it, when, what the fields mean
- `#[non_exhaustive]` is on the enum (line 27), not on individual variants — DO NOT add it to variants
- `AgentEvent` does NOT derive `Clone` or `Serialize` — do not add those derives

**Unit test** to add at the end of the `phase3_additions` test module (or a new `phase7_additions` module):
```rust
#[test]
fn context_truncated_variant_shape() {
    let ev = AgentEvent::ContextTruncated { dropped_symbols: 3, budget_tokens: 7168 };
    let dbg = format!("{ev:?}");
    assert!(dbg.contains("ContextTruncated"), "{dbg}");
    assert!(dbg.contains("3"), "{dbg}");
    assert!(dbg.contains("7168"), "{dbg}");
}

#[test]
fn index_progress_variant_shape() {
    let ev = AgentEvent::IndexProgress { indexed: 10, total: 100 };
    let dbg = format!("{ev:?}");
    assert!(dbg.contains("IndexProgress"), "{dbg}");
    assert!(dbg.contains("10"), "{dbg}");
    assert!(dbg.contains("100"), "{dbg}");
}
```

---

### `crates/kay-tools/src/events_wire.rs` — Adding 2 new match arms (DL-12)

**Analog:** Existing match arms in the same file — specifically the simple struct-variant arms.

**Insertion point:** Inside `impl<'a> Serialize for AgentEventWire<'a>`, after the `AgentEvent::Aborted` arm (lines 169–174), before the closing `}` of the match.

**Exact pattern to follow** — copy the `Aborted` arm shape (2-field struct variant):
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

**Critical conventions from the module header and existing arms:**
- `type` tag uses `snake_case` of the variant name (lines 37–38 of module doc)
- Map size = number of fields including `"type"` (so 3 for 2-field variants)
- Field name in JSON matches the Rust field name exactly (`dropped_symbols` → `"dropped_symbols"`)
- NO new helper newtypes needed — `usize` fields serialize natively via `serde`
- The compiler enforces exhaustiveness: adding a variant without a match arm is a compile error

**Snapshot tests** to add to `crates/kay-tools/tests/events_wire_snapshots.rs`:
```rust
#[test]
fn snap_context_truncated_wire() {
    let ev = AgentEvent::ContextTruncated { dropped_symbols: 3, budget_tokens: 7168 };
    insta::assert_json_snapshot!(wire_value(&ev));
}

#[test]
fn snap_index_progress_wire() {
    let ev = AgentEvent::IndexProgress { indexed: 10, total: 100 };
    insta::assert_json_snapshot!(wire_value(&ev));
}
```

---

### `crates/kay-tools/src/registry.rs` — Adding `schemas()` method (DL-13)

**Analog:** Existing `tool_definitions()` method in the same file (lines 39–53).

**Insertion point:** Inside `impl ToolRegistry`, after `tool_definitions()` and before `len()` (line 55).

**Exact method to add** (from DL-13):
```rust
    /// Return the raw `input_schema()` JSON Value for each registered tool.
    /// Consumed by `ContextEngine::retrieve` (Phase 7 DL-13) so the context
    /// engine can apply `SchemaHardener` to the schemas in-context.
    /// Iteration order is not stable (HashMap) — callers must not rely on order.
    pub fn schemas(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|t| t.input_schema()).collect()
    }
```

**Key conventions from `tool_definitions()` (lines 39–53):**
- `&self` receiver (not `&mut self`) — registry is immutable for reads
- Return type is owned `Vec<T>` — callers own the result, no borrowing from registry
- One-liner `.values().map(|t| ...).collect()` — no intermediate `let` bindings
- Doc comment explains purpose and notes the HashMap ordering caveat (same caveat as `tool_definitions`)

**Test to add** inside the existing `#[cfg(test)] mod tests` block in `registry.rs`:
```rust
    #[test]
    fn schemas_returns_one_per_tool() {
        let mut r = ToolRegistry::new();
        r.register(dummy("alpha"));
        r.register(dummy("beta"));
        let schemas = r.schemas();
        assert_eq!(schemas.len(), 2);
        for s in &schemas {
            assert!(s.get("type").is_some() || s.get("properties").is_some(),
                "schema must be a JSON object: {s}");
        }
    }
```

---

### `crates/kay-core/src/loop.rs` — Adding 3 fields to `RunTurnArgs` (DL-9)

**Analog:** Existing fields in `RunTurnArgs` struct (lines 107–142 of the same file).

**Field ordering convention** (from the struct doc, lines 98–106): fields mirror the priority ordering of the `select!` — control first, model last, then execution surface. The 3 new fields are context-injection surface, grouped after `tool_ctx`:

**Insertion point:** After `pub tool_ctx: ToolCallContext` (line 141), before the closing `}` of the struct:

```rust
    /// Context engine consulted at turn start (before the event loop).
    /// `NoOpContextEngine` is the default until Phase 8 wires real retrieval.
    /// `Arc` so callers can share a single engine across multiple turns.
    pub context_engine: Arc<dyn kay_context::engine::ContextEngine>,

    /// Token budget for context assembly this turn.
    /// `ContextBudget::default()` = 8192 max / 1024 reserve (DL-7).
    pub context_budget: kay_context::budget::ContextBudget,

    /// The user's prompt for this turn. Passed to `context_engine.retrieve()`
    /// so the retrieval can bias toward symbols relevant to the current task.
    pub initial_prompt: String,
```

**Call-site pattern at top of `run_turn()`** (DL-9 — before the `loop { tokio::select! {...} }` block):
```rust
    // Context retrieval at turn start (Phase 7 DL-9).
    // _ctx_packet unused in Phase 7 — Phase 8+ injects it into the
    // OpenRouter request. #[allow(unused)] acceptable here per DL-9.
    #[allow(unused)]
    let _ctx_packet = args.context_engine
        .retrieve(&args.initial_prompt, &args.registry.schemas())
        .await
        .unwrap_or_default();
```

**`unwrap_or_default()` rationale:** `ContextEngine::retrieve` returns `Result<ContextPacket, ContextError>`. Since Phase 7 only calls `NoOpContextEngine` (which never errors), `.unwrap_or_default()` is safe AND avoids a `?` that would require `LoopError` to gain a `Context(ContextError)` variant before Phase 8 is ready. This is explicitly called out in DL-9.

**Import addition** to `use` block at top of `loop.rs`:
```rust
use std::sync::Arc;   // already present (line 83)
// Add:
use kay_context::engine::ContextEngine;
use kay_context::budget::ContextBudget;
```

**`kay-cli/src/run.rs` update** — pass new fields when constructing `RunTurnArgs` (line 334 in current file):
```rust
    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        // Phase 7 additions (DL-9):
        context_engine: Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: prompt.clone(),   // prompt is moved into offline_provider above;
                                          // save a clone before the spawn
    }));
```

Note: `prompt` is moved into `offline_provider(prompt, model_tx)` at line 304. Save `initial_prompt` before that move.

---

## Shared Patterns

### Error handling — `thiserror` in library crates
**Source:** `crates/kay-session/src/error.rs` + `crates/kay-tools/src/error.rs`
**Apply to:** `crates/kay-context/src/error.rs` and all modules that return `Result`

- Library crates (`kay-context`, `kay-session`, `kay-tools`) use `thiserror` for typed error enums
- `anyhow` appears only in binaries (`kay-cli`) and in `ToolError::ExecutionFailed { source: anyhow::Error }` (one-off for opaque tool execution failures)
- Always `#[non_exhaustive]` on public error enums
- `#[from]` transparent wrappers for `std::io::Error`, `rusqlite::Error`, `serde_json::Error` at the bottom of the enum

### `#[deny(clippy::unwrap_used, clippy::expect_used)]` lint gate
**Source:** `crates/kay-session/src/lib.rs` line 4, `crates/kay-tools/src/lib.rs` line 8
**Apply to:** `crates/kay-context/src/lib.rs`

Every `lib.rs` in the kay-* crates carries this deny gate. Test modules always carry `#[allow(clippy::unwrap_used, clippy::expect_used)]` to counterbalance.

### `#[non_exhaustive]` on public enums
**Source:** `crates/kay-tools/src/events.rs` line 27, `crates/kay-tools/src/error.rs` line 8, `crates/kay-session/src/error.rs` line 4
**Apply to:** All new `pub enum` types in `kay-context`

Every publicly-exported enum in the kay-* crates carries `#[non_exhaustive]`. This is load-bearing for additive evolution without breaking downstream match exhaustiveness.

### `async_trait` for DI seam traits
**Source:** `crates/kay-tools/src/seams/verifier.rs` lines 19–24, `crates/kay-tools/src/seams/sandbox.rs` lines 13–21
**Apply to:** `ContextEngine` trait in `engine.rs`, `EmbeddingProvider` trait in `embedder.rs`

Trait methods that are `async` use `#[async_trait::async_trait]` (via the `async-trait = "0.1"` workspace dep). Both `Send + Sync` bounds always present on DI seam traits.

### Test isolation with `tempfile::TempDir`
**Source:** `crates/kay-session` tests (SQLite isolation pattern implied by `tempfile` dev-dep)
**Apply to:** All `store.rs` tests in `kay-context`

```rust
fn open_temp() -> (SymbolStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = SymbolStore::open(dir.path()).unwrap();
    (store, dir)
}
```
`TempDir` must be returned and held for the lifetime of the test — dropping it early deletes the directory.

### Insta snapshot tests for wire serialization
**Source:** `crates/kay-tools/tests/events_wire_snapshots.rs` lines 1–60
**Apply to:** New snapshot tests for `ContextTruncated` and `IndexProgress` wire forms

Pattern: `wire_value(&ev)` helper → `insta::assert_json_snapshot!(...)`. Accept snapshots with `cargo insta accept` after first RED run. Snapshot files land in `crates/kay-tools/tests/snapshots/`.

### `Arc<dyn Trait>` for DI seam injection
**Source:** `crates/kay-core/src/loop.rs` lines 133–141 (`registry: Arc<ToolRegistry>`, `tool_ctx: ToolCallContext`)
**Apply to:** `context_engine: Arc<dyn ContextEngine>` field in `RunTurnArgs`

The `Arc` pattern avoids ownership conflicts when the same engine needs to be shared across multiple turns. `ContextEngine: Send + Sync` (required by the `async_trait` bounds) makes `Arc<dyn ContextEngine>` `Send`.

---

## No Analog Found

All 14 files have analogs in the existing codebase. No entries in this section.

---

## Metadata

**Analog search scope:** `crates/kay-session/src/`, `crates/kay-tools/src/`, `crates/kay-core/src/`, `crates/kay-cli/src/`
**Files scanned:** 18 source files read in full
**Pattern extraction date:** 2026-04-22
