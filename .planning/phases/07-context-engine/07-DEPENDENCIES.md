# Phase 7: Context Engine — Dependencies

**Date:** 2026-04-22
**Phase:** 07-context-engine
**Analysis:** gsd-analyze-dependencies

---

## Phase Dependency Analysis

### Phase 7: Context Engine

**Scope:** New `kay-context` crate (10 source files) + `RunTurnArgs` extension + two new `AgentEvent` variants + `ToolRegistry::schemas()` method + rusqlite workspace promotion.

**Files modified by Phase 7:**

| File | Phase that created/owns it | Dependency type |
|------|---------------------------|-----------------|
| `Cargo.toml` (workspace.dependencies) | Phase 1 scaffold | file overlap |
| `crates/kay-session/Cargo.toml` | Phase 6 | file overlap → **Depends on Phase 6** |
| `crates/kay-core/src/loop.rs` | Phase 5 | semantic → **Depends on Phase 5** |
| `crates/kay-cli/src/run.rs` | Phase 5 | semantic → **Depends on Phase 5** |
| `crates/kay-tools/src/events.rs` | Phase 5 | semantic → **Depends on Phase 5** |
| `crates/kay-tools/src/events_wire.rs` | Phase 5 | semantic → **Depends on Phase 5** |
| `crates/kay-tools/src/registry.rs` | Phase 3 | semantic (additive) |
| `crates/kay-context/**` | NEW (Phase 7) | no prior dependency |

---

## Detected Dependencies

### → Depends on Phase 5 (Agent Loop + Canonical CLI)

**Reason (semantic + file overlap):**
1. `crates/kay-core/src/loop.rs::RunTurnArgs` — Phase 5 created this struct. Phase 7 adds 3 fields (`context_engine`, `context_budget`, `initial_prompt`). Phase 5 must be merged first.
2. `crates/kay-cli/src/run.rs` — Phase 5 created the `RunTurnArgs` construction site. Phase 7 adds `NoOpContextEngine` + `ContextBudget` + prompt fields.
3. `crates/kay-tools/src/events.rs` — Phase 5 added `Paused` + `Aborted` variants. Phase 7 adds `ContextTruncated` + `IndexProgress`. Requires Phase 5's enum shape to be final.
4. `crates/kay-tools/src/events_wire.rs` — Phase 5 established the `AgentEventWire` borrowing-newtype + hand-written `Serialize` pattern. Phase 7 adds match arms following this exact pattern.
5. **QG-C4 carry-forward** — `event_filter.rs` must be byte-identical. This constraint originates in Phase 5 and must be respected in Phase 7.

**Status:** ✅ SATISFIED — Phase 5 merged to main at `95412f0` (2026-04-22)

---

### → Depends on Phase 6 (Session Store + Transcript)

**Reason (file overlap):**
1. `crates/kay-session/Cargo.toml` — Phase 6 created this file with `rusqlite = { version = "0.38", features = ["bundled"] }` as a crate-local dep. Phase 7 modifies this line to `rusqlite = { workspace = true }` as part of the workspace promotion.
2. The rusqlite workspace promotion requires Phase 6's `kay-session` crate to already exist in the workspace with the crate-local dep.

**Status:** ✅ SATISFIED — Phase 6 merged to main at `793317c` (2026-04-22)

---

### → Phase 3 (Tool Registry + KIRA Core Tools) — additive only

**Reason (additive, not blocking):**
- `crates/kay-tools/src/registry.rs` — Phase 3 created `ToolRegistry`. Phase 7 adds `schemas()` method (one new public method, no modifications to existing methods). Phase 3 is already merged; this is purely additive.

**Status:** ✅ SATISFIED — Phase 3 merged (v0.1.1)

---

## Downstream Impact

### Phase 8 depends on Phase 7

Phase 8 (Multi-Perspective Verification) uses the `ContextEngine` trait and `ContextPacket` that Phase 7 introduces. ROADMAP.md already shows `**Depends on**: Phase 5, Phase 7` for Phase 8 ✅.

---

## Intra-Phase Wave DAG (for planner)

Phase 7's 7 waves have internal dependencies:

```
W-1 (SymbolStore CRUD) ──────────────────────────────────┐
                                                          ├──► W-7 (FileWatcher + E2E)
W-2 (Indexer) ──────────────────────────────────────────┐│
                                                          ││
W-3 (FTS5 Retriever) ──────────────────────────────────┐ ├┘
W-4 (sqlite-vec + RRF) ──► requires W-1 (store schema) ┘ │
W-5 (ContextBudget) ────────────────────────────────────┘ │
W-6 (SchemaHardener) ───────────────────────────────────┘ │
                                                           │
Task 0 (Workspace prep) → ALL waves (must run first)
Task 1 (AgentEvent variants) → W-7 E2E tests
Task 2 (ToolRegistry.schemas()) → W-6 hardener tests
Task 10 (CLI wiring) → W-7 E2E tests
```

**Critical ordering:**
- Task 0 (workspace Cargo.toml changes) must run before any `cargo build` in waves
- W-4 requires W-1 to have created the `symbols` table and `enable_vector_search()` API
- W-7 integration tests require all prior waves to be GREEN

---

## Summary

```
Suggested ROADMAP.md updates:
  Phase 7: change "Depends on: Phase 5 (parallelizable with Phase 6)"
           → "Depends on: Phase 5, Phase 6"   ✅ APPLIED
  Phase 8: "Depends on: Phase 5, Phase 7"     ✅ already correct — no change needed
```

ROADMAP.md updated. Phase 7 may proceed to planning.
