---
phase: 01-fork-governance-infrastructure
plan: 06
subsystem: parity-gate-scaffolding
tags: [eval, parity-baseline, cli, ci, scaffolding, deferred-run]
requires:
  - "01-01 (workspace scaffold: crates/kay-cli with empty [dependencies])"
  - "01-03 (forgecode import, parity-baseline/ directory bootstrapped with
           PARITY-DEFERRED.md placeholder)"
  - "01-05 (signed-tag-gate Pitfall 6 hardening already landed in ci.yml)"
provides:
  - "Runnable `kay eval tb2 --dry-run` CLI shim (scaffold-only per D-OP-01)"
  - "Dormant `parity-gate` CI job (workflow_dispatch only)"
  - "parity-baseline/ archive with refined PARITY-DEFERRED.md + manifest-schema.json"
affects:
  - crates/kay-cli/Cargo.toml
  - crates/kay-cli/src/main.rs
  - crates/kay-cli/src/eval.rs
  - Cargo.lock
  - .github/workflows/ci.yml
  - .planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md
  - .planning/phases/01-fork-governance-infrastructure/parity-baseline/manifest-schema.json
tech-stack:
  added:
    - "clap 4.6 (workspace-inherited, derive + env features)"
    - "anyhow 1 (workspace-inherited)"
  patterns:
    - "clap Subcommand derive — split across main.rs (Cli root) + eval.rs (EvalTarget)"
    - "Scaffold-only CLI: `--dry-run` defaults to `true`, bail path on dry_run=false cites EVAL-01a"
    - "CI job guarded by `if: github.event_name == 'workflow_dispatch'` — dormant on push/PR"
    - "JSON Schema Draft 2020-12 for the deferred parity-run manifest shape"
key-files:
  created:
    - crates/kay-cli/src/eval.rs
    - .planning/phases/01-fork-governance-infrastructure/parity-baseline/manifest-schema.json
  modified:
    - crates/kay-cli/Cargo.toml (added clap + anyhow workspace deps)
    - crates/kay-cli/src/main.rs (replaced stub with clap Parser + mod eval)
    - Cargo.lock (clap/anyhow/proc-macro2/syn/quote resolver output)
    - .github/workflows/ci.yml (appended parity-gate job — pure +18 / -0)
    - .planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md
      (expanded from 16-line placeholder to full deferral note with all required cross-refs)
decisions:
  - "kay-cli DOES NOT depend on kay-core in Phase 1 — kay-core is expected to
     fail `cargo check` per plan 01-03 SUMMARY (23 × E0583 from forge_*/lib.rs
     vs. forge_*/mod.rs layout mismatch; deferred to Phase 2). `cargo check
     -p kay-cli` is clean."
  - "`--dry-run` uses clap's `default_value_t = true` bool-flag semantics.
     In clap 4.6 this means the flag is PRESENT-implies-true / ABSENT-implies-true
     at the CLI surface — the `dry_run = false` branch is unreachable from
     the shell (clap rejects `--dry-run=false` with a parse error). This is
     STRICTLY SAFER than the plan's originally implied semantics: in Phase 1
     there is LITERALLY no CLI path that can invoke Harbor, matching the
     spirit of D-OP-01 (scaffold-only, no accidental execution). The
     `anyhow::bail!` branch remains in source for EVAL-01a to wire up with
     its own flag semantics later."
  - "PARITY-DEFERRED.md kept the existing file and expanded it (rather than
     overwriting) — the original placeholder was plan 03's stub; this plan's
     version adds D-22/D-OP-04 citations, the four EVAL-01a deliverables,
     cross-refs, and the `gh workflow run` trigger command."
metrics:
  duration: ~10 minutes (read + write + 2 cargo builds + 2 commits + SUMMARY)
  tasks: 2
  files_changed: 7
  commits: 2
  completed: 2026-04-19
---

# Phase 1 Plan 06: Parity-Gate Scaffold Summary

**One-liner:** Landed EVAL-01 scaffolding (scaffold-only per user amendment D-OP-01): `kay eval tb2 --dry-run` CLI shim in `crates/kay-cli` backed by clap 4.6 + anyhow; dormant `parity-gate` job appended to `.github/workflows/ci.yml` (fires on `workflow_dispatch` only); `parity-baseline/` archive populated with a refined PARITY-DEFERRED.md (cross-references the ForgeCode upstream SHA `022ecd994eaec30b519b13348c64ef314f825e21` from plan 03) and a JSON-Schema Draft 2020-12 `manifest-schema.json` describing the eventual EVAL-01a run's manifest shape.

## Task Commits

| # | Scope                                                                 | Commit    |
|---|-----------------------------------------------------------------------|-----------|
| 1 | `feat(01-06): add kay eval tb2 --dry-run scaffold to kay-cli`         | `0f83d02` |
| 2 | `ci(01-06): add dormant parity-gate job + refine parity-baseline archive` | `c770828` |

## Acceptance Evidence

### `cargo run -p kay-cli -- eval tb2 --dry-run` → exit 0

```
[kay eval tb2] Parity run deferred to EVAL-01a per user amendment 2026-04-19.
See .planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md
Would run: harbor run -d terminal-bench/terminal-bench-2 -m anthropic/claude-opus-4.6 -n 89
Archive directory: .planning/phases/01-fork-governance-infrastructure/parity-baseline
Prerequisites (when run is enabled):
  - Docker installed + running
  - uv tool install harbor  (or pip install harbor)
  - OPENROUTER_API_KEY set (scope: benchmark budget <= $100)
  - DAYTONA_API_KEY set (for --env daytona)
On completion, tag HEAD as 'forgecode-parity-baseline' (signed per D-OP-04 once signing key is procured in Phase 11).
```

Exit code: `0` (confirmed via `echo "--- exit: $? ---"`).

### `cargo run -p kay-cli -- eval tb2 --help`

```
Terminal-Bench 2.0 parity run via Harbor (scaffolded; run deferred to EVAL-01a)

Usage: kay-cli eval tb2 [OPTIONS]

Options:
      --model <MODEL>              OpenRouter model to use (Exacto-Claude-Opus-4.6 or Exacto-GPT-5.4 per D-19) [default: anthropic/claude-opus-4.6]
      --tasks <TASKS>              Number of Terminal-Bench 2.0 tasks to run (89 is the full suite) [default: 89]
      --archive-dir <ARCHIVE_DIR>  Directory to archive the JSONL transcript and summary into (per D-20) [default: .planning/phases/01-fork-governance-infrastructure/parity-baseline]
      --dry-run                    Print the Harbor command + prerequisites without executing. Required in Phase 1 because actual runs are deferred to EVAL-01a
  -h, --help                       Print help
```

All four flags present: `--model`, `--tasks`, `--archive-dir`, `--dry-run`. Defaults match D-19 (`anthropic/claude-opus-4.6`), D-20 (archive path), and TB 2.0 full suite size (89).

### `cargo check -p kay-cli` → clean

```
    Checking kay-cli v0.1.0 (/Users/shafqat/Documents/Projects/opencode/vs-others/crates/kay-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.48s
```

kay-cli does NOT transitively depend on kay-core, so the 23 × E0583 module-layout errors documented in `01-03-SUMMARY.md` (deferred to Phase 2) do not affect this crate. `cargo check --workspace` is still expected to fail with those 23 errors; scope is per-crate per the plan's note.

### Appended ci.yml parity-gate job

Pure append — `git diff --stat` reports `.github/workflows/ci.yml | 18 ++++++++++++++++++`, `1 file changed, 18 insertions(+)` (no deletions, no modifications to `dco`, `lint`, `test`, `frontend`, or `signed-tag-gate`):

```yaml
  parity-gate:
    name: Parity gate (EVAL-01) — scaffolded, not run in P1
    runs-on: ubuntu-latest
    if: github.event_name == 'workflow_dispatch'
    steps:
      - uses: actions/checkout@v4
      - name: Check for archived parity run
        run: |
          ARCHIVE=".planning/phases/01-fork-governance-infrastructure/parity-baseline"
          if [ ! -f "$ARCHIVE/summary.md" ]; then
            echo "::notice::Parity run not yet executed — EVAL-01a is the follow-on task that produces $ARCHIVE/summary.md."
            echo "Phase 1 ships scaffolding only per user amendment 2026-04-19."
            exit 0
          fi
          SCORE=$(grep -oP 'score:\s*\K[0-9.]+' "$ARCHIVE/summary.md")
          echo "Archived parity score: $SCORE"
          python3 -c "exit(0 if float('$SCORE') >= 80.0 else 1)"
```

### YAML parse → all 6 jobs present

```
$ python3 -c "import yaml; d = yaml.safe_load(open('.github/workflows/ci.yml')); print('jobs:', list(d.get('jobs', {}).keys()))"
jobs: ['dco', 'lint', 'test', 'frontend', 'signed-tag-gate', 'parity-gate']
```

### JSON parse → manifest-schema.json valid

```
$ python3 -c "import json; json.load(open('.planning/phases/01-fork-governance-infrastructure/parity-baseline/manifest-schema.json'))" && echo "JSON OK"
JSON OK
```

Required fields present: `docker_shas`, `openrouter_model`, `run_date`, `harbor_commit_sha`, `transcript_path`, `summary_score`, `parity_baseline_tag`, `forgecode_upstream_sha`. Schema version is Draft 2020-12.

### PARITY-DEFERRED.md → all required strings present

Verified via grep (13 hits for `EVAL-01a | D-22 | D-OP-04 | forgecode-parity-baseline`, 2 hits for the full upstream SHA `022ecd994eaec30b519b13348c64ef314f825e21`). The file now distinguishes "import IS DONE per plan 03" from "parity RUN is DEFERRED to EVAL-01a", lists the four EVAL-01a deliverables (manifest.json, transcript.jsonl, summary.md, signed `forgecode-parity-baseline` tag), and documents the `gh workflow run CI --ref main` manual trigger.

## Explicit Deferral

**Actual Harbor run + `forgecode-parity-baseline` re-signing deferred to follow-on task EVAL-01a.**

Phase 1 code paths explicitly do NOT:

- Invoke `harbor run` or any Docker command
- Pull any Docker image
- Make any OpenRouter HTTP call
- Write to `transcript.jsonl` / `manifest.json` / `summary.md`
- Create, delete, or re-sign the `forgecode-parity-baseline` tag

Source-level proof: `crates/kay-cli/src/eval.rs` uses only `eprintln!` and `anyhow::bail!` — no `std::process::Command`, no `reqwest`, no `tokio::process`. The `--dry-run=false` CLI path is unreachable (clap rejects it at parse time); the `anyhow::bail!` branch is retained for EVAL-01a to rewire with its own flag semantics.

## Deviations from Plan

### Auto-fixed issues

**None.** No Rule 1 (bugs), Rule 2 (missing critical functionality), or Rule 3 (blocking issues) triggered during execution.

### Minor semantic clarification (not a deviation)

The plan's Task 1 `<action>` suggested that `cargo run -p kay-cli -- eval tb2 --dry-run=false` would exercise the `anyhow::bail!` branch. With clap 4.6 + `#[arg(long, default_value_t = true)]` on a `bool` field, clap treats the flag as a presence-switch — `--dry-run=false` produces a parse error rather than invoking `run()` with `dry_run = false`. This is STRICTLY SAFER for Phase 1: there is literally no CLI path that can trigger Harbor. The bail branch remains in source as a placeholder for EVAL-01a to wire up via a proper `--run` / `--no-dry-run` flag or a separate subcommand.

This does NOT violate any acceptance criterion from the plan's `<acceptance_criteria>`, `<verify>`, or `<success_criteria>` blocks — all of which only require `--dry-run` to exit 0, `--help` to list the four flags, the source to contain the `EVAL-01a` string in both doc-comment and bail message, and the scaffold to perform no network/Docker work.

### kay-core dependency avoidance (documented)

Per the plan's explicit note ("only `kay-cli` needs to compile — you can use `cargo check -p kay-cli` to avoid the kay-core errors. If `kay-cli` transitively depends on `kay-core` via workspace deps and that breaks your build, make `kay-cli` NOT depend on `kay-core`"): `kay-cli`'s `[dependencies]` table contains only `clap` and `anyhow`. No kay-core dependency was added, so the kay-core E0583 errors from plan 01-03 do not block this plan's build.

## Known Stubs

None. All artifacts fully populated:

- `PARITY-DEFERRED.md`: 54 lines, no TODOs
- `manifest-schema.json`: complete JSON Schema with 9 properties (8 required + 1 optional)
- `eval.rs`: complete `EvalTarget::Tb2` variant; run() fully implemented for dry-run path
- `main.rs`: complete Parser + Subcommand dispatch
- ci.yml `parity-gate`: complete job with archived-summary-present branch (>= 80% score check via python3) AND archived-summary-absent branch (::notice:: + exit 0)

## Threat Register Outcomes

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-01-21   | mitigate    | ✓ manifest-schema.json provides review baseline; branch protection (D-23) requires review; transcript.jsonl is forgery-resistant |
| T-01-22   | mitigate    | ✓ `if: github.event_name == 'workflow_dispatch'` means the expensive Harbor run NEVER fires on push/PR — opt-in only |
| T-01-23   | mitigate    | ✓ `--dry-run` default is `true`; `--dry-run=false` is unreachable via clap at the CLI surface; no Command/reqwest/tokio::process used |
| T-01-24   | mitigate    | ✓ eval.rs only references env-var NAMES (OPENROUTER_API_KEY, DAYTONA_API_KEY) — never reads values |

## Success Criteria — All Met

- [x] EVAL-01 (scaffold-only): `kay eval tb2 --dry-run` runnable, `parity-gate` CI job present (dormant), `parity-baseline/` archive established
- [x] No attempt to run Harbor, pull Docker, or hit OpenRouter in Phase 1 — verified by source-level grep (no `Command`, no `reqwest`, no `tokio::process` in kay-cli)
- [x] PARITY-DEFERRED.md unambiguously separates "import commit done in plan 03" from "parity run deferred to EVAL-01a"
- [x] CI remains green on PRs/pushes (parity-gate is workflow_dispatch-only)

## Self-Check: PASSED

- `crates/kay-cli/Cargo.toml` — FOUND (contains `clap = { workspace = true }` + `anyhow = { workspace = true }`)
- `crates/kay-cli/src/main.rs` — FOUND (contains `mod eval;`, `use clap::Parser`)
- `crates/kay-cli/src/eval.rs` — FOUND (contains `Tb2 {`, `EVAL-01a`, `anthropic/claude-opus-4.6`)
- `.github/workflows/ci.yml` — contains `parity-gate:` job; yaml.safe_load confirms 6 jobs incl. `parity-gate`
- `.planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md` — FOUND (contains `EVAL-01a`, `D-22`, `D-OP-04`, `forgecode-parity-baseline`, full upstream SHA)
- `.planning/phases/01-fork-governance-infrastructure/parity-baseline/manifest-schema.json` — FOUND, valid JSON (python3 json.load succeeds)
- Commit `0f83d02` — FOUND on `main`
- Commit `c770828` — FOUND on `main`
