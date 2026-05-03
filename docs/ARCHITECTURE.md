# Architecture and Design

High-level architecture and design principles for Kay. Detailed phase-level designs live in `docs/superpowers/specs/` as they are produced.

> The authoritative architectural research is in `.planning/research/ARCHITECTURE.md`. This document is the living summary; the research file is frozen evidence.

## System Overview

Kay is a workspace of **8 Rust crates** with **three frontends over one core**:

- **Canonical backend:** `kay-cli` (rebrands ForgeCode's `forge_main`) exposes a structured-event JSONL contract on stdout. Works standalone over SSH, in CI, for TB 2.0 benchmarks — zero GUI/TUI dependencies.
- **Desktop frontend:** `kay-tauri` — native Tauri 2.x GUI that frontends the same CLI contract (no parallel agent-loop).
- **Full-screen terminal frontend:** `kay-tui` — ratatui multi-pane UI for SSH / low-bandwidth / keyboard-first users.

Codex-rs (OpenAI) is the closest reference shape. The harness is compiled into the main binary, **not** shipped as a Tauri sidecar (sidecar notarization is broken on macOS per Tauri #11992).

## Core Components

- **`kay-core`** — Agent loop, tool registry, context engine, session store, verifier. Currently holds imported ForgeCode source (unmodified; tagged `forgecode-parity-baseline`). No UI or runtime dependencies.
- **`kay-cli`** — **Canonical user-facing surface.** Rebrands `forge_main`; interactive mode inherits ForgeCode's completer, editor, banner, stream-renderer, and syntax highlighter. Headless mode for CI. Structured-event JSONL stream that GUI/TUI frontends consume.
- **`kay-tui`** — Full-screen ratatui frontend (multi-pane session view, tool-call inspector, cost meter). Consumes `kay-cli`'s JSONL stream. Keyboard-first, SSH-friendly, no mouse required. Installable standalone via `cargo install kay-tui` or invoked as `kay tui`.
- **`kay-tauri`** — Desktop GUI (macOS/Windows/Linux). Tauri 2.x with typed `#[tauri::command]` + `ipc::Channel<AgentEvent>` streaming. Same contract as the CLI — no parallel implementation. React 19 + TypeScript + Vite frontend.
- **`kay-provider-openrouter`** — OpenRouter HTTP + SSE client with strict Exacto-leaning model allowlist and tolerant JSON parser.
- **`kay-verifier`** — `MultiPerspectiveVerifier` with 3 KIRA critics (test-engineer, QA, end-user). Implements `TaskVerifier` trait. Separate crate to avoid circular deps with `kay-provider-openrouter`.
- **`kay-sandbox-{macos,linux,windows}`** — Per-OS sandbox implementations behind the `Sandbox` trait: macOS `sandbox-exec`, Linux `landlock` + `seccompiler`, Windows Job Objects + restricted token.
- **Frontend sources:** `kay-tauri/ui/` (React 19 + Vite + `tauri-specta` v2); `kay-tui/` (ratatui + crossterm).

**Personas** (`forge` = write, `sage` = research, `muse` = plan) are data (YAML per-persona), not code — names inherited from ForgeCode to minimize launch-day drift. Kay is brand/binary name; personas stay.

## Design Principles

1. **Three frontends, one contract** — The CLI's structured-event JSONL is the API boundary between core and frontends. GUI and TUI are pure consumers; neither reimplements the agent loop.
2. **CLI is canonical, not CI-only** — `cargo install kay` yields a fully functional agent. GUI and TUI are additive, never load-bearing for a benchmark run.
3. **Merged binary (no sidecars)** — Tauri bundle ships `kay-core` compiled in. No `externalBin` — macOS notarization blocked by Tauri #11992.
4. **Event-driven core** — `tokio::select!` over input / model / tool / control channels; `AgentEvent` (`#[non_exhaustive]`) is the single streaming surface.
5. **Personas as data** — `forge`, `sage`, `muse` share one code path parameterized by YAML (prompt + tool filter + model).
6. **Schema discipline** — all tool schemas pass through ForgeCode's hardening post-process (required-before-properties, flattening, explicit truncation reminders) before reaching the provider.
7. **Fail loud, never silent** — sandbox escapes, schema errors, and provider variance surface as typed events and `AgentEvent` frames; never swallowed.
8. **Extension-point discipline** — all core traits are object-safe and async; `#[non_exhaustive]` on public enums. v2 wedges (ACE / dynamic routing / verification-first depth / multi-agent orchestration) slot in additively without breaking v1 consumers.

## Current State (Phase 8 COMPLETE, 2026-04-22)

**Completed phases (1–8):**
- Phase 1: Fork, governance, CI infrastructure. `forgecode-parity-baseline` tag set.
- Phase 2: Provider HAL (`kay-provider-openrouter`) — OpenRouter HTTP+SSE client, tolerant JSON parser, streaming events, retry/backoff, `CostCap`, model allowlist. PROV-01..08. 79 tests.
- Phase 2.5: Kay-core sub-crate split — ForgeCode 23-module source integrated as 23 independent workspace sub-crates. `cargo check --workspace` clean.
- Phase 3: Tool registry + KIRA core tools — `ToolRegistry`, `Tool` trait, `kay-tools` crate, 8 built-in tools, ForgeCode JSON schema hardening, adversarial proptest suite. 174 tests. v0.1.1 signed.
- Phase 4: Sandbox — `Sandbox` trait + macOS `sandbox-exec` + Linux Landlock+seccomp + Windows Job Objects; escape suite (36 attempts denied); v0.2.0 signed.
- Phase 5: Agent loop — `run_turn` in `kay-core/src/loop.rs` with `tokio::select!` biased priority; `ControlMsg` (Pause/Resume/Abort); `TurnResult` enum; `TaskComplete` verify gate; `AgentEvent` streaming; `RunTurnArgs` struct; `kay-cli` binary.
- Phase 6: Session store — `SessionStore` (SQLite/rusqlite 0.38 bundled), `TranscriptStore`, event persistence, session resume. SESS-01..05 + CLI-02.
- Phase 7: Context engine — `ContextEngine` trait, `NoOpContextEngine` stub, `ContextBudget`, tree-sitter symbol store + SQLite FTS5 + sqlite-vec hybrid retrieval, `FileWatcher` (500ms debounce). 70 tests.
- Phase 8: Multi-Perspective Verification — `crates/kay-verifier/` with `MultiPerspectiveVerifier` (3 KIRA critics: test-engineer, QA, end-user), `AgentEvent::Verification` + `AgentEvent::VerifierDisabled`, `VerifierMode` (Interactive/Disabled), `VerifierConfig`, `run_with_rework` outer retry wrapper in `kay-core/src/loop.rs`, `VerificationOutcome`, `CriticResponse` JSON parsing, cost ceiling guard. VERIFY-01..04. PR #17 → b21897a2.

**Not yet started:**
- Phase 9: TUI (`kay-tui`, ratatui multi-pane).
- Phase 10: Tauri GUI (`kay-tauri`, React 19 + TypeScript).
- Phase 11: Code signing + release infrastructure.
- Phase 12: Terminal-Bench 2.0 run + leaderboard submission.

## Technology Choices

See `.planning/research/STACK.md` for the authoritative pinned dependency table. High-level:

- **Rust 2024** with tokio 1.51 LTS, reqwest 0.13 + reqwest-eventsource, rustls 0.23
- **Tauri 2.10+** (merged binary), `tauri-specta` v2 for IPC bindings
- **Frontend**: React 19 + TypeScript + Vite; Monaco or CodeMirror for the diff viewer
- **Context engine**: tree-sitter + sqlite-vec (pinned `=0.1.9`)
- **Shell**: `tokio::process` for non-PTY; `portable-pty` fallback for TTY-requiring commands
- **Sandbox**: `sandbox-exec` (macOS), `landlock` + `seccompiler` (Linux), Job Objects + restricted token (Windows)
- **Signing**: Apple Developer ID + Azure Code Signing; signed GPG/SSH release tags
