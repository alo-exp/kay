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

## Regression Gates

- **Phase 1 parity gate**: forked ForgeCode baseline ≥ 80% on TB 2.0 before any harness mod merges (EVAL-01). Enforced in CI via a stored reference transcript.
- **Nightly real-repo eval**: drop > 2pp on any repo blocks main merges until investigated.
- **Canary memory delta**: > 50 MB/hour in the 4-hour run fails the nightly pipeline.
- **TB 2.0 submission acceptance**: public score ≥ 81.8% with archived transcript (EVAL-05).

## Tooling

- `cargo-nextest` for faster CI
- `cargo-llvm-cov` for coverage
- `proptest` / `quickcheck` for property tests on the schema-hardening pipeline
- `insta` for snapshot tests on agent-event sequences
