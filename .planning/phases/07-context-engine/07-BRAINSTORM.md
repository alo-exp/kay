# Phase 7: Context Engine — Brainstorm

> **Mode:** autonomous (§10e) — all OQs resolved deterministically.
> **Date:** 2026-04-22
> **Inputs:** CTX-01..CTX-05, ROADMAP.md §Phase 7, codebase intel scan, crate availability check

---

## 1. Problem Statement

Kay's agent loop currently operates with "empty registry + minimal context" — no structured code context is injected into prompts. ForgeCode's context engine is gRPC-cloud (upload workspace to a remote service, run semantic search via proto). Kay needs a **local**, offline-capable replacement that:
- Extracts function signatures and module boundaries via tree-sitter (not full file bodies)
- Retrieves relevant context per turn via hybrid structural + vector search
- Enforces an explicit per-turn token budget
- Applies ForgeCode's schema hardening to all in-context tool schemas

---

## 2. Open Questions → Decisions

### OQ-1: Architecture for `kay-context` crate

**Decision:** 7 types forming a clean pipeline:

| Type | Role |
|------|------|
| `SymbolStore` | rusqlite DB wrapper; schema with symbols, FTS5, sqlite-vec, index_state |
| `Indexer` | tree-sitter parsing → Symbol extraction; per-language queries |
| `EmbeddingProvider` (trait) | Pluggable; `OpenRouterEmbedder` + `NoOpEmbedder` |
| `Retriever` | Hybrid: FTS5 structural + sqlite-vec ANN → RRF merge |
| `ContextBudget` | Token budget per turn (char/4 heuristic); explicit truncation |
| `SchemaHardener` | Applies `harden_tool_schema()` to all in-context tool schemas (CTX-05) |
| `ContextEngine` (trait) | DI seam for kay-cli: `retrieve(query, budget)` + `invalidate(path)` |

### OQ-2: tree-sitter language support

**Decision:** V1 supports Rust, TypeScript/TSX, Python, Go.
- Crates: `tree-sitter-rust 0.24`, `tree-sitter-typescript 0.23`, `tree-sitter-python 0.25`, `tree-sitter-go 0.25`
- Unknown languages: file-level boundary only (path + first 10 lines as summary line)
- Language detection: file extension → enum `Language { Rust, TypeScript, Python, Go, Unknown }`

**Rationale:** Covers >90% of TB 2.0 benchmark tasks. Unknown fallback ensures correctness (no panic on `.json`, `.md`, etc).

### OQ-3: sqlite-vec — embedding generation

**Decision:** Lazy embedding via OpenRouter text-embedding endpoint.
- Default: `NoOpEmbedder` — symbol store built with FTS5 only; vector column empty
- With API key at index time: `OpenRouterEmbedder` populates sqlite-vec vectors for ANN search
- Retrieval: FTS5-only when no vectors; adds ANN dimension when vectors present
- Avoids `fastembed` (ONNX runtime, heavy binary); satisfies CTX-03 (sqlite-vec is wired even if vectors are lazy)

**Why not fastembed:** 22MB quantized model + ONNX runtime adds ~15MB to binary and 200ms startup. Unacceptable for `kay run` latency.

### OQ-4: File watch strategy

**Decision:** `notify = "9.0.0-rc.3"` with 500ms debounce.
- FSEvents (macOS) / inotify (Linux) / ReadDirectoryChangesW (Windows)
- Debounced `notify::RecommendedWatcher` → batches rapid edits into one invalidation call
- `invalidate(path)` re-indexes only the changed file; updates `index_state` (mtime + hash check)
- Monorepo >10k files: initial index is lazy (index on first `retrieve()` call per file); watcher activated after first index pass

### OQ-5: Context budget token counting

**Decision:** `(content_bytes / 4)` byte estimate — no tiktoken-rs.
- Good enough for English identifiers and code signatures
- `ContextBudget::max_tokens` read from persona YAML `context_budget_tokens` field (default: 8192)
- When budget exceeded: sort symbols by score descending, include until budget full, emit `AgentEvent::ContextTruncated { dropped_symbols: usize, budget_tokens: usize }`

### OQ-6: retrieve() wiring into kay-cli

**Decision:** `ContextEngine` trait object passed via `RunTurnArgs`.
- `RunTurnArgs` gains `context_engine: Arc<dyn ContextEngine>`
- `NoOpContextEngine` implements the trait (returns empty `ContextPacket`) for backward compat
- `run_turn` calls `context_engine.retrieve(last_user_message, budget)` before assembling the provider request
- `ContextPacket` is injected as a system-prompt section: `<context>\n{symbol sigs}\n</context>`
- `hardened_schemas` from `ContextPacket` replace the raw tool schemas before sending to provider

### OQ-7: TDD wave breakdown

| Wave | Core deliverable | REQs |
|------|-----------------|------|
| W-1 | `kay-context` scaffold: SQLite schema v1, `SymbolStore::open`, `index_state` CRUD | CTX-01 partial |
| W-2 | `Indexer`: tree-sitter parsing for Rust/TS/Py/Go + unknown fallback | CTX-01, CTX-02 |
| W-3 | `Retriever` structural (FTS5): `retrieve_by_name`, `retrieve_by_path` | CTX-03 partial |
| W-4 | sqlite-vec loading + `OpenRouterEmbedder` + RRF hybrid merge | CTX-03 full |
| W-5 | `ContextBudget` + `AgentEvent::ContextTruncated` + explicit truncation | CTX-04 |
| W-6 | `SchemaHardener` (CTX-05) + prompt assembly integration in `run_turn` | CTX-05 |
| W-7 | `FileWatcher` (notify) + incremental re-index + full kay-cli E2E test | CTX-01 SC-5 |

---

## 3. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        kay-context crate                        │
│                                                                 │
│  ┌──────────────┐    ┌──────────────────┐    ┌─────────────┐   │
│  │   Indexer    │───▶│   SymbolStore    │◀───│ FileWatcher │   │
│  │  (tree-sitter│    │ (rusqlite + FTS5 │    │  (notify)   │   │
│  │  Rust/TS/    │    │  + sqlite-vec)   │    └─────────────┘   │
│  │  Py/Go)      │    └────────┬─────────┘                      │
│  └──────────────┘             │                                 │
│                               │                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                      Retriever                           │   │
│  │   FTS5 structural ─────────────────────┐                 │   │
│  │   sqlite-vec ANN (when embeddings avail)├── RRF merge    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                               │                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │               ContextBudget + SchemaHardener             │   │
│  │   sort by score → truncate to budget → harden schemas    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                               │                                 │
│  ContextEngine trait ◀────────┘                                 │
└─────────────────────────────────────────────────────────────────┘
         │
         │  Arc<dyn ContextEngine>
         ▼
┌────────────────┐
│  kay-cli run.rs │  RunTurnArgs.context_engine
│  run_turn()     │  → retrieve() → inject into system prompt
└────────────────┘
```

---

## 4. SQLite Schema v1

```sql
-- Symbol index
CREATE TABLE IF NOT EXISTS symbols (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL,          -- 'function' | 'module' | 'class' | 'trait' | 'impl' | 'boundary'
    file_path   TEXT NOT NULL,
    start_line  INTEGER NOT NULL,
    end_line    INTEGER NOT NULL,
    signature   TEXT NOT NULL,          -- one-line sig; NOT full body
    doc_comment TEXT
);

-- FTS5 virtual table for structural text search
CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
    name, signature, doc_comment,
    content=symbols, content_rowid=id
);

-- File watch state for incremental re-indexing
CREATE TABLE IF NOT EXISTS index_state (
    file_path   TEXT PRIMARY KEY,
    mtime_secs  INTEGER NOT NULL,
    file_hash   TEXT NOT NULL,          -- SHA-256 hex
    indexed_at  INTEGER NOT NULL        -- unix timestamp
);

-- Optional: sqlite-vec virtual table (populated lazily when embeddings available)
-- CREATE VIRTUAL TABLE symbols_vec USING vec0(embedding float[384]);
-- Created dynamically when EmbeddingProvider is OpenRouterEmbedder
```

---

## 5. Public API Surface

```rust
// ContextEngine trait (DI seam)
pub trait ContextEngine: Send + Sync {
    fn retrieve(&self, query: &str, budget: ContextBudget) -> Result<ContextPacket>;
    fn invalidate(&self, path: &Path) -> Result<()>;
    fn index_project(&self, root: &Path) -> Result<IndexStats>;
}

// ContextPacket — returned by retrieve()
pub struct ContextPacket {
    pub symbols: Vec<Symbol>,          // sorted by relevance, within budget
    pub hardened_schemas: Vec<Value>,  // CTX-05: all tool schemas post-hardening
    pub truncated: bool,               // CTX-04: true if budget was hit
    pub dropped_count: usize,          // CTX-04: count of dropped symbols
}

// ContextBudget — per-turn budget config
pub struct ContextBudget {
    pub max_tokens: usize,             // default 8192
    pub reserve_tokens: usize,         // tokens reserved for user message + tool calls (default 2048)
}

// Symbol — what tree-sitter extracts
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,             // Function | Module | Class | Trait | Impl | FileBoundary
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub signature: String,            // one-line; never full body
    pub doc_comment: Option<String>,
    pub relevance_score: f32,         // combined FTS5 + vec score after RRF
}
```

---

## 6. AgentEvent Extension

```rust
// additive #[non_exhaustive] extension to AgentEvent in kay-tools/src/events.rs
AgentEvent::ContextTruncated {
    dropped_symbols: usize,
    budget_tokens: usize,
}
```

---

## 7. Key Non-Decisions (out of scope for Phase 7)

- Dense embedding generation at index time: deferred to post-v1 (sqlite-vec wired but `NoOpEmbedder` default)
- Multi-language tree-sitter beyond Rust/TS/Py/Go: deferred to Phase 12 hardening
- Interactive context budget editing via GUI/TUI: Phase 10
- Cross-session context reuse: Phase 10

---

## 8. Wave Scope Summary

7 TDD waves, each with RED commit before GREEN:
- W-1: scaffold + schema (3-4 tests)
- W-2: indexer (8-10 tests, incl. proptest for symbol extraction correctness)
- W-3: FTS5 retriever (4-6 tests)
- W-4: sqlite-vec + RRF (4-5 tests)
- W-5: budget + truncation event (5-6 tests)
- W-6: schema hardener + prompt assembly (4-5 tests)
- W-7: file watcher + E2E (6-8 tests)

**Total target:** ≥35 tests

---

*Generated 2026-04-22 — autonomous brainstorm for /silver:feature Phase 7*
