# Phase 7: Context Engine — Test Strategy

**Phase:** 07-context-engine  
**Date:** 2026-04-22  
**Spec ref:** `docs/superpowers/specs/2026-04-22-phase7-context-engine-design.md`

---

## Testing Pyramid

```
         /   W-7 Integration   \      8 tests — FileWatcher + E2E kay-cli
        /  W-3/W-4/W-5/W-6 Unit \    23 tests — retrieval, budget, hardener
       /    W-1/W-2 Unit Tests    \   16 tests — store CRUD + indexer
```

Total target: **≥ 47 tests** (all synchronous/deterministic; no network calls; no OpenRouter).

---

## Wave-by-Wave Plan

### W-1 — SymbolStore CRUD (5 tests)

**File:** `crates/kay-context/tests/store.rs`  
**What to test:** SQLite schema creation, insert/query/delete lifecycle, FTS5 trigger sync, index_state hash-skip logic.

| Test | Type | Assertion |
|------|------|-----------|
| `schema_creates_tables` | unit | `PRAGMA table_info(symbols)` returns expected columns; `symbols_fts` and `index_state` exist |
| `insert_and_query_by_name` | unit | Insert 1 symbol; `query_by_name("fn_foo")` returns it with all fields intact |
| `delete_clears_fts` | unit | Insert + delete; FTS5 search returns empty (trigger fires correctly) |
| `index_state_skip_on_same_hash` | unit | Call `re_index` twice with identical content; second call returns `skipped_files=1` |
| `index_state_updates_on_hash_change` | unit | Mutate file content; second `re_index` returns `skipped_files=0`, new symbol present |

**Skip:** internal SQLite WAL/journal mechanics — those belong to rusqlite's own tests.

---

### W-2 — Indexer: per-language + property tests (11 tests)

**File:** `crates/kay-context/tests/indexer.rs`  
**What to test:** tree-sitter extraction correctness per language, max-sig truncation at 256 chars, unknown-language fallback to `FileBoundary`.

| Test | Type | Assertion |
|------|------|-----------|
| `rust_fn_extracted` | unit | Parse `fn foo(x: i32) -> i32 {}` → `Symbol { kind: Function, name: "foo", sig: "fn foo(x: i32) -> i32" }` |
| `rust_trait_extracted` | unit | Parse `trait Bar { fn baz(); }` → trait symbol + fn symbol |
| `rust_mod_boundary` | unit | Parse `mod utils {}` → `SymbolKind::Module` |
| `typescript_function_extracted` | unit | Parse `function greet(name: string): string {}` → function symbol |
| `typescript_class_extracted` | unit | Parse `class Foo {}` → class symbol |
| `python_def_extracted` | unit | Parse `def compute(x): ...` → function symbol |
| `python_class_extracted` | unit | Parse `class Solver: ...` → class symbol |
| `go_func_extracted` | unit | Parse `func Run(ctx context.Context) error {}` → func symbol |
| `sig_truncated_at_256` | unit | Signature >256 chars → stored sig ends with `…`, `len ≤ 257` |
| `unknown_extension_file_boundary` | unit | `.toml` file → 1 symbol with `kind=FileBoundary`, sig = first 10 lines |
| `proptest_sig_never_exceeds_256` | proptest | For any ASCII function body, extracted sig len ≤ 257 chars |

**Skip:** tree-sitter internals, grammar test cases (covered by upstream grammar repos).

---

### W-3 — FTS5 Retriever (6 tests)

**File:** `crates/kay-context/tests/retriever_fts.rs`  
**What to test:** FTS5 match/no-match, ordering by bm25, name-bonus (+0.5) application.

| Test | Type | Assertion |
|------|------|-----------|
| `fts_exact_match_returns_symbol` | unit | Insert `run_loop` fn; query `"run_loop"` → result contains it |
| `fts_no_match_returns_empty` | unit | Query `"zzznomatch"` against populated store → empty results |
| `fts_prefix_match` | unit | Query `"run_lo*"` → matches `run_loop` |
| `fts_name_bonus_applied` | unit | Insert 2 symbols; query exact name of one → that one ranked higher |
| `fts_ranking_order` | unit | Insert 5 symbols; query term appearing 3× in one sig → that symbol ranked first |
| `fts_multi_word_query` | unit | Query `"run loop"` (two tokens) → symbols containing both ranked above single-token matches |

---

### W-4 — sqlite-vec + RRF Merge (6 tests)

**File:** `crates/kay-context/tests/retriever_vec.rs`  
**Embedder:** `FakeEmbedder { dimensions: 1536 }` — deterministic, no network calls.

| Test | Type | Assertion |
|------|------|-----------|
| `vec_table_created_with_fake_embedder` | unit | `enable_vector_search()` → `symbols_vec` table exists |
| `fake_embedder_insert_and_ann` | unit | Insert 3 symbols with FakeEmbedder; ANN query for symbol 0's vector → symbol 0 in top-1 |
| `rrf_merge_prefers_fts_winner` | unit | FTS5 top = symbol A; ANN top = symbol B; RRF output: A ranked above B when FTS5 score is dominant |
| `rrf_merge_prefers_vec_winner` | unit | Zero FTS5 signal; ANN strong hit → vec winner ranks first in merged output |
| `rrf_k60_score_formula` | unit | 2 lists, known ranks → verify `rrf_score = 1/(60+r1) + 1/(60+r2)` for a symbol appearing in both |
| `noop_embedder_skips_vec` | unit | With NoOpEmbedder, `retrieve()` completes; no attempt to access `symbols_vec` (table absent → no error) |

---

### W-5 — ContextBudget + Truncation (6 tests)

**File:** `crates/kay-context/tests/budget.rs`  
**What to test:** token estimate formula, exact-fit, one-over, zero-available, `ContextTruncated` emission.

| Test | Type | Assertion |
|------|------|-----------|
| `token_estimate_formula` | unit | Symbol with name=`"foo"` (3), sig=`"fn foo() -> i32"` (15) → estimate = `(3+15+10)/4 = 7` |
| `exact_fit_no_truncation` | unit | Budget=100 tokens, symbols totalling exactly 100 → `truncated=false`, `dropped_count=0` |
| `one_over_truncates` | unit | Budget=100 tokens, symbols totalling 101 → `truncated=true`, `dropped_count≥1` |
| `zero_available_returns_empty` | unit | `max_tokens=0, reserve_tokens=0` → `symbols=[]`, `truncated=false` (nothing to drop) |
| `reserve_tokens_reduces_available` | unit | `max_tokens=200, reserve_tokens=150` → `available()=50`; large symbol list truncates at 50 tokens |
| `chars_count_not_bytes` | unit | Symbol with non-ASCII sig (e.g. `"fn résumé()"`) → estimate uses `.chars().count()` not `.len()` |

---

### W-6 — SchemaHardener + NoOp CTX-05 path (5 tests)

**File:** `crates/kay-context/tests/hardener.rs`  
**What to test:** `SchemaHardener::harden()` idempotency, ForgeCode post-process applied, NoOpContextEngine still hardens schemas.

| Test | Type | Assertion |
|------|------|-----------|
| `harden_moves_required_before_properties` | unit | Input schema has `properties` before `required`; hardened schema has `required` first |
| `harden_is_idempotent` | unit | `harden(harden(schemas)) == harden(schemas)` |
| `harden_adds_truncation_reminder` | unit | Output schema contains truncation reminder field (per `TruncationHints::default()`) |
| `noop_engine_hardens_schemas` | unit | `NoOpContextEngine::retrieve()` returns `ContextPacket` with `hardened_schemas` non-empty when given non-empty schemas |
| `tool_registry_schemas_method` | unit | `ToolRegistry::schemas()` returns 1 `Value` per registered tool; each is a JSON object |

---

### W-7 — Integration: FileWatcher + E2E kay-cli (8 tests)

**File:** `crates/kay-context/tests/watcher.rs` (5) + `crates/kay-cli/tests/context_e2e.rs` (3)

| Test | File | Type | Assertion |
|------|------|------|-----------|
| `watcher_debounce_coalesces_events` | watcher.rs | integration | Write file 3× within 100ms → `invalidate()` called exactly once after 500ms |
| `watcher_triggers_on_create` | watcher.rs | integration | Create new `.rs` file in watched dir → `invalidate()` called |
| `watcher_triggers_on_modify` | watcher.rs | integration | Modify existing `.rs` file → `invalidate()` called |
| `watcher_triggers_on_remove` | watcher.rs | integration | Delete `.rs` file → symbols for that file removed from store |
| `watcher_ignores_non_source` | watcher.rs | integration | Write `.lock` file → no `invalidate()` call |
| `context_injected_into_system_prompt` | context_e2e.rs | E2E | `run_turn()` with populated store → system prompt contains `<context>` block |
| `truncated_event_emitted` | context_e2e.rs | E2E | Budget too small for all symbols → `AgentEvent::ContextTruncated` emitted via event stream |
| `noop_engine_backward_compat` | context_e2e.rs | E2E | Existing `run_turn()` call sites with `NoOpContextEngine::default()` compile and pass existing tests |

**Tooling for W-7:** `tempfile` for scratch dirs; `tokio::time::sleep` for debounce waits (500ms + buffer); `assert!` on event stream captures.

---

## Coverage Targets

| Area | Target | Rationale |
|------|--------|-----------|
| `kay-context` crate (new code) | ≥ 80% line coverage | All components are new — no legacy debt to exempt |
| `kay-tools/src/registry.rs` addition | 100% | Single new public method, trivial to cover |
| `kay-tools/src/events.rs` new variants | 100% via insta snapshots | Schema-stability requirement in module doc |
| `kay-cli/src/run.rs` injection point | covered by W-7 E2E | Integration path; not isolated |
| ForgeCode ported code | exempt | Upstream-tested; we only call it |

---

## What to Skip

- **SQLite WAL/journal behavior** — rusqlite's own test suite owns this
- **tree-sitter grammar correctness** — upstream grammar repos own this
- **OpenRouter API** — no network in CI; `OpenRouterEmbedder` tested via `FakeEmbedder` stand-in
- **forge_walker traversal** — tested in Phase 5; we only call `Walker::max_all().cwd(root).get().await`
- **`event_filter.rs`** — QG-C4 carry-forward: byte-identical, must NOT be touched

---

## TDD Iron Law (per spec)

Every wave follows RED → GREEN → commit:
1. Write failing test(s) — commit with `test(wN): RED — <description>`
2. Write minimal production code to pass — commit with `feat(ctx-NN): GREEN — <description>`
3. Refactor if needed — commit with `refactor(ctx): <description>`

DCO `Signed-off-by:` required on every commit.

---

## Snapshot Tests

Two insta snapshot tests required in `crates/kay-tools/tests/events_wire_snapshots.rs`:
- `snap_context_truncated_wire` — serialized `AgentEvent::ContextTruncated { dropped_symbols: 3, budget_tokens: 8192 }`
- `snap_index_progress_wire` — serialized `AgentEvent::IndexProgress { indexed: 100, total: 500 }`

Run `cargo insta review` after first GREEN to accept snapshots.

---

## Existing Coverage Impact

Phase 7 adds `kay-context` as a new crate. No existing test files are modified except:
- `crates/kay-tools/tests/events_wire_snapshots.rs` — 2 new snapshot tests added (additive)
- `crates/kay-tools/src/registry.rs` — 1 new method, covered by W-6
- `crates/kay-cli/src/run.rs` — injection point covered by W-7 E2E

**`event_filter.rs` and all Phase 5/6 test files remain byte-identical.**
