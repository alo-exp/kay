---
phase: 01-fork-governance-infrastructure
plan: 03
subsystem: fork-import
tags: [fork, governance, attribution, apache-2.0, parity-baseline]
requires:
  - "01-01 (workspace scaffold: crates/kay-core/Cargo.toml + stub lib.rs)"
  - "01-02 (governance files: NOTICE + ATTRIBUTIONS.md with placeholders)"
provides:
  - "ForgeCode source imported verbatim into crates/kay-core/src/"
  - "Recorded upstream SHA (machine + human readable)"
  - "Git tag `forgecode-parity-baseline` (annotated, unsigned per D-OP-04)"
affects:
  - crates/kay-core/src/
  - crates/kay-core/Cargo.toml
  - crates/kay-core/NOTICE
  - NOTICE
  - ATTRIBUTIONS.md
  - .forgecode-upstream-sha
tech-stack:
  added: []
  patterns:
    - "Single-import-commit fork pattern (D-01: clean-cut copy, no subtree/submodule)"
    - "Quadruple SHA recording (file + NOTICE + ATTRIBUTIONS + commit body)"
    - "forge_* subtree naming preserved under single kay-core crate (D-05)"
key-files:
  created:
    - .forgecode-upstream-sha
    - crates/kay-core/NOTICE
    - crates/kay-core/src/forge_api/ (+22 sibling forge_* subdirs, 440 .rs files total)
    - .planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md
  modified:
    - NOTICE (SHA placeholder substituted)
    - ATTRIBUTIONS.md (UPSTREAM_COMMIT placeholder substituted)
    - crates/kay-core/src/lib.rs (stub → forge_* module declarations)
    - crates/kay-core/Cargo.toml (description updated with short SHA)
decisions:
  - "Tag is annotated (`-a`) and UNSIGNED per D-OP-04 amendment — signing deferred to Phase 11"
  - "cargo check failure (E0583 × 23) accepted — modifying forge_* source to compile would corrupt the parity baseline"
  - "Flat forge_*/… subtree structure preserved under a single kay-core crate (D-05) rather than split into 23 sub-crates"
metrics:
  duration: ~5 minutes (plan 01-03 start → final commit)
  tasks: 3
  files_changed: 694
  completed: 2026-04-19
---

# Phase 1 Plan 03: Import ForgeCode Source Summary

**One-liner:** Imported ForgeCode `main` HEAD (SHA `022ecd994eaec30b519b13348c64ef314f825e21`) verbatim into `crates/kay-core/src/` as a single commit, substituted SHA placeholders in NOTICE and ATTRIBUTIONS.md, added a crate-level NOTICE, and tagged the commit `forgecode-parity-baseline` (annotated, UNSIGNED per D-OP-04 deferral).

## Captured Upstream SHA

- **Full:** `022ecd994eaec30b519b13348c64ef314f825e21`
- **Short:** `022ecd994eae`
- **Upstream:** https://github.com/antinomyhq/forgecode @ `main`
- **Capture date:** 2026-04-19

## Import Footprint

| Metric                        | Value |
| ----------------------------- | ----- |
| forge_* subdirs imported      | 23    |
| .rs files under kay-core/src/ | 440   |
| Additional non-rs files       | 254 (mostly .md/.toml/.json fixtures from upstream src/) |
| Files staged in import commit | 694   |

All 23 upstream crates under `forgecode/crates/forge_*/src/**` were copied verbatim into `crates/kay-core/src/forge_*/`. The upstream `lib.rs` for each crate now sits at `crates/kay-core/src/forge_<name>/lib.rs` — unmodified.

## Import Commit & Tag

- **Commit SHA:** `8af1f2b8622084268d5462aa08579d0fb541492d`
- **Tag object SHA:** `9985d77997d18c6e880af2f24a3eb2a8ed0286f8`
- **Tag:** `forgecode-parity-baseline` (type: `tag`, i.e. annotated)
- **Trailers verified:**
  - `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` ✓ (DCO)
  - `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>` ✓
- **Commit body references:** full upstream SHA ✓, short SHA ✓, "unsigned" ✓, `Refs: GOV-01` ✓
- **Tag message references:** full SHA ✓, `UNSIGNED` ✓, Phase 11 re-tag note ✓
- **`git tag -v forgecode-parity-baseline`:** exits with "no signature found" as expected — D-OP-04 defers tag signing to Phase 11 when the signing key is procured.

## cargo check --workspace Smoke Check

- **Exit code:** `101` (compile failure)
- **Error count:** 23 × `E0583: file not found for module`
- **Root cause:** Rust expects each `pub mod forge_X;` declared in `crates/kay-core/src/lib.rs` to be backed by either `crates/kay-core/src/forge_X.rs` OR `crates/kay-core/src/forge_X/mod.rs`. The upstream source ships its own `lib.rs` per crate, so the imported subtree has `crates/kay-core/src/forge_X/lib.rs` — NOT `mod.rs`. This is a structural mismatch between upstream crate-layout and Kay's single-crate module-layout.
- **First 20 lines of error log:**
  ```
      Checking kay-core v0.1.0 (/Users/shafqat/Documents/Projects/opencode/vs-others/crates/kay-core)
  error[E0583]: file not found for module `forge_api`
    --> crates/kay-core/src/lib.rs:20:1
     |
  20 | pub mod forge_api;
     | ^^^^^^^^^^^^^^^^^^
     |
     = help: to create the module `forge_api`, create file "crates/kay-core/src/forge_api.rs" or "crates/kay-core/src/forge_api/mod.rs"
     = note: if there is a `mod forge_api` elsewhere in the crate already, import it with `use crate::...` instead

  error[E0583]: file not found for module `forge_app`
    --> crates/kay-core/src/lib.rs:21:1
     |
  21 | pub mod forge_app;
     | ^^^^^^^^^^^^^^^^^^
     |
     = help: to create the module `forge_app`, create file "crates/kay-core/src/forge_app.rs" or "crates/kay-core/src/forge_app/mod.rs"
     = note: if there is a `mod forge_app` elsewhere in the crate already, import it with `use crate::...` instead
  ```
- **Resolution (deferred):** Phase 2's first harness modification will either (a) rename each `forge_X/lib.rs` → `forge_X/mod.rs` (mechanical, preserves content), (b) add thin `forge_X.rs` shims that `include!` the upstream lib.rs, or (c) split kay-core into multiple sub-crates per the plan's escalation path. Plan 01-03 Task 3 step 4 explicitly allows this — "cross-module reference errors are both acceptable; the parity baseline's integrity is what matters." No source modification was made to fix the build.

## Deviations from Plan

**None.** Plan executed exactly as written:

- `.forgecode-upstream-sha` created at repo root with the raw 40-char SHA (plus trailing newline).
- Root `NOTICE` `<SHA>` placeholder substituted with the full SHA.
- `ATTRIBUTIONS.md` `<UPSTREAM_COMMIT>` placeholder substituted with the full SHA.
- `crates/kay-core/NOTICE` created with the Tailcall + Apache-2.0 + repo-root-reference text verbatim from the plan's `<interfaces>` template.
- `crates/kay-core/Cargo.toml` already had `authors.workspace = true`; updated only the `description` field with the short SHA.
- `crates/kay-core/src/lib.rs` rewritten with header comment (mentions import SHA) and 23 `pub mod forge_*;` declarations.
- Single import commit created with `--signoff`, HEREDOC body, and `Co-Authored-By` trailer.
- Tag created with `-a` (annotated) and explicitly NOT `-s` or `-u` (unsigned per D-OP-04).

The cargo-check compile failure was anticipated by the plan and does not constitute a deviation — the plan explicitly permits either outcome.

## Attribution Chain (Apache §4(d))

All five attribution anchors now reference the captured SHA:

1. `.forgecode-upstream-sha` — `022ecd994eaec30b519b13348c64ef314f825e21`
2. `NOTICE` — "Imported from ForgeCode at commit 022ecd994eaec30b519b13348c64ef314f825e21 on 2026-04-19"
3. `ATTRIBUTIONS.md` — "Upstream commit: `022ecd994eaec30b519b13348c64ef314f825e21`"
4. `crates/kay-core/src/lib.rs` header — "imported from ForgeCode … at commit 022ecd994eaec30b519b13348c64ef314f825e21"
5. Import commit body — full SHA in subject and body, short SHA as context
6. Tag `forgecode-parity-baseline` — full SHA in tag message

T-01-06 (Tampering: SHA misrecorded) is fully mitigated — sextuple recording.

## Threat Register Outcomes

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-01-06   | mitigate    | ✓ SHA recorded in 6 places |
| T-01-07   | mitigate    | ✓ `Signed-off-by:` trailer present on import commit |
| T-01-08   | accept      | — cloned from canonical `github.com/antinomyhq/forgecode`; cargo-deny/audit run in plan 01-04 |
| T-01-09   | accept      | — tag explicitly UNSIGNED per D-OP-04; message flags this; Phase 11 will re-tag |
| T-01-10   | mitigate    | ✓ `.github/workflows/` NOT copied; upstream LICENSE/README/CONTRIBUTING not copied |

## Deferred Items

- **EVAL-01a (parity run):** deferred per user amendment; see `.planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md`.
- **Tag signing:** deferred to Phase 11 per D-OP-04. Phase 11 plan must re-tag (or super-sede) `forgecode-parity-baseline` with a signed equivalent.
- **cargo check --workspace clean compile:** deferred to Phase 2 first harness modification task (rename `lib.rs` → `mod.rs` inside each forge_* subdir, or introduce shims). NOT tracked as a bug — it is an accepted post-import structural adjustment.

## Known Stubs

None. All artifacts (NOTICE, ATTRIBUTIONS.md, crate NOTICE, .forgecode-upstream-sha, lib.rs module declarations) are fully populated — no placeholders, no "TODO"s.

## Success Criteria — All Met

- [x] GOV-01: attribution across NOTICE, README Acknowledgments (from plan 01-02), crate NOTICE, ATTRIBUTIONS.md, and workspace-inherited `authors` field — all now reference the actual upstream SHA
- [x] D-01: Single import commit (`8af1f2b`), no subtree/submodule, tagged `forgecode-parity-baseline`
- [x] D-OP-04: Tag is annotated but UNSIGNED; message explicitly documents Phase 11 re-tag obligation
- [x] Apache §4(d): NOTICE names Tailcall (actual copyright holder) + import SHA + date
- [x] Post-import cargo-check result captured (RC=101, 23 E0583 errors — source untouched)

## Self-Check: PASSED

- `.forgecode-upstream-sha` — FOUND
- `crates/kay-core/NOTICE` — FOUND
- `crates/kay-core/src/forge_api/lib.rs` — FOUND (sample forge_* subdir confirmed)
- `crates/kay-core/src/lib.rs` — FOUND (contains `pub mod forge_api;` … and 40-char SHA)
- `NOTICE` — contains captured SHA; no `<SHA>` placeholder
- `ATTRIBUTIONS.md` — contains captured SHA; no `<UPSTREAM_COMMIT>` placeholder
- Commit `8af1f2b` — FOUND on `main`
- Tag `forgecode-parity-baseline` → tag object `9985d77` → commit `8af1f2b` — FOUND
- `git cat-file -t forgecode-parity-baseline` = `tag` (annotated) — CONFIRMED
- `git tag -v forgecode-parity-baseline` = "no signature found" — CONFIRMED (expected per D-OP-04)
