# Phase 12: Terminal-Bench 2.0 Submission + v1 Hardening

## Goal

Kay posts a public Terminal-Bench 2.0 score ≥81.8% with a documented reference run (model pinned, seed pinned, transcript archived), validated against a held-out task subset and a parallel real-repo eval — and the v1.0 release ships with that score in the README.

## Requirements

From `.planning/REQUIREMENTS.md`:

| REQ-ID | Requirement | Phase |
|--------|-------------|-------|
| EVAL-01 | Parity baseline: unmodified fork scores ≥80% on TB 2.0 | 1 (deferred to EVAL-01a) |
| EVAL-01a | Run EVAL-01 with MiniMax-M2.7 + Harbor harness | 12 |
| EVAL-02 | `kay eval tb2` command with pinned Docker images | 12 |
| EVAL-03 | Held-out task subset scoring within 2pp of full set | 12 |
| EVAL-04 | Real-repo eval (Rails, React+TS, Rust crate, Python package) | 12 |
| EVAL-05 | Public leaderboard listing with archived transcript | 12 |

## Success Criteria

1. A single `kay eval tb2` command runs the Harbor harness locally with pinned Docker images, seed, and model allowlist, matching official submission settings exactly.
2. A held-out task subset (never referenced during development) is revealed for final validation and scores within 2 percentage points of the full-set local score.
3. Nightly real-repo eval (Rails, React+TS, Rust crate, Python package, monorepo >10k files) passes and its result is published alongside the TB 2.0 score.
4. The public TB 2.0 leaderboard lists Kay ≥81.8% with a documented, model-pinned reference run whose full transcript is archived in the repo.
5. **Phase 12 entry gate — EVAL-01a carried from Phase 1** — the unmodified `forgecode-parity-baseline` fork has been executed against TB 2.0 end-to-end and scored ≥80%. This closes the empirical half of NN#1 (ForgeCode parity gate): Phase 3 proved *structural* parity via byte-diff; Phase 12 proves *behavioral* parity via live benchmark.

## Prerequisites

- **MiniMax API key** configured as `MINIMAX_API_KEY` env var
- **Harbor harness** installed and working
- **Terminal-Bench 2.0** Docker images pulled
- **~$100 budget** for eval runs

## Dependencies

- Phase 8 (Multi-Perspective Verification) — verifier must be active
- Phase 10 (Multi-Session Manager) — export format needed for TB 2.0 submission
- Phase 11 (Release Pipeline) — v1.0 tag must be signed

## Task Breakdown

### Wave 1: EVAL-01a Parity Baseline

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W1-T1 | Harbor harness setup | `.planning/phases/01-fork-governance-infrastructure/parity-baseline/` | Document Harbor setup, Docker pull, seed pinning | Harbor runs locally |
| W1-T2 | MiniMax-M2.7 model pinning | `crates/kay-cli/eval.md` | Document model version, temperature, max_tokens | Model responds to test prompt |
| W1-T3 | Baseline run (80% target) | `.planning/phases/01-fork-governance-infrastructure/parity-baseline/run-YYYY-MM-DD/` | Archive: full stdout, JSONL transcript, score summary | Score ≥80% |
| W1-T4 | Archive reference run | `.planning/phases/01-fork-governance-infrastructure/parity-baseline/REFERENCE.md` | Document: model, seed, Docker SHA, score, commit | Reference exists |

### Wave 2: `kay eval tb2` Command

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W2-T1 | `kay eval tb2 --dry-run` | `crates/kay-cli/src/eval.rs` | Prints eval command that would run | Prints correct Docker + model flags |
| W2-T2 | `kay eval tb2 --run` | `crates/kay-cli/src/eval.rs` | Executes Harbor harness with Kay | Harbor runs, score returned |
| W2-T3 | Score parsing + exit code | `crates/kay-cli/src/eval.rs` | Exit 0 if ≥81.8%, non-zero otherwise | Exit code reflects score |

### Wave 3: Held-Out Task Subset Validation

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W3-T1 | Identify held-out tasks | `.planning/phases/12-terminal-bench-submission/HELD-OUT.md` | List task IDs not used in development | 10-20 tasks |
| W3-T2 | Run held-out subset | `.planning/phases/12-terminal-bench-submission/held-out-results/` | Score held-out tasks | Score within 2pp of full |
| W3-T3 | Document validation | `.planning/phases/12-terminal-bench-submission/HELD-OUT.md` | Update with scores | Validation recorded |

### Wave 4: Real-Repo Eval

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W4-T1 | Define real-repo suite | `.planning/phases/12-terminal-bench-submission/REAL-REPOS.md` | Rails, React+TS, Rust crate, Python package, monorepo >10k | 5 repos defined |
| W4-T2 | Run real-repo eval | `.planning/phases/12-terminal-bench-submission/real-repo-results/` | Score each repo | All pass (≥80% each) |
| W4-T3 | Publish real-repo results | `docs/real-repo-eval.md` | Summary with scores, links to transcripts | Results public |

### Wave 5: v1.0 Release

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W5-T1 | Update README with score | `README.md` | Add TB 2.0 score badge, reference link | Badge visible |
| W5-T2 | Tag v1.0.0 (signed) | `RELEASE.md` | GPG/SSH signed tag, changelog | Tag exists, verified |
| W5-T3 | GitHub release | `.github/workflows/release.yml` | Auto-create release from signed tag | Release created |
| W5-T4 | Archive all transcripts | `.planning/phases/01-fork-governance-infrastructure/parity-baseline/archives/` | ZIP all reference runs | Archives exist |

## Verification Steps

1. `kay eval tb2 --dry-run` → prints correct Harbor command
2. MiniMax API responds to test prompt
3. Harbor harness pulls Docker images
4. Baseline run completes with score ≥80%
5. Held-out subset scores within 2pp of full set
6. Real-repo eval passes all 5 repos
7. v1.0.0 tag signed and verified
8. README shows TB 2.0 score badge
9. GitHub release published with all transcripts

## Threat Model

### API Key Exposure
- `MINIMAX_API_KEY` stored only in local shell environment
- Never committed to repo
- `grep -r "sk-cp-" .` pre-commit check

### Eval Cost
- ~$100 budget for baseline + held-out + real-repo eval
- Monitor costs via MiniMax dashboard
- Stop if >$150 accumulated

### Benchmark Integrity
- Held-out tasks never referenced during development
- Reference run archived before any Phase 12 development
- Seed pinned to prevent score gaming

## Rollback

- Revert README badge changes
- `git tag -d v1.0.0` to remove signed tag
- GitHub release can be unpublished (draft)
- Reference run archives preserved for re-run

## Dependencies

- **Depends on**: Phase 8 (verifier), Phase 10 (export), Phase 11 (release pipeline)
- **Unblocks**: v1.0.0 release

## Exit Condition

PLAN.md exists, passes all quality gates (no ❌), and has been reviewed.

---

*Phase 12 plan created: 2026-04-26*