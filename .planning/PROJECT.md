# Kay

## What This Is

Kay is an open-source terminal coding agent — a Rust fork of ForgeCode, hardened with Terminus-KIRA's harness techniques and delivered through a native Tauri desktop UI that no other OSS agent offers. It targets developers who want the top-tier agentic coding experience (currently locked inside Claude Code, Codex, and ForgeCode) as a single permissively-licensed binary on macOS, Windows, and Linux.

## Core Value

**Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.** If the score target fails, Kay has no reason to exist; if the UI fails to ship, Kay is just another ForgeCode fork. Both must hold.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

<!-- Current scope. Building toward v1 (ForgeCode parity + KIRA harness techniques + Tauri UI + OpenRouter). -->

**Core harness (ForgeCode parity):**

- [ ] Rust workspace forked from ForgeCode with attribution, Apache-2.0 + NOTICE preserved
- [ ] Three specialized agents: `kay` (write), `sage` (read-only research), `muse` (planning)
- [ ] Semantic context engine: index function signatures and module boundaries (not raw file dumps)
- [ ] OpenRouter provider integration with API-key auth (no OAuth)
- [ ] Bash-tool execution loop with sandboxed subprocess isolation
- [ ] Mandatory verification pass before task-complete signal
- [ ] JSON schema hardening: field reordering, flattening, explicit truncation reminders

**KIRA harness techniques (quality bar for beating ForgeCode):**

- [ ] Native LLM tool calling via `tools` parameter (replace any ICL-style parsing)
- [ ] Marker-based command-completion polling (`__CMDEND__<seq>__` sentinel)
- [ ] Multi-perspective completion verification (test engineer + QA engineer + end-user critics)
- [ ] Multimodal `image_read` tool for base64 terminal-state screenshots

**Tauri desktop UI (Kay's distinguishing surface):**

- [ ] Native Tauri 2.x desktop app (macOS, Windows, Linux) bundled with the Kay binary
- [ ] Session view: live agent trace, tool-call timeline, token/cost meter
- [ ] Multi-session manager: spawn, pause, resume, fork sessions from the GUI
- [ ] Project workspace: directory picker, env/key management, OpenRouter account binding
- [ ] Log/export: structured session export for reproducibility and benchmark submission
- [ ] Headless CLI mode preserved for CI/scripting (desktop UI is additive, not mandatory)

**Governance & distribution:**

- [ ] Apache-2.0 license with Contributor License Agreement (CLA)
- [ ] Signed release tags (GPG or SSH) — no unsigned tags in the release flow
- [ ] Binary distribution: macOS (arm64 + x64), Windows (x64), Linux (x64 + arm64)
- [ ] `cargo install kay` published on crates.io
- [ ] Terminal-Bench 2.0 submission achieving >81.8% with a documented reference run

### Out of Scope

<!-- Explicit boundaries for v1. Each with reasoning to prevent re-adding. -->

- **Direct Anthropic / OpenAI / Gemini / Groq APIs** — OpenRouter covers all of these for v1; direct integrations add surface area for little v1 benefit. Revisit in v2.
- **Local model support (Ollama / LM Studio / llama.cpp)** — OpenCode's differentiator, but irrelevant to beating ForgeCode on TB 2.0. Deferred to v2 under the "privacy" milestone.
- **IDE extensions (VS Code / JetBrains / Xcode)** — Claude Code and Codex lead here; competing on IDE surface is a v3+ concern.
- **Self-improving context / ACE** — One of the four post-v1 wedges. Requires a stable harness to build on.
- **Dynamic model routing per subtask** — v2 wedge. v1 uses a single user-selected model per session.
- **Deep multi-perspective verification-first loop** — v1 ships the KIRA-level baseline; the deeper verification-first architecture is v2.
- **Re-enterable hierarchical multi-agent orchestration** — v2 wedge. Addresses OpenCode issue #11012 but requires session-persistence primitives that don't exist in ForgeCode yet.
- **Non-Terminal-Bench benchmarks (SWE-bench, MLE-bench, etc.)** — Picking one benchmark as the north star keeps the team honest. SWE-bench gets attention only if the TB 2.0 goal is met.
- **Web dashboard / browser UI** — Tauri covers the "not-a-terminal" surface. A web UI is redundant for v1 and fragments the effort.
- **TUI (terminal UI)** — ForgeCode's existing CLI surface is sufficient; Kay's distinguishing surface is Tauri. Revisit if users demand it.

## Context

**Benchmark landscape (as of April 2026):**
- ForgeCode holds TB 2.0 #1 at 81.8% (with GPT-5.4 and Claude Opus 4.6).
- Terminus-KIRA (KRAFTON AI) sits at #9–10 with 74.4–74.8% on the same Apache-2.0 base Kay will adopt ideas from.
- OpenCode scores 51.7% (rank #51) — 30+ points below ForgeCode on the same model class. The gap is *harness*, not LLM.

**Why the "fork ForgeCode, import KIRA" wedge exists:**
- ForgeCode is Apache-2.0 and ships a proven top-tier context engine.
- KIRA's four techniques (native tool calling, marker polling, multi-perspective verification, multimodal `image_read`) are well-documented and portable.
- Neither has a desktop UI — OpenCode's terminal-first stance leaves the GUI surface unclaimed in OSS.

**Community signal:**
- OpenCode issues #11012, #12661, #9649, #8456, #15456, #10761 all surface problems Kay fixes from day one (closed subagents, lack of agent teams, no multi-agent collaboration, no dynamic routing, no ACE, generic performance).
- The March 2026 Claude Code leak (and subsequent claw-code cleanroom — 100K stars in hours) shows appetite for a "real" OSS Claude Code alternative. Kay is positioned to be that alternative with a better license story than claw-code's murky origins.

**Reference implementations to study, in priority order:**
1. claw-code (full-stack ecosystem: hooks, memory, Agent Teams) — blueprint for v2 wedges
2. Terminus-KIRA — proven harness improvements with public technical writeup
3. ForgeCode — Kay's direct base; context engine and JSON hardening
4. Terminus 2 — benchmark-reference shape
5. Codex CLI — speed/integration patterns
6. mini-swe-agent — minimalism discipline check

## Constraints

- **Tech stack**: Rust (harness) + Tauri 2.x + TypeScript/React for the UI layer — chosen to match ForgeCode's stack and inherit top-tier performers' language choice. Single-binary distribution is non-negotiable.
- **License**: Apache-2.0 + CLA — inherits ForgeCode's license requirements and protects future relicensing paths. Every contributor must sign the CLA before a PR can merge.
- **Benchmark**: Terminal-Bench 2.0 at >81.8% is the v1 acceptance gate — no release without a clean reference run on the public leaderboard.
- **Provider**: OpenRouter-only for v1 — keeps the provider-abstraction layer scoped. Direct APIs are deliberately deferred.
- **Supply chain**: Signed release tags, reproducible builds, published SHA attestations. Claw-code's "100K stars overnight, legal uncertainty" trajectory is a cautionary tale — Kay trades ambition for auditability.
- **Timeline**: v1 is a ~3-month milestone targeting ForgeCode feature parity + KIRA techniques + Tauri UI. The four wedges are explicitly v2+ — no scope creep into v1.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Language: Rust | Matches ForgeCode, claw-code, Codex CLI — every top performer chose Rust; single binary, best perf | — Pending |
| Base: fork ForgeCode | Apache-2.0 permissive, proven #1 harness, context engine already solved | — Pending |
| UI: Tauri desktop (not TUI, not web) | No other OSS agent has a native desktop GUI; differentiator visible at install time | — Pending |
| License: Apache-2.0 + CLA | Matches base; CLA protects future paths and gives contributor-identity clarity | — Pending |
| v1 providers: OpenRouter only | 300+ models via one key; direct APIs deferred to v2 | — Pending |
| v1 benchmark target: >81.8% TB 2.0 | Beating ForgeCode is the only acceptable v1 quality bar | — Pending |
| Four wedges deferred to v2 | ACE, routing, verification-first, multi-agent need stable harness foundation first | — Pending |
| Non-negotiable: signed release tags | Silver-Bullet#28 filed today — Kay adopts signing from v0.0.1 to avoid the same ecosystem complaint | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-19 after initialization*
