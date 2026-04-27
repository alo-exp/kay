---
phase: 3
plan: 05
wave: 4
subsystem: kay-tools
tags: [parity, delegation, image-read, task-complete, default-set, kay-cli, kira]
requires: [TOOL-01, TOOL-03, TOOL-04, TOOL-06, T-3-06, T-3-07]
provides:
  - ServicesHandle facade (Arc<dyn ServicesHandle>) wrapping four parity service trait objects
  - ForgeServicesFacade production impl emitting byte-identical ToolOutput at the service layer
  - 4 parity tool delegations (fs_read, fs_write, fs_search, net_fetch)
  - ImageReadTool (KIRA trio) with ImageQuota-backed cap enforcement
  - TaskCompleteTool (KIRA trio) emitting AgentEvent::TaskComplete
  - ImageQuota real impl (AtomicU32 per-turn + per-session with rollback-on-breach)
  - default_tool_set(project_root, quota) factory → 7-tool ToolRegistry
  - kay-cli boot::install_tool_registry() and `kay tools list` subcommand
affects:
  - crates/kay-tools/src/runtime/context.rs
  - crates/kay-tools/src/forge_bridge.rs
  - crates/kay-tools/src/quota.rs
  - crates/kay-tools/src/builtins/{fs_read,fs_write,fs_search,net_fetch,image_read,task_complete}.rs
  - crates/kay-tools/src/builtins/mod.rs
  - crates/kay-tools/src/default_set.rs
  - crates/kay-tools/src/lib.rs
  - crates/kay-tools/Cargo.toml
  - crates/kay-cli/Cargo.toml
  - crates/kay-cli/src/main.rs
  - crates/kay-cli/src/boot.rs
tech_stack_added: [base64 = "0.22"]
patterns_added:
  - service-layer parity (facade → individual service trait objects → format_* helpers) instead of ToolExecutor bundle
  - quota as shared Arc<ImageQuota> on ToolCallContext rather than field on each tool
  - pub format_* helpers for deterministic output so integration tests compare byte-for-byte
key_files_created:
  - crates/kay-tools/src/forge_bridge.rs
  - crates/kay-tools/tests/parity_delegation.rs
  - crates/kay-tools/tests/image_quota.rs
  - crates/kay-tools/tests/default_set_wiring.rs
  - crates/kay-cli/src/boot.rs
key_files_modified:
  - crates/kay-tools/src/runtime/context.rs (ServicesHandle got 4 async methods)
  - crates/kay-tools/src/quota.rs (real try_consume + counters + limit_for)
  - crates/kay-tools/src/builtins/{fs_read,fs_write,fs_search,net_fetch}.rs (stub → real thin adapters)
  - crates/kay-tools/src/builtins/image_read.rs (stub → real body w/ quota + emit + base64)
  - crates/kay-tools/src/builtins/task_complete.rs (stub → real body w/ verifier + emit)
  - crates/kay-tools/src/default_set.rs (todo!() → factory)
  - crates/kay-tools/src/lib.rs (re-exports ForgeServicesFacade, 7 tools, default_tool_set)
  - crates/kay-cli/src/main.rs (added `tools list` subcommand)
decisions:
  - Service-layer parity (not ToolExecutor parity) for Wave 4 — upgrade in Phase 5 when full ForgeServices bundle is available in kay-cli
  - ForgeConfig.image_read config NOT added — kay-cli hardcodes (2, 20) defaults for Phase 3, ForgeConfig integration deferred to Phase 5
  - default_tool_set takes 2 args (project_root, quota), not 5 — services/sandbox/verifier are per-turn ToolCallContext concerns
  - ImageQuota::try_consume checks PerTurn then PerSession atomically in a single call, not two — prevents in-flight reservation leaks
  - quota reservation is held across IO failures in image_read — explicit "budget, not success-counter" semantics pinned by test
metrics:
  duration: ~4 hours (wall clock, across session boundary)
  tasks_completed: 3
  commits: 3
  files_created: 5
  files_modified: 11
  tests_added: 13 (4 quota unit + 3 outcome_body/detect_mime unit + 5 parity + 5 image_quota + 3 default_set_wiring, minus overlap)
  tests_total_kay_tools: 62 unit + 22 integration all green
completed: 2026-04-21
---

# Phase 3 Plan 05 (Wave 4): Service Parity + KIRA Core Tools + Registry Wire-Up Summary

Wave 4 delivers the proof point that Phase 3 is observably complete: a single binary (`kay tools list`) that loads seven hardened-schema tools, each dispatching through an object-safe `Arc<dyn Tool>` registry, with byte-identical service-layer output to ForgeCode. The KIRA trio (`execute_commands` from Wave 3, plus `image_read` and `task_complete` added here) is wired end-to-end; the four parity tools delegate through a new `ServicesHandle` facade that preserves Wave-1's frozen `ToolCallContext` shape (B7/VAL-007).

## Tasks

### Task 1 — ServicesHandle facade + 4 parity tool delegations (`216e758`)

Extended the previously-empty `ServicesHandle` trait with four async methods (`fs_read`, `fs_write`, `fs_search`, `net_fetch`) and introduced `ForgeServicesFacade`, a production impl holding `Arc<dyn FsReadService>` + sibling service trait objects. Each facade method calls the concrete `forge_services::Forge*` service impl and renders the structured output via a free `format_*` helper into a deterministic `ToolOutput::text` body. Re-wrote the four parity built-ins as ~20-LOC thin adapters that deserialize their `FSRead`/`FSWrite`/`FSSearch`/`NetFetch` input, hand off to `ctx.services.<op>`, and map errors.

Integration test `tests/parity_delegation.rs` (5 tests, all green) compares the `Tool::invoke` path against a direct-service-call path byte-for-byte using `pretty_assertions::assert_eq`, and proves `Arc<dyn Tool>` object-safety for all four tools.

### Task 2 — ImageQuota real body + ImageReadTool + TaskCompleteTool (`17fe0d4`)

Replaced `ImageQuota::try_consume`'s `todo!()` with atomic PerTurn-then-PerSession cap checks plus rollback-on-breach; added `reset_turn()`, `per_turn_count()`, `per_session_count()`, `limit_for(scope)`. Wrote `ImageReadTool` (Kay-specific; ForgeCode has no `ToolCatalog::ImageRead`) that reserves a quota slot → resolves MIME from extension (jpg/jpeg/png/webp/gif) → `tokio::fs::read` → emits `AgentEvent::ImageRead { path, bytes }` with raw bytes → returns `data:<mime>;base64,<blob>`. Wrote `TaskCompleteTool` that calls `ctx.verifier.verify(summary)`, emits `AgentEvent::TaskComplete { call_id, verified, outcome }` with `verified = matches!(outcome, Pass {..})` (Phase 3 NoOpVerifier locks this to `false` — T-3-06 invariant), and returns an outcome-derived body.

Integration test `tests/image_quota.rs` (5 tests, all green) covers per-turn breach, per-session breach across `reset_turn()`, event-emission with raw bytes, missing-file IO error, and unsupported-extension `InvalidArgs`.

### Task 3 — default_tool_set factory + kay-cli wire-up (`7ed458c`)

Replaced `default_set`'s `todo!()` with a two-arg factory (`project_root`, `Arc<ImageQuota>`) that constructs each built-in once and registers seven `Arc<dyn Tool>`. Added `kay-cli::boot::install_tool_registry()` and a `kay tools list` subcommand that prints each tool's name + description via `ToolRegistry::tool_definitions()`, proving the hardened-schema round-trip end-to-end.

Integration test `tests/default_set_wiring.rs` (3 tests, all green) locks the 7-tool name set, asserts every schema has `additionalProperties: false` + non-empty `required`, and confirms factory determinism across repeated calls.

## Deviations from Plan (Rule-3 Reconciliations)

All six Rule-3 reconciliations below were applied without user prompt, documented in source module docs + commit messages, and covered by at least one test.

### 1. [Rule 3 — Scaffold mismatch] Service-layer parity instead of ToolExecutor parity

- **Found during:** Task 1 pre-work, reading `forge_app::ToolExecutor::execute` signature.
- **Issue:** The 03-05 plan text mandated delegating through `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)`, which requires the full `forge_app::Services` trait bundle (25+ associated types + `Clone`). `Services` is NOT dyn-compatible; its only concrete impl (`forge_services::ForgeServices`) needs a deep infrastructure stack (snapshots, validation, auth, providers) that Phase 5 owns.
- **Fix:** Pivoted to service-layer parity. `ForgeServicesFacade` holds individual service trait objects (which ARE dyn-safe) and renders structured outputs via deterministic `format_*` helpers. Phase 5 will swap the facade body to call `ToolExecutor::execute` once the bundle lands in kay-cli; tools are unchanged.
- **Files:** `crates/kay-tools/src/forge_bridge.rs` (module doc block).
- **Commit:** `216e758`.

### 2. [Rule 3 — Plan vs. scaffold] ServicesHandle trait got its methods this wave

- **Found during:** Task 1 setup.
- **Issue:** Wave 1 (03-01) deferred `ServicesHandle` method bodies — the trait was empty, deferred to Wave 4 per the scaffold-compiles invariant.
- **Fix:** Added four methods in the same commit that introduced the facade. No breaking change to the scaffold; `ToolCallContext`'s shape is unchanged.
- **Files:** `crates/kay-tools/src/runtime/context.rs`.
- **Commit:** `216e758`.

### 3. [Rule 3 — Test plumbing] format_* helpers made `pub`

- **Found during:** Task 1 parity test authoring.
- **Issue:** Integration tests need to invoke the same format helpers the facade uses — otherwise "parity" is circular (`format_read_output(x) == format_read_output(x)`). `pub(crate)` blocks external-to-crate access.
- **Fix:** Promoted `format_read_output`, `format_write_output`, `format_search_match`, `format_search_output`, `format_fetch_output` to `pub`. The helpers are already small, deterministic free functions — no internal contract surface area increase.
- **Files:** `crates/kay-tools/src/forge_bridge.rs`.
- **Commit:** `216e758`.

### 4. [Rule 3 — API simplification] ImageQuota::try_consume takes NO scope argument

- **Found during:** Task 2 quota design.
- **Issue:** Plan text specified `try_consume(CapScope::Turn)` followed by `try_consume(CapScope::Session)` — two sequential calls. That pattern leaks a per-turn reservation if the per-session call then fails.
- **Fix:** Single atomic `try_consume()` that checks PerTurn first (tighter window), then PerSession, and rolls back all in-flight increments on breach. Tool callers handle the returned `CapScope` and pair it with `quota.limit_for(scope)` to build a `ToolError::ImageCapExceeded`.
- **Files:** `crates/kay-tools/src/quota.rs`, `crates/kay-tools/src/builtins/image_read.rs`.
- **Commit:** `17fe0d4`.

### 5. [Rule 3 — Scope boundary] ForgeConfig.image_read field NOT added

- **Found during:** Task 2 integration design.
- **Issue:** Plan text said to add `image_read: ImageReadConfig { max_per_turn, max_per_session }` to `forge_config::ForgeConfig`. But caps are a Kay-level concern (the agent-harness budget), not a ForgeCode-level one. Threading them through `ForgeConfig` couples the fork point unnecessarily and introduces a new schema field that Phase 5 (when we wire real config) will probably repath anyway.
- **Fix:** `kay-cli::boot::install_tool_registry()` hardcodes the D-07 reference defaults `(2, 20)` in `const` at the kay-cli layer. Phase 5 can move these into a Kay-owned config type without touching forge_config.
- **Files:** `crates/kay-cli/src/boot.rs`.
- **Commit:** `7ed458c`.

### 6. [Rule 3 — Factory simplification] default_tool_set takes 2 args, not 5

- **Found during:** Task 3 factory design.
- **Issue:** Plan text signature was `default_tool_set(services, sandbox, verifier, project_root, quota)` — five args. But `services`, `sandbox`, and `verifier` belong on `ToolCallContext` (they vary per-turn); they are NOT per-tool construction inputs.
- **Fix:** Factory takes only the two arguments that are genuinely per-tool (`project_root` for the shell tool's relative-path resolution; `Arc<ImageQuota>` for image_read's caps). The context-owned seams flow in separately via `ToolCallContext::new`.
- **Files:** `crates/kay-tools/src/default_set.rs` (module doc block documents the rationale).
- **Commit:** `7ed458c`.

## Authentication Gates

None encountered. This plan is pure Rust + in-process tests; no external services or credentials touched.

## Test Posture

- **Unit tests:** 62 passing across kay-tools (4 quota, 3 image_read, 3 task_complete, 1 default_set, existing 51).
- **Integration tests (kay-tools/tests):** 22 passing — parity_delegation (5), image_quota (5), default_set_wiring (3), registry_integration (4), schema_hardening_property (2), pty_integration (2), timeout_cascade (1).
- **Binary smoke test:** `cargo run -p kay-cli -- tools list` emits all 7 tools with hardened descriptions.
- **Clippy:** `cargo clippy -p kay-tools -p kay-cli --all-targets -- -D warnings` clean.

## Out-of-Scope Issues (Deferred)

- `forge_domain` test-target compile error in `conversation_html.rs` — references `forge_test_kit::json_fixture` behind a disabled `json` feature. Pre-existing on base branch (verified via stash), unrelated to Wave 4 changes. Filed for phase-boundary cleanup.

## Self-Check: PASSED

- Created `crates/kay-tools/src/forge_bridge.rs` — exists.
- Created `crates/kay-tools/tests/parity_delegation.rs` — exists (5 tests green).
- Created `crates/kay-tools/tests/image_quota.rs` — exists (5 tests green).
- Created `crates/kay-tools/tests/default_set_wiring.rs` — exists (3 tests green).
- Created `crates/kay-cli/src/boot.rs` — exists.
- Commit `216e758` — present in `git log --oneline`.
- Commit `17fe0d4` — present in `git log --oneline`.
- Commit `7ed458c` — present in `git log --oneline`.
- `cargo run -p kay-cli -- tools list` emits exactly 7 tools, names match the reference set.
