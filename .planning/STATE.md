---
gsd_state_version: 1.0
milestone: v2.0.0
milestone_name: milestone
status: phase_2_in_progress
stopped_at: Phase 2 Plan 02 complete (Wave 2 structural rename); 8 plans remaining (02-03..02-10)
last_updated: "2026-04-19T18:57:35Z"
last_activity: 2026-04-19 -- Phase 2 Plan 02 executed (1 task, 1 atomic commit bb57694); 23 forge_*/lib.rs renamed to mod.rs (R100 on every file, zero content edits); E0583 count 24->0; E0432/E0433 baseline 2025 for Wave 3
progress:
  total_phases: 13
  completed_phases: 1
  total_plans: 16
  completed_plans: 8
  percent: 11
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-19)

**Core value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.
**Current focus:** Phase 2 — Provider HAL + Tolerant JSON Parser

## Current Position

Phase: 2 of 12 (Provider HAL + Tolerant JSON Parser)
Plan: 2 of 10 executed in current phase (02-01 Wave 0 + 02-02 Wave 2 complete; 02-03..02-10 remaining)
Status: Wave 2 structural rename landed; Wave 3 (02-03/04/05 path rewrites) unblocked
Last activity: 2026-04-19 -- Plan 02-02 executed (1 atomic commit bb57694); 23 forge_*/lib.rs renamed to mod.rs; E0583 count 24->0; next plan 02-03 rewrites ~1k crate::X -> crate::forge_X::Y import paths

Progress: [█░░░░░░░░░░░] 11% (1 of 12 phases done; Phase 2: 2 of 10 plans done)

## Performance Metrics

**Velocity:**

- Total plans completed: 8 (6 in Phase 1 + 2 in Phase 2)
- Average duration: ~8 min/plan (weighted across phases)
- Total execution time: ~53 min of direct plan execution (excludes review loops, fixes, release)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01    | 6     | 6     | ~5 min   |
| 02    | 2     | 10    | ~11 min  |

**Recent Trend:**

- Last 5 plans: 01-04, 01-05, 01-06, 02-01, 02-02 (all PASS; sequential/main-worktree mode with DCO signoff on every commit)
- Trend: Stable; Phase 2 Wave 0 + Wave 2 shipped with one minor doc-only deviation (requirement-mapping on 02-02 deferred to 02-06/08 per ROADMAP)

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
- Phase 2 Plan 02: Verified the rename is byte-identical via triple-check — git R100 similarity score on every file + numstat `0\t0` on every file + combined sha256 before/after match (`e749ea93...e098e8d238a`). Going forward, this triple-check is the canonical invariant for any atomic N-file rename in this project.
- Phase 2 Plan 02: PROV-01 checkbox NOT marked despite plan frontmatter listing it — rationale: PROV-01 is a behavioral requirement (Provider trait + tool calling + SSE + typed AgentEvent), which a structural file rename cannot fulfill. ROADMAP correctly labels 02-02 as "(PROV-01 prereq)". Downstream plans 02-06 (trait scaffolding) and 02-08 (OpenRouterProvider impl) own PROV-01 completion. Documented as Rule-4 interpretation deviation in 02-02-SUMMARY.

### Pending Todos

None carried over — Phase 1 closed cleanly.

### Blockers/Concerns

- **Phase 2 structural integration (cross-subtree path rewrites)**: RESOLVED for module-declaration layer as of plan 02-02 (E0583 count is 0). REMAINING: 2,025 × E0432/E0433 cross-subtree import-path errors in kay-core — addressed by plans 02-03/04/05 (Wave 3). `cargo check --workspace` (without `--exclude kay-core`) will remain failing until 02-05 lands; that's by design. Pre-flagged in 01-03-SUMMARY.md and VERIFICATION.md §SC-4.
- Phase 2 research flag: OpenRouter SSE retry semantics need real-trace validation (flag for `/gsd-research-phase` at Phase 2 planning).
- Phase 1 external dependency (still ticking): Apple Developer ID and Azure Code Signing enrollment has 2-4 week lead time. Started at Phase 1; certificates land by Phase 11.
- Phase 7 research flag: SQLite schema for function signatures + vector embeddings is an open design question; audit ForgeCode indexer before reimplementing.
- Phase 9 research flag: Tauri IPC memory leak status (issues #12724/#13133) needs upstream check before building session view.

## Deferred Items

Items acknowledged and carried forward:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| EVAL | EVAL-01a — run unmodified fork on TB 2.0 ≥80% | Blocked on OpenRouter key + ~$100 budget | Phase 1 (D-OP-01) |
| Compile | kay-core 23 × E0583 (forge_* naming) | RESOLVED in plan 02-02 (2026-04-19, commit bb57694 — 23 renames, R100 on every file) | Phase 1 (01-03-SUMMARY §Deferrals) |
| Compile | kay-core 2,025 × E0432/E0433 (cross-subtree import paths) | Scheduled for Wave 3 — plans 02-03/04/05 | Phase 2 (02-02-SUMMARY §Expected Downstream Errors) |
| Release signing | GPG/SSH signing of release tags | v0.0.x unsigned carve-out; mandatory v0.1.0+ | Phase 1 (SECURITY.md §Release Signing) |

## Session Continuity

Last session: 2026-04-19 — Phase 2 Plan 02 executed (Wave 2 structural rename)
Stopped at: Plan 02-02 complete (1 task, 1 atomic DCO-signed commit bb57694 — 23 forge_*/lib.rs renamed to mod.rs with R100 similarity on every file, zero content edits); proceeding to plan 02-03 (Wave 3 sub-wave A+B+C path rewrites: 17 leaf subtrees + forge_domain + forge_domain-dependents)
Resume file: `.planning/phases/02-provider-hal-tolerant-json-parser/02-03-PLAN.md`
