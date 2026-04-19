# Phase 1: Fork, Governance, Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `01-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 01-fork-governance-infrastructure
**Mode:** Autonomous (`--auto`) — user delegated all non-critical decisions
**Areas discussed:** Fork & Attribution, Workspace, DCO Enforcement, Signed Tags, Clean-room Attestation, Parity Gate, Branch Protection

---

## Fork & Attribution

| Option | Description | Selected |
|--------|-------------|----------|
| git subtree import | Track upstream as a subtree for future merges | |
| git submodule | Submodule ForgeCode; pin to SHA | |
| Clean-cut clone + copy | Clone at SHA, copy source into workspace, single import commit | ✓ |

**User's choice (auto-recommended):** Clean-cut clone + copy.
**Notes:** Subtree/submodule add upstream-drift complexity v1 doesn't need. Clean cut with a signed `forgecode-parity-baseline` tag is sufficient provenance.

---

## Workspace Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Flat single-crate | Monolithic `kay` crate | |
| codex-rs mirror (5-7 crates) | kay-core, kay-cli, kay-tauri, kay-provider-openrouter, kay-sandbox-* | ✓ |
| Per-feature crate explosion | 20+ crates by subsystem | |

**User's choice (auto-recommended):** codex-rs mirror.
**Notes:** Aligns with research recommendation (STACK.md, ARCHITECTURE.md). Clean split between core library and frontends.

---

## DCO Enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| tim-actions/dco (Apache-2.0) | Linux-kernel / ASF standard | ✓ |
| dcoapp/app (bot-based) | GitHub App flow | |
| Custom bash check | Home-grown verifier | |

**User's choice (auto-recommended):** tim-actions/dco.
**Notes:** Already scaffolded in `.github/workflows/ci.yml`. Mature, Apache-licensed, aligns with Linux kernel / ASF projects.

---

## Signed Tags

| Option | Description | Selected |
|--------|-------------|----------|
| GPG only | Require RFC 4880 signatures | |
| SSH only | Require git 2.34+ SSH signatures | |
| Both GPG and SSH accepted | Maximum flexibility for contributors | ✓ |

**User's choice (auto-recommended):** Both.
**Notes:** SSH signing lowers the contributor bar; GPG remains supported. `git tag -v` validates both.

---

## Clean-room Attestation

| Option | Description | Selected |
|--------|-------------|----------|
| CLA with explicit clean-room clause | High friction, strong shield | |
| DCO + CONTRIBUTING.md attestation | Low friction, honor-system + PR review | ✓ |
| No attestation | Fastest, weak legal shield | |

**User's choice (auto-recommended, aligned with PROJECT.md DCO switch):** DCO + CONTRIBUTING.md attestation.
**Notes:** PROJECT.md §Key Decisions locked Apache-2.0 + DCO earlier this session. Attestation text cites the specific leak date + version.

---

## Parity Gate Harness

| Option | Description | Selected |
|--------|-------------|----------|
| Re-implement TB 2.0 scoring locally | Custom harness mimicking TB 2.0 | |
| Use official harbor-framework/terminal-bench-2 via Docker | Official, reproducible | ✓ |
| Skip local gate; submit to public leaderboard only | Cheapest | |

**User's choice (auto-recommended):** Official Harbor + Docker.
**Notes:** Pinned Docker SHAs + Harbor commit SHA. Models: OpenRouter Exacto Claude Opus 4.6 or GPT-5.4 (same ones ForgeCode scored 81.8% on). $100 budget scoped for initial baseline.

---

## CI Scope at Phase 1

| Option | Description | Selected |
|--------|-------------|----------|
| Full CI (lint + test + parity + notarize) now | All gates live at Phase 1 end | |
| Staged: lint + test + DCO + signed-tag at P1; parity gate wired in P2; notarize in P11 | Incremental landing matching the phase the capability actually needs | ✓ |
| Bare minimum (lint only) | Defer everything else | |

**User's choice (auto-recommended):** Staged.
**Notes:** Parity gate is scaffolded at P1 (job stub) but only fires on P2+ when harness changes land. Notarization waits for Apple Developer ID onboarding (D-OP-02).

---

## Claude's Discretion

- Exact content of `kay-core/src/` module stubs (emerge organically in Phase 2+)
- `rust-toolchain.toml` pin (pick stable at execution day)
- `deny.toml` initial advisory list (start from cargo-deny default; tune on false positives)

## Deferred Ideas

- JSON-twin `NOTICE.json` for supply-chain tooling (v1.x)
- Automated clean-room leak scanner (indefinite defer)
- Dedicated release-signing key separate from maintainer's personal key (post-v0.1)
- Reproducible-build guarantee (v1+)

---

*Discussion mode: autonomous. All options above were evaluated against PROJECT.md, REQUIREMENTS.md, and the four research dimensions (STACK, FEATURES, ARCHITECTURE, PITFALLS) before locking the recommended choice.*
