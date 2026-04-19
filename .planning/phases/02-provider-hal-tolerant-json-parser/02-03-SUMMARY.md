---
phase: 02-provider-hal-tolerant-json-parser
plan: 03
subsystem: infra
tags: [rust, cargo, module-system, refactor, structural-integration, import-path-rewrite]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser
    provides: "Plan 02-02 — 23 forge_*/lib.rs renamed to mod.rs atomically (commit bb57694). E0583 count dropped 24->0; E0432|E0433 baseline for Wave 3 = 2025."
provides:
  - "17 forge_* subtrees path-rewritten (sub-waves A, B, C) — 12 leaves + forge_domain + 4 forge_domain-dependents."
  - "125 import-statement rewrites across 83 files in 17 DCO-signed commits (one per subtree)."
  - "E0432|E0433 count reduced 2025 -> 1902 (123 errors resolved by this plan)."
  - "Sub-wave A (12 leaves): forge_json_repair, forge_stream, forge_template, forge_tool_macros, forge_walker, forge_test_kit, forge_embed, forge_ci, forge_config, forge_display, forge_select, forge_markdown_stream — 44 errors resolved."
  - "Sub-wave B (forge_domain): 46 files, 61 imports (54 intra + 6 inter + 1 pub-use) — 60 errors resolved."
  - "Sub-wave C (4 forge_domain-dependents): forge_fs, forge_spinner, forge_tracker, forge_snaps — 19 errors resolved."
  - "Baseline for plan 02-04: residual E0432|E0433 count = 1902, recorded in /tmp/02-03-residual-errors.txt."
  - "forgecode-parity-baseline tag semantic integrity preserved — all 17 commits are import-path-syntax-only changes (zero non-use-statement lines in combined diff)."
affects: [02-04-forge-app-rewrites, 02-05-forge-services-final-rewrites, 02-06-provider-hal, eval-01a]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-subtree atomic commit pattern: one DCO-signed commit per forge_X subtree for bisect granularity (even when a subtree has zero imports to rewrite — empty marker commits preserve the 12/1/4 sub-wave structure)."
    - "perl -i -pe with | as regex delimiter (not s{}{}) — avoids brace-matching issues when the replacement contains literal { for grouped-import rewrites like `use crate::{`."
    - "Three-rule rewrite sequence: Rule 1a (`^use crate::X` non-grouped, not forge_), Rule 1b (`^use crate::{` grouped opener), Rule 2 (`^use forge_Y::` inter-subtree). Applied per subtree with post-rewrite invariant checks."
    - "pub use handled as a fourth rule for the single case (forge_domain/session_metrics.rs:9) where the plan's regex extension to ^(pub )?use crate:: surfaced a residual. Documented as pattern extension for plans 02-04/05."
    - "Monotonic error-count reduction gate: cargo check -p kay-core 2>&1 | grep -cE 'E0432|E0433' must be non-increasing across each sub-wave commit. Proven across all 17 commits (trajectory: 2025 -> 2023 -> 1981 -> 1921 -> 1902)."

key-files:
  created:
    - ".planning/phases/02-provider-hal-tolerant-json-parser/02-03-SUMMARY.md (this file)"
  modified:
    - "crates/kay-core/src/forge_json_repair/ (2 files, 2 imports)"
    - "crates/kay-core/src/forge_ci/ (8 files, 11 imports)"
    - "crates/kay-core/src/forge_config/ (6 files, 9 imports incl. 1 multi-line grouped)"
    - "crates/kay-core/src/forge_display/ (1 file, 1 import)"
    - "crates/kay-core/src/forge_select/ (3 files, 6 imports)"
    - "crates/kay-core/src/forge_markdown_stream/ (7 files, 16 imports)"
    - "crates/kay-core/src/forge_domain/ (46 files, 61 imports: 54 intra + 6 inter + 1 pub-use)"
    - "crates/kay-core/src/forge_fs/ (3 files, 5 imports: 2 intra + 1 inter + 2 pub-use)"
    - "crates/kay-core/src/forge_spinner/ (1 file, 1 inter-subtree import)"
    - "crates/kay-core/src/forge_tracker/ (5 files, 11 imports: 8 intra + 2 inter + 1 grouped)"
    - "crates/kay-core/src/forge_snaps/ (1 file, 2 inter-subtree imports)"

key-decisions:
  - "Phase 2 Plan 03: Used `perl -i -pe` with `|` as regex delimiter rather than `s{}{}` form to sidestep brace-matching issues when the replacement contained literal `{` (for grouped `use crate::{` rewrites). The alternate delimiter is now the canonical tooling for plans 02-04/05."
  - "Phase 2 Plan 03: Extended the rewrite-subtree invariant check to account for `pub use crate::` (not just `use crate::`) after discovering one instance in forge_domain/session_metrics.rs. Plans 02-04/05 must apply all four rewrite rules (Rule 1a, 1b, 2, and pub-use variant) for completeness."
  - "Phase 2 Plan 03: Empty marker commits (via `git commit --allow-empty`) used for 4 leaf subtrees that had zero intra/inter imports to rewrite (forge_stream, forge_template, forge_tool_macros, forge_walker, forge_test_kit, forge_embed — correction: 6, not 4). This preserves the 12-commit sub-wave-A gate from the plan's verify block without fabricating content changes. These subtrees only reference external crates (std, anyhow, handlebars, proc_macro, etc.) — no forge_*-internal paths."

patterns-established:
  - "Per-subtree path-rewrite pattern: for each forge_X, run Rule 1a/1b/2 via perl -i on all files under crates/kay-core/src/forge_X/, verify three invariants (intra-remain=0, pub-use-remain=0, inter-legacy=0), commit with DCO signoff. Dependency order matters: leaves first, then forge_domain (depends on 3 leaves), then forge_domain-dependents."
  - "Error-count sanity gate: after each subtree's rewrite commit, cargo check -p kay-core 2>&1 | grep -cE 'E0432|E0433' must be non-increasing. This catches mis-rewrites that introduce NEW unresolved imports (a category of regression not caught by the post-rewrite invariant grep, since mis-targeted paths still look `correct` syntactically)."
  - "Zero non-use-statement lines across the combined diff of all N commits: git diff HEAD~N..HEAD | grep -vE 'use |pub use |^(diff|index|---|\\+\\+\\+|@@|\\s)' must be empty. This is the content-preservation gate for the forgecode-parity-baseline tag's semantic integrity."

requirements-completed: []  # PROV-01 is a behavioral requirement — still in progress via 02-06/08. This plan is a "PROV-01 prereq" per ROADMAP.md §Phase 2 Plans.

# Metrics
duration: 42min
completed: 2026-04-20
---

# Phase 2 Plan 03: kay-core Path Rewrites (Sub-waves A, B, C) Summary

**17 DCO-signed commits path-rewrite 17 forge_* subtrees — 125 import statements across 83 files, driving E0432|E0433 count from 2025 to 1902 (123 errors resolved) with zero non-use-statement changes (parity-baseline semantics preserved).**

## Performance

- **Duration:** ~42 min (2026-04-20T03:15:00Z start → 2026-04-20T03:57:00Z metadata-commit)
- **Tasks:** 3 (Task 1 sub-wave A 12 commits, Task 2 sub-wave B 1 commit, Task 3 sub-wave C 4 commits)
- **Commits:** 17 path-rewrite + 1 metadata = 18 DCO-signed commits total on plan 02-03
- **Files modified:** 83 (all in `crates/kay-core/src/forge_*/`)
- **Imports rewritten:** 125 (+125 lines, -125 lines; delta-neutral)

## Accomplishments

- **Sub-wave A (12 leaf subtrees, zero inter-subtree deps) — 12 DCO-signed commits:**
  1. `forge_json_repair` (commit `9c515a7`) — 2 files, 2 imports — E0432|E0433: 2025 → 2023
  2. `forge_stream` (commit `e3f8f73`) — empty marker, 0 imports (std/futures/tokio only)
  3. `forge_template` (commit `a07fdf0`) — empty marker, 0 imports
  4. `forge_tool_macros` (commit `a01a1c1`) — empty marker, 0 imports (proc_macro/quote/syn only)
  5. `forge_walker` (commit `5ed3ac7`) — empty marker, 0 imports
  6. `forge_test_kit` (commit `43d5c8b`) — empty marker, 0 imports
  7. `forge_embed` (commit `f77d0ab`) — empty marker, 0 imports (handlebars/include_dir only)
  8. `forge_ci` (commit `9d012b0`) — 8 files, 11 imports — E0432|E0433: 2023 → 2013
  9. `forge_config` (commit `594b545`) — 6 files, 9 imports (incl. 1 multi-line grouped) — 2013 → 2004
  10. `forge_display` (commit `fb23dbf`) — 1 file, 1 import — 2004 → 2003
  11. `forge_select` (commit `ddc5fa7`) — 3 files, 6 imports — 2003 → 1997
  12. `forge_markdown_stream` (commit `3587c7d`) — 7 files, 16 imports — 1997 → 1981

- **Sub-wave B (forge_domain, central domain types) — 1 DCO-signed commit:**
  - `forge_domain` (commit `88f9ad3`) — 46 files, 61 imports (54 intra + 6 inter to forge_json_repair/forge_template/forge_tool_macros + 1 pub-use in session_metrics.rs) — E0432|E0433: 1981 → 1921 (60 errors resolved)

- **Sub-wave C (4 forge_domain-dependents) — 4 DCO-signed commits:**
  13. `forge_fs` (commit `7c5c1d7`) — 3 files, 5 imports (2 intra + 1 inter to forge_domain + 2 pub-use) — E0432|E0433: 1921 → 1916
  14. `forge_spinner` (commit `3147961`) — 1 file, 1 inter-subtree import to forge_domain — 1916 → 1915
  15. `forge_tracker` (commit `e1bdad3`) — 5 files, 11 imports (8 intra + 2 inter + 1 grouped) — 1915 → 1904
  16. `forge_snaps` (commit `674f0d0`) — 1 file, 2 inter-subtree imports (forge_domain + forge_fs) — 1904 → 1902

- **Parity-baseline integrity preserved:** Combined diff across HEAD~17..HEAD contains zero non-use-statement lines. 125 insertions = 125 deletions. The forgecode-parity-baseline tag (commit 8af1f2b) remains semantically valid — these 17 commits are syntactic reshaping of import paths only.
- **Non-regression intact:** `cargo check --workspace --exclude kay-core` continues to exit 0; `cargo check -p kay-provider-openrouter --tests` continues to exit 0. Plan 02-01 (MockServer + cassettes) unaffected.

## Task Commits

Each task was committed atomically (DCO-signed, per-subtree):

1. **Task 1 (Sub-wave A: 12 leaf subtrees) — 12 commits:**
   - `9c515a7` refactor(02-03): path-rewrite forge_json_repair (sub-wave A, leaf)
   - `e3f8f73` refactor(02-03): path-rewrite forge_stream (sub-wave A, leaf)
   - `a07fdf0` refactor(02-03): path-rewrite forge_template (sub-wave A, leaf)
   - `a01a1c1` refactor(02-03): path-rewrite forge_tool_macros (sub-wave A, leaf)
   - `5ed3ac7` refactor(02-03): path-rewrite forge_walker (sub-wave A, leaf)
   - `43d5c8b` refactor(02-03): path-rewrite forge_test_kit (sub-wave A, leaf)
   - `f77d0ab` refactor(02-03): path-rewrite forge_embed (sub-wave A, leaf)
   - `9d012b0` refactor(02-03): path-rewrite forge_ci (sub-wave A, leaf)
   - `594b545` refactor(02-03): path-rewrite forge_config (sub-wave A, leaf)
   - `fb23dbf` refactor(02-03): path-rewrite forge_display (sub-wave A, leaf)
   - `ddc5fa7` refactor(02-03): path-rewrite forge_select (sub-wave A, leaf)
   - `3587c7d` refactor(02-03): path-rewrite forge_markdown_stream (sub-wave A, leaf)

2. **Task 2 (Sub-wave B: forge_domain) — 1 commit:**
   - `88f9ad3` refactor(02-03): path-rewrite forge_domain (sub-wave B)

3. **Task 3 (Sub-wave C: forge_fs, forge_spinner, forge_tracker, forge_snaps) — 4 commits:**
   - `7c5c1d7` refactor(02-03): path-rewrite forge_fs (sub-wave C)
   - `3147961` refactor(02-03): path-rewrite forge_spinner (sub-wave C)
   - `e1bdad3` refactor(02-03): path-rewrite forge_tracker (sub-wave C)
   - `674f0d0` refactor(02-03): path-rewrite forge_snaps (sub-wave C)

**Plan metadata:** (pending — `docs(02-03): complete path-rewrite plan` commit includes SUMMARY + STATE + ROADMAP updates)

## E0432|E0433 Error Trajectory

| After commit | Sub-wave | Subtree | E0432\|E0433 | Δ | Cumulative |
|---|---|---|---|---|---|
| *pre-plan baseline (post 02-02)* | — | — | 2025 | — | — |
| 9c515a7 | A | forge_json_repair | 2023 | -2 | -2 |
| e3f8f73 | A | forge_stream | 2023 | 0 | -2 |
| a07fdf0 | A | forge_template | 2023 | 0 | -2 |
| a01a1c1 | A | forge_tool_macros | 2023 | 0 | -2 |
| 5ed3ac7 | A | forge_walker | 2023 | 0 | -2 |
| 43d5c8b | A | forge_test_kit | 2023 | 0 | -2 |
| f77d0ab | A | forge_embed | 2023 | 0 | -2 |
| 9d012b0 | A | forge_ci | 2013 | -10 | -12 |
| 594b545 | A | forge_config | 2004 | -9 | -21 |
| fb23dbf | A | forge_display | 2003 | -1 | -22 |
| ddc5fa7 | A | forge_select | 1997 | -6 | -28 |
| 3587c7d | A | forge_markdown_stream | 1981 | -16 | -44 |
| 88f9ad3 | B | forge_domain | 1921 | -60 | -104 |
| 7c5c1d7 | C | forge_fs | 1916 | -5 | -109 |
| 3147961 | C | forge_spinner | 1915 | -1 | -110 |
| e1bdad3 | C | forge_tracker | 1904 | -11 | -121 |
| 674f0d0 | C | forge_snaps | 1902 | -2 | -123 |

**Trajectory:** strictly monotonically non-increasing across all 17 commits. Plan 02-04 inherits residual = **1902** (recorded in `/tmp/02-03-residual-errors.txt`).

## Verification Evidence

| Check | Expected | Actual | Source |
|---|---|---|---|
| Sub-wave A commits | 12 | 12 | `git log --oneline -50 \| grep -c "refactor(02-03): path-rewrite forge_.* (sub-wave A"` |
| Sub-wave B commits | 1 | 1 | `git log --oneline -50 \| grep -c "refactor(02-03): path-rewrite forge_domain (sub-wave B)"` |
| Sub-wave C commits | 4 | 4 | `git log --oneline -50 \| grep -c "refactor(02-03): path-rewrite forge_.* (sub-wave C)"` |
| Total 02-03 commits | 17 | 17 | `git log --oneline -50 \| grep -c "refactor(02-03):"` |
| DCO signoffs | 17/17 | 17/17 | Verified per-commit via `git log -1 --format=%B <sha> \| grep -c "Signed-off-by:"` for each of 17 commits |
| All 17 subtrees invariant-clean | 17/17 | 17/17 | For each subtree: `grep -rh "^use crate::"` ∩ not-self ∩ not-forge_* → 0 lines; `grep -rh "^use forge_X::"` → 0 lines |
| Non-use-statement diff lines | 0 | 0 | `git diff HEAD~17..HEAD -- 'crates/kay-core/src/forge_*' \| grep -vE "^(diff\|index\|---\|\\+\\+\\+\|@@\|-use \|-pub use \|+use \|+pub use \|\\s)"` |
| Cross-subtree contamination | 0 commits | 0 commits | Per-commit scan: each of 17 commits touches only files under its own subtree dir |
| Diff balance (+ = -) | equal | 125 = 125 | `git log HEAD~17..HEAD --numstat --format= \| awk '{a+=$1; d+=$2} END {print "+"a" -"d}'` |
| `cargo check -p kay-core` E0583 | 0 | 0 | Tail of cargo output across all 17 per-subtree checks |
| `cargo check -p kay-core` E0432\|E0433 | < 2025 | 1902 | Final post-plan count |
| `cargo check --workspace --exclude kay-core` | exit 0 | exit 0 | "Finished \`dev\` profile" |
| `cargo check -p kay-provider-openrouter --tests` | exit 0 | exit 0 | "Finished \`dev\` profile" |

## Decisions Made

1. **`perl -i -pe` with `|` delimiter:** The helper script `/tmp/rewrite-subtree.sh` initially used `s{pattern}{replacement}g` form (matching the plan text's regex shape), which failed with `Bareword found where operator expected at -e line 1, near '}g'` when the replacement contained literal `{` (for the grouped `use crate::{` case). Switched to `s|pattern|replacement|g` form — clean, unambiguous, no escape games. Plan 02-04/05 should use the same delimiter.

2. **Empty marker commits for zero-import subtrees:** 6 of the 12 sub-wave A subtrees (`forge_stream`, `forge_template`, `forge_tool_macros`, `forge_walker`, `forge_test_kit`, `forge_embed`) have zero intra-subtree `use crate::` and zero inter-subtree `use forge_` imports — they only reference external crates (std, futures, handlebars, proc_macro, etc.). The plan's verify gate hard-requires 12 sub-wave A commits (line 248). Reconciling: used `git commit -s --allow-empty` with a descriptive message explaining the zero-rewrite reason. This preserves bisectability and the 12-commit gate without fabricating content. Documented as a Rule-3 style reconciliation (gate vs. action text mismatch).

3. **`pub use crate::` as a fourth rewrite rule:** Plan action text (Task 2 fallback guidance) anticipates `pub use crate::X` as a corner case, suggesting pattern extension to `^(pub )?use crate::`. Discovered one instance in `forge_domain/session_metrics.rs:9` — `pub use crate::file_operation::FileOperation` — which slipped past my first-pass script (which only matched `^use crate::`). Handled via a targeted follow-on `perl -i -pe` invocation on the single file. For plans 02-04/05, the rewrite helper should fold this into the main rule set.

4. **Sub-wave C execution order confirmed:** forge_snaps last, per plan — it depends on forge_fs (rewritten first in sub-wave C) AND forge_domain (sub-wave B). Order forge_fs → forge_spinner → forge_tracker → forge_snaps preserves dependency-lowest-first.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Perl regex delimiter bug in helper script**
- **Found during:** Task 1 (forge_config subtree, 9th commit)
- **Issue:** Helper script's `s{^use crate::\{}{use crate::$SUBTREE::{}g` form caused perl to fail with syntax error ("Bareword found where operator expected near '}g'") because the literal `{` in the replacement confused perl's brace-matching for the `s{}{}` delimiter pair. Rule 1a ran first and rewrote non-grouped imports; Rule 1b then aborted, leaving grouped `use crate::{...}` imports un-rewritten. Verify gate caught this (residual=1 for forge_config/config.rs line 11).
- **Fix:** Switched the helper to `s|^use crate::\{|use crate::${SUBTREE}::\{|g` (pipe delimiter). Re-ran on forge_config; all grouped imports rewrote cleanly. Re-verified all subsequent subtree rewrites with the fixed helper.
- **Files modified:** `/tmp/rewrite-subtree.sh` (script only, not in-repo).
- **Verification:** Forge_config multi-line grouped import at `config.rs:11` correctly rewrote to `use crate::forge_config::{...`. All subsequent subtree rewrites had `remain=0 legacy=0` invariant checks pass.
- **Committed in:** 594b545 (forge_config commit includes the fixed-helper output)

**2. [Rule 2 - Missing Critical] pub use crate:: variant not in initial helper**
- **Found during:** Task 2 (forge_domain subtree, sub-wave B)
- **Issue:** Helper only handled `^use crate::` forms, missed `^pub use crate::file_operation::FileOperation` in `forge_domain/session_metrics.rs:9`. This was flagged in the plan's action text as a known corner case ("likely causes are...`pub use crate::X` which should also be rewritten"). If left un-rewritten, the `pub use` exports an unresolved path (still crashes the re-export layer).
- **Fix:** Applied a targeted `perl -i -pe 's|^pub use crate::(?!forge_)(?!\{)|pub use crate::forge_domain::|g'` on the single affected file. Then re-verified the 3-class invariant: use-remain=0, pub-use-remain=0, forge-legacy=0. All clear.
- **Files modified:** `crates/kay-core/src/forge_domain/session_metrics.rs`.
- **Verification:** `grep -rh "^pub use crate::" crates/kay-core/src/forge_domain | grep -v "forge_" | wc -l` = 0.
- **Committed in:** 88f9ad3 (forge_domain sub-wave B commit)

**3. [Rule 3 - Blocking] 6 leaf subtrees with zero imports — empty marker commit reconciliation**
- **Found during:** Task 1 (forge_stream, forge_template, forge_tool_macros, forge_walker, forge_test_kit, forge_embed)
- **Issue:** The plan's verify gate hard-requires 12 sub-wave A commits (`test $(git log --oneline -50 | grep -c "refactor(02-03): path-rewrite forge_.* (sub-wave A") -eq 12`) but the action text says "If the subtree just introduced no net changes (e.g., it had no forge_* deps to rewrite), that's OK; skip the commit." These two rules contradict when a subtree has zero imports.
- **Fix:** Used `git commit -s --allow-empty` to create a no-op marker commit per such subtree, with a descriptive message explaining the zero-rewrite reason. This satisfies the 12-commit gate, preserves per-subtree bisectability, makes no content changes, and carries DCO signoff.
- **Files modified:** None (empty commits).
- **Verification:** `git log --oneline -50 | grep -c "refactor(02-03): path-rewrite forge_.* (sub-wave A"` = 12 as required.
- **Committed in:** e3f8f73, a07fdf0, a01a1c1, 5ed3ac7, 43d5c8b, f77d0ab (6 empty marker commits)

---

**Total deviations:** 3 auto-fixed (1 blocking script bug, 1 missing critical pattern, 1 gate-reconciliation)
**Impact on plan:** All three are necessary for correctness / completeness. Scope fully preserved; no architectural changes. Plans 02-04/05 can adopt the fixed helper pattern directly.

## Issues Encountered

- **None material.** The `forge_stream` subtree initially surfaced the zero-import empty-marker question (resolved as Deviation #3 above). The `forge_config` subtree surfaced the perl-delimiter bug (resolved as Deviation #1). The `forge_domain` subtree surfaced the `pub use` variant (resolved as Deviation #2). All three were self-diagnosed and self-corrected within task execution.

## Known Stubs

None. This plan performs import-path syntactic rewrites only — no placeholders, no hardcoded empty data, no TODO markers introduced.

## Next Phase Readiness

- **Ready for Plan 02-04 (Wave 3 sub-wave forge_app):** 17 of 23 forge_* subtrees path-rewritten. Remaining: `forge_app` (~211 imports across 103 files per RESEARCH §Pitfall 1), `forge_services`, `forge_repo`, `forge_infra`, `forge_api`, `forge_main`. Baseline E0432|E0433 for plan 02-04 = 1902.
- **Ready for Plan 02-05 (Wave 3 final + CI cleanup):** Once forge_app is done, plans 02-05 closes the path-rewrite work and removes the `--exclude kay-core` escape clause from ci.yml / CONTRIBUTING.md / docs/CICD.md. STATE.md updates per ROADMAP §Phase 2.
- **Parity-baseline integrity intact:** `forgecode-parity-baseline` tag (commit 8af1f2b) still points at the unmodified import. The 17 commits landed in plan 02-03 are downstream import-path-syntax reshapes — they do NOT alter the byte content of any ForgeCode logic (confirmed via zero-non-use-line combined diff).
- **Non-regression intact:** Plan 02-01 (MockServer + cassettes) continues to build clean. Wave 0 deliverables unaffected.

## Self-Check: PASSED

Verified claims (commands re-run after SUMMARY drafted):

- **17 path-rewrite commits landed:** `git log --oneline HEAD~17..HEAD | grep -c "refactor(02-03):"` → 17.
- **Sub-wave counts:** 12 A + 1 B + 4 C = 17 (all three grep-counts match).
- **All 17 commits DCO-signed:** per-commit signoff-presence loop → 17/17.
- **Each commit scoped to its own subtree:** per-commit `git show --stat --name-only` → all file paths under the commit's subject-declared subtree; zero cross-subtree contamination.
- **Combined diff balance:** +125 = -125 (import-path reshape is additively neutral).
- **Zero non-use-statement lines in combined diff:** `git diff HEAD~17..HEAD ... | grep -vE '^(diff|index|---|\\+\\+\\+|@@|-use |+use |-pub use |+pub use |\\s)' | wc -l` → 0.
- **E0432|E0433 trajectory monotonically non-increasing:** confirmed per-commit (2025 → 2023 → 2023 (×5 no-op) → 2013 → 2004 → 2003 → 1997 → 1981 → 1921 → 1916 → 1915 → 1904 → 1902).
- **Residual E0432|E0433 count = 1902:** strictly less than post-02-02 baseline of 2025 (Δ = -123); recorded in `/tmp/02-03-residual-errors.txt`.
- **Non-regression checks green:** `cargo check --workspace --exclude kay-core` exit 0; `cargo check -p kay-provider-openrouter --tests` exit 0.

All success criteria met. All acceptance criteria from the plan met. Three documented deviations (perl delimiter bug, pub use pattern extension, empty-marker reconciliation) — all self-corrected within task execution, no external input needed.

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Completed: 2026-04-20*
