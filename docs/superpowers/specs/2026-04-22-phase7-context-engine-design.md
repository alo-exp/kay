# Phase 7: Context Engine — Design Spec

**Date:** 2026-04-22  
**Feature:** CTX-01..CTX-05 — local tree-sitter symbol store + SQLite + sqlite-vec hybrid retrieval  
**Branch:** phase/07-context-engine  
**Status:** approved (autonomous mode §10e)  
**Rev:** r3 — NEW-1..NEW-4 from second spec review resolved

---

## 1. Overview

Kay's agent loop currently sends prompts with no structured code context — prompts are built from raw tool registry schemas and the user's message only. Phase 7 adds a local context engine that extracts function signatures and module boundaries from the project via tree-sitter, retrieves relevant symbols per turn via hybrid search, enforces an explicit per-turn token budget, and ensures all tool schemas are hardened before injection into the provider payload.

**This replaces ForgeCode's gRPC cloud workspace engine** (`forge_repo/src/context_engine.rs`) with a fully local, offline-capable implementation requiring no external service.

---

## 2. Architecture

### 2.1 New Crate: `kay-context`

Add `"crates/kay-context"` to `[workspace.members]` in root `Cargo.toml` (explicit list, line 6–42).

Directory layout:
```
crates/kay-context/
├── Cargo.toml
└── src/
    ├── lib.rs          — public re-exports + ContextEngine trait
    ├── store.rs        — SymbolStore (rusqlite + FTS5 + sqlite-vec)
    ├── indexer.rs      — tree-sitter parsing → Symbol extraction
    ├── language.rs     — Language enum + extension detection
    ├── retriever.rs    — Hybrid FTS5 + sqlite-vec → RRF merge
    ├── budget.rs       — ContextBudget + explicit truncation
    ├── hardener.rs     — SchemaHardener (wraps harden_tool_schema)
    ├── watcher.rs      — FileWatcher (notify debounced)
    ├── embedder.rs     — EmbeddingProvider trait + NoOpEmbedder + FakeEmbedder (test)
    └── engine.rs       — KayContextEngine (concrete impl of ContextEngine)
```

### 2.2 Pipeline

```
project root
     │
     ▼
FileWatcher (notify) ──invalidate(path)──▶ SymbolStore::re_index(path)
                                                    │
                              Indexer (tree-sitter) │
                              Rust / TS / Py / Go   │
                              + Unknown fallback     │
                                                    ▼
                                          SQLite: symbols + FTS5 + sqlite-vec (optional)
                                                    │
per-turn retrieve(query, schemas, budget)           │
     │                                             ▼
     │              ┌──────────────── Retriever ──────────────────┐
     │              │  FTS5 structural (always)                   │
     └──────────────│  sqlite-vec ANN (when embeddings present)   │
                    │  RRF merge → sorted Vec<Symbol>             │
                    └─────────────────────────────────────────────┘
                                          │
                                   ContextBudget
                              (sort by score, truncate)
                              → AgentEvent::ContextTruncated if hit
                                          │
                                   SchemaHardener
                              (harden all tool schemas)
                                          │
                                   ContextPacket
                                          │
                                   run_turn() injection
                        ┌─────────────────────────────────┐
                        │ system prompt: <context>         │
                        │   {symbol signatures}            │
                        │ </context>                       │
                        │ tool schemas: hardened           │
                        └─────────────────────────────────┘
```

---

## 3. Data Model

### 3.1 SQLite Schema v1

Database location: `~/.kay/symbols/<sha256_of_canonical_project_root_first8chars>.db`

```sql
CREATE TABLE IF NOT EXISTS symbols (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK(kind IN ('function','module','class','trait','impl','boundary')),
    file_path   TEXT NOT NULL,
    start_line  INTEGER NOT NULL,
    end_line    INTEGER NOT NULL,
    signature   TEXT NOT NULL,     -- one-line; NEVER full body; max 256 chars
    doc_comment TEXT
);

CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);

-- FTS5 content table — synchronization via triggers (see §3.2 below)
CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
    name, signature, doc_comment,
    content=symbols, content_rowid=id
);

CREATE TABLE IF NOT EXISTS index_state (
    file_path   TEXT PRIMARY KEY,
    mtime_secs  INTEGER NOT NULL,
    file_hash   TEXT NOT NULL,   -- SHA-256 hex (first 16 chars for compactness)
    indexed_at  INTEGER NOT NULL -- unix epoch seconds
);
```

### 3.2 FTS5 Synchronization Triggers

FTS5 content tables require explicit triggers to stay in sync with the source table. Create these triggers alongside the schema:

```sql
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
```

The `rebuild` command (`INSERT INTO symbols_fts(symbols_fts) VALUES('rebuild')`) is reserved for database recovery scenarios only — not used in normal incremental re-index flow.

### 3.3 sqlite-vec Virtual Table

Created only when `OpenRouterEmbedder` is configured. Embedding dimension: 1536 (OpenAI `text-embedding-3-small` via OpenRouter):

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS symbols_vec USING vec0(
    embedding float[1536]
);
```

This DDL is executed by `SymbolStore::enable_vector_search()` called at startup only if `EmbeddingProvider` is not `NoOpEmbedder`. The table is absent from `NoOpEmbedder` deployments.

### 3.4 Symbol Type

```rust
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: i64,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub signature: String,           // one-liner; max 256 chars, truncated with `…` if longer
    pub doc_comment: Option<String>, // first doc comment line only
    pub relevance_score: f32,        // post-RRF; 0.0 for raw inserts
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Module,
    Class,
    Trait,
    Impl,
    FileBoundary, // unknown language: file-level boundary entry
}
```

### 3.5 IndexStats

```rust
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub indexed_files: usize,
    pub total_symbols: usize,
    pub skipped_files: usize,   // unchanged by hash check
    pub duration_ms: u64,
}
```

---

## 4. Component Contracts

### 4.1 ContextEngine Trait

```rust
pub trait ContextEngine: Send + Sync {
    /// Build or update the symbol index for a project root.
    /// Lazy: called on first `retrieve()` if not yet indexed.
    fn index_project(&self, root: &Path) -> Result<IndexStats>;

    /// Retrieve relevant symbols for the current turn.
    fn retrieve(
        &self,
        query: &str,
        schemas: &[serde_json::Value],
        budget: ContextBudget,
    ) -> Result<ContextPacket>;

    /// Invalidate and re-index a single file (called by FileWatcher).
    fn invalidate(&self, path: &Path) -> Result<()>;
}

/// No-op implementation — returns empty ContextPacket. Used in all existing tests.
pub struct NoOpContextEngine;
impl ContextEngine for NoOpContextEngine { /* ... */ }
```

### 4.2 ContextPacket

```rust
#[derive(Debug, Clone)]
pub struct ContextPacket {
    pub symbols: Vec<Symbol>,          // within budget, sorted by relevance desc
    pub hardened_schemas: Vec<serde_json::Value>, // CTX-05: post-hardening
    pub truncated: bool,               // CTX-04
    pub dropped_count: usize,          // CTX-04: symbols dropped to fit budget
    pub total_tokens_estimate: usize,  // estimated tokens used by included symbols
}
```

### 4.3 ContextBudget

```rust
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,      // from persona YAML; default 8192
    pub reserve_tokens: usize,  // reserved for user msg + tool outputs; default 2048
}

impl ContextBudget {
    pub fn available(&self) -> usize {
        self.max_tokens.saturating_sub(self.reserve_tokens)
    }
}
```

**Token estimate formula:** `(sym.signature.chars().count() + sym.name.chars().count() + 10) / 4`

Uses `.chars().count()` (Unicode scalar values, not bytes) for correctness with non-ASCII identifiers in Python and Go.

### 4.4 SchemaHardener

```rust
pub struct SchemaHardener;

impl SchemaHardener {
    /// Apply ForgeCode's enforce_strict_schema + truncation reminder to all schemas.
    pub fn harden(&self, schemas: &[serde_json::Value]) -> Vec<serde_json::Value> {
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

### 4.5 ToolRegistry::schemas() — New Method

Add this `impl` block **directly within `crates/kay-tools/src/registry.rs`** (not a separate file — `self.tools` is a private field accessible only within the same module):

```rust
impl ToolRegistry {
    /// Return the raw JSON schemas for all registered tools.
    /// Called by SchemaHardener before sending to provider.
    pub fn schemas(&self) -> Vec<serde_json::Value> {
        self.tools                      // private HashMap<ToolName, Arc<dyn Tool>> — same file required
            .values()
            .map(|t| t.input_schema())  // Tool::input_schema() — existing trait method (TOOL-05)
            .collect()
    }
}
```

Must be added inside `registry.rs`, not in `engine.rs` or any other file in `kay-context` — private fields are not accessible across module boundaries even within the same crate.

### 4.6 AgentEvent Extensions

Add both new variants to `AgentEvent` in `crates/kay-tools/src/events.rs`:

```rust
// Additive; #[non_exhaustive] already applied to AgentEvent
AgentEvent::ContextTruncated {
    dropped_symbols: usize,
    budget_tokens: usize,
},

AgentEvent::IndexProgress {
    indexed: usize,
    total: usize,
},
```

**Also wire into `AgentEventWire`** in `crates/kay-tools/src/events_wire.rs`.

`AgentEventWire<'a>` is a borrowing newtype `pub struct AgentEventWire<'a>(pub &'a AgentEvent)` with a hand-written `Serialize` impl that `match self.0 { ... }`. There are no enum variants to add — instead, add two new match arms to the existing `impl<'a> Serialize for AgentEventWire<'a>` block:

```rust
// In the `match self.0 { ... }` block inside impl Serialize for AgentEventWire:
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

Also add insta snapshot tests in `crates/kay-tools/tests/events_wire_snapshots.rs` for both new variants (per the schema-stability requirement in events_wire.rs module doc).

---

## 5. Language Support

| Language | Extension(s) | tree-sitter crate | Extracted kinds |
|----------|-------------|------------------|----|
| Rust | `.rs` | `tree-sitter-rust 0.23` | fn, mod, trait, impl |
| TypeScript | `.ts`, `.tsx` | `tree-sitter-typescript 0.23` | function, class, module |
| Python | `.py` | `tree-sitter-python 0.23` | function, class, module |
| Go | `.go` | `tree-sitter-go 0.23` | func, type, package |
| Unknown | `*` | — | `FileBoundary` (path + first 10 lines) |

**Version note:** All grammar crates pinned at `0.23.x` to match `tree-sitter` core `0.23.x` API. Using `tree-sitter` core `0.23` (not `0.26`) because `tree-sitter-rust 0.24` and `tree-sitter-typescript 0.23` are built against the `0.23` API. Lock core at `0.23.2` in `[workspace.dependencies]`.

---

## 6. Retrieval Algorithm

### Step 1: FTS5 Structural Search

```sql
SELECT s.id, s.name, s.kind, s.file_path, s.start_line, s.end_line, s.signature,
       bm25(symbols_fts) as score
FROM symbols_fts
JOIN symbols s ON symbols_fts.rowid = s.id
WHERE symbols_fts MATCH ?
ORDER BY score ASC   -- bm25 returns negative; more negative = better match
LIMIT 50;
```

Query term: whitespace-tokenized words from the user's last message.

### Step 2: Structural Name Match (bonus)

Symbols whose `name` appears verbatim in the query get +0.5 relevance bonus applied after FTS5 scoring.

### Step 3: sqlite-vec ANN (when embeddings present)

```sql
SELECT id, distance
FROM symbols_vec
WHERE embedding MATCH vec_f32(?1)
ORDER BY distance
LIMIT 20;
```

If `symbols_vec` does not exist (NoOpEmbedder), this step is skipped.

Result merged with FTS5 results via Reciprocal Rank Fusion (k=60):
`rrf_score(d) = Σ 1/(60 + rank_i(d))` across result lists. Final sort: descending rrf_score.

### Step 4: ContextBudget Truncation

Sort by combined score descending. Accumulate until `ContextBudget::available()` token estimate exhausted. If any symbols dropped: emit `AgentEvent::ContextTruncated` to event stream and set `ContextPacket.truncated = true`.

---

## 7. Incremental Re-indexing

On `invalidate(path)`:
1. Read file bytes; compute `sha256(content)[..16]` (first 16 hex chars)
2. Check `index_state.file_hash` — if hash matches, return `Ok(())` early (no change)
3. `DELETE FROM symbols WHERE file_path = ?` (triggers handle FTS5 sync via §3.2 triggers)
4. Re-run tree-sitter parse → batch-insert new symbols (triggers populate FTS5)
5. Update `index_state` (new mtime_secs, new hash, current epoch)

**Initial `index_project()` call:**

```rust
// Uses forge_walker::Walker::max_all() for full traversal
// Verified: Walker::max_all() exists; .cwd() setter generated by derive_setters; get().await returns Result<Vec<forge_walker::File>>
let walker = forge_walker::Walker::max_all().cwd(root.to_path_buf());
let files: Vec<forge_walker::File> = walker.get().await?;
// file.path is String (not PathBuf) — use PathBuf::from(&file.path) to convert
```

`Walker::max_all()` sets `max_files = usize::MAX`, `max_depth = usize::MAX`, `hidden = false` (includes dotfiles). `forge_walker::File` has fields `path: String`, `file_name: Option<String>`, `size: u64`. Chunked in transactions of 256 files to avoid SQLite lock contention. Progress streamed as `AgentEvent::IndexProgress { indexed, total }` every 256 files.

---

## 8. FileWatcher

```rust
pub struct FileWatcher {
    _watcher: notify::RecommendedWatcher, // kept alive by struct lifetime
}

impl FileWatcher {
    pub fn start(
        root: PathBuf,
        engine: Arc<dyn ContextEngine>,
    ) -> Result<Self>;
}
```

- `notify::RecursiveMode::Recursive` on project root
- `notify` version: `6.1` (stable release, not 9.0.0-rc.3 — see §10)
- Debounce: 500ms via `notify-debouncer-mini = "0.4"` (stable companion crate for `notify 6.x`)
- Filter: `Create | Modify | Remove` events on files matching known extensions
- On `Remove`: `DELETE FROM symbols WHERE file_path = ?` (triggers handle FTS5)

---

## 9. Embedding Provider

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
}

/// Default: no embedding generation; vector search disabled.
pub struct NoOpEmbedder;

/// Test: deterministic fake embeddings (random-but-reproducible f32 vectors).
#[cfg(any(test, feature = "testing"))]
pub struct FakeEmbedder {
    pub dimensions: usize, // default 1536 to match prod schema
}

/// Production: OpenRouter text-embedding-3-small endpoint.
pub struct OpenRouterEmbedder {
    pub api_key: String,
    pub model: String, // default "openai/text-embedding-3-small"
}
```

W-4 tests use `FakeEmbedder` to populate `symbols_vec` with deterministic vectors, enabling sqlite-vec ANN tests without any OpenRouter calls.

---

## 10. kay-cli Wiring

### `RunTurnArgs` change (crates/kay-cli/src/run.rs)

```rust
pub struct RunTurnArgs {
    // ... existing fields unchanged ...
    pub context_engine: Arc<dyn ContextEngine>,  // NEW — default: NoOpContextEngine
    pub context_budget: ContextBudget,           // NEW — default: ContextBudget::default()
}
```

### `run_turn` injection point

Before assembling the provider request:

```rust
let raw_schemas = args.tool_registry.schemas();
let packet = args.context_engine.retrieve(
    &last_user_message,
    &raw_schemas,
    args.context_budget.clone(),
)?;

if packet.truncated {
    event_tx.send(AgentEvent::ContextTruncated {
        dropped_symbols: packet.dropped_count,
        budget_tokens: args.context_budget.max_tokens,
    }).await?;
}

// inject packet.symbols into system prompt as <context> block
// use packet.hardened_schemas instead of raw_schemas when building provider request
```

System prompt injection format:
```
<context>
{for each symbol: "{kind} {file_path}:{start_line} — {signature}"}
</context>
```

### Backward compatibility

All existing `run.rs` call sites use `RunTurnArgs` with `..Default::default()` for the new fields. `NoOpContextEngine::retrieve()` returns an empty `ContextPacket` with `hardened_schemas` populated from `SchemaHardener::harden(schemas)` — so CTX-05 applies even in the no-op case.

---

## 11. Dependencies

### New entries for `[workspace.dependencies]`

```toml
# Tree-sitter — pin core + grammars at 0.23.x for API compatibility
tree-sitter = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"

# Vector search SQLite extension
sqlite-vec = "0.1.10-alpha.3"

# File watching (stable 6.x + debouncer)
notify = "6.1"
notify-debouncer-mini = "0.4"
```

**Note on rusqlite:** `rusqlite = "0.38" bundled` was added to `crates/kay-session/Cargo.toml` as a **crate-local dep** in Phase 6 — it is NOT in `[workspace.dependencies]`. Add it there now so `kay-context` can reference `{ workspace = true }`:

```toml
# Add to [workspace.dependencies]:
rusqlite = { version = "0.38", features = ["bundled"] }
```

**Note on sqlite-vec:** `"0.1.10-alpha.3"` is an alpha pre-release — no stable version exists as of 2026-04-22. Pin the exact version to prevent `cargo update` from pulling an incompatible alpha:

```toml
sqlite-vec = "=0.1.10-alpha.3"   # exact pin — no stable release exists yet
```

### `crates/kay-context/Cargo.toml` — selected deps

```toml
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

---

## 12. Testing Strategy

| Wave | Test type | Key cases | Count |
|------|-----------|----------|-------|
| W-1 | Unit: SymbolStore CRUD, index_state hash check, schema creation | insert/query/delete, mtime skip, triggers | 5 |
| W-2 | Unit: Indexer Rust/TS/Py/Go + proptest sig extraction | correctness per lang, max-sig truncation, unknown fallback | 11 |
| W-3 | Unit: FTS5 retriever | match/no-match, ranking, name-bonus | 6 |
| W-4 | Unit: sqlite-vec + RRF merge | FakeEmbedder insert, ANN recall, RRF ordering | 6 |
| W-5 | Unit: budget + ContextTruncated | exact-fit, one-over, zero-available, chars estimate | 6 |
| W-6 | Unit: SchemaHardener + prompt assembly | hardening idempotent, NoOp CTX-05 path | 5 |
| W-7 | Integration: FileWatcher + E2E kay-cli | debounce, invalidate path, full run with context injection | 8 |
| **Total** | | | **≥47** |

All W-4 tests use `FakeEmbedder` — no OpenRouter calls in test suite.

---

## 13. Non-Goals (Phase 7)

- Dense embedding generation at index time (sqlite-vec wired but `NoOpEmbedder` default; OpenRouterEmbedder tested via FakeEmbedder stand-in)
- Multi-language grammar coverage beyond Rust/TS/Py/Go
- Interactive context budget control in GUI/TUI (Phase 10)
- Cross-session context reuse
- Context merging with Phase 6 session transcripts

---

*Spec rev r4 — 2026-04-22 — NEW-5/NEW-6 resolved: schemas() must be in registry.rs (private field), Walker API verified (get().await → Vec<forge_walker::File>, file.path: String)*
