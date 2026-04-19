---
phase: 02-provider-hal-tolerant-json-parser
plan: 02
subsystem: infra
tags: [rust, cargo, module-system, refactor, structural-integration]

# Dependency graph
requires:
  - phase: 01-fork-governance-infrastructure
    provides: "Imported ForgeCode source tree under crates/kay-core/src/forge_*/lib.rs (frozen at commit 8af1f2b — forgecode-parity-baseline tag). 23 × E0583 pre-flagged in 01-03-SUMMARY §cargo check + VERIFICATION.md §SC-4 with `--exclude kay-core` escape clause."
  - phase: 02-provider-hal-tolerant-json-parser
    provides: "Plan 02-01 — MockServer + SSE cassettes in kay-provider-openrouter (unaffected by kay-core rename)."
provides:
  - "23 forge_*/lib.rs renamed to mod.rs atomically in commit bb57694 (R100 on every file — byte-identical content under new path)."
  - "kay-core module-declaration layer now compiles (pub mod forge_X; in src/lib.rs resolves for all 23 subtrees)."
  - "Pre-path-rewrite baseline for Wave 3 (02-03 / 02-04 / 02-05): E0583 = 0, E0432|E0433 = 2025."
  - "forgecode-parity-baseline tag semantic integrity preserved — the rename commit does not alter any file content; combined sha256 of 23 files unchanged across rename."
affects: [02-03-kay-core-path-rewrites, 02-04, 02-05, 02-06-provider-hal, 02-07-allowlist, eval-01a]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Atomic structural rename: single commit with N × `git mv` for related modules; verified via git's R100 (100% similarity) similarity score + numstat 0/0 per file."
    - "Byte-identity verification via combined sha256 before/after rename (belt-and-suspenders on top of git's similarity detector)."

key-files:
  created: []
  modified:
    - "crates/kay-core/src/forge_api/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_app/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_ci/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_config/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_display/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_domain/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_embed/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_fs/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_infra/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_json_repair/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_main/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_markdown_stream/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_repo/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_select/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_services/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_snaps/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_spinner/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_stream/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_template/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_test_kit/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_tool_macros/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_tracker/lib.rs → mod.rs"
    - "crates/kay-core/src/forge_walker/lib.rs → mod.rs"

key-decisions:
  - "Executed D-01 Step 1 verbatim: 23 × `git mv forge_X/lib.rs forge_X/mod.rs` in a single shell invocation, zero content edits, one atomic commit."
  - "Did NOT modify crates/kay-core/src/lib.rs — its 23 × `pub mod forge_X;` declarations already resolve correctly once forge_X/mod.rs exists."
  - "Did NOT re-tag forgecode-parity-baseline — per CONTEXT.md D-01 note, the tag captures the unmodified import at commit 8af1f2b; the rename commit is an independent downstream commit."

patterns-established:
  - "Structural rename pattern: Rust's canonical way to register a subdirectory module is `forge_X/mod.rs` when parent declares `pub mod forge_X;`. ForgeCode's imported `forge_X/lib.rs` format only works as a crate root, not a submodule — Kay's mono-crate form requires mod.rs."
  - "Atomic rename verification: R100 (100% git-similarity) on every file + numstat `0\t0` on every file + combined sha256 match before/after. Anything less than 100% similarity on any file means content was altered and the rename is invalid."

requirements-completed: []  # PROV-01 is a behavioral requirement — unblocked by this plan but completed by 02-06 (Provider trait) + 02-08 (OpenRouterProvider impl). See Deviations section.

# Metrics
duration: 8min
completed: 2026-04-19
---

# Phase 2 Plan 02: Structural Rename (kay-core E0583 Fix) Summary

**23 forge_*/lib.rs renamed to mod.rs atomically — module-declaration layer of kay-core now compiles; E0583 count dropped from 24 to 0 while source content stayed byte-identical (R100 on every file, 0 insertions + 0 deletions).**

## Performance

- **Duration:** ~8 min (2026-04-19T18:50:00Z → 2026-04-19T18:57:35Z)
- **Started:** 2026-04-19T18:50:00Z
- **Completed:** 2026-04-19T18:57:35Z
- **Tasks:** 1
- **Files modified:** 23 (all renames, zero content edits)

## Accomplishments
- Resolved all 23 × E0583 "file not found for module" errors in kay-core with a single atomic commit.
- Preserved `forgecode-parity-baseline` tag's semantic integrity: combined sha256 of all 23 files is unchanged (`e749ea93836a8ff7017a3cc4b18d08b9da56f1559fed9913eaba2e098e8d238a`) before and after rename. Content is byte-identical under the new filename.
- Established the pre-path-rewrite baseline for Wave 3 (plans 02-03 / 02-04 / 02-05): `E0432|E0433 count = 2025`. This is the error budget those plans drive to 0.
- Non-regression confirmed: `cargo check --workspace --exclude kay-core` and `cargo check -p kay-provider-openrouter --tests` both still exit 0.

## Task Commits

Each task was committed atomically:

1. **Task 1: Atomically rename all 23 forge_*/lib.rs to forge_*/mod.rs via git mv** — `bb57694` (refactor)

No metadata commit yet — will be added after STATE.md / ROADMAP.md / REQUIREMENTS.md updates land.

## Files Created/Modified
- 23 × `crates/kay-core/src/forge_*/lib.rs → mod.rs` (pure rename; R100 similarity on every file; 0 insertions, 0 deletions per file per `git show --numstat HEAD -M`).

## Verification Evidence

| Check | Expected | Actual | Source |
|---|---|---|---|
| `forge_*/lib.rs` remaining | 0 | 0 | `ls crates/kay-core/src/forge_*/lib.rs` |
| `forge_*/mod.rs` present | 23 | 23 | `ls crates/kay-core/src/forge_*/mod.rs` |
| R100 renames in HEAD | 23 | 23 | `git show --name-status HEAD -M \| grep -cE "^R100"` |
| numstat `0\t0` file rows | 23 | 23 | `git show --numstat HEAD -M \| grep -cE "^0\s+0\s+"` |
| Non-zero delta file rows | 0 | 0 | `git show --numstat HEAD -M \| grep -E "^[0-9-]+\s+[0-9-]+\s+" \| grep -cvE "^0\s+0\s+"` |
| Combined sha256 pre-rename | identical | `e749ea93...e098e8d238a` | `shasum -a 256 crates/kay-core/src/forge_*/lib.rs \| awk '{print $1}' \| shasum -a 256` |
| Combined sha256 post-rename | identical | `e749ea93...e098e8d238a` | `shasum -a 256 crates/kay-core/src/forge_*/mod.rs \| awk '{print $1}' \| shasum -a 256` |
| E0583 in kay-core | 0 | 0 | `cargo check -p kay-core 2>&1 \| grep -c "E0583"` |
| E0432\|E0433 in kay-core | > 0 (expected) | 2025 | `cargo check -p kay-core 2>&1 \| grep -cE "E0432\|E0433"` |
| `cargo check --workspace --exclude kay-core` | exit 0 | exit 0 | stderr tail: "Finished dev profile" |
| `cargo check -p kay-provider-openrouter --tests` | exit 0 | exit 0 | stderr tail: "Finished dev profile" |
| Commit prefix `refactor(02-02):` | yes | yes | `git log -1 --format=%s` |
| `Signed-off-by:` trailer | >= 1 | 1 | `git log -1 --format=%B \| grep -c "Signed-off-by:"` |
| `D-01 Step 1` mention | >= 1 | 1 | `git log -1 --format=%B \| grep -c "D-01 Step 1"` |

## Decisions Made

- **Executed D-01 Step 1 verbatim** — no deviation in the core task.
- **PROV-01 requirement checkbox left UN-marked in REQUIREMENTS.md.** The plan's frontmatter lists `requirements: [PROV-01]`, but PROV-01 is "Provider trait supports chat completion + tool calling + streaming SSE with typed AgentEvent output" — a behavioral spec that this plan does not deliver. The ROADMAP describes this plan as "(PROV-01 prereq)", not "(PROV-01)". The requirement will be completed by plans 02-06 (Provider trait definition) and 02-08 (OpenRouterProvider impl). This SUMMARY documents the unblocking step; downstream plans check PROV-01 off when the Provider trait + streaming SSE + typed AgentEvent actually exist.

## Deviations from Plan

**1. [Rule 4 - architectural judgment / interpretation] PROV-01 requirement not marked complete despite plan frontmatter.**
- **Found during:** State-update step (after Task 1 commit).
- **Issue:** Plan 02-02 frontmatter lists `requirements: [PROV-01]`, which, if followed literally, would mark PROV-01 complete. But PROV-01 is a behavioral requirement ("Provider trait supports chat completion + tool calling + streaming SSE with typed AgentEvent output") that a structural file-rename does not and cannot deliver. The ROADMAP correctly labels this plan as "(PROV-01 prereq)". Marking PROV-01 complete here would make the requirement traceability table lie.
- **Fix:** Left PROV-01 unchecked. Downstream plans 02-06 (Provider trait scaffolding) and 02-08 (OpenRouterProvider impl) own the actual PROV-01 deliverables and will mark it complete then. This SUMMARY records the requirement-mapping correction.
- **Files modified:** None (kept REQUIREMENTS.md PROV-01 unchecked).
- **Verification:** REQUIREMENTS.md line 219 still shows `| PROV-01 | Phase 2 | Pending |`.
- **Impact:** Traceability integrity preserved. Future plans 02-06/02-08 mark PROV-01 complete when the behavioral spec is actually delivered.

---

**Total deviations:** 1 (interpretation / requirement-mapping correction — no code impact).
**Impact on plan:** Zero impact on the structural rename itself. The deviation only affects requirement-status accounting.

## Issues Encountered

None. The `git status --short | grep -cE "^R"` acceptance check used a pre-defined format; git showed 23 R-prefix lines as expected. Initial `cargo check` output was ~1 MB (26,751 lines) — had to redirect to a file for grep counting; that's a tooling artifact, not an execution issue.

## Expected Downstream Errors

Per RESEARCH §Pitfall 1 lines 458-482, the rename UNMASKS hundreds of cross-subtree `use forge_Y::X` path errors. **These are expected and out of scope for this plan.** They will be resolved by:

- **Plan 02-03** — Wave 3 — rewrites absolute `crate::X` paths inside each `forge_X` subtree to `crate::forge_X::X` form.
- **Plan 02-04** — Wave 3 — rewrites cross-subtree `forge_Y::Z` references (e.g., `forge_infra` code importing from `forge_domain`) to `crate::forge_Y::Z` form.
- **Plan 02-05** — Wave 3 — catchall for remaining edge-cases (macro paths, test module paths, external crate re-exports in the import tree).

The E0432/E0433 count of 2025 is the **baseline** those plans drive toward 0. `cargo check --workspace` (without `--exclude kay-core`) remains failing until 02-05 lands; that's by design per the phase plan.

Additional error codes surfaced in the full cargo-check output (out of scope for this plan):
- **E0038** (trait cannot be made into an object) — some `dyn Trait` usages, likely related to downstream path resolution failures.
- **E0220** (associated type not found) — cascading from E0432/E0433 unresolved imports.
- **E0223** (ambiguous associated type) — same root cause.

These are all downstream consequences of the unresolved path imports; fixing E0432/E0433 in 02-03/04/05 should substantially reduce E0038/E0220/E0223 too. Final error-count triage lands in 02-05.

## Known Stubs

None. This plan performs a structural rename only — no stub code, no placeholder implementations, no hardcoded empty data.

## Next Phase Readiness

- **Ready for Plan 02-03 (Wave 3 path rewrites):** The module-declaration layer is clean. `cargo check -p kay-core` now reports only import-path errors, not module-system errors — Wave 3 is unblocked.
- **Non-regression intact:** Wave 0 deliverables (`kay-provider-openrouter` MockServer + cassettes from Plan 02-01) continue to build. No cross-wave contamination.
- **Parity-baseline integrity intact:** `forgecode-parity-baseline` tag (at commit 8af1f2b) still points at the unmodified import. Commit `bb57694` is a downstream refactor of the module system, not the source tree.

## Self-Check: PASSED

Verified claims:
- **Commit `bb57694` exists:** `git log --oneline | grep -q "^bb57694"` → present (`bb57694 refactor(02-02): rename 23 forge_*/lib.rs to mod.rs`).
- **23 × `forge_*/mod.rs` files exist:** `ls crates/kay-core/src/forge_*/mod.rs | wc -l` → 23.
- **0 × `forge_*/lib.rs` files remain:** `ls crates/kay-core/src/forge_*/lib.rs 2>/dev/null | wc -l` → 0.
- **Commit is 100%-similarity rename:** `git show --name-status HEAD -M | grep -cE "^R100"` → 23.
- **DCO trailer present:** `git log -1 --format=%B | grep -c "Signed-off-by:"` → 1.

All success criteria met. All acceptance criteria from the plan met. One documented deviation (requirement-mapping interpretation — see §Deviations above) with no code impact.

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Completed: 2026-04-19*
