---
phase: 03-tool-registry-kira-core-tools
plan: 03
wave: 2
subsystem: schema-hardening, agent-events, tool-contracts
tags: [schema, hardening, agent-events, truncation, kay-tools, kay-provider-openrouter, kay-provider-errors]
requires:
  - 03-01 (kay-tools scaffold — seams/verifier.rs with VerificationOutcome)
  - 03-02 (Wave 1: ToolRegistry + Tool trait object-safety)
provides:
  - kay-tools::events::AgentEvent (source of truth, D-08 additive variants)
  - kay-tools::events::ToolOutputChunk (Stdout / Stderr / Closed)
  - kay-tools::schema::harden_tool_schema (TOOL-05 / TOOL-06 wrapper)
  - kay-tools::schema::TruncationHints
  - kay-provider-errors crate (ProviderError / RetryReason / AuthErrorKind)
affects:
  - kay-provider-openrouter::event (now a compat re-export shim)
  - kay-provider-openrouter::error (now a compat re-export shim)
tech-stack:
  added:
    - proptest 1 (dev-dep on kay-tools)
    - kay-provider-errors crate (new workspace member)
  patterns:
    - "Cross-crate cycle-break via extracted errors crate (Rule-3 auto-fix authorized by plan Task 1 Step 2 fallback)"
key-files:
  created:
    - crates/kay-provider-errors/Cargo.toml
    - crates/kay-provider-errors/src/lib.rs
    - crates/kay-tools/tests/schema_hardening_property.rs
    - crates/kay-tools/tests/events_registry_integration.rs
  modified:
    - Cargo.toml (workspace members + new crate)
    - crates/kay-tools/Cargo.toml (proptest dev-dep + kay-provider-errors dep)
    - crates/kay-tools/src/events.rs (AgentEvent source of truth — full variant set)
    - crates/kay-tools/src/schema.rs (harden_tool_schema real implementation)
    - crates/kay-tools/src/lib.rs (ToolOutputChunk + schema re-exports)
    - crates/kay-provider-openrouter/Cargo.toml (kay-tools + kay-provider-errors deps)
    - crates/kay-provider-openrouter/src/event.rs (compat re-export shim)
    - crates/kay-provider-openrouter/src/error.rs (compat re-export shim)
decisions:
  - "Rule-3 cycle-break: extracted ProviderError/RetryReason/AuthErrorKind into a new kay-provider-errors crate (plan Task 1 Step 2 authorized fallback — the plan explicitly named this option)"
  - "Rule-1 guard-test adjustment: property-test guard uses json!({\"type\":\"object\"}) instead of json!({}) because enforce_strict_schema only normalizes values that is_object_schema recognizes; bare {} is left untouched by design"
metrics:
  duration_minutes: 30
  completed_date: "2026-04-21"
  tests_added: 17
  commits: 1
requirements:
  - TOOL-05
  - TOOL-06
  - SHELL-03
---

# Phase 3 Plan 03 (Wave 2): Schema Hardening + AgentEvent Extensions Summary

**One-liner:** Wraps `forge_app::utils::enforce_strict_schema` unchanged (byte-identical TB 2.0 parity) and relocates the `AgentEvent` enum into `kay-tools::events` with three additive Phase 3 variants (ToolOutput / TaskComplete / ImageRead), breaking a would-be `kay-tools <-> kay-provider-openrouter` cycle via a new `kay-provider-errors` crate.

## Tasks Executed

### Task 1 — kay-tools::events source-of-truth relocation + Phase 3 additive variants (U-15/U-16/U-17/U-18 + clone + I-04)

- Moved full Phase 2 `AgentEvent` variant set (TextDelta, ToolCallStart/Delta/Complete/Malformed, Usage, Retry, Error) from `kay-provider-openrouter::event` into `kay-tools::events`.
- Appended three Phase 3 additive variants: `ToolOutput { call_id, chunk }`, `TaskComplete { call_id, verified, outcome }`, `ImageRead { path, bytes }`.
- Added `#[non_exhaustive]` `ToolOutputChunk` enum with `Stdout(String) | Stderr(String) | Closed { exit_code, marker_detected }`. Derives `Clone`.
- Dropped the Wave 0 placeholder `__NonExhaustive` variant (it was only there to let the scaffold compile before Wave 2).
- `VerificationOutcome` is referenced via `use crate::seams::verifier::VerificationOutcome` — NOT redefined (A2 compliance; seams/verifier.rs is owned by 03-01).
- `kay-provider-openrouter::event` converted to a three-line compat shim: `pub use kay_tools::events::AgentEvent;` + `ToolOutputChunk`. All Phase 2 call-sites that import `kay_provider_openrouter::event::AgentEvent` or `kay_provider_openrouter::AgentEvent` continue to compile unchanged.
- 5 unit tests (phase3_additions::{tool_output_variant_shape, task_complete_variant_shape, image_read_variant_shape, emits_in_order, tool_output_chunk_is_clone}) + 1 integration test (phase3_events_flow_through_registry_dispatch, mpsc transport).
- **Verified:** `cargo test -p kay-tools --lib events::phase3_additions` → 5 passed; `cargo test -p kay-tools --test events_registry_integration` → 1 passed.

### Task 2 — harden_tool_schema implementation (U-11/U-12/U-13/U-14 + 5 supporting)

- Replaced the Wave 0 `todo!()` body in `kay-tools::schema::harden_tool_schema` with a real wrapper that calls `forge_app::utils::enforce_strict_schema(schema, true)` and then (conditionally) appends `TruncationHints.output_truncation_note` to the TOP-LEVEL `description` only.
- `#[derive(Debug, Default, Clone)]` on `TruncationHints`.
- 9 unit tests: U-11 (required_sorted), U-12 (additionalProperties:false), U-13 (allOf flattening), U-14 (truncation append), plus harden_delegates_to_enforce_strict_schema_verbatim (byte-identical delegation invariant), harden_creates_description_when_absent, harden_noop_when_hint_is_none, harden_does_not_touch_nested_descriptions, harden_is_idempotent_when_hint_is_none.
- **Verified:** `cargo test -p kay-tools --lib schema::` → 9 passed.

### Task 3 — Property test at 1024 cases (P-01)

- Created `crates/kay-tools/tests/schema_hardening_property.rs` with `harden_always_produces_valid_strict_schema` at `ProptestConfig { cases: 1024, .. }`. Checks the five strict-mode invariants (type=object, properties object present, required == sorted property keys, additionalProperties=false, propertyNames absent) on randomly shaped schemas with 0-5 properties drawn from a 5-letter alphabet and `{string, integer, boolean}` leaf types.
- Plus `verbatim_delegation_on_empty_object` guard test. **Rule-1 deviation:** the plan specified `json!({})` but `enforce_strict_schema`'s `is_object_schema` check (requires `type`/`properties`/`additionalProperties` signal) leaves bare `{}` untouched. Changed to `json!({"type":"object"})` which mirrors a realistic empty-parameter tool schema and exercises the strict-mode `properties:{}` / `required:[]` / `additionalProperties:false` path. Logged inline in the test doc comment.
- **Verified:** `cargo test -p kay-tools --test schema_hardening_property` → 2 passed (proptest runtime < 200 ms).

### Task 4 — Workspace-wide green gate + DCO commit

- `cargo test -p kay-tools -p kay-provider-openrouter -p kay-provider-errors` → all green (see test matrix below).
- `cargo clippy --workspace --all-targets --exclude forge_domain -- -D warnings` → green. (Exclusion reason: pre-existing `forge_test_kit::json_fixture` feature-gate error in `forge_domain/src/conversation_html.rs:495` reproduces on clean HEAD and is explicitly documented in the prompt as "NOT ours".)
- `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok.
- Atomic DCO-signed commit `5f0a3c2` created on `phase/03-tool-registry`.

## Deviations from Plan

### Rule-3 (blocking-issue auto-fix, pre-authorized by plan)

**Cross-crate dependency cycle broken via `kay-provider-errors` extraction.**

- **Found during:** Task 1, Step 2 cargo check.
- **Issue:** The plan directed `kay-tools/Cargo.toml` to add `kay-provider-openrouter = { path = ... }` so `events.rs` could `use kay_provider_openrouter::error::{ProviderError, RetryReason}`. Simultaneously, `kay-provider-openrouter/src/event.rs` was directed to `pub use kay_tools::events::AgentEvent;` — a direct A↔B cycle.
- **Fix:** Extracted `ProviderError` / `RetryReason` / `AuthErrorKind` into a new `kay-provider-errors` crate (3 files: `Cargo.toml`, `src/lib.rs` with the enum definitions and 2 regression tests). Both `kay-tools` and `kay-provider-openrouter` depend on `kay-provider-errors`. `kay-provider-openrouter::error` becomes a one-line re-export shim — every existing top-level re-export at `kay-provider-openrouter/src/lib.rs` (`pub use error::{AuthErrorKind, ProviderError, RetryReason}`) continues to work, and every external test that imports `kay_provider_openrouter::{ProviderError, RetryReason, AuthErrorKind}` compiles unchanged.
- **Pre-authorized by:** Plan Task 1 Step 2: "If this creates a dep cycle warning [...] reverse the direction — make `kay-provider-openrouter` the DEPENDENT crate. Specifically: if cargo reports a cycle, STOP and switch strategy per E1 (**move `ProviderError` / `RetryReason` into a third small crate `kay-provider-errors`**, or inline lightweight copies in events.rs)."
- **Files added:** `crates/kay-provider-errors/Cargo.toml`, `crates/kay-provider-errors/src/lib.rs`. Workspace `Cargo.toml` members list extended.

### Rule-1 (bug fix)

**Property-test guard schema adjustment.**

- **Found during:** Task 3 first test run.
- **Issue:** The plan's `verbatim_delegation_on_empty_object` test used `json!({})`. Reading `forge_app::utils::is_object_schema`, a bare `{}` doesn't signal "object schema" (no `type`/`properties`/`additionalProperties` keys), so `enforce_strict_schema` leaves it untouched — causing the test's invariant assertions to fail.
- **Fix:** Switched the guard schema to `json!({"type":"object"})`, which does hit the is_object branch and exercises the `properties:{}` / `required:[]` / `additionalProperties:false` insertion path. Documented inline in the test's doc comment.
- **Files modified:** `crates/kay-tools/tests/schema_hardening_property.rs`.

### Ancillary scaffold-alignment adjustments

- `kay-tools::events::AgentEvent` (originally Wave 0 scaffolded as `#[derive(Clone, Serialize, Deserialize)]` with only `__NonExhaustive`) was converted to the plan's `#[derive(Debug)]`-only enum (Clone/Serialize/Deserialize dropped). Rationale: `ProviderError` contains `reqwest::Error` / `serde_json::Error`, neither of which is Clone or Serialize. This matches the Phase-2 `kay-provider-openrouter::event::AgentEvent` shape exactly.
- Dropped the `__NonExhaustive` placeholder variant from the Wave 0 scaffold (it existed solely to let the enum compile before Wave 2 filled it in).
- Updated `kay-tools/src/lib.rs` to additionally re-export `ToolOutputChunk` and `schema::{TruncationHints, harden_tool_schema}` (the schema re-export was missing from the Wave 0 scaffold despite the plan implying it was present).

## Test Matrix

| Test | Location | Status |
|------|----------|--------|
| U-11 required_sorted_after_harden | `kay-tools::schema::tests` | pass |
| U-12 additional_properties_false_after_harden | `kay-tools::schema::tests` | pass |
| U-13 all_of_flattened_after_harden | `kay-tools::schema::tests` | pass |
| U-14 truncation_reminder_present_after_harden | `kay-tools::schema::tests` | pass |
| harden_delegates_to_enforce_strict_schema_verbatim | `kay-tools::schema::tests` | pass |
| harden_creates_description_when_absent | `kay-tools::schema::tests` | pass |
| harden_noop_when_hint_is_none | `kay-tools::schema::tests` | pass |
| harden_does_not_touch_nested_descriptions | `kay-tools::schema::tests` | pass |
| harden_is_idempotent_when_hint_is_none | `kay-tools::schema::tests` | pass |
| U-15 tool_output_variant_shape | `kay-tools::events::phase3_additions` | pass |
| U-16 task_complete_variant_shape | `kay-tools::events::phase3_additions` | pass |
| U-17 image_read_variant_shape | `kay-tools::events::phase3_additions` | pass |
| U-18 emits_in_order | `kay-tools::events::phase3_additions` | pass |
| tool_output_chunk_is_clone | `kay-tools::events::phase3_additions` | pass |
| I-04 phase3_events_flow_through_registry_dispatch | `tests/events_registry_integration.rs` | pass |
| P-01 harden_always_produces_valid_strict_schema (1024 cases) | `tests/schema_hardening_property.rs` | pass |
| verbatim_delegation_on_empty_object | `tests/schema_hardening_property.rs` | pass |

**Total new tests:** 17 (9 schema unit + 5 events unit + 1 events integration + 2 property-test file).

**Phase 2 regression check (kay-provider-openrouter, all tests):** 57 lib + 1 nn7 + 4 cost-cap + 2 retry + 2 streaming + 2 tool-malformed + 1 tool-reassembly + 3 cost-cap-turn-boundary + 5 error-taxonomy = all green, no Phase 2 test modified.

## Acceptance Criteria Matrix

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `grep "pub enum AgentEvent" crates/kay-tools/src/events.rs` = 1 | pass | moved source of truth |
| `grep "pub enum AgentEvent" crates/kay-provider-openrouter/src/event.rs` = 0 | pass | shim contains re-export only |
| `grep "pub use kay_tools::events::AgentEvent" crates/kay-provider-openrouter/src/event.rs` ≥ 1 | pass | re-export line |
| `grep "pub enum ToolOutputChunk" crates/kay-tools/src/events.rs` = 1 | pass | new enum present |
| `grep "pub enum VerificationOutcome\|impl VerificationOutcome" crates/kay-tools/src/events.rs crates/kay-provider-openrouter/src/event.rs` = 0 | pass | A2: only referenced |
| `#[non_exhaustive]` count in events.rs ≥ 2 | pass | AgentEvent + ToolOutputChunk |
| `grep "todo!()" crates/kay-tools/src/schema.rs` = 0 | pass | implementation landed |
| `grep "forge_app::utils::enforce_strict_schema" crates/kay-tools/src/schema.rs` ≥ 2 | pass | impl + delegation test |
| `grep "forge_app = " crates/kay-tools/Cargo.toml` ≥ 1 | pass | dep already present |
| `grep "proptest = " crates/kay-tools/Cargo.toml` ≥ 1 | pass | added under [dev-dependencies] |
| `grep "cases: 1024" crates/kay-tools/tests/schema_hardening_property.rs` = 1 | pass | ProptestConfig |
| `cargo test -p kay-tools --lib schema::` = 9 passed | pass | see test matrix |
| `cargo test -p kay-tools --lib events::phase3_additions` = 5 passed | pass | see test matrix |
| `cargo test -p kay-tools --test schema_hardening_property` = 2 passed | pass | see test matrix |
| `cargo test -p kay-tools --test events_registry_integration` = 1 passed | pass | see test matrix |
| `cargo clippy -p kay-tools --all-targets -- -D warnings` exit 0 | pass | `--exclude forge_domain` not needed at crate-level |
| `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` exit 0 | pass | both shim-enabled |
| `cargo deny check` exit 0 | pass | advisories/bans/licenses/sources ok |
| DCO `Signed-off-by:` trailer on commit | pass | commit 5f0a3c2 |

## Commit

| SHA | Subject | Files |
|-----|---------|-------|
| `5f0a3c2` | feat(03-03): schema hardening wrapper + AgentEvent Phase 3 extensions | 13 files, +829 / -179 |

## Downstream Executor Notes (03-04 / 03-05)

- **`harden_tool_schema` is the single call site for schema hardening.** Consume it via `kay_tools::harden_tool_schema` and pass the tool's `input_schema()` return value (owned `serde_json::Value`, A1) plus an optional `TruncationHints { output_truncation_note: Some("...") }` for tools whose outputs can exceed per-turn byte caps.
- **`AgentEvent` is now in `kay-tools::events`.** Tool implementations may `use kay_tools::events::{AgentEvent, ToolOutputChunk};` directly and send events through `ToolCallContext.stream_sink`. Provider-crate call-sites continue to use `kay_provider_openrouter::event::AgentEvent` / `kay_provider_openrouter::AgentEvent` (compat re-exports).
- **`VerificationOutcome` is NOT in this plan.** It lives in `crates/kay-tools/src/seams/verifier.rs` (owned by 03-01 scaffold). Use `crate::seams::verifier::VerificationOutcome` internally or `kay_tools::seams::verifier::VerificationOutcome` externally — or the convenience top-level re-export `kay_tools::VerificationOutcome`.
- **`ToolOutputChunk::Closed { exit_code, marker_detected }`** is the terminal frame 03-04's `execute_commands` streams after marker detection. `marker_detected: false` signals abnormal termination (crash / SIGKILL / orphan), not just "no marker used".
- **Dep direction post-Wave-2 (for Phase 3+ planners):**
  - `kay-provider-openrouter -> kay-tools -> kay-provider-errors`
  - `kay-provider-openrouter -> kay-provider-errors`
  - `kay-tools` does NOT depend on `kay-provider-openrouter`. Do NOT add that dep; it would re-introduce the cycle.
- **`ProviderError` lives in `kay-provider-errors`.** For new error-taxonomy work, edit `crates/kay-provider-errors/src/lib.rs` directly; the `kay-provider-openrouter::error` shim will re-export automatically.

## Self-Check: PASSED

Files verified to exist:
- `crates/kay-provider-errors/Cargo.toml` — FOUND
- `crates/kay-provider-errors/src/lib.rs` — FOUND
- `crates/kay-tools/src/events.rs` — FOUND (full AgentEvent enum)
- `crates/kay-tools/src/schema.rs` — FOUND (no todo!() remaining)
- `crates/kay-tools/tests/schema_hardening_property.rs` — FOUND
- `crates/kay-tools/tests/events_registry_integration.rs` — FOUND
- `crates/kay-provider-openrouter/src/event.rs` — FOUND (re-export shim)
- `crates/kay-provider-openrouter/src/error.rs` — FOUND (re-export shim)

Commit verified:
- `5f0a3c2` in `git log --oneline` — FOUND on `phase/03-tool-registry`
- DCO trailer `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` — FOUND
