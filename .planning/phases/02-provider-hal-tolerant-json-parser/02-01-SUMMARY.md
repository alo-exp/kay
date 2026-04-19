---
phase: 02-provider-hal-tolerant-json-parser
plan: 01
subsystem: testing
tags: [mockito, proptest, sse, openrouter, cassettes, fixtures, tokio]

# Dependency graph
requires:
  - phase: 01-fork-governance-infrastructure
    provides: "kay-provider-openrouter skeleton crate with empty [dependencies]; ForgeCode's forge_repo/provider/mock_server.rs pattern available as a reference analog"
provides:
  - "mockito 1.7 + proptest 1.11 + pretty_assertions 1 + tokio test features as dev-dependencies on kay-provider-openrouter"
  - "Standalone MockServer wrapper (tests/common/mock_server.rs) exposing mock_openrouter_chat_stream, mock_rate_limit, mock_server_error_503, and load_sse_cassette — independent of kay-core so it compiles through the 02-02..02-05 rename"
  - "Six canonical OpenRouter SSE cassettes (happy_path, tool_call_fragmented, tool_call_malformed, rate_limit_429, server_error_503, usage_without_cost) that every downstream Phase 2 plan reuses"
  - "D-07 launch allowlist fixture (anthropic/claude-sonnet-4.6, anthropic/claude-opus-4.6, openai/gpt-5.4) locked into the test tree as tests/fixtures/config/allowlist.json"
  - "wave_0_complete: true in 02-VALIDATION.md frontmatter — advances the Wave 0 gate for plans 02-06..02-10"
affects: [02-06, 02-07, 02-08, 02-09, 02-10, provider HAL impl, tolerant JSON parser, allowlist gate, cost cap, retry policy]

# Tech tracking
tech-stack:
  added:
    - "mockito 1.7 — HTTP mocking (matches the version ForgeCode uses in forge_repo/provider/mock_server.rs)"
    - "proptest 1.11 — property-based fuzzing (new dep; no in-tree analog)"
    - "pretty_assertions 1 — readable assertion diffs"
    - "tokio test-util feature — deterministic async tests"
  patterns:
    - "Standalone test helper: tests/common/mock_server.rs does NOT import kay-core; mirrors the shape of forge_repo/provider/mock_server.rs but is self-contained so it compiles during the kay-core structural rename"
    - "JSONL-per-line SSE cassette format: one JSON object per non-blank line; loader adds the `data: ` SSE prefix + blank-line separators at mockito assembly time"
    - "CARGO_MANIFEST_DIR-rooted cassette paths: compile-time path rooting keeps tests reproducible without relative-path fragility"
    - "Config fixture with inline _comment field (serde-ignored) for provisional-status documentation — avoids sidecar README files while staying JSON-spec compliant"

key-files:
  created:
    - "crates/kay-provider-openrouter/tests/common/mod.rs — subdirectory module anchor"
    - "crates/kay-provider-openrouter/tests/common/mock_server.rs — standalone MockServer wrapper (81 lines)"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/happy_path.jsonl — plain completion with usage.cost"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/tool_call_fragmented.jsonl — one tool_call across 3 SSE chunks"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/tool_call_malformed.jsonl — trailing comma + unquoted key (seed for plan 02-09)"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/rate_limit_429.jsonl — post-retry success body"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/server_error_503.jsonl — post-retry success body"
    - "crates/kay-provider-openrouter/tests/fixtures/sse/usage_without_cost.jsonl — usage frame missing cost (D-10 fallback case)"
    - "crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json — D-07 canonical launch allowlist"
  modified:
    - "crates/kay-provider-openrouter/Cargo.toml — [dev-dependencies] block added"
    - "Cargo.lock — lockfile updates for the new dev-deps tree"
    - ".planning/phases/02-provider-hal-tolerant-json-parser/02-VALIDATION.md — wave_0_complete flag flipped false → true"

key-decisions:
  - "Chose standalone MockServer over importing from kay-core — kay-core is pre-rename and would break compilation; the helper mirrors the ForgeCode analog shape and will converge post-rename"
  - "Adopted JSONL-per-line cassette format (one JSON object per line + loader-added `data: ` prefix) over pre-wrapped SSE strings — keeps fixtures diff-readable and lets the loader do `data: ` assembly consistently"
  - "Inline _comment field in allowlist.json instead of sidecar README — serde-ignored extra field keeps the config self-documenting without extra files (flagged for plan 02-07 to either allow unknown fields or move to a sidecar)"
  - "Did NOT add insta snapshot testing — PATTERNS.md suggestion, but not tied to any REQ-ID; deferred"
  - "[dependencies] stays empty in this plan — Provider impl lands in plan 02-06; this plan is pure Wave 0 scaffolding"

patterns-established:
  - "Test infrastructure landed before the kay-core rename so Wave 0 can merge in parallel with the rename plans (02-02..02-05)"
  - "Six-cassette coverage of OpenRouter SSE variance: happy path, fragmented tool_calls, malformed tool_calls, 429 Retry-After, 503, usage-without-cost"
  - "DCO signoff (`git commit -s`) applied on every commit per CLAUDE.md non-negotiable #3"

requirements-completed: [PROV-01, PROV-04, PROV-05, PROV-07]

# Metrics
duration: ~15min
completed: 2026-04-19
---

# Phase 02 Plan 01: Wave 0 Test Scaffolding Summary

**Six canonical OpenRouter SSE cassettes + standalone MockServer helper + D-07 allowlist fixture landed ahead of the kay-core structural rename, unblocking Phase 2 downstream plans.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-19T18:10:00Z
- **Completed:** 2026-04-19T18:25:37Z
- **Tasks:** 2 of 2 completed
- **Files modified:** 12 (10 created + 2 modified: Cargo.toml, 02-VALIDATION.md; Cargo.lock also regenerated by the new dev-deps)

## Accomplishments

- Installed the test toolkit (mockito 1.7 + proptest 1.11 + pretty_assertions 1 + tokio with test-util) as dev-dependencies on `kay-provider-openrouter`; `[dependencies]` stays empty (Provider impl lands in plan 02-06).
- Built a standalone `MockServer` wrapper that does NOT import from kay-core — fully self-contained so this plan can merge in parallel with the 02-02..02-05 structural rename work.
- Captured six canonical OpenRouter SSE event sequences as cassettes (happy_path, tool_call_fragmented, tool_call_malformed, rate_limit_429, server_error_503, usage_without_cost) so plans 02-07 through 02-10 reuse the same fixtures instead of re-inventing them.
- Locked the D-07 launch allowlist (`anthropic/claude-sonnet-4.6`, `anthropic/claude-opus-4.6`, `openai/gpt-5.4`) into the test tree as `tests/fixtures/config/allowlist.json` with an inline `_comment` field documenting the `openai/gpt-5.4` provisional status.
- Advanced the Wave 0 gate: `02-VALIDATION.md` frontmatter now has `wave_0_complete: true`.

## Task Commits

Each task committed atomically with DCO signoff:

1. **Task 1: Install dev dependencies + test directory layout** — `b107e7e` (chore)
2. **Task 2: Create MockServer helper + six SSE cassette fixtures** — `50d8020` (test)

_(plus this SUMMARY.md commit landing after self-check — see Plan metadata at the bottom)_

## Files Created/Modified

**Created:**
- `crates/kay-provider-openrouter/tests/common/mod.rs` — subdirectory module anchor (`pub mod mock_server;`)
- `crates/kay-provider-openrouter/tests/common/mock_server.rs` — MockServer wrapper (mock_openrouter_chat_stream, mock_rate_limit, mock_server_error_503, load_sse_cassette)
- `crates/kay-provider-openrouter/tests/fixtures/sse/happy_path.jsonl` — plain completion + `usage.cost`
- `crates/kay-provider-openrouter/tests/fixtures/sse/tool_call_fragmented.jsonl` — one tool_call across 3 SSE chunks (RESEARCH §Code Example 4)
- `crates/kay-provider-openrouter/tests/fixtures/sse/tool_call_malformed.jsonl` — trailing comma + unquoted key; seed for plan 02-09 never-panic test
- `crates/kay-provider-openrouter/tests/fixtures/sse/rate_limit_429.jsonl` — post-retry success body
- `crates/kay-provider-openrouter/tests/fixtures/sse/server_error_503.jsonl` — post-retry success body
- `crates/kay-provider-openrouter/tests/fixtures/sse/usage_without_cost.jsonl` — usage frame missing `cost` (D-10 price-table fallback)
- `crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json` — D-07 launch allowlist with inline `_comment`

**Modified:**
- `crates/kay-provider-openrouter/Cargo.toml` — `[dev-dependencies]` block added; `[dependencies]` stays empty
- `Cargo.lock` — lockfile regenerated for the new dev-deps tree
- `.planning/phases/02-provider-hal-tolerant-json-parser/02-VALIDATION.md` — `wave_0_complete: false` → `true`

## Decisions Made

- **Standalone MockServer (no kay-core import)** — kay-core is pre-rename and `use kay_core::forge_repo::provider::mock_server` would not compile; a self-contained helper that mirrors the ForgeCode shape keeps Wave 0 independent of the rename plans.
- **JSONL-per-line cassette format** — one JSON object per non-blank line; the loader adds the `data: ` SSE prefix at assembly time. Keeps fixtures diff-readable and moves the SSE-wrapping concern into the loader where it belongs.
- **Inline `_comment` field in `allowlist.json`** — serde-ignored extra field documents `openai/gpt-5.4`'s provisional status (per RESEARCH §Open Questions Q1) without adding a sidecar README. Flagged for plan 02-07 to decide whether to use `#[serde(deny_unknown_fields)]` (then the comment moves) or a permissive struct.

## Deviations from Plan

None — plan executed exactly as written.

Both tasks proceeded through their action blocks and `verify` steps on the first attempt. No Rule 1 (bugs), Rule 2 (missing critical functionality), Rule 3 (blocking issues), or Rule 4 (architectural) deviations arose.

## Issues Encountered

None. `cargo check -p kay-provider-openrouter --tests` compiled clean after Task 1 and remained clean after Task 2. The baseline non-regression `cargo check --workspace --exclude kay-core` passed both before and after.

One verification-tool note: the plan's draft acceptance criterion used `cargo metadata --no-deps --format-version 1 -p kay-provider-openrouter`, but `cargo metadata` does not accept `-p`. The equivalent command is `cargo metadata --manifest-path crates/kay-provider-openrouter/Cargo.toml --no-deps --format-version 1 | grep -oE '"name":"(mockito|proptest|tokio|pretty_assertions)"'`. All four dev-deps were confirmed present in the metadata. This is a documentation gap in the plan's verify block, not a behavioral deviation; no commit was needed.

## User Setup Required

None — no external services, credentials, or dashboards touched by this plan.

## Next Phase Readiness

- `cargo check -p kay-provider-openrouter --tests` exits 0 (clean; zero warnings) — fixtures are loadable and MockServer compiles.
- `cargo check --workspace --exclude kay-core` still exits 0 — Phase 1 baseline not regressed.
- Six SSE cassettes + allowlist fixture are committed and immutable in the test tree; plans 02-06 through 02-10 can reference them with `MockServer::load_sse_cassette("<name>")`.
- `wave_0_complete: true` flag flipped in 02-VALIDATION.md — Wave 0 gate advanced for downstream plans.
- The `_comment` field convention in `allowlist.json` is flagged for plan 02-07 to either tolerate via permissive struct or migrate to a sidecar README.

**Ready for plan 02-02 (kay-core structural rename — the E0583 unblock).**

## Self-Check: PASSED

All claimed artifacts verified on disk:
- `crates/kay-provider-openrouter/Cargo.toml` FOUND — contains `[dev-dependencies]` with mockito, proptest, tokio, pretty_assertions
- `crates/kay-provider-openrouter/tests/common/mod.rs` FOUND — contains `pub mod mock_server;`
- `crates/kay-provider-openrouter/tests/common/mock_server.rs` FOUND — contains `mock_openrouter_chat_stream`, `mock_rate_limit`, `mock_server_error_503`, `load_sse_cassette`
- 6 SSE cassettes under `tests/fixtures/sse/` FOUND (happy_path, tool_call_fragmented, tool_call_malformed, rate_limit_429, server_error_503, usage_without_cost)
- `tests/fixtures/config/allowlist.json` FOUND — contains all three D-07 models + `_comment`, NO `:exacto`
- `tests/fixtures/sse/usage_without_cost.jsonl` FOUND — contains `"usage"` and does NOT contain `"cost"`

Commits verified in `git log --oneline -5`:
- `b107e7e` FOUND — Task 1 commit (chore(02-01): install dev-deps + tests/common scaffold)
- `50d8020` FOUND — Task 2 commit (test(02-01): MockServer helper + 6 SSE cassettes + D-07 allowlist fixture)

Both commits verified as carrying `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` trailers (DCO compliance per CLAUDE.md non-negotiable #3).

Final verifications:
- `cargo check -p kay-provider-openrouter --tests` → exit 0, zero warnings, zero errors
- `cargo check --workspace --exclude kay-core` → exit 0 (baseline non-regression)
- `ls tests/fixtures/sse/*.jsonl | wc -l` → 6
- `grep -q 'wave_0_complete: true' .planning/phases/02-provider-hal-tolerant-json-parser/02-VALIDATION.md` → exits 0

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Completed: 2026-04-19*
