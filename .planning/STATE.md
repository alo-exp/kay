---
gsd_state_version: 1.0
milestone: v2.0.0
milestone_name: milestone
status: phase_2_blocked_on_2_5_inserted
stopped_at: Phase 2.5 inserted 2026-04-20 after structural finding during plan 02-05. kay-core mono-crate approach ruled out; D-01 revised to Option (c) sub-crate split. Plans 02-06..02-10 blocked on Phase 2.5 execution.
last_updated: "2026-04-20T07:30:00Z"
last_activity: 2026-04-20 -- Plan 02-05 partial (5 upper subtrees committed: 404ff21 a6f37f7 4991060 57045a4 3d85520); 1323 residual errors exceed mechanical rewrite scope; Phase 2.5 inserted with CONTEXT.md authored (ready for /gsd-plan-phase 2.5)
progress:
  total_phases: 14
  completed_phases: 1
  total_plans: 16
  completed_plans: 15
  percent: 12
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-19)

**Core value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.
**Current focus:** Phase 2.5 — kay-core sub-crate split (INSERTED 2026-04-20; blocks remaining Phase 2 work)

## Current Position

Phase: 2.5 of 13 (kay-core sub-crate split, INSERTED)
Plan: 0 of TBD — ready for `/gsd-plan-phase 2.5`
Status: Phase 2 paused at 4.5/10 plans executed (02-01..02-04 complete; 02-05 partial with 5 upper-subtree rewrites committed but kay-core still has 1323 residual errors that require Option-(c) sub-crate split to resolve). Phase 2.5 CONTEXT.md authored; planning next.
Last activity: 2026-04-20 -- 5 upper subtree rewrites in Phase 2 Plan 05 committed; structural finding documented; Phase 2.5 inserted with CONTEXT.md

Progress: [█░░░░░░░░░░░░] 12% (1 of 13 phases done; Phase 2: 4 of 10 plans done + partial 02-05; Phase 2.5: CONTEXT ready)

## Performance Metrics

**Velocity:**

- Total plans completed: 10 (6 in Phase 1 + 4 in Phase 2)
- Average duration: ~11 min/plan (weighted across phases)
- Total execution time: ~101 min of direct plan execution (excludes review loops, fixes, release)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01    | 6     | 6     | ~5 min   |
| 02    | 4     | 10    | ~17 min  |

**Recent Trend:**

- Last 5 plans: 01-06, 02-01, 02-02, 02-03, 02-04 (all PASS; sequential/main-worktree mode with DCO signoff on every commit)
- Trend: Stable; Phase 2 Wave 0 + Wave 2 + Wave 3 (sub-waves A/B/C + forge_app) shipped. Plan 02-04 executed with zero deviations (reused the 4-rule rewrite tooling from plan 02-03 directly; only minor artifact: commit message count was off by 18 due to regex double-counting grouped imports, corrected in SUMMARY).

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
- Phase 2 Plan 03: Used `perl -i -pe` with `|` delimiter (not `s{}{}`) for all path rewrites — the alternate delimiter avoids brace-matching failures when the replacement contains literal `{` for grouped `use crate::{` forms. This tooling choice is canonical for plans 02-04/05.
- Phase 2 Plan 03: Extended the path-rewrite rule set to include `pub use crate::X` as a fourth rule (alongside Rule 1a, 1b, Rule 2). One instance in `forge_domain/session_metrics.rs:9` would have re-exported an unresolved path. Plans 02-04/05 must apply all four rules for completeness.
- Phase 2 Plan 03: Used `git commit -s --allow-empty` for 6 of 12 sub-wave A leaf subtrees (`forge_stream`, `forge_template`, `forge_tool_macros`, `forge_walker`, `forge_test_kit`, `forge_embed`) that had zero intra/inter-subtree imports — these only reference external crates (std/futures/handlebars/proc_macro). Empty marker commits satisfied the plan's 12-commit sub-wave A gate without fabricating content. Rule-3 style reconciliation between the plan's action text ("skip commit if no-op") and its verify gate ("require 12 commits"). Preserved per-subtree bisectability; zero impact on parity-baseline integrity.
- Phase 2 Plan 03: PROV-01 checkbox NOT marked — same rationale as plan 02-02; this plan remains a "PROV-01 prereq" per ROADMAP. Behavioral PROV-01 completion still owned by 02-06/08.
- Phase 2 Plan 04: Single atomic commit for 83-file / 212-import forge_app rewrite (vs. per-subtree in 02-03) — forge_app IS one subtree, so finer-than-subtree commits would be false granularity. git diff --name-only scope-check (100% under crates/kay-core/src/forge_app/) guarantees no cross-subtree contamination despite the large file-count. Pattern canonicalized for "one subtree per commit, regardless of size within subtree."
- Phase 2 Plan 04: Count-reconciliation convention — pre-scan's `^use crate::` regex matches BOTH non-grouped AND grouped openers, so naive sum (116 + 18 + 95 + 1) double-counts grouped imports. True count for any subtree = (Rule 1a pre-scan count - Rule 1b count) + Rule 1b + Rule 2 + pub-use. Apply this reconciliation in plan 02-05's SUMMARY.
- Phase 2 Plan 04: PROV-01 checkbox NOT marked — same rationale as plans 02-02/03; this plan remains a "PROV-01 prereq" per ROADMAP. Behavioral PROV-01 completion still owned by 02-06/08.
- Phase 2 Plan 05 PARTIAL (2026-04-20): Committed 5 upper-subtree rewrites (`404ff21` forge_services, `a6f37f7` forge_infra, `4991060` forge_repo, `57045a4` forge_api, `3d85520` forge_main). STRUCTURAL FINDING: kay-core mono-crate approach hit a wall at 1323 residual errors from proc-macro self-reference, missing `include_str!` files, missing dependencies, trait object-safety, and ambiguous path issues. Mechanical path-rewrite approach (D-01 Options a/b) ruled out. **D-01 decision revised to Option (c): split kay-core into 23 workspace sub-crates** preserving ForgeCode's original structure. Plan 02-05 Task 2 (CI cleanup) absorbed into Phase 2.5. 132 files remain uncommitted in working tree (extended indented-use rewrite) — recommended to revert as obsolete since sub-crate split will redo module structure.

### Roadmap Evolution

- **2026-04-20:** Phase 2.5 inserted after Phase 2 — "kay-core sub-crate split" (URGENT, D-01 Option c). Discovered during Phase 2 plan 02-05 execution that ForgeCode's 23-crate source cannot run as a single mono-crate. Scope captured in `.planning/phases/02.5-kay-core-sub-crate-split/02.5-CONTEXT.md`. Plans 02-06..02-10 blocked on 2.5 completion; planning artifacts for all 10 Phase-2 plans remain valid (minor import path adjustments expected: `kay_core::forge_X::Y` → `kay_forge_X::Y`).

### Pending Todos

None carried over — Phase 1 closed cleanly.

### Blockers/Concerns

- **Phase 2 structural integration (cross-subtree path rewrites)**: PARTIALLY RESOLVED. Module-declaration layer resolved in plan 02-02 (E0583 = 0); sub-waves A/B/C path rewrites resolved in plan 02-03 (17/23 subtrees done, E0432|E0433 count 2025→1902); forge_app rewrite resolved in plan 02-04 (18/23 subtrees done, E0432|E0433 count 1902→1712). REMAINING: 1,712 × E0432|E0433 in the still-pending 5 subtrees (forge_services, forge_repo, forge_infra, forge_api, forge_main) — addressed by plan 02-05 (remaining subtrees + CI cleanup). `cargo check --workspace` (without `--exclude kay-core`) stays failing until 02-05 lands; that's by design. Pre-flagged in 01-03-SUMMARY.md and VERIFICATION.md §SC-4.
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
| Compile | kay-core E0432/E0433 (cross-subtree import paths) | PARTIALLY RESOLVED in plans 02-03+02-04 (2026-04-20, 17 commits 9c515a7..674f0d0 + 1 commit 808edcc — 18 of 23 subtrees rewritten, count 2025→1712). REMAINING 1,712 scheduled for plan 02-05 (forge_services/infra/repo/api/main + --exclude kay-core CI cleanup) | Phase 2 (02-02-SUMMARY §Expected Downstream Errors; 02-03/04-SUMMARY §Next Phase Readiness) |
| Release signing | GPG/SSH signing of release tags | v0.0.x unsigned carve-out; mandatory v0.1.0+ | Phase 1 (SECURITY.md §Release Signing) |

## Session Continuity

Last session: 2026-04-20 — Phase 2 Plan 04 executed (Wave 3 forge_app path rewrite)
Stopped at: Plan 02-04 complete (1 task, 1 DCO-signed path-rewrite commit 808edcc — forge_app subtree with 83 files + 212 imports; E0432|E0433 count 1902→1712, 190 resolved; zero non-use-statement content lines in commit diff; parity-baseline integrity preserved); proceeding to plan 02-05 (Wave 3 final — forge_services/infra/repo/api/main + --exclude kay-core CI cleanup, baseline E0432|E0433 = 1712)
Resume file: `.planning/phases/02-provider-hal-tolerant-json-parser/02-05-PLAN.md`
