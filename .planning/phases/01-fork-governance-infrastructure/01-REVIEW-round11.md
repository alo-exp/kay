---
phase: 01-fork-governance-infrastructure
reviewed: 2026-04-19T00:00:00Z
depth: deep
round: 11
model: claude-sonnet-4-6
files_reviewed: 39
files_reviewed_list:
  - Cargo.toml
  - rust-toolchain.toml
  - .rustfmt.toml
  - deny.toml
  - .gitignore
  - LICENSE
  - NOTICE
  - README.md
  - CONTRIBUTING.md
  - SECURITY.md
  - CODE_OF_CONDUCT.md
  - ATTRIBUTIONS.md
  - crates/kay-cli/Cargo.toml
  - crates/kay-cli/src/main.rs
  - crates/kay-cli/src/eval.rs
  - crates/kay-core/Cargo.toml
  - crates/kay-core/NOTICE
  - crates/kay-provider-openrouter/Cargo.toml
  - crates/kay-provider-openrouter/src/lib.rs
  - crates/kay-sandbox-linux/Cargo.toml
  - crates/kay-sandbox-linux/src/lib.rs
  - crates/kay-sandbox-macos/Cargo.toml
  - crates/kay-sandbox-macos/src/lib.rs
  - crates/kay-sandbox-windows/Cargo.toml
  - crates/kay-sandbox-windows/src/lib.rs
  - crates/kay-tauri/Cargo.toml
  - crates/kay-tauri/src/lib.rs
  - crates/kay-tui/Cargo.toml
  - crates/kay-tui/src/main.rs
  - .github/workflows/ci.yml
  - .github/workflows/audit.yml
  - .github/pull_request_template.md
  - .github/dependabot.yml
  - tests/governance/check_attribution.sh
  - docs/signing-keys/README.md
  - docs/ARCHITECTURE.md
  - docs/PRD-Overview.md
  - docs/CICD.md
  - docs/TESTING.md
  - docs/CHANGELOG.md
  - docs/knowledge/INDEX.md
  - docs/knowledge/2026-04.md
  - docs/lessons/2026-04.md
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 01: Code Review Report — Round 11

**Reviewed:** 2026-04-19
**Depth:** deep
**Model:** claude-sonnet-4-6 (reviewer diversity from round 10's opus)
**Round:** 11 of 2-consecutive-clean stopping condition
**Files Reviewed:** 39
**Status:** issues_found

## Summary

Full exhaustive re-read of all 39 files with deliberate skepticism across the six focus areas prescribed for round 11. All Rust source, CI workflows, governance scripts, and documentation were read in full.

The release-flow dry-run is correct: pushing tag `v0.0.1` fires `lint` + `test` matrix + `frontend` (skips all steps) and correctly skips both `signed-tag-gate` (v0.0.x carve-out) and `parity-gate` (not workflow_dispatch). `Swatinem/rust-cache@v2` degrades gracefully on tag refs (read-only restore from main-branch cache). The `pnpm/action-setup` `cache-dependency-path` issue does not arise because the entire `setup-node` step is gated on `has_ui == 'true'`. The `cargo fmt` 7-crate list is consistent between ci.yml and CONTRIBUTING.md. No stale `v0.1.0` first-release claims exist in the 39-file scope — all `v0.1.0` references are correct policy descriptions of the signing boundary. Windows `cargo test` (no `shell: bash`) is not a bug: the command is a pure cargo invocation with no Bash-isms.

One warning and three info items were found, all in documentation — no source code defects.

---

## Warnings

### WR-R11-01: CICD.md falsely claims audit.yml triggers on Cargo.lock PRs

**File:** `docs/CICD.md:16`

**Issue:** The line reads:

> `.github/workflows/audit.yml` — Nightly cargo-audit at 04:17 UTC via `rustsec/audit-check@v2.0.0`; **also on `Cargo.toml` / `Cargo.lock` PRs**

`audit.yml` has exactly two triggers: `schedule` (cron `17 4 * * *`) and `workflow_dispatch`. There is no `pull_request` or `push` trigger with a paths filter. The nightly run is the only automated execution. A maintainer reading CICD.md would believe that opening a Cargo.lock-bumping PR causes an immediate security audit — it does not. The advisory-DB lag (up to ~23 hours) is the actual exposure window, not "PR-time."

**Fix:** Correct line 16 of CICD.md to remove the false claim:

```markdown
- **`.github/workflows/audit.yml`** — Nightly cargo-audit at 04:17 UTC via
  `rustsec/audit-check@v2.0.0`; also triggerable manually via `workflow_dispatch`.
  (No PR-time path filter — advisory-DB lag is up to ~23 h between nightly runs.)
```

Alternatively, add the missing trigger to `audit.yml`:

```yaml
on:
  schedule:
    - cron: '17 4 * * *'
  workflow_dispatch:
  push:
    paths:
      - 'Cargo.toml'
      - 'Cargo.lock'
      - '**/Cargo.toml'
```

---

## Info

### IN-R11-01: CICD.md misdescribes parity-gate as running the CLI binary

**File:** `docs/CICD.md:15`

**Issue:** The line states the parity-gate "runs `kay eval tb2 --dry-run` and exits 0". The actual job (ci.yml lines 188-198) runs a bash script that checks for `$ARCHIVE/summary.md` existence and exits 0 if absent. The CLI binary is never invoked by the CI job. The confusion matters for anyone who expects the parity-gate to exercise the CLI scaffolding.

**Fix:** Update to accurately describe what the job does:

```markdown
`parity-gate` — `workflow_dispatch` trigger only (Phase 1 scaffold per D-OP-01
user amendment); checks for an archived `parity-baseline/summary.md` and exits 0
(noticing the run is pending) if absent; upgrades to real Harbor invocation in Phase 2
```

---

### IN-R11-02: TESTING.md claims check_attribution.sh verifies "antinomyhq" in NOTICE — it does not

**File:** `docs/TESTING.md:33`

**Issue:** The line reads:

> `NOTICE` contains the ForgeCode SHA + "ForgeCode" + "antinomyhq"

The actual script checks NOTICE for "ForgeCode" (line 101), "Tailcall" (line 102), and "Apache License, Version 2.0" (line 103) — no check for "antinomyhq" and no check for the ForgeCode SHA string. NOTICE does happen to contain both, but the script does not enforce them. This is a documentation mismatch: contributors reading TESTING.md believe an invariant is enforced that is not.

**Fix:** Either update TESTING.md to reflect what the script actually checks, or add the missing checks to check_attribution.sh. Updating the doc is the lower-risk option:

```markdown
- `NOTICE` contains "ForgeCode", "Tailcall", and "Apache License, Version 2.0"
- `NOTICE` is ≤ 20 lines and has no `<SHA>` placeholder
```

---

### IN-R11-03: eval.rs dry-run output instructs user to create a tag that already exists

**File:** `crates/kay-cli/src/eval.rs:62`

**Issue:** The dry-run output eprintln at lines 61-63 reads:

> "On completion, tag HEAD as 'forgecode-parity-baseline' (signed per D-OP-04 once signing key is procured in Phase 11)."

The `forgecode-parity-baseline` tag was already created during Phase 1. A maintainer or contributor running `kay eval tb2 --dry-run` in a future phase would read this instruction and attempt to create a tag that already exists, causing a `git tag` error. The message should either be omitted or updated to reflect that the tag already exists and the real action in EVAL-01a is to archive the run transcript.

**Fix:**
```rust
eprintln!(
    "On completion, archive the transcript to {archive_dir}/summary.md \
     (the forgecode-parity-baseline tag already exists from Phase 1)."
);
```

---

## Release-Flow Dry-Run Analysis (Round 11 Deep Check)

Confirming the exact job-firing sequence for `git push --tags` with `v0.0.1`:

| Job | Fires? | Reason |
|---|---|---|
| `dco` | No | `if: github.event_name == 'pull_request'` — tag push is not a PR |
| `lint` | Yes | No `if:` guard; runs ubuntu-latest |
| `test (ubuntu)` | Yes | No `if:` guard |
| `test (macos)` | Yes | No `if:` guard |
| `test (windows)` | Yes | No `if:` guard; `cargo test` runs in PowerShell default shell — no Bash-isms, safe |
| `frontend` | Yes (all steps skip) | No `if:` guard on job; `has_ui=false` skips all steps |
| `signed-tag-gate` | No | `!startsWith(github.ref_name, 'v0.0.')` is false for `v0.0.1` — carve-out applies |
| `parity-gate` | No | `if: github.event_name == 'workflow_dispatch'` |

Result: lint + 3-OS test matrix + frontend (no-op) run. Signing is not required. No undesired jobs fire. The `Swatinem/rust-cache@v2` action degrades to read-only restore on the tag ref (no cross-ref cache pollution). The frontend `actions/setup-node` `cache-dependency-path` is never evaluated because the `has_ui == 'true'` gate prevents the step from running.

## cargo fmt List Consistency (Round 11 Check)

The 7-crate explicit list in ci.yml (lines 69-77) and CONTRIBUTING.md (lines 48-49) are consistent:
`kay-cli`, `kay-provider-openrouter`, `kay-sandbox-linux`, `kay-sandbox-macos`, `kay-sandbox-windows`, `kay-tauri`, `kay-tui`.

`kay-core` is correctly excluded from both. `Cargo.toml` workspace members = 8 crates = the 7 above + `kay-core`. Adding a 9th crate requires updating 3 locations (ci.yml + CONTRIBUTING.md + Cargo.toml). Cargo.toml comment on line 3-5 already documents this maintenance obligation. No structural defect; tracking is a known TODO per the workspace comment.

## SECURITY.md Actionability Assessment (Round 11 Check)

SECURITY.md is actionable for an external security researcher:
- Primary channel: GitHub Security Advisories (direct URL provided)
- Fallback: `security@kay.dev` with explicit field list (description, repro steps, versions affected, reporter handle)
- SLAs documented: 72h ack, 7d triage, 30d/90d fix-by severity
- Coordinated disclosure: 90-day default window, advisory published with fix
- Reporter credit: explicitly promised unless anonymity requested
- Signing verification instructions for releases

No gaps found.

---

_Reviewed: 2026-04-19_
_Reviewer: Claude Sonnet 4.6 (claude-sonnet-4-6) — round 11 reviewer diversity from round 10 opus_
_Depth: deep_
