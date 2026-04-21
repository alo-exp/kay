---
spec-version: 1
status: Active
jira-id: ""
figma-url: ""
source-artifacts:
  - ".planning/PROJECT.md"
  - ".planning/REQUIREMENTS.md"
  - ".planning/ROADMAP.md"
created: 2026-04-20
last-updated: 2026-04-20
scope: project
---

# Kay — Project Spec

> **Scope note.** Kay is a *project*, not a single feature. This SPEC.md is the
> project-level spec that Silver Bullet's spec-floor hook requires before
> `/gsd-plan-phase`. It intentionally cross-references the canonical artifacts
> rather than duplicating them; PROJECT.md + REQUIREMENTS.md + ROADMAP.md are
> the sources of truth.
>
> Per-phase implementation decisions live in
> `.planning/phases/NN-.../NN-CONTEXT.md`, not here.

## Overview

Kay is an open-source terminal coding agent — a Rust fork of
[ForgeCode](https://github.com/antinomyhq/forgecode) hardened with
Terminus-KIRA's harness techniques and delivered through three frontends
(canonical CLI, ratatui TUI, Tauri 2.x desktop GUI) over one `kay-core`
library contract.

**Problem:** OpenCode scores 51.7% on Terminal-Bench 2.0 — ~30 points below
ForgeCode's 81.8% — despite using the same LLM class. The gap is *harness*,
not the model. At the same time, no permissively-licensed OSS agent ships a
native desktop GUI, so developers wanting a "real Claude Code alternative"
have to choose between Claude Code's proprietary client, claw-code's murky
clean-room provenance, or OpenCode's terminal-only surface.

**Who it serves:** developers who want the top-tier agentic coding
experience (currently locked inside Claude Code, Codex, and ForgeCode) as a
single Apache-2.0 binary on macOS, Windows, and Linux, with a first-class
CLI for CI/benchmark/SSH workflows and a native desktop GUI for session
management.

**How it wins:** (a) fork ForgeCode's proven #1 harness (Apache-2.0
inheritable), (b) import KIRA's four techniques (native tool calling,
marker polling, multi-perspective verification, multimodal `image_read`),
(c) ship the GUI + TUI + CLI wedge that no OSS competitor owns,
(d) OpenRouter-only with an Exacto-leaning model allowlist to eliminate
tool-calling variance, (e) clean-room attestation + DCO + signed releases
to earn the provenance trust that claw-code lacks.

## User Stories

- **As a developer running `cargo install kay`, I want** a single binary
  that drives an agentic coding loop against OpenRouter **so that** I can
  use Kay headless in CI and Terminal-Bench runs without any GUI
  dependency.
- **As a developer over SSH, I want** a full-screen ratatui TUI (`kay tui`)
  that shows session state, tool-call timeline, and cost meter **so that**
  I can drive a long-running agent session without a desktop.
- **As a developer on a laptop, I want** a native Tauri desktop app that
  shows live agent traces and lets me manage multiple sessions **so that**
  I get a "real Claude Code alternative" with an OSS license.
- **As a contributor, I want** DCO + clean-room attestation + signed
  releases **so that** I can trust Kay's provenance chain in a post-leak
  ecosystem.
- **As a benchmark reviewer, I want** Kay's public TB 2.0 submission to
  include a reproducible reference run (pinned model, pinned Docker
  images, archived transcript) **so that** I can verify the >81.8% claim.

## UX Flows

### Primary flow — headless TB 2.0 run
1. User sets `OPENROUTER_API_KEY` env var (or `[provider.openrouter] api_key` in `kay.toml`)
2. User runs `kay eval tb2 --model anthropic/claude-sonnet-4.6 --max-usd 50`
3. Harbor harness drives Kay through the benchmark task set
4. Kay streams `AgentEvent` frames to stdout in JSONL
5. Final score + reference-run manifest written to `.kay/runs/<id>/`

### Primary flow — interactive CLI
1. User runs `kay` (no args) in a project directory
2. Interactive REPL launches (inherited from ForgeCode's `forge_main`: completer, editor, banner, stream-renderer)
3. User types a task description
4. Kay executes through `forge` persona (write), `sage` persona (read-only research), or `muse` persona (planning) per prompt context
5. Multi-perspective verifier confirms task completion before `task_complete` returns
6. Session transcript saved to `~/.kay/sessions/<id>/`

### Primary flow — Tauri desktop GUI
1. User double-clicks `Kay.app` / `Kay.exe` / `Kay.AppImage`
2. Native window opens with session list + agent trace + cost meter
3. User picks a project directory, binds an OpenRouter key to OS keychain
4. User clicks "New session" → task input
5. Live `AgentEvent` stream renders via `ipc::Channel`; tool-call timeline + diffs via Monaco/CodeMirror
6. User can spawn/pause/resume/fork sessions from the GUI

## Acceptance Criteria

> Every v1 requirement in REQUIREMENTS.md must be satisfied for Kay to
> ship. The criteria below are the **project-level acceptance gates** —
> the "Kay is done and shippable" signals. Phase-level success criteria
> live in ROADMAP.md per phase and are the actual quality gates.

- [ ] **AC-01 (benchmark):** Kay posts a public Terminal-Bench 2.0 score
  **> 81.8%** with a documented, model-pinned, Docker-image-pinned,
  transcript-archived reference run. Derived from `EVAL-05` + core value.
- [ ] **AC-02 (parity gate, never skipped):** The forked ForgeCode
  baseline — imported at SHA
  `022ecd994eaec30b519b13348c64ef314f825e21`, tagged
  `forgecode-parity-baseline` — reproduces **≥ 80%** on TB 2.0 unmodified
  before any harness modification merges to `main`. Derived from `EVAL-01`.
  (Scaffold shipped in Phase 1; actual run = EVAL-01a follow-on.)
- [ ] **AC-03 (three frontends, one core):** `cargo install kay` yields a
  working headless CLI; `cargo install kay-tui` yields a working TUI;
  `Kay.app` / `Kay.exe` / `Kay.AppImage` yields a working desktop GUI.
  All three consume the same `kay-core` contract — no parallel agent-loop
  implementations. Derived from `CLI-06`, `TUI-04`, `TAURI-02`.
- [ ] **AC-04 (governance):** Every merged commit carries
  `Signed-off-by:`; every release tag from `v0.1.0` forward is GPG- or
  SSH-signed; clean-room attestation is required on first merge for any
  contributor with exposure to the 2026-03-31 Claude Code leak. Derived
  from `GOV-03`, `GOV-05`, `GOV-07`.
- [ ] **AC-05 (distribution):** Signed and notarized bundles exist for
  macOS (arm64 + x64), Windows (x64 Authenticode), Linux (x64 + arm64
  AppImage + tar.gz with SHA attestations) produced from every merge to
  `main` (not only release tags). Derived from `REL-01..REL-06`.
- [ ] **AC-06 (supply chain):** `cargo-deny` blocks GPL/AGPL transitive
  deps; `cargo-audit` runs nightly and on every PR; `openssl` is banned
  (rustls only); no unsigned tag shape `vX.Y.Z` from `v0.1.0` forward.
  Derived from `WS-03`, `WS-04`, Phase 1 D-08.
- [ ] **AC-07 (model policy):** Provider rejects requests for
  non-allowlisted models with typed `ProviderError::ModelNotAllowlisted`
  — no silent fallback. Launch allowlist targets OpenRouter Exacto
  endpoints. Derived from `PROV-04`.
- [ ] **AC-08 (safety):** Every shell/file/network action runs through
  the per-OS sandbox; escape attempts produce
  `AgentEvent::SandboxViolation`, never silent fall-through. Derived
  from `SBX-01..SBX-04`.
- [ ] **AC-09 (semver + release policy):** Kay follows semver with a
  **never-major** rule — breaking bumps are treated as minor releases
  and documented in CHANGELOG. The `v0.0.x` pre-stable series is exempt
  from signed-tag enforcement; `v0.1.0` forward is mandatory. Derived
  from project memory `project_kay_versioning.md` + Phase 1 SECURITY.md
  §Release Signing.
- [ ] **AC-10 (reproducibility):** A user can `kay session export
  <session-id>` and receive a self-contained JSONL bundle that replays
  deterministically on another machine (same model, same seed, same
  transcript). Derived from `SESS-05`.

## Assumptions

<!-- Assumptions inherited from PROJECT.md / REQUIREMENTS.md / ROADMAP.md and Phase 1 decisions. All Resolved at project-init or explicitly deferred. -->

- [ASSUMPTION: OpenRouter Exacto endpoints are accessible via the same API
  as standard endpoints with a `:exacto` model-ID suffix | Status: Accepted
  | Owner: Phase 2 executor — verify during first streaming call]
- [ASSUMPTION: ForgeCode's TB 2.0 81.8% run used
  `openai/gpt-5.4` + `anthropic/claude-opus-4.6` + `anthropic/claude-sonnet-4.6`
  | Status: Follow-up-required | Owner: Phase 12 reference-run capture
  (verify against ForgeCode public submission manifest before launch)]
- [ASSUMPTION: Apple Developer ID and Azure Code Signing enrollments
  complete within the 2-4 week window started at Phase 1
  | Status: Follow-up-required | Owner: Maintainer — tracked under
  Phase 1 D-OP-05 and Phase 11 dependency]
- [ASSUMPTION: `forge_json_repair` (ForgeCode import) covers OpenRouter's
  actual tool-call malformations observed in production
  | Status: Accepted | Owner: Phase 2 executor — verify with real traces]
- [ASSUMPTION: The Rust-toolchain pin (stable 1.95, 2024 edition) remains
  stable for the ~3-month v1 cycle without a breaking `rustc` change
  | Status: Accepted | Owner: Maintainer — tracked in `rust-toolchain.toml`]
- [ASSUMPTION: Tauri 2.x IPC memory leak status (issues #12724, #13133)
  is either fixed or worked around before Phase 9 ships
  | Status: Follow-up-required | Owner: Phase 9 researcher — upstream check]

## Open Questions

<!-- Flagged for resolution before or during the phase they impact. -->

- [ ] **TB 2.0 budget:** approximate $ for a full TB 2.0 submission run.
  Owner: Maintainer — resolve before EVAL-01a.
- [ ] **OpenRouter key procurement:** account + billing + key for
  EVAL-01a. Owner: Maintainer — blocks Phase 2 smoke test and
  Phase 12 submission.
- [ ] **Signing key procurement:** GPG or SSH key for maintainer; Apple
  Developer ID + Azure Code Signing for release bundles.
  Owner: Maintainer — blocks `v0.1.0` release.
- [ ] **Windows sandbox hardening depth:** Job Objects + restricted token
  is the minimum; integrity-level + capability restrictions are the
  maximum. Owner: Phase 4 researcher — tracked in
  `ROADMAP.md §Phase 999.1 (backlog)`.
- [ ] **Tauri `externalBin` vs merged binary choice for frontend assets:**
  confirmed merged binary (no sidecar) for the Rust backend per TAURI-02.
  UI assets are still bundled — verify notarization path.
  Owner: Phase 9 researcher.

## Out of Scope

> These are explicit v1 exclusions. All mirrored from PROJECT.md §Out of
> Scope and REQUIREMENTS.md §Out of Scope. Listed here only to satisfy
> the SB SPEC.md template contract.

- Direct Anthropic / OpenAI / Gemini / Groq APIs (OpenRouter covers
  these for v1; direct integrations are v2)
- Local model support (Ollama / LM Studio / llama.cpp) — v2 privacy
  milestone
- IDE extensions (VS Code / JetBrains / Xcode) — v3+
- Self-improving context / ACE — v2 wedge
- Dynamic model routing per subtask — v2 wedge
- Deep multi-perspective verification-first loop — v2 wedge (v1 ships
  the KIRA-level baseline)
- Re-enterable hierarchical multi-agent orchestration — v2 wedge
- Web dashboard / browser UI — Tauri covers the "not-a-terminal" surface
- Plugin marketplace — OpenClaw's supply-chain incident (21K exposed
  instances) proves marketplaces shift security burden to solo maintainers
- Auto-memory across sessions at v1 — lands with ACE (WEDGE-01)
- Hooks (pre/post tool, session) — Claude Code's feature; distracts from
  TB 2.0 target
- 75+ model providers like OpenCode — tool-calling variance destroys
  benchmark scores; allowlist is the inverse bet
- Voice input — Codex CLI has this; off-strategy for v1
- Cerebras / WSE-3 specific optimizations — locks us into one hardware
  vendor
- CLA for contributors — switched to DCO based on pitfalls research
  (2026-04-19)
- Non-Terminal-Bench benchmarks (SWE-bench, MLE-bench) — picking one
  benchmark as north star keeps the team honest

## Traceability Map (project → sources of truth)

- **Vision + scope:** `.planning/PROJECT.md` (authoritative)
- **v1 requirements (83 REQ-IDs with phase mapping):** `.planning/REQUIREMENTS.md`
- **12 phases + success criteria:** `.planning/ROADMAP.md`
- **Per-phase implementation decisions:**
  `.planning/phases/NN-*/NN-CONTEXT.md`
- **Per-phase plans:** `.planning/phases/NN-*/NN-MM-PLAN.md`
- **Per-phase verification:** `.planning/phases/NN-*/VERIFICATION.md`
- **Semver + release policy:** memory note `project_kay_versioning.md`
  + `SECURITY.md §Release Signing`

## Implementations

<!-- Populated automatically by SB pr-traceability.sh hook post-merge. -->
- PR: https://github.com/alo-exp/kay/pull/8 | Date: 2026-04-22 | Spec-version: 1
- PR: https://github.com/alo-exp/kay/pull/5 | Date: 2026-04-21 | Spec-version: 1
- PR: https://github.com/alo-exp/kay/pull/4 | Date: 2026-04-21 | Spec-version: 1

- 2026-04-19 — Phase 1 shipped as `v0.0.1` (commit `22890ad`, tag
  `v0.0.1`); 6/6 plans complete, CI 5/5 green,
  `signed-tag-gate` correctly skipped under v0.0.x carve-out.
