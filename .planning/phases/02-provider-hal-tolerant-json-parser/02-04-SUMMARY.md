---
phase: 02-provider-hal-tolerant-json-parser
plan: 04
subsystem: infra
tags: [rust, cargo, module-system, refactor, structural-integration, import-path-rewrite, forge_app]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser
    provides: "Plan 02-03 — 17 forge_* subtrees path-rewritten (12 leaves + forge_domain + 4 forge_domain-dependents); E0432|E0433 count 2025->1902. Upstream dependencies of forge_app (forge_config/display/domain/json_repair/stream/template) all have canonical `crate::forge_X::` paths, so forge_app's rewritten paths resolve to real targets."
provides:
  - "forge_app path-rewritten in a single DCO-signed atomic commit (808edcc) — largest single-subtree rewrite of Phase 2."
  - "83 files modified (20 forge_app files reference only external crates — correctly untouched)."
  - "212 import-statement rewrites applied: 98 Rule 1a (intra non-grouped) + 18 Rule 1b (intra grouped opener) + 95 Rule 2 (inter `use forge_X::`) + 1 pub-use Rule (forge_domain::truncate_key)."
  - "E0432|E0433 count reduced 1902 -> 1712 (-190 errors resolved by this plan)."
  - "Baseline for plan 02-05: residual E0432|E0433 count = 1712, recorded in /tmp/02-04-residual-errors.txt."
  - "DTO layer at forge_app/dto/openai/ (target of plan 02-08's translator) has rewritten intra-subtree paths — `use crate::forge_app::` appears in request.rs (2 occurrences). normalize_tool_schema.rs (NN-7 schema hardening) body unchanged — path reshape only."
  - "forgecode-parity-baseline tag semantic integrity preserved — zero non-use-statement content diff lines in commit 808edcc; +213/-213 import-path reshape is additively neutral."
affects: [02-05-forge-services-repo-infra-api-main-final, 02-06-provider-hal, 02-08-provider-hal-openai-translator, eval-01a]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Single atomic commit for 83-file rewrite (vs. per-subtree in 02-03) — forge_app is ONE subtree so bisect granularity is already at its finest. Scoping verified via git diff --name-only: 100% under crates/kay-core/src/forge_app/."
    - "Same 4-rule rewrite set as 02-03 (Rule 1a, Rule 1b, Rule 2, pub-use variant) applied with perl `|` delimiter to sidestep brace-matching issue with literal `{` in replacements. No new tooling needed."
    - "Pre-rewrite counting used as invariant check: measured 116 `^use crate::` + 18 `^use crate::\\{` + 95 `^use forge_*::` + 1 `^pub use forge_*::`. Post-rewrite invariant = 0 for all four categories. `grep -rh '^use crate::' | grep -v forge_*` = 0."
    - "Count reconciliation: pre-scan's `^use crate::` pattern matched BOTH non-grouped and grouped (116 inclusive of 18 grouped). True breakdown: 98 non-grouped + 18 grouped = 116 + 95 inter + 1 pub-use = 212 imports rewritten. Commit message overcounts slightly (230) due to double-counting grouped in its stated breakdown — corrected in this SUMMARY. Content integrity is unaffected; 212 is the canonical number."
    - "DTO-layer smoke signal as acceptance criterion: grep for `use crate::forge_app::` in forge_app/dto/openai/request.rs returns >0 (got 2). Confirms Rule 1a propagated into the DTO layer that plan 02-08 will consume."

key-files:
  created:
    - ".planning/phases/02-provider-hal-tolerant-json-parser/02-04-SUMMARY.md (this file)"
  modified:
    - "crates/kay-core/src/forge_app/ — 83 files modified under this subtree (of 103 total .rs files); see commit 808edcc for full list"

key-decisions:
  - "Phase 2 Plan 04: Applied the same 4-rule rewrite sequence as plan 02-03 (Rule 1a/1b/2/pub-use). No pattern variant surfaced in forge_app that wasn't already handled. `pub(crate) use` inside forge_app/transformers/mod.rs:9 is a LOCAL module re-export (not prefixed with crate:: or forge_*), correctly left untouched by the regex anchors."
  - "Phase 2 Plan 04: Single-commit-per-subtree pattern preserved (same as 02-03 sub-wave B's forge_domain commit). Atomic commit for 83 files keeps bisect granularity at subtree level (the finest granularity that makes sense — within a subtree, all rewrites are same-class transforms)."
  - "Phase 2 Plan 04: PROV-01 checkbox NOT marked — same rationale as plans 02-02/03. PROV-01 is a behavioral requirement (Provider trait + tool calling + SSE + typed AgentEvent), which structural path rewriting cannot fulfill. ROADMAP correctly labels 02-04 as '(PROV-01 prereq)'. Behavioral completion still owned by 02-06 (trait scaffolding) and 02-08 (OpenRouterProvider impl)."

patterns-established:
  - "Per-subtree atomic rewrite for large subtrees: when a subtree has >50 files or >150 imports (forge_app had 103/212), still use a single atomic commit — finer granularity within a subtree would be false precision (all rewrites are mechanical same-class transforms), and the per-commit file-scope invariant (verified via git diff --name-only) keeps the commit bounded."
  - "Non-use-statement content-preservation invariant: git diff HEAD~1..HEAD | grep -E '^[-+]' | grep -vE '^[-+]{3}' | grep -vE '^[-+](use |pub use )' | grep -vE '^[-+]$' = 0. The `-/+` bare lines (blank lines in import blocks) are semantically neutral and may be non-zero, but true content changes must be zero. Proven for commit 808edcc."
  - "Post-rewrite error-count gate as smoke signal: cargo check -p kay-core 2>&1 | grep -cE '(E0432|E0433)' must be strictly less than the pre-rewrite count. 1902 -> 1712 (-190) satisfies this; confirms the rewritten paths resolve to real targets (i.e., the upstream subtrees' rewrites from 02-03 are correct)."

requirements-completed: []  # PROV-01 is behavioral; this plan is a "PROV-01 prereq" per ROADMAP.md §Phase 2 Plans.

# Metrics
duration: ~6min
completed: 2026-04-20
---

# Phase 2 Plan 04: forge_app Path Rewrites Summary

**One DCO-signed commit (808edcc) path-rewrites 83 files in forge_app — 212 import statements across the largest single subtree of Phase 2, driving E0432|E0433 count from 1902 to 1712 (190 errors resolved) with zero non-use-statement changes (parity-baseline semantics preserved).**

## Performance

- **Duration:** ~6 min (invariant-check + rewrite + commit + verify)
- **Tasks:** 1 (single atomic forge_app rewrite)
- **Commits:** 1 rewrite commit (808edcc) + 1 metadata commit (pending) = 2 DCO-signed commits on plan 02-04
- **Files modified:** 83 (of 103 .rs files in forge_app; 20 files reference only external crates)
- **Imports rewritten:** 212 (+213 lines, -213 lines; delta-neutral per rustfmt blank-line reshuffle)

## Accomplishments

- **forge_app path-rewritten in a single atomic commit (808edcc)** — the largest single-subtree rewrite of Phase 2 (83 files, 212 imports).
- **Category breakdown:**
  - 98 × Rule 1a (intra-subtree non-grouped `use crate::X`)
  - 18 × Rule 1b (intra-subtree grouped opener `use crate::{...}`)
  - 95 × Rule 2 (inter-subtree `use forge_X::Y` to 6 upstream subtrees: forge_config, forge_display, forge_domain, forge_json_repair, forge_stream, forge_template)
  - 1 × pub-use Rule (`pub use forge_domain::truncate_key` at utils.rs:46)
- **DTO layer smoke signal confirmed:** `use crate::forge_app::` appears 2× in forge_app/dto/openai/request.rs. This is the surface plan 02-08's translator will consume; Rule 1a landed cleanly in the DTO layer.
- **Zero cross-subtree contamination:** git diff --name-only HEAD~1..HEAD confirmed all 83 files are under `crates/kay-core/src/forge_app/`. Zero files outside.
- **Parity-baseline integrity preserved:** Combined diff shows +213/-213 (perfectly balanced) and zero non-use-statement content lines. The forgecode-parity-baseline tag (commit 8af1f2b) remains semantically valid — commit 808edcc is syntactic reshaping of import paths only.
- **Non-regression intact:** `cargo check --workspace --exclude kay-core` continues to exit 0; `cargo check -p kay-provider-openrouter --tests` continues to exit 0.

## Task Commits

1. **Task 1 (forge_app single atomic rewrite):** `808edcc` refactor(02-04): rewrite imports for forge_app (83 files, 230 imports)
   - *Note: commit message overcounts slightly at 230 due to double-counting grouped imports in its stated breakdown. True count is 212 (reconciled in this SUMMARY's tech-stack patterns). Content integrity unaffected.*

**Plan metadata:** pending — `docs(02-04): complete forge_app rewrite plan — SUMMARY + STATE + ROADMAP` commit includes SUMMARY.md + STATE.md + ROADMAP.md updates.

## E0432|E0433 Error Trajectory

| After commit | Subtree | E0432\|E0433 | Δ | Cumulative |
|---|---|---|---|---|
| *pre-plan baseline (post 02-03)* | — | 1902 | — | — |
| 808edcc | forge_app | 1712 | -190 | -190 |

**Trajectory:** single monotonic reduction step (1902 -> 1712, -190). Plan 02-05 inherits residual = **1712** (recorded in `/tmp/02-04-residual-errors.txt`).

**Remaining subtrees for plan 02-05:** forge_services, forge_infra, forge_repo, forge_api, forge_main (5 subtrees). Expected error-count reduction at 02-05: from 1712 down to a low-3-digit or 2-digit residual (or 0 if the remaining subtrees account for all cross-subtree import resolution). Exact count will be driven by 02-05.

## Verification Evidence

| Check | Expected | Actual | Source |
|---|---|---|---|
| Single path-rewrite commit landed | 1 | 1 | `git log --oneline HEAD~1..HEAD \| grep -c "refactor(02-04):"` → 1 |
| Commit is DCO-signed | yes | yes | `git log -1 --pretty=%B \| grep -c "Signed-off-by:"` → 1 |
| All modified files under forge_app/ | 83/83 | 83/83 | `git diff --name-only HEAD~1..HEAD \| grep -v "^crates/kay-core/src/forge_app/" \| wc -l` → 0 |
| Un-rewritten `use crate::X` (non-forge) in forge_app | 0 | 0 | `grep -rhE "^(pub )?use crate::" forge_app \| grep -v "use crate::forge_app" \| grep -v "use crate::forge_" \| wc -l` → 0 |
| Un-rewritten bare `use forge_X::` in forge_app | 0 | 0 | `grep -rhE "^(pub )?use forge_[a-z_]+::" forge_app \| wc -l` → 0 |
| Non-use-statement diff lines | 0 | 0 | `git diff HEAD~1..HEAD \| grep -E "^[-+]" \| grep -vE "^[-+]{3}" \| grep -vE "^[-+](use \|pub use )" \| grep -vE "^[-+]$" \| wc -l` → 0 |
| Diff balance (+ = -) | equal | 213 = 213 | `git diff HEAD~1..HEAD --numstat \| awk '{a+=$1; d+=$2} END {print "+"a" -"d}'` |
| DTO layer smoke signal | >0 | 2 | `grep -c "use crate::forge_app::" forge_app/dto/openai/request.rs` → 2 |
| `cargo check -p kay-core` E0432\|E0433 | < 1902 | 1712 | `cargo check -p kay-core 2>&1 \| grep -cE "(E0432\|E0433)"` |
| `cargo check --workspace --exclude kay-core` | exit 0 | exit 0 | "Finished `dev` profile" |
| `cargo check -p kay-provider-openrouter --tests` | exit 0 | exit 0 | "Finished `dev` profile" |
| `/tmp/02-04-residual-errors.txt` contents | non-neg int | 1712 | `cat /tmp/02-04-residual-errors.txt` |

## Decisions Made

1. **Same 4-rule rewrite as plan 02-03, single atomic commit.** No new pattern variants surfaced in forge_app. The perl `|` delimiter choice from 02-03 handled grouped `use crate::{` correctly. Rule 1a, Rule 1b, Rule 2, pub-use Rule all fired; post-rewrite invariants returned 0 for all four categories.

2. **Count reconciliation between commit message and SUMMARY.** Commit message states "230 imports" (sum of pre-scan counts: 116 + 18 + 95 + 1). True count is 212 because pre-scan's Rule 1a regex (`^use crate::`) matched BOTH non-grouped AND grouped openers, so 18 grouped were counted twice. Canonical count reconciled in this SUMMARY: 98 Rule 1a (non-grouped) + 18 Rule 1b (grouped opener) + 95 Rule 2 (inter) + 1 pub-use = 212. Content integrity unaffected; the commit itself is correct (changes 213 lines in each direction, perfectly balanced).

3. **PROV-01 checkbox remains unchecked.** This plan is structural-only (path rewriting) and cannot behaviorally satisfy PROV-01 (Provider trait + tool calling + SSE + typed AgentEvent). ROADMAP correctly labels 02-04 as "(PROV-01 prereq)". Behavioral completion still owned by 02-06 (trait scaffolding) and 02-08 (OpenRouterProvider impl).

## Deviations from Plan

None. Plan 02-04 executed exactly as specified. The plan's 4-rule rewrite (inherited from 02-03 as "plan-02-03 transform applied to forge_app") fired with zero surprises. The commit message's "230 imports" slight overcount (versus true 212) is documented in this SUMMARY's Decisions section and tech-stack patterns, but does NOT constitute a deviation — the commit content is correct; only the stated breakdown number is off by 18 due to regex double-counting. Content integrity, DCO signoff, file scoping, error-count reduction, and all acceptance criteria are satisfied.

## Issues Encountered

- **None material.** Pre-rewrite scan counts showed the same pattern distribution as plan 02-03 predicted (116 Rule 1a + 18 Rule 1b + 95 Rule 2 + 1 pub-use). Post-rewrite invariants returned 0 for all four categories on first pass. No hand-edits needed.

## Known Stubs

None. This plan performs import-path syntactic rewrites only — no placeholders, no hardcoded empty data, no TODO markers introduced.

## Next Phase Readiness

- **Ready for Plan 02-05 (Wave 3 final: forge_services, forge_infra, forge_repo, forge_api, forge_main + CI cleanup):** 18 of 23 forge_* subtrees path-rewritten (12 leaves + forge_domain + 4 forge_domain-dependents from 02-03 + forge_app from 02-04). Remaining: 5 subtrees — forge_services (service layer), forge_infra (infrastructure), forge_repo (ForgeCode's OpenAI/OpenRouter provider — D-02 delegation target), forge_api (API surface), forge_main (CLI entrypoint). Baseline E0432|E0433 for plan 02-05 = 1712.
- **Ready for Plan 02-08 (Provider translator):** forge_app/dto/openai/ layer has canonical `use crate::forge_app::` paths internally. Once kay-core becomes a Cargo dep of kay-provider-openrouter in plan 02-06, the translator can consume `use kay_core::forge_app::dto::openai::{Request, Response, ...}` — and the `dto/openai/transformers/normalize_tool_schema.rs` body (NN-7 schema hardening) is unchanged by 02-04 (import paths only), so semantic behavior remains parity-preserved.
- **Parity-baseline integrity intact:** `forgecode-parity-baseline` tag (commit 8af1f2b) still points at the unmodified import. Commits 02-02, 02-03 (17 commits), and 02-04 (1 commit) landed downstream are structural-rename or import-path-syntax reshapes — they do NOT alter the byte content of any ForgeCode logic (confirmed via +213/-213 balance and zero non-use-statement lines in commit 808edcc; same invariant held across plan 02-03's combined 17-commit diff).
- **Non-regression intact:** Plan 02-01 (MockServer + cassettes) continues to build clean. `cargo check --workspace --exclude kay-core` and `cargo check -p kay-provider-openrouter --tests` both exit 0. Wave 0 deliverables unaffected.

## Self-Check: PASSED

Verified claims (commands re-run after SUMMARY drafted):

- **Single path-rewrite commit landed:** `git log --oneline HEAD~1..HEAD | grep -c "refactor(02-04):"` → 1 (commit 808edcc).
- **Commit is DCO-signed:** `git log -1 --pretty=%B 808edcc | grep -c "Signed-off-by:"` → 1.
- **All 83 modified files scoped to forge_app:** `git diff --name-only HEAD~1..HEAD | grep -v "^crates/kay-core/src/forge_app/"` → 0 lines.
- **Total file count:** `git diff --name-only HEAD~1..HEAD | wc -l` → 83.
- **Post-rewrite invariants clean:** `grep -rhE "^(pub )?use crate::" forge_app | grep -v "use crate::forge_app" | grep -v "use crate::forge_" | wc -l` → 0; `grep -rhE "^(pub )?use forge_[a-z_]+::" forge_app | wc -l` → 0.
- **Diff balance:** +213 = -213 (import-path reshape is additively neutral).
- **Zero non-use-statement content changes:** `git diff | grep -E "^[-+]" | grep -vE "^[-+]{3}" | grep -vE "^[-+](use |pub use )" | grep -vE "^[-+]$" | wc -l` → 0.
- **E0432|E0433 trajectory monotonically decreased:** 1902 → 1712 (-190, confirmed via `cargo check -p kay-core 2>&1 | grep -cE "(E0432|E0433)"`).
- **Residual E0432|E0433 count = 1712:** strictly less than pre-plan baseline of 1902 (Δ = -190); recorded in `/tmp/02-04-residual-errors.txt`.
- **Non-regression checks green:** `cargo check --workspace --exclude kay-core` exit 0; `cargo check -p kay-provider-openrouter --tests` exit 0.
- **DTO smoke signal:** `grep -c "use crate::forge_app::" crates/kay-core/src/forge_app/dto/openai/request.rs` → 2.

All success criteria met. All acceptance criteria from the plan met. Zero deviations (the commit message count slight-overcount is documented not as a deviation but as a reconciliation in Decisions §2).

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Completed: 2026-04-20*
