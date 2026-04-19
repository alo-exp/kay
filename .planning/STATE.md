---
gsd_state_version: 1.0
milestone: v2.0.0
milestone_name: milestone
status: phase_2_in_progress
stopped_at: Phase 2 Plan 01 complete (Wave 0 test scaffolding); 9 plans remaining (02-02..02-10)
last_updated: "2026-04-19T18:25:37Z"
last_activity: 2026-04-19 -- Phase 2 Plan 01 executed (2 tasks, 2 commits b107e7e + 50d8020); Wave 0 test scaffolding landed ahead of kay-core rename
progress:
  total_phases: 13
  completed_phases: 1
  total_plans: 16
  completed_plans: 7
  percent: 10
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-19)

**Core value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.
**Current focus:** Phase 2 — Provider HAL + Tolerant JSON Parser

## Current Position

Phase: 2 of 12 (Provider HAL + Tolerant JSON Parser)
Plan: 1 of 10 executed in current phase (02-01 Wave 0 complete; 02-02..02-10 remaining)
Status: Wave 1 in progress; Wave 0 test scaffolding landed
Last activity: 2026-04-19 -- Plan 02-01 executed (b107e7e + 50d8020); cargo check -p kay-provider-openrouter --tests clean; wave_0_complete flag flipped in 02-VALIDATION.md

Progress: [█░░░░░░░░░░░] 10% (1 of 12 phases done; Phase 2: 1 of 10 plans done)

## Performance Metrics

**Velocity:**

- Total plans completed: 7 (6 in Phase 1 + 1 in Phase 2)
- Average duration: ~8 min/plan (weighted across phases)
- Total execution time: ~45 min of direct plan execution (excludes review loops, fixes, release)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01    | 6     | 6     | ~5 min   |
| 02    | 1     | 10    | ~15 min  |

**Recent Trend:**

- Last 5 plans: 01-03, 01-04, 01-05, 01-06, 02-01 (all PASS; 02-01 executed in sequential/main-worktree mode with DCO signoff)
- Trend: Stable; Phase 2 Wave 0 shipped with zero deviations

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Init: License Apache-2.0 + DCO (not CLA) — pitfalls research showed CLAs cause measurable contributor drop-off.
- Init: Fork ForgeCode as the Apache-2.0 base; import KIRA's four harness techniques; layer Tauri 2.x desktop UI.
- Init: OpenRouter-only provider for v1 with an Exacto-leaning model allowlist (not "300+ models").
- Init: 12-phase roadmap with Phase 1 parity gate (EVAL-01) — forked baseline must hit >=80% on TB 2.0 before any harness change merges.
- Phase 1 D-OP-01: parity-gate **scaffold only** in Phase 1; actual run re-scoped to EVAL-01a follow-on (unblocks on OpenRouter key + ~$100 budget).
- Phase 1 D-OP-04: `forgecode-parity-baseline` tag left unsigned (v0.0.x pre-stable carve-out); v0.1.0+ release signing mandatory.
- Phase 1 release policy: semver with **never-major** policy — breaking bumps treated as minor releases.
- Phase 2 Plan 01: chose standalone MockServer (no kay-core import) so Wave 0 scaffolding can merge in parallel with the 02-02..02-05 kay-core rename plans — self-contained helper that mirrors the ForgeCode `forge_repo/provider/mock_server.rs` analog shape.
- Phase 2 Plan 01: JSONL-per-line SSE cassette format (one JSON object per non-blank line; loader adds `data: ` prefix at mockito assembly time) — keeps fixtures diff-readable and isolates the SSE-wrapping concern in the loader.
- Phase 2 Plan 01: Inline `_comment` field in `allowlist.json` (serde-ignored extra field) documents `openai/gpt-5.4` provisional status without requiring a sidecar README — flagged for plan 02-07 to decide between permissive struct and sidecar file.

### Pending Todos

None carried over — Phase 1 closed cleanly.

### Blockers/Concerns

- **Phase 2 structural integration** (first task): fix 23 × `E0583` errors in `kay-core` from ForgeCode's `forge_*/lib.rs` naming. Blocks `cargo check --workspace` without `--exclude kay-core`. Pre-flagged in 01-03-SUMMARY.md and VERIFICATION.md §SC-4.
- Phase 2 research flag: OpenRouter SSE retry semantics need real-trace validation (flag for `/gsd-research-phase` at Phase 2 planning).
- Phase 1 external dependency (still ticking): Apple Developer ID and Azure Code Signing enrollment has 2-4 week lead time. Started at Phase 1; certificates land by Phase 11.
- Phase 7 research flag: SQLite schema for function signatures + vector embeddings is an open design question; audit ForgeCode indexer before reimplementing.
- Phase 9 research flag: Tauri IPC memory leak status (issues #12724/#13133) needs upstream check before building session view.

## Deferred Items

Items acknowledged and carried forward:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| EVAL | EVAL-01a — run unmodified fork on TB 2.0 ≥80% | Blocked on OpenRouter key + ~$100 budget | Phase 1 (D-OP-01) |
| Compile | kay-core 23 × E0583 (forge_* naming) | Addressed in Phase 2 structural-integration task | Phase 1 (01-03-SUMMARY §Deferrals) |
| Release signing | GPG/SSH signing of release tags | v0.0.x unsigned carve-out; mandatory v0.1.0+ | Phase 1 (SECURITY.md §Release Signing) |

## Session Continuity

Last session: 2026-04-19 — Phase 2 Plan 01 executed (Wave 0 test scaffolding)
Stopped at: Plan 02-01 complete (2 tasks, 2 atomic commits b107e7e + 50d8020, DCO-signed); proceeding to plan 02-02 (kay-core structural rename)
Resume file: `.planning/phases/02-provider-hal-tolerant-json-parser/02-02-PLAN.md`
