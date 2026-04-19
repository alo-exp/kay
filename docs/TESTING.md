# Testing Strategy and Plan

## Philosophy

Kay's testing strategy is benchmark-anchored. Terminal-Bench 2.0 is the north star; unit, integration, and nightly eval tiers exist to keep the benchmark reproducible and regressions visible. Every tier must be runnable in CI.

## Test Pyramid

1. **Unit tests** (`cargo test` per crate) — pure logic: context engine retrieval, schema hardening, tolerant JSON parser, persona loader, sandbox policy parser.
2. **Integration tests** (`cargo test --workspace` with fixtures) — tool registry, agent loop on mocked providers, session store resume, Tauri command round-trips (on headless frontend).
3. **Cross-platform sandbox tests** — matrix: macOS arm64, Ubuntu x64, Windows x64. Each runs the sandbox-escape assertion suite.
4. **Canary tests** — 4-hour Tauri long-session memory canary on macOS + Linux nightly (guards Tauri #12724 / #13133). Budget: memory delta per frame bounded.
5. **Real-repo eval set** — nightly run against 5 curated repositories (Rails, React+TS, Rust crate, Python, monorepo > 10k files) to catch regressions outside TB 2.0's Docker shape.
6. **Terminal-Bench 2.0 harness** — scripted via `kay eval tb2` with pinned Docker images, pinned model, pinned seed. Produces a reproducible reference transcript.
7. **Held-out TB subset** — maintained locally, never submitted. Used to detect overfitting.

## Coverage Goals

- `kay-core`: unit + integration coverage ≥ 80% line / 70% branch.
- Provider layer: 100% coverage of SSE parse + tool-call-reassembly paths.
- Sandbox layer: 100% policy-violation assertion coverage per OS.
- Agent loop: behavioral coverage (event sequences) rather than line coverage.

## Non-Goals

- Pursuing 100% line coverage in the UI layer (cost > value; covered by end-to-end canaries).
- Duplicating TB 2.0 tasks as unit tests — the benchmark harness is the one source of truth.

## Governance Invariants (active now)

**`tests/governance/check_attribution.sh`** — 35 grep-based assertions run locally (and wireable into CI):

- `NOTICE` contains the ForgeCode SHA + "ForgeCode" + "antinomyhq"
- `README.md` has an `## Acknowledgments` section mentioning ForgeCode + Terminus-KIRA
- `CONTRIBUTING.md` contains exact clean-room attestation strings `@anthropic-ai/claude-code`, `v2.1.88`, `2026-03-31`, `Developer Certificate of Origin`
- `SECURITY.md` references `security@kay.dev` + `docs/signing-keys`
- `.forgecode-upstream-sha` exists with a 40-char hex SHA
- `ATTRIBUTIONS.md` has `<UPSTREAM_COMMIT>` placeholder substituted
- `forgecode-parity-baseline` tag exists and is annotated

Run: `bash tests/governance/check_attribution.sh`. Exits 0 on all-pass; non-zero on any failure. `--help` lists each check.

## Regression Gates

- **Phase 1 parity gate**: forked ForgeCode baseline ≥ 80% on TB 2.0 before any harness mod merges (EVAL-01). Scaffolded in Phase 1 (`kay eval tb2 --dry-run`); actual run = EVAL-01a follow-on.
- **Nightly real-repo eval**: drop > 2pp on any repo blocks main merges until investigated (Phase 8+ onward).
- **Canary memory delta**: > 50 MB/hour in the 4-hour Tauri run fails the nightly pipeline (Phase 9+ onward).
- **TB 2.0 submission acceptance**: public score ≥ 81.8% with archived transcript (EVAL-05, Phase 12).

## Tooling

- `cargo-nextest` for faster CI
- `cargo-llvm-cov` for coverage
- `proptest` / `quickcheck` for property tests on the schema-hardening pipeline
- `insta` for snapshot tests on agent-event sequences
- `cargo-deny` + `cargo-audit` — active in CI since Phase 1 (lint job + nightly workflow)
