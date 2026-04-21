# Phase 6 Dependencies — Session Store + Transcript

> **Date:** 2026-04-22
> **Phase:** 6 — Session Store + Transcript
> **Produced by:** PATH 9 — ANALYZE-DEPS (autonomous §10e)
> **Consumed by:** gsd-plan-phase (PLAN.md must not diverge from this map)

---

## §1 — New Workspace Member

Add `crates/kay-session` to the workspace `members` list in `Cargo.toml` (root):

```toml
# In [workspace].members — add after "crates/kay-tools":
"crates/kay-session",
```

**Invariant:** `kay-session` is a library crate (`lib.rs`) with no binary. The `kay` binary lives exclusively in `kay-cli`.

---

## §2 — `crates/kay-session/Cargo.toml` (new file)

### §2.1 Production dependencies

| Dep | Source | Version/path | Rationale |
|-----|--------|-------------|-----------|
| `rusqlite` | **local** (not workspace) | `{ version = "0.32", features = ["bundled"] }` | SQLite via bundled C; avoids system libsqlite3 on CI. Only `kay-session` uses rusqlite — no need to promote to workspace yet. |
| `kay-tools` | intra-workspace | `{ path = "../kay-tools" }` | `AgentEventWire` — the serde struct that emits one JSONL line per event. Reused directly; no new serialization layer. |
| `uuid` | workspace | `{ workspace = true }` | Session IDs as UUID v4 |
| `chrono` | workspace | `{ workspace = true }` | `start_time`, `end_time` timestamps (serde-enabled) |
| `dirs` | workspace | `{ workspace = true }` | `dirs::home_dir()` for `kay_home()` default (`~/.kay`) |
| `serde` | workspace | `{ workspace = true }` | Struct derive for `SessionMeta`, `ExportManifest` |
| `serde_json` | workspace | `{ workspace = true }` | JSONL serialization + manifest.json write |
| `thiserror` | workspace | `{ workspace = true }` | Typed error enums (`SessionError`, `PathTraversalRejected`, `TranscriptDeleted`, etc.) |
| `sha2` | workspace | `{ workspace = true }` | SHA-256 of snapshot bytes for integrity check (T-5) |
| `tracing` | workspace | `{ workspace = true }` | Structured logging for open/resume/append operations |

### §2.2 Dev-only dependencies

| Dep | Source | Version | Rationale |
|-----|--------|---------|-----------|
| `tempfile` | workspace | `{ workspace = true }` | All tests use `TempDir`; no hardcoded paths |
| `insta` | workspace | `{ workspace = true }` | Snapshot assertions for manifest JSON shape |
| `assert_cmd` | workspace | `{ workspace = true }` | CLI E2E tests in T-8 (subprocess harness) |
| `predicates` | workspace | `{ workspace = true }` | Predicate combinators for assert_cmd output |
| `proptest` | **local** | `"1"` | Property-based tests T-2p, T-7p; not yet workspace-level |
| `trybuild` | **local** | `"1.0"` | Compile-fail canaries T-9; not yet workspace-level |

---

## §3 — Changes to Existing Crates

### §3.1 `crates/kay-cli/Cargo.toml` — add `kay-session` dep

```toml
[dependencies]
# Session persistence (Phase 6: SESS-01..05 + CLI-02)
kay-session = { path = "../kay-session" }
```

This dep is added under the `# Kay core + tools (intra-workspace)` block. No version pin — path dep from the same workspace.

**Files in `kay-cli` that will be modified in W-7:**
- `src/run.rs` — event-tap fan-out in the drain loop; `kay-session::Session::append_event` called for each `AgentEvent`
- `src/main.rs` — `--session`, `--resume`, subcommand dispatch for `kay session *` and `kay rewind`

**Invariant:** `kay-cli` gains `kay-session`; `kay-session` does NOT depend on `kay-cli`. The dependency graph remains a DAG.

### §3.2 No changes to `crates/kay-core/`

`kay-core/src/event_filter.rs` is **untouched**. QG-C4 coverage gate stays green. The event-tap pattern means the drain loop in `kay-cli` is the only subscriber — `run_turn` and `kay-core` are unmodified.

---

## §4 — Dependency Invariants (planner must enforce)

| # | Invariant | Violation = BLOCK |
|---|-----------|-------------------|
| I-1 | `kay-session` ← `kay-tools` (one-way). `kay-tools` must NEVER import `kay-session`. | Circular dep breaks compile |
| I-2 | `kay-session` does NOT depend on `kay-core`. Session storage is pure I/O; no agent-loop types leak in. | Coupling breaks testability |
| I-3 | `rusqlite` uses `features = ["bundled"]`. The `bundled` feature is MANDATORY for Windows CI (no system SQLite). | Windows CI will break without it |
| I-4 | All `kay-session` tests use `tempfile::TempDir` for the session store root. No test writes to `~/.kay` directly. | Tests pollute real user state |
| I-5 | `event_filter.rs` is not modified in any wave. Any wave task that touches `crates/kay-core/` must be rejected. | QG-C4 CI gate fails |
| I-6 | `proptest` and `trybuild` stay as local dev-deps in `kay-session`. Do NOT promote to workspace yet. | Premature generalization |

---

## §5 — Wave Dependency DAG

Sequential — each wave depends on the previous:

```
W-1 (SessionStore::open + SQLite schema v1)
  ↓
W-2 (Session::append_event + JSONL write + last-line recovery)
  ↓
W-3 (SQLite CRUD: create, list, close, resume)
  ↓
W-4 (record_snapshot + byte cap + LRU eviction)
  ↓
W-5 (fork_session + parent_id FK)
  ↓
W-6 (export_session + import_session + replay)
  ↓
W-7 (kay session * CLI + kay rewind — wires everything)
```

**All waves are sequential.** No parallel execution: each wave's GREEN gate produces types consumed by the next wave.

---

## §6 — CI Impact

| Concern | Action |
|---------|--------|
| 3-OS matrix (macOS-14/ubuntu/windows) | No change — existing matrix covers `kay-session` automatically once declared as workspace member |
| `coverage-event-filter` CI job | No change — `event_filter.rs` is untouched; job stays green |
| New coverage job for `kay-session`? | No new job in Phase 6. `kay-session` coverage is collected by the existing `cargo llvm-cov` run; the 90% line target is enforced by code review, not a separate CI gate (deferred to Phase 7 if needed) |
| Windows `rusqlite` bundled | `features = ["bundled"]` compiles SQLite from source — no system dep, no CI install step needed |
| `proptest` determinism | `proptest` uses a fixed seed by default in CI (`PROPTEST_CASES` env var can tune iter count) — no CI config change needed |
