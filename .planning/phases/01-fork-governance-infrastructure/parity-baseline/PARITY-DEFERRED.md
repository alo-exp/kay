# Parity Baseline — DEFERRED to EVAL-01a

**Status (Phase 1):** The FORKING work (single import commit, `crates/kay-core/src/forge_*`
source, `forgecode-parity-baseline` annotated tag) IS DONE per plan 03 — see
`../01-03-SUMMARY.md`. The imported ForgeCode upstream SHA is
`022ecd994eaec30b519b13348c64ef314f825e21` (also recorded in `.forgecode-upstream-sha`
at the repo root and in the root `NOTICE`). The parity GATING RUN — executing Harbor
against Terminal-Bench 2.0 to reproduce ≥80% on the unmodified fork — is DEFERRED
to follow-on task **EVAL-01a** per CONTEXT.md §User Amendments (2026-04-19).

## Why deferred

- **Tag signing (D-OP-04):** The `forgecode-parity-baseline` tag created in plan 03 is
  UNSIGNED per the D-OP-04 amendment. When EVAL-01a runs, the tag may be re-cut
  (signed) to match the parity-run's source SHA exactly. Signing key procurement
  is deferred to Phase 11.
- **CI cost:** Running Harbor inside GitHub Actions is expensive; the gate CI
  job stays on `workflow_dispatch` until Phase 2 requires it.
- **OPENROUTER API KEY (RESOLVED 2026-04-25):** MiniMax-M2.7 API key is configured.
  Use `MINIMAX_API_KEY` environment variable (not OpenRouter). MiniMax-M2.7
  is sufficient for EVAL-01a — no OpenRouter key required.

## What EVAL-01a must produce

When executed, EVAL-01a writes the following into THIS directory (same path as this note):

- `manifest.json` — conforms to `manifest-schema.json` in this directory; captures
  Docker image SHAs, MiniMax model + date, submission seed, Harbor commit SHA,
  per-task pass/fail, summary score. The schema's `forgecode_upstream_sha` field
  cross-checks against `../../../../.forgecode-upstream-sha`
  (`022ecd994eaec30b519b13348c64ef314f825e21`).
- `transcript.jsonl` — full Harbor run transcript (source of truth for downstream
  reproducers).
- `summary.md` — human-readable summary with a single `score: <NN.N>` line that
  the `parity-gate` CI job (in `.github/workflows/ci.yml`) greps.
- `forgecode-parity-baseline` annotated + GPG/SSH-signed git tag pointing at the
  exact commit the run was executed against (may be re-created from the unsigned
  Phase 1 tag at commit `8af1f2b` or a successor commit).

## How to trigger the gate manually (after EVAL-01a lands)

```
gh workflow run CI --ref main
```

and select the `parity-gate` job. It will read `summary.md`'s score and fail if < 80%.
The job is otherwise dormant — it does NOT run on normal `push` or `pull_request`
events.

## Cross-references

- Import commit: `8af1f2b` (plan 01-03; see `../01-03-SUMMARY.md`)
- Import tag: `forgecode-parity-baseline` → commit `8af1f2b` (annotated, UNSIGNED per D-OP-04)
- Upstream SHA file: `../../../../.forgecode-upstream-sha`
- Governance decisions: `../01-CONTEXT.md` §Decisions D-18, D-19, D-20, D-21, D-OP-04;
  §User Amendments (2026-04-19)
- Scaffolded CLI shim: `crates/kay-cli/src/eval.rs` — `kay eval tb2 --dry-run`
- Scaffolded CI job: `.github/workflows/ci.yml` — `parity-gate` (workflow_dispatch only)
