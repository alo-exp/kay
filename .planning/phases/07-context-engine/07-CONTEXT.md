# Phase 7: Context Engine — Context

**Gathered:** 2026-04-22
**Status:** Ready for planning
**Mode:** autonomous (§10e) — all open questions resolved deterministically from upstream artifacts + codebase facts

<domain>
## Phase Boundary

Phase 7 delivers a local context engine for Kay's agent loop. It replaces ForgeCode's cloud gRPC context engine with a new `kay-context` crate containing: a tree-sitter symbol store backed by SQLite (CTX-01, CTX-02), hybrid FTS5 + sqlite-vec retrieval with RRF merge (CTX-03), per-turn token budget enforcement with explicit truncation events (CTX-04), and consistent ForgeCode schema hardening on all tool schemas in-context (CTX-05).

**What this phase delivers:** `kay-context` crate + two new `AgentEvent` variants + `ToolRegistry::schemas()` method + context injection seam in `run_turn`.

**What this phase does NOT deliver:** Real system prompt injection into the OpenRouter request (Phase 8+), OpenRouter-based embeddings (Phase 8+), TUI/GUI context visualization (Phase 9/9.5).

</domain>

<decisions>
## Implementation Decisions

### DL-1 — New crate: `kay-context`

Phase 7 ships as a new top-level workspace member `crates/kay-context/`. It is NOT an extension of `kay-core`, `kay-tools`, or `kay-session`. This crate owns: `SymbolStore`, `TreeSitterIndexer`, `Retriever`, `ContextBudget`, `SchemaHardener`, `FileWatcher`, `ContextEngine` trait, `NoOpContextEngine`, `EmbeddingProvider` trait.

**Source files (10 total):**
- `src/lib.rs` — public re-exports
- `src/store.rs` — SymbolStore + SQLite schema + FTS5 triggers
- `src/indexer.rs` — TreeSitterIndexer + IndexStats
- `src/language.rs` — Language enum + extension detection
- `src/retriever.rs` — rrf_merge() + apply_name_bonus()
- `src/budget.rs` — ContextBudget + ContextPacket + estimate_tokens()
- `src/hardener.rs` — SchemaHardener wrapping harden_tool_schema()
- `src/watcher.rs` — FileWatcher (notify debounced, 500ms)
- `src/embedder.rs` — EmbeddingProvider trait + NoOpEmbedder + FakeEmbedder (cfg(test))
- `src/engine.rs` — ContextEngine trait + NoOpContextEngine + KayContextEngine stub

---

### DL-2 — rusqlite workspace promotion

`rusqlite = { version = "0.38", features = ["bundled"] }` moves from `crates/kay-session/Cargo.toml` (crate-local) to `[workspace.dependencies]` in the root `Cargo.toml`. `kay-session/Cargo.toml` changes to `rusqlite = { workspace = true }`. This is the ONLY change to `kay-session/Cargo.toml` in Phase 7.

---

### DL-3 — sqlite-vec pinned exactly

`sqlite-vec = "=0.1.10-alpha.3"` — exact pin required (no stable release; semver range picks up breaking alpha changes). This is the only viable sqlite-vec version at Phase 7 time.

---

### DL-4 — tree-sitter 0.23.x (NOT 0.26)

All tree-sitter crates pinned at `0.23.x`: `tree-sitter = "0.23"`, `tree-sitter-rust = "0.23"`, `tree-sitter-typescript = "0.23"`, `tree-sitter-python = "0.23"`, `tree-sitter-go = "0.23"`. The 0.26 API has breaking `Query`/`Language` API changes that are not yet stabilized; 0.23 is proven. Grammar crates MUST match the runtime version.

---

### DL-5 — v1 language support: Rust, TypeScript, Python, Go + FileBoundary fallback

Four languages in Phase 7: Rust, TypeScript (covers .ts + .tsx), Python, Go. Unknown extensions get a single `SymbolKind::FileBoundary` symbol with first 10 lines as signature — this handles TOML/JSON/Markdown without crashing the indexer.

---

### DL-6 — Embedding strategy: NoOpEmbedder default

- `NoOpEmbedder`: default in production (no network calls during indexing; ANN path disabled; only FTS5 retrieval active)
- `FakeEmbedder { dimensions: 1536 }`: deterministic test double (`#[cfg(test)]`); identity function on input vector (ANN for same-vector input returns that vector)
- `OpenRouterEmbedder`: deferred to Phase 8+ (needs provider HAL integration)

When `NoOpEmbedder` is active, `retrieve()` MUST NOT attempt to access `symbols_vec` table — table is absent and any access panics.

---

### DL-7 — Token counting: chars().count() / 4 heuristic

Token estimate formula: `(name.chars().count() + sig.chars().count() + 10) / 4`. Constant 10 represents average whitespace + punctuation overhead. This uses `.chars()` (Unicode scalar values), NOT `.len()` (bytes) — non-ASCII signatures must not be under-counted.

**Default budget:** `max_tokens = 8192`, `reserve_tokens = 1024` → `available = 7168`. Configurable via `ContextBudget::new(max, reserve)`.

**Rationale:** tiktoken-rs requires a C++ toolchain (cc crate + llvm); chars/4 is within 20% for code and the 12.5% reserve absorbs the error.

---

### DL-8 — FileWatcher: notify 6.1 + notify-debouncer-mini 0.4, 500ms debounce

- Crate: `notify = "6.1"` + `notify-debouncer-mini = "0.4"`
- Debounce: 500ms (coalesces burst writes from IDE autosave)
- Ignored paths: `*.lock`, `target/`, `.git/`, `*.tmp`, `*.swp`
- Watched extensions: `.rs`, `.ts`, `.tsx`, `.py`, `.go`
- Watch scope: project root from `forge_walker` (same root passed to `Walker::max_all().cwd(root)`)

---

### DL-9 — ContextEngine wiring into run_turn

Phase 7 adds 3 fields to `RunTurnArgs` in `crates/kay-core/src/loop.rs`:

```rust
pub struct RunTurnArgs {
    // ... existing 6 fields unchanged ...
    pub context_engine: Arc<dyn kay_context::engine::ContextEngine>,
    pub context_budget: kay_context::budget::ContextBudget,
    pub initial_prompt: String,
}
```

At the top of `run_turn()` (before the event loop), call:
```rust
let _ctx_packet = args.context_engine
    .retrieve(&args.initial_prompt, &args.registry.schemas())
    .await
    .unwrap_or_default();
```

This proves the plumbing compiles and runs. The `ContextPacket` is not yet injected into the OpenRouter request — that wiring is Phase 8+ (when the provider adapter is extended). The `_ctx_packet` is intentionally unused in Phase 7 to avoid dead-code warnings; a `#[allow(unused)]` annotation is acceptable here.

**`kay-cli/src/run.rs` update:** Pass `NoOpContextEngine::default()` + `ContextBudget::default()` + `prompt.clone()` when constructing `RunTurnArgs`.

**Rationale:** `run_turn` is an event consumer loop (it drains `model_rx`); the actual OpenRouter request is assembled upstream. Injecting a real system prompt requires extending the provider adapter in Phase 8+. Phase 7's scope is the retrieval infrastructure and budget enforcement — not the provider wiring.

---

### DL-10 — FTS5 + sqlite-vec RRF merge, k=60

Hybrid retrieval uses RRF (Reciprocal Rank Fusion) with k=60:
- `rrf_score(rank) = 1 / (60 + rank)`
- FTS5 results get a name-bonus of +0.5 when the query term exactly matches the symbol name
- ANN results from sqlite-vec provide the second ranking list
- Merged scores determine final ordering in `ContextPacket.symbols`

---

### DL-11 — Signature truncation at 256 chars

`TreeSitterIndexer` truncates any extracted signature at 256 chars, appending `…` (U+2026). Stored `sig` field length ≤ 257 chars (256 + ellipsis). This is enforced in `Symbol::new()` — not in the tree-sitter query itself. The `proptest_sig_never_exceeds_256` property test verifies this invariant.

---

### DL-12 — New AgentEvent variants: ContextTruncated + IndexProgress

Added to `crates/kay-tools/src/events.rs`:
```rust
ContextTruncated {
    dropped_symbols: usize,
    budget_tokens: usize,
},
IndexProgress {
    indexed: usize,
    total: usize,
},
```

Wire serialization added to `crates/kay-tools/src/events_wire.rs` (match arms, NOT new structs — QG-C4 pattern).

Two insta snapshot tests added to `crates/kay-tools/tests/events_wire_snapshots.rs`:
- `snap_context_truncated_wire`
- `snap_index_progress_wire`

**QG-C4 carry-forward:** `event_filter.rs` MUST NOT be touched. No new match arms in event_filter — these new variants pass through unfiltered.

---

### DL-13 — ToolRegistry::schemas() method

Added to `crates/kay-tools/src/registry.rs` (inside `impl ToolRegistry`):
```rust
pub fn schemas(&self) -> Vec<serde_json::Value> {
    self.tools.values().map(|t| t.input_schema()).collect()
}
```

This CANNOT be added to `engine.rs` or any other file because `self.tools` is private to `registry.rs`.

---

### DL-14 — SchemaHardener delegates to existing harden_tool_schema()

`SchemaHardener::harden()` in `src/hardener.rs` calls the existing `harden_tool_schema()` from `kay-tools/src/schema.rs`. No duplicate hardening logic. The `NoOpContextEngine` also calls `SchemaHardener::harden()` on its schema list — CTX-05 applies even in no-op mode.

---

### DL-15 — Session title injection: [USER_DATA: ...] delimiter required

Per Phase 6 DL-7: if Phase 7's context engine ever injects `session.title` into the system prompt, it MUST delimit it as `[USER_DATA: session_title]`. In Phase 7 scope, session metadata is NOT injected (only symbols are injected). This constraint is documented in `src/engine.rs` as a comment for Phase 8+ implementors.

---

### Claude's Discretion

- Internal field ordering within `SymbolStore`, `ContextBudget`, `ContextPacket` structs
- Whether `FakeEmbedder` returns zeros or a deterministic hash of the input (either is acceptable; zeros is simpler)
- Whether `KayContextEngine` stub in `src/engine.rs` is an empty struct or has placeholder fields
- Whether the `<context>` block in the system prompt uses XML-style tags or markdown fences (Phase 8+ decision; Phase 7 only assembles the `ContextPacket`)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 7 Spec and Plan
- `docs/superpowers/specs/2026-04-22-phase7-context-engine-design.md` — Full design spec: architecture, component contracts, open questions resolved
- `docs/superpowers/plans/2026-04-22-phase7-context-engine.md` — 10-task TDD implementation plan with exact file paths and code snippets
- `.planning/phases/07-context-engine/07-TEST-STRATEGY.md` — 7-wave test strategy, 47+ tests, exact test names and assertions

### Upstream Context Consumed
- `.planning/phases/05-agent-loop/05-CONTEXT.md` — DL decisions that constrain Phase 7: AgentEvent #non_exhaustive (DL-4), AgentEventWire borrowing newtype pattern, event_filter QG-C4
- `.planning/phases/06-session-store/06-CONTEXT.md` — DL-3 (rusqlite 0.38 bundled in kay-session to promote), DL-7 (session title = untrusted data)

### Code to Read Before Implementing
- `crates/kay-core/src/loop.rs` — RunTurnArgs struct shape (DL-9); run_turn() signature
- `crates/kay-tools/src/events.rs` — AgentEvent enum + existing variant shapes
- `crates/kay-tools/src/events_wire.rs` — AgentEventWire Serialize pattern (match arms required)
- `crates/kay-tools/src/registry.rs` — ToolRegistry struct (private `self.tools`) + where to add schemas()
- `crates/kay-tools/src/schema.rs` — harden_tool_schema() signature (consumed by SchemaHardener)
- `crates/kay-session/Cargo.toml` — rusqlite dependency to be promoted to workspace
- `Cargo.toml` — workspace.dependencies section to receive rusqlite + tree-sitter + sqlite-vec + notify

### External Crate Docs
- sqlite-vec docs at `https://alexgarcia.xyz/sqlite-vec/` — ANN search API, table creation, vec0 virtual table syntax
- tree-sitter 0.23 docs — Language::new(), Query construction, Node traversal API (0.26 API is different — do NOT use 0.26 examples)
- notify-debouncer-mini 0.4 docs — DebouncedWatcher, DebouncedEvent construction

### Validation
- `.planning/VALIDATION.md` — 0 BLOCK, 8 WARN (all future-phase concerns), 4 INFO; workflow may proceed

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/kay-tools/src/schema.rs::harden_tool_schema()` — wraps `enforce_strict_schema(schema, true)`; consumed directly by `SchemaHardener::harden()`
- `crates/kay-tools/src/registry.rs::ToolRegistry` — public registry with private `self.tools: HashMap<ToolName, Arc<dyn Tool>>`; `schemas()` method added in Task 2
- `forge_walker::Walker::max_all().cwd(root).get().await` — `.gitignore`-aware file traversal; returns `Result<Vec<forge_walker::File>>` where `File.path: String`
- `crates/kay-tools/tests/events_wire_snapshots.rs` — existing insta snapshot test file; Phase 7 adds 2 new snapshot tests here (additive)
- `crates/kay-session/Cargo.toml::rusqlite 0.38 bundled` — to be promoted to workspace dep

### Established Patterns
- `AgentEventWire` borrowing newtype: `pub struct AgentEventWire<'a>(pub &'a AgentEvent)` with hand-written `Serialize` impl using `serialize_map`. New variants require new match arms — NOT new struct variants.
- `#[non_exhaustive]` on `AgentEvent` means downstream `match` arms need `_` catch-all — Phase 7 additions are safe under this contract.
- TDD iron law: RED commit (`test(wN): RED — <description>`) before GREEN commit (`feat(ctx-NN): GREEN — <description>`). DCO `Signed-off-by:` on every commit.
- `tempfile::TempDir` for SQLite test isolation — every test gets its own ephemeral DB.
- `proptest` already in workspace for property tests (see Phase 3 marker-forgery proptest).

### Integration Points
- `crates/kay-core/src/loop.rs::RunTurnArgs` — add 3 fields (DL-9)
- `crates/kay-cli/src/run.rs` — pass `NoOpContextEngine::default()` + `ContextBudget::default()` + `prompt.clone()` when constructing `RunTurnArgs`
- `Cargo.toml` workspace members: add `"crates/kay-context"`
- `Cargo.toml` workspace.dependencies: add rusqlite (promote), tree-sitter 0.23.x (4 crates), sqlite-vec exact pin, notify + notify-debouncer-mini

### Files That Must NOT Be Modified
- `crates/kay-tools/src/event_filter.rs` — QG-C4 carry-forward: byte-identical; no new match arms for ContextTruncated/IndexProgress
- `crates/kay-session/**` except `Cargo.toml` (only the rusqlite workspace promotion)
- All Phase 5/6 test files — must remain byte-identical

</code_context>

<specifics>
## Specific Ideas

- `FakeEmbedder` returns a vector of zeros of length `self.dimensions` — simpler than hashing and still passes ANN tests because the query uses the same zero vector
- `SymbolStore::open(path: &Path)` mirrors `SessionStore::open(root: &Path)` from Phase 6 for API consistency
- `ContextBudget::default()` = `ContextBudget::new(8192, 1024)` — matches the Phase 7 spec default
- The `<context>` block XML tag format is a Phase 8+ decision; Phase 7 assembles `ContextPacket` but does not serialize it into the system prompt string

</specifics>

<deferred>
## Deferred Ideas

- **OpenRouterEmbedder**: sqlite-vec ANN with real embeddings from OpenRouter — Phase 8+ (needs provider HAL integration and cost accounting)
- **System prompt injection**: real `<context>` block insertion into the OpenRouter request — Phase 8+ (OpenRouter adapter extension)
- **Project-scoped context** (grouping multiple sessions' symbols): Phase 10 (session manager)
- **Language support expansion** (Java, C++, C, Swift): Phase 11+ backlog (add grammar crates + language.rs match arm)
- **tiktoken-rs token counting**: accurate BPE counting — deferred until C++ build toolchain is guaranteed in CI
- **sqlite-vec upgrade to stable**: once sqlite-vec publishes a non-alpha release, unpin the exact version

</deferred>

---

*Phase: 07-context-engine*
*Context gathered: 2026-04-22*
