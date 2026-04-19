---
phase: 1
status: passed_with_amendments
verified: 2026-04-19
---

# Phase 1 — Fork, Governance, Infrastructure · Verification

## Success Criteria (from ROADMAP.md)

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | Anyone can clone `kay/` and see ForgeCode attribution in `NOTICE`, `README`, and crate `authors`, with Apache-2.0 `LICENSE` at repo root | ✅ PASS | `NOTICE` contains SHA `022ecd994eaec30b519b13348c64ef314f825e21` + "ForgeCode"; `README.md` has `## Acknowledgments`; workspace `Cargo.toml` has `authors = ["Kay Contributors <contributors@kay.dev>"]` inherited by all 8 crates; `LICENSE` is verbatim Apache-2.0 at repo root |
| 2 | PR without Signed-off-by on every commit is auto-blocked by CI; `CONTRIBUTING.md` documents DCO + clean-room + PR process; `SECURITY.md` publishes private-advisory flow | ✅ PASS | `.github/workflows/ci.yml` has `dco` job using `tim-actions/dco@master` + `tim-actions/get-pr-commits@v1.3.1`; `CONTRIBUTING.md` has DCO text (exact verbatim `@anthropic-ai/claude-code` / `v2.1.88` / `2026-03-31`); `SECURITY.md` with GitHub Security Advisory flow + `security@kay.dev` + 72h/7d/30d/90d SLA |
| 3 | Maintainer cutting release tag without GPG/SSH signature is rejected by release workflow | ✅ PASS | `signed-tag-gate` job with Pitfall 6–hardened `if:` condition (`github.ref_type == 'tag' && startsWith(github.ref_name, 'v') && contains(github.ref_name, '.')`) runs `git tag -v` and exits 1 on unsigned tags |
| 4 | `cargo check --workspace --deny warnings` passes on macOS/Linux/Windows; cargo-deny + cargo-audit run on every PR and reject GPL/AGPL or known-vulnerable transitive deps | ⚠️ PARTIAL (expected) | cargo-deny configured via `deny.toml` (blocks GPL/AGPL/LGPL/SSPL/openssl per D-08) + wired into `ci.yml`; cargo-audit nightly via `.github/workflows/audit.yml` + inline CI step. **Pending:** workspace compile fails at `kay-core` with 23 × `E0583` errors from ForgeCode's `forge_*/lib.rs` naming convention. Known, documented in `01-03-SUMMARY.md`, and deferred to Phase 2 structural integration (first harness modification) per EVAL-01 parity-preservation principle — modifying forge_* source to make it compile would corrupt the parity baseline. All other workspace crates (kay-cli, kay-tauri, kay-tui, kay-provider-openrouter, kay-sandbox-*) compile clean individually. |
| 5 | Forked ForgeCode baseline reproduces ≥80% on TB 2.0 with archived reference run, enforced in CI before harness modifications merge | ⚠️ DEFERRED (per user amendment) | Scaffold only per D-OP-01. `kay eval tb2 --dry-run` is runnable and exits 0. `parity-gate` CI job appended (workflow_dispatch only). `PARITY-DEFERRED.md` + `manifest-schema.json` archive the expected run shape. Actual run re-scoped to follow-on task **EVAL-01a**, unblocking when OpenRouter key + ~$100 budget are supplied. |

## Phase Goal

> Kay exists as a clean, Apache-2.0-compliant ForgeCode fork on signed infrastructure with DCO-enforced contribution, and the unmodified fork reproduces ForgeCode's TB 2.0 baseline before any harness change is allowed to merge.

**Verdict:** Goal achieved within the scope of user amendments. The deferred parity run (EVAL-01a) is the only success criterion not delivered in this phase; its scaffolding is in place and the follow-on task is explicitly tracked. The workspace compile failure is structural, predictable, and out-of-scope for Phase 1 (which preserves the parity baseline verbatim).

## Requirement Coverage

All 13 REQ-IDs from ROADMAP Phase 1 are covered:

| REQ-ID | Plan | Status |
|--------|------|--------|
| GOV-01 | 01-02, 01-03 | ✅ |
| GOV-02 | 01-02 | ✅ |
| GOV-03 | 01-05 | ✅ |
| GOV-04 | 01-02 | ✅ |
| GOV-05 | 01-05 | ✅ (gate active; no tags cut this phase per D-OP-04) |
| GOV-06 | 01-02 | ✅ |
| GOV-07 | 01-02 | ✅ |
| WS-01 | 01-01 | ✅ (8 workspace members — 7 original + kay-tui from arch amendment) |
| WS-02 | 01-01 | ✅ |
| WS-03 | 01-04 | ✅ |
| WS-04 | 01-04 | ✅ |
| WS-05 | 01-01, 01-04, 01-05 | ⚠️ PARTIAL — lint/deny/audit paths green; workspace compile deferred to Phase 2 per above |
| EVAL-01 | 01-06 | ⚠️ SCAFFOLD-ONLY per D-OP-01 user amendment; run = EVAL-01a follow-on |

## Commits

**Phase 1 produced 20 commits** on `main` (all DCO-signed, all Co-Authored-By: Claude Sonnet 4.6):

| Area | Commits |
|------|---------|
| Planning (context, research, validation, plans) | 7 |
| Plan 01-01 (workspace scaffold) | 3 |
| Plan 01-02 (governance files) | 3 |
| Plan 01-03 (ForgeCode import + unsigned tag) | 2 |
| Plan 01-04 (supply-chain gates) | 3 |
| Plan 01-05 (CI verification + governance checker) | 3 |
| Plan 01-06 (parity-gate scaffold) | 3 |
| Architectural amendment (CLI-first + TUI + forge/sage/muse) | 1 |

## Deliverables

**Files created (repo-root):**
- `LICENSE`, `NOTICE`, `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `ATTRIBUTIONS.md`
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.rustfmt.toml`, `deny.toml`
- `.forgecode-upstream-sha`
- `docs/signing-keys/README.md`
- `.github/pull_request_template.md`, `.github/workflows/audit.yml`
- `tests/governance/check_attribution.sh` (executable)

**Files modified:**
- `.github/workflows/ci.yml` — Pitfall 6 hardening (Plan 01-05) + parity-gate job appended (Plan 01-06)

**Crates (8-crate workspace):**
- `kay-core/` — 694 files (imported ForgeCode source at `022ecd994eaec30b519b13348c64ef314f825e21`, UNMODIFIED)
- `kay-cli/` — workspace entry point; `eval tb2 --dry-run` scaffolded
- `kay-tauri/` — skeleton for Phase 9
- `kay-tui/` — skeleton for Phase 9.5 (added per architectural amendment)
- `kay-provider-openrouter/` — skeleton for Phase 2
- `kay-sandbox-macos/`, `kay-sandbox-linux/`, `kay-sandbox-windows/` — skeletons for Phase 4

**Tag:** `forgecode-parity-baseline` → annotated, UNSIGNED per D-OP-04 deferral (signing key procurement → Phase 11)

## Follow-On Tasks

| Task | Blocker | Phase |
|------|---------|-------|
| EVAL-01a — run parity baseline ≥80% on TB 2.0 | OpenRouter API key + ~$100 budget | Unblocks Phase 2 harness merges |
| Sign `forgecode-parity-baseline` tag | GPG/SSH signing key procurement | Phase 11 (cert/key procurement) |
| Structural integration of ForgeCode source into kay-core (fix 23 × E0583) | None — starts Phase 2 | Phase 2 first harness modification |
| Apple Developer ID + Azure Code Signing enrollment | User/org procurement | Start at Phase 9 for Phase 11 readiness |

## Phase 1 — Complete ✓ (with documented amendments)
