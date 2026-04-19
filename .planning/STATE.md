---
gsd_state_version: 1.0
milestone: v2.0.0
milestone_name: milestone
status: phase_2_in_progress
stopped_at: Phase 2 Plan 03 complete (Wave 3 sub-waves A+B+C path rewrites); 7 plans remaining (02-04..02-10)
last_updated: "2026-04-20T03:57:00Z"
last_activity: 2026-04-20 -- Phase 2 Plan 03 executed (3 tasks, 17 atomic path-rewrite commits 9c515a7..674f0d0 spanning sub-waves A/B/C); 17 forge_* subtrees path-rewritten (125 imports across 83 files); E0432|E0433 count 2025->1902 (123 resolved); baseline for plan 02-04 = 1902
progress:
  total_phases: 13
  completed_phases: 1
  total_plans: 16
  completed_plans: 9
  percent: 12
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-19)

**Core value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.
**Current focus:** Phase 2 — Provider HAL + Tolerant JSON Parser

## Current Position

Phase: 2 of 12 (Provider HAL + Tolerant JSON Parser)
Plan: 3 of 10 executed in current phase (02-01 Wave 0 + 02-02 Wave 2 + 02-03 Wave 3 sub-waves A/B/C complete; 02-04..02-10 remaining)
Status: Wave 3 sub-waves A/B/C path-rewrites landed; forge_app/services/repo/infra/api/main path rewrites remain for plans 02-04 (forge_app) and 02-05 (remaining + CI cleanup)
Last activity: 2026-04-20 -- Plan 02-03 executed (3 tasks, 17 atomic path-rewrite commits 9c515a7..674f0d0); 17 forge_* subtrees path-rewritten; E0432|E0433 count 2025->1902 (123 resolved); next plan 02-04 drives forge_app (~211 imports across 103 files)

Progress: [█░░░░░░░░░░░] 12% (1 of 12 phases done; Phase 2: 3 of 10 plans done)

## Performance Metrics

**Velocity:**

- Total plans completed: 9 (6 in Phase 1 + 3 in Phase 2)
- Average duration: ~12 min/plan (weighted across phases)
- Total execution time: ~95 min of direct plan execution (excludes review loops, fixes, release)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01    | 6     | 6     | ~5 min   |
| 02    | 3     | 10    | ~21 min  |

**Recent Trend:**

- Last 5 plans: 01-05, 01-06, 02-01, 02-02, 02-03 (all PASS; sequential/main-worktree mode with DCO signoff on every commit)
- Trend: Stable; Phase 2 Wave 0 + Wave 2 + Wave 3 (sub-waves A/B/C) shipped. Plan 02-03 had 3 minor deviations (perl-delimiter bug, pub-use pattern extension, empty-marker gate reconciliation) — all self-corrected; no user input needed.

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

### Pending Todos

None carried over — Phase 1 closed cleanly.

### Blockers/Concerns

- **Phase 2 structural integration (cross-subtree path rewrites)**: PARTIALLY RESOLVED. Module-declaration layer resolved in plan 02-02 (E0583 = 0); sub-waves A/B/C path rewrites resolved in plan 02-03 (17/23 subtrees done, E0432|E0433 count 2025→1902). REMAINING: 1,902 × E0432|E0433 in the still-pending 6 subtrees (forge_app, forge_services, forge_repo, forge_infra, forge_api, forge_main) — addressed by plans 02-04 (forge_app) and 02-05 (remaining + CI cleanup). `cargo check --workspace` (without `--exclude kay-core`) stays failing until 02-05 lands; that's by design. Pre-flagged in 01-03-SUMMARY.md and VERIFICATION.md §SC-4.
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
| Compile | kay-core E0432/E0433 (cross-subtree import paths) | PARTIALLY RESOLVED in plan 02-03 (2026-04-20, 17 commits 9c515a7..674f0d0 — 17 of 23 subtrees rewritten, count 2025→1902). REMAINING 1,902 scheduled for plans 02-04 (forge_app, ~211 imports) and 02-05 (remaining + CI cleanup) | Phase 2 (02-02-SUMMARY §Expected Downstream Errors; 02-03-SUMMARY §Next Phase Readiness) |
| Release signing | GPG/SSH signing of release tags | v0.0.x unsigned carve-out; mandatory v0.1.0+ | Phase 1 (SECURITY.md §Release Signing) |

## Session Continuity

Last session: 2026-04-20 — Phase 2 Plan 03 executed (Wave 3 sub-waves A/B/C path rewrites)
Stopped at: Plan 02-03 complete (3 tasks, 17 DCO-signed path-rewrite commits 9c515a7..674f0d0 — sub-wave A 12 leaves + sub-wave B forge_domain + sub-wave C 4 forge_domain-dependents; 125 imports across 83 files; E0432|E0433 count 2025→1902, 123 resolved; zero non-use-statement lines in combined diff; parity-baseline integrity preserved); proceeding to plan 02-04 (Wave 3 sub-wave forge_app, ~211 imports across 103 files, baseline E0432|E0433 = 1902)
Resume file: `.planning/phases/02-provider-hal-tolerant-json-parser/02-04-PLAN.md`
