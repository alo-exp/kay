---
phase: 02-provider-hal-tolerant-json-parser
plan: 06
subsystem: provider-hal
tags: [rust, async-trait, futures-stream, thiserror, non-exhaustive-enum, openrouter, crate-lint-deny-unwrap]

# Dependency graph
requires:
  - phase: 02.5-kay-core-sub-crate-split
    provides: "direct forge_* sub-crate path-deps in kay-provider-openrouter/Cargo.toml (commit 9d6a32a); aggregator kay-core crate (not used here but compiles clean)"
  - phase: 02-provider-hal-tolerant-json-parser (plans 02-01..02-04)
    provides: "MockServer + SSE cassette scaffolding (02-01); workspace builds cleanly with 0 exclusions (02-04 precursor + 2.5 closure)"
provides:
  - "Provider trait (object-safe via async_trait)"
  - "ChatRequest / Message / ToolSchema input types"
  - "AgentEventStream<'a> lifetime-bounded stream type alias"
  - "AgentEvent enum (8 variants, #[non_exhaustive]) — delta-granular provider frames"
  - "ProviderError enum (11 variants, #[non_exhaustive]) with thiserror Display"
  - "AuthErrorKind (3 variants) + RetryReason (2 variants), both #[non_exhaustive]"
  - "crate-root #![deny(clippy::unwrap_used, clippy::expect_used)] lint"
affects: [plan-02-07, plan-02-08, plan-02-09, plan-02-10, phase-03-tool-registry, phase-05-agent-loop]

# Tech tracking
tech-stack:
  added: [backon-1.6, async-trait-0.1, futures-0.3, tokio-stream-0.1]
  patterns:
    - "non-exhaustive public enums for additive evolution across phases"
    - "crate-root lint deny(unwrap_used, expect_used) to enforce PROV-05 never-panic at compile time"
    - "typed ProviderError taxonomy (not anyhow::Error) at HAL boundary"
    - "Pin<Box<dyn Stream + Send + 'a>> lifetime-bounded event streams"

key-files:
  created:
    - "crates/kay-provider-openrouter/src/error.rs — ProviderError + AuthErrorKind + RetryReason + 2 unit tests"
    - "crates/kay-provider-openrouter/src/event.rs — AgentEvent enum (8 variants)"
    - "crates/kay-provider-openrouter/src/provider.rs — Provider trait + ChatRequest/Message/ToolSchema + AgentEventStream"
  modified:
    - "crates/kay-provider-openrouter/Cargo.toml — added backon, async-trait, futures, tokio-stream"
    - "crates/kay-provider-openrouter/src/lib.rs — module decls + re-exports + crate-level lint"
    - "Cargo.lock — regenerated for new deps"

key-decisions:
  - "AgentEvent dropped `Clone` derive — ProviderError::Network(reqwest::Error) and ProviderError::Serialization(serde_json::Error) both contain non-Clone types. Consumers receive events by move; Clone is not required by any planned downstream path. Phase 5 LOOP-02 may revisit if the cross-phase contract needs it."
  - "No kay_core imports in any of the four files — planned imports (forge_json_repair, forge_domain::provider, forge_app::dto::openai) land in plan 02-08 when the concrete OpenRouterProvider impl is assembled."
  - "Cargo.toml Task 1 was a LIGHT touch per Appendix A Rule 1 — forge_* direct path-deps were already wired by Phase 2.5-04 (commit 9d6a32a). Only the 4 NEW deps (backon/async-trait/futures/tokio-stream) were added. Plan's literal text (`kay-core = { path = \"../kay-core\" }`) is superseded by the post-2.5 direct-dep layout."
  - "ToolSchema.input_schema uses serde_json::Value (not schemars::Schema). Phase 3 TOOL-05 introduces schemars for tool registration; keeping Phase 2 free of schemars avoids cross-phase coupling per plan's action note."

patterns-established:
  - "non-exhaustive enum discipline: every public enum in the provider HAL is #[non_exhaustive] so Phase 3/4/5/8 can add variants without breaking downstream consumers"
  - "crate-root never-panic lint: `#![deny(clippy::unwrap_used, clippy::expect_used)]` at lib.rs top; tests in #[cfg(test)] modules auto-exempt. Pattern to reuse for future security-critical HAL crates."
  - "AgentEventStream<'a> lifetime-binding pattern: Stream lifetime is tied to &self so provider state can be borrowed; consumers requiring 'static must Arc-clone state into the closure"

requirements-completed: []  # PROV-01, PROV-02, PROV-08 are BEHAVIORAL — this plan only ships the type contract. Completion belongs to plans 02-07/08/09/10 when the actual impl lands.

# Metrics
duration: 7min
completed: 2026-04-20
---

# Phase 02 Plan 06: kay-provider-openrouter Public Contract Summary

**Provider HAL type contract frozen — `Provider` trait (async_trait, object-safe), `AgentEvent` (8 variants, non-exhaustive), `ProviderError` (11 variants, thiserror), helper types, and crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` lint locking PROV-05 never-panic at compile time.**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-04-20T03:10:53Z
- **Completed:** 2026-04-20T03:18:15Z
- **Tasks:** 2
- **Files created:** 3 (error.rs, event.rs, provider.rs)
- **Files modified:** 3 (Cargo.toml, lib.rs, Cargo.lock)

## Accomplishments

- Four-module public surface in `kay-provider-openrouter` (error, event, provider, lib) defining the contract plans 02-07..02-10 will implement against
- All enums are `#[non_exhaustive]` — phase-additive evolution is breakage-free for the three user-facing frontends (kay-cli Phase 5, kay-tauri Phase 9, kay-tui Phase 9.5)
- Crate-root lint `#![deny(clippy::unwrap_used, clippy::expect_used)]` enforces PROV-05 + TM-01 at compile time — any `.unwrap()` or `.expect()` in non-test code now fails clippy
- 2 unit tests passing (Debug redaction + Display context preservation)
- `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` exits 0

## Task Commits

Each task was committed atomically with DCO sign-off (`git commit -s`):

1. **Task 1: Install runtime deps (backon + async-trait + futures + tokio-stream)** — `f36083f` (feat)
2. **Task 2: Scaffold public contract (error.rs + event.rs + provider.rs + lib.rs rewrite)** — `b0bcc8d` (feat)

Both commits include `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`. CI pre-commit hook (`cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings`) passed without `--no-verify` on each commit.

## Files Created/Modified

- `crates/kay-provider-openrouter/src/error.rs` (113 lines) — `ProviderError` (11 variants, thiserror), `AuthErrorKind` (3 variants), `RetryReason` (2 variants); unit tests for Debug-no-leak + Display-carries-context
- `crates/kay-provider-openrouter/src/event.rs` (61 lines) — `AgentEvent` (8 variants per D-06): TextDelta, ToolCallStart, ToolCallDelta, ToolCallComplete, ToolCallMalformed, Usage, Retry, Error
- `crates/kay-provider-openrouter/src/provider.rs` (79 lines) — `Provider` trait (async_trait), `AgentEventStream<'a>` alias, `ChatRequest` / `Message` / `ToolSchema` structs
- `crates/kay-provider-openrouter/src/lib.rs` (28 lines) — module decls + `pub use` re-exports + crate-level `#![deny(clippy::unwrap_used, clippy::expect_used)]` + `#![allow(dead_code)]`
- `crates/kay-provider-openrouter/Cargo.toml` — added 4 deps (backon 1.6, async-trait 0.1, futures 0.3, tokio-stream 0.1)
- `Cargo.lock` — regenerated to include the 4 new deps in kay-provider-openrouter's dependency section

## Decisions Made

- **Clone derive dropped from `AgentEvent`** (forced by type system): the PLAN.md snippet specified `#[derive(Debug, Clone)]`, but `AgentEvent::Error { error: ProviderError }` carries a `ProviderError` that holds `reqwest::Error` (not Clone) and `serde_json::Error` (not Clone). Keeping Clone would require a wholesale redesign (either owning error types or Arc-wrapping). The HAL's planned consumers — SSE translator (02-08), downstream Stream adapters — receive events by move, so Clone adds no value at this boundary. Recorded as a Rule-1 auto-fix deviation; documented in event.rs's derive and here for traceability.
- **Light-touch Cargo.toml Task 1 per Appendix A Rule 1**: the plan's literal `kay-core = { path = "../kay-core" }` is superseded by the direct forge_* path-deps committed in Phase 2.5-04 (`9d6a32a`). Task 1 reduced to adding the 4 NEW deps the plan required on top of the existing wiring.
- **ToolSchema.input_schema stays `serde_json::Value`** (not schemars::Schema) — explicit in the plan's action: Phase 3 TOOL-05 owns the schemars introduction. Decision upheld.
- **No kay_core / forge_* imports anywhere in this plan's code** — zero imports from the forge_* sub-crates. Those land in plan 02-08 when the concrete delegation layer assembles. Matches plan's explicit "DO NOT import from kay_core in ANY of the four files in this task" directive.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Dropped `Clone` derive on `AgentEvent`**
- **Found during:** Task 2 (cargo check compilation attempt)
- **Issue:** Plan 02-06's interface block (lines 134-146) specified `#[derive(Debug, Clone)]` on `AgentEvent`. But `AgentEvent::Error { error: ProviderError }` holds a `ProviderError`, which carries `reqwest::Error` (via `Network` variant) and `serde_json::Error` (via `Serialization` variant). Neither of those error types implements `Clone`, so `#[derive(Clone)]` on `AgentEvent` fails to compile with E0277 on those two variants.
- **Fix:** Dropped `Clone` from `AgentEvent`'s derive list. Only `Debug` remains. The provider-HAL consumers (SSE translator in 02-08, Stream combinators, CLI/Tauri/TUI frame sinks) receive events by move via the Stream `Item = Result<AgentEvent, ProviderError>` contract — cloning is never needed along that path. Phase 5 LOOP-02 may revisit if cross-phase subscription requires it; at that point `AgentEvent::Error` can be redesigned (e.g., hold `Arc<ProviderError>`) without breaking the stream contract since the enum is `#[non_exhaustive]`.
- **Files modified:** `crates/kay-provider-openrouter/src/event.rs`
- **Verification:** `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` exits 0; 2 unit tests pass.
- **Committed in:** `b0bcc8d` (Task 2 commit)

**2. [Rule 2 - Post-2.5 Realignment, Appendix A Rule 1] Cargo.toml kay-core dep substitution**
- **Found during:** Task 1 (Cargo.toml edit)
- **Issue:** Plan 02-06's frontmatter `must_haves.truths` lists "kay-provider-openrouter has runtime dependencies on kay-core, reqwest, ..." and `key_links` asserts `kay-core = { path = "../kay-core" }` must be present in Cargo.toml. Post-Phase-2.5 the canonical layout uses direct forge_* path-deps (`forge_domain`, `forge_config`, `forge_services`, `forge_repo`, `forge_json_repair`) committed in 9d6a32a instead of a kay-core aggregator dep.
- **Fix:** Applied Appendix A Rule 1 — kept the existing direct forge_* path-deps intact; added only the 4 NEW runtime deps the plan required (`backon 1.6`, `async-trait 0.1`, `futures 0.3`, `tokio-stream 0.1`). Did NOT add `kay-core = { path = "../kay-core" }` (adding it would regress against 2.5-04's architectural choice).
- **Files modified:** `crates/kay-provider-openrouter/Cargo.toml`
- **Verification:** `cargo tree -p kay-provider-openrouter --depth 1` shows all 4 new deps resolve alongside the existing forge_* path-deps. `cargo check -p kay-provider-openrouter --lib` exits 0.
- **Committed in:** `f36083f` (Task 1 commit)

**3. [Rule 2 - Post-2.5 Realignment, Appendix A Rule 2] No-op — no `kay_core::forge_*` import-path substitution needed**
- **Found during:** Task 2 planning phase
- **Issue:** Plan 02-06 objective mentions "the crate links against kay-core (path dep) for the `forge_json_repair` and `forge_app::dto::openai` types that plan 02-08 will consume." Post-2.5 the substitution would be `use forge_json_repair::...` / `use forge_app::dto::openai::...`.
- **Fix:** Recorded as applicable but NOT EXERCISED in plan 02-06 — the four files created in Task 2 explicitly have zero `kay_core::` or `forge_*::` imports per the plan's own directive ("DO NOT import from kay_core in ANY of the four files in this task"). The substitution rule will actually fire in plan 02-08.
- **Files modified:** none (preemptive documentation)
- **Verification:** `grep -rn "use kay_core" crates/kay-provider-openrouter/src/` → 0 matches (exit 1); `grep -rn "use forge_" crates/kay-provider-openrouter/src/` → 0 matches. Clean.
- **Committed in:** N/A (documentation-only in this SUMMARY)

---

**Total deviations:** 3 auto-fixed (1 Rule-1 bug-fix for Clone derive; 2 Rule-2 post-2.5 realignment substitutions per Appendix A).

**Impact on plan:** No scope creep. Deviation #1 (Clone drop) is a mechanical Rust-type-system constraint that was latent in the plan's snippet and surfaced at first compile — the fix is the minimal correct change. Deviations #2 and #3 are pre-authorized by 02-CONTEXT.md Appendix A; they keep the post-2.5 layout intact and do not alter the plan's intent. All acceptance criteria still met.

## Issues Encountered

- Minor: rustfmt applied a formatting normalization to `event.rs` on the second `cargo fmt --all` pass (collapsed `ToolCallDelta { id: String, arguments_delta: String }` to a single line because it fits width). Not a bug — expected rustfmt behavior. No re-commit needed; fmt was applied before Task 2's commit.

## User Setup Required

None — no external service configuration required. The crate scaffolds a type contract; no HTTP, no auth, no runtime side effects yet.

## Threat Flags

No threat flags. The four files created confine security-relevant surface to the threat register's existing TM-01 / T-02-06-01..03 threats, all of which are mitigated:

- **TM-01** (API key leakage via Debug): `AuthErrorKind` is a unit-discriminant enum; Debug output cannot contain a credential. Unit test `debug_impl_never_prints_credential_material` asserts this structurally.
- **TM-07** (tampering via weak error coverage): all 11 D-05 variants present with thiserror-derived Display carrying triage context.
- **T-02-06-03** (panic via unwrap/expect): crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` fails CI on any such call in non-test code.

## Next Phase Readiness

- **Plans 02-07..02-10 unblocked.** Provider trait + AgentEvent + ProviderError are the exact types the next four plans implement against. Contract is frozen.
- **Plan 02-07** (allowlist gate + auth) can now reference `ProviderError::ModelNotAllowlisted` and `ProviderError::Auth { reason: AuthErrorKind::Missing }` as return types.
- **Plan 02-08** (OpenRouterProvider impl) will wire the first `use forge_json_repair::...` / `use forge_domain::...` imports — Appendix A Rule 2 substitution fires for real there.
- **Plan 02-09** (tolerant JSON parser) will emit `AgentEvent::ToolCallMalformed { id, raw, error }` and `ProviderError::ToolCallMalformed { id, error }` as the failure path.
- **Plan 02-10** (retry + cost cap + error taxonomy) will exercise `AgentEvent::Retry`, `ProviderError::RateLimited`, `ProviderError::CostCapExceeded` end-to-end.

## Self-Check: PASSED

Verification performed after SUMMARY draft:

**Files exist:**
- FOUND: crates/kay-provider-openrouter/src/error.rs
- FOUND: crates/kay-provider-openrouter/src/event.rs
- FOUND: crates/kay-provider-openrouter/src/provider.rs
- FOUND: crates/kay-provider-openrouter/src/lib.rs (modified)
- FOUND: crates/kay-provider-openrouter/Cargo.toml (modified)

**Commits exist (verified via `git log --oneline --all | grep`):**
- FOUND: f36083f (Task 1)
- FOUND: b0bcc8d (Task 2)

**Acceptance gates (verified during execution):**
- PASS: `cargo check -p kay-provider-openrouter --lib`
- PASS: `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` (0 warnings)
- PASS: `cargo test -p kay-provider-openrouter --lib` (2 tests: `debug_impl_never_prints_credential_material`, `provider_error_display_includes_context`)
- PASS: `cargo check --workspace` (0 exclusions, clean)
- PASS: `cargo fmt --all -- --check`
- PASS: `tests/governance/check_attribution.sh` (ALL INVARIANTS PASS)
- PASS: `grep "use kay_core" crates/kay-provider-openrouter/src/` → no matches (per plan directive)

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Plan: 06*
*Completed: 2026-04-20*
