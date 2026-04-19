# Architecture and Design

High-level architecture and design principles for Kay. Detailed phase-level designs live in `docs/specs/YYYY-MM-DD-<topic>-design.md` as they are produced.

> The authoritative architectural research is in `.planning/research/ARCHITECTURE.md`. This document is the living summary; the research file is frozen evidence.

## System Overview

Kay is a workspace of Rust crates consumed by two surfaces: a headless CLI (`kay`) and a Tauri desktop app. Codex-rs (OpenAI) is the closest reference shape — a pure `kay-core` library backing multiple frontends. The ForgeCode harness is compiled into the main binary, **not** shipped as a Tauri sidecar (sidecar notarization is broken on macOS per Tauri #11992).

## Core Components

- **`kay-core`** — Agent loop, tool registry, context engine, session store, verifier. No UI or runtime dependencies.
- **`kay-cli`** — Headless entry point; used for TB 2.0 submission and CI.
- **`kay-tauri`** — Tauri 2.x app; exposes agent via typed `#[tauri::command]` + `ipc::Channel<AgentEvent>` streaming.
- **`kay-sandbox-{macos,linux,windows}`** — Per-OS sandbox implementations behind the `Sandbox` trait.
- **`kay-provider-openrouter`** — OpenRouter HTTP + SSE client with strict model allowlist and tolerant JSON parser.
- **Frontend (`kay-tauri/ui/`)** — React 19 + TypeScript + Vite; `tauri-specta` v2 for typed IPC bindings.

## Design Principles

1. **Headless first** — every capability is usable from `kay-cli`. The desktop UI is additive, never load-bearing for a benchmark run.
2. **Merged binary** — the harness and UI ship together. No `externalBin` sidecars; no split process model on the happy path.
3. **Event-driven core** — `tokio::select!` over input / model / tool / control channels; `AgentEvent` is the single streaming interface between core and all frontends.
4. **Personas as data** — `kay`, `sage`, `muse` share one code path, parameterized by YAML (prompt + tool filter + model).
5. **Schema discipline** — all tool schemas pass through the ForgeCode hardening post-process (required-before-properties, flattening, explicit truncation reminders) before reaching the provider.
6. **Fail loud, never silent** — sandbox escapes, schema errors, and provider variance surface as typed events and `AgentEvent` frames; never swallowed.
7. **Extension-point discipline** — all core traits are object-safe and async; `#[non_exhaustive]` on public enums. v2 wedges slot in additively without breaking v1 consumers.

## Technology Choices

See `.planning/research/STACK.md` for the authoritative pinned dependency table. High-level:

- **Rust 2024** with tokio 1.51 LTS, reqwest 0.13 + reqwest-eventsource, rustls 0.23
- **Tauri 2.10+** (merged binary), `tauri-specta` v2 for IPC bindings
- **Frontend**: React 19 + TypeScript + Vite; Monaco or CodeMirror for the diff viewer
- **Context engine**: tree-sitter + sqlite-vec
- **Shell**: `tokio::process` for non-PTY; `portable-pty` fallback for TTY-requiring commands
- **Sandbox**: `sandbox-exec` (macOS), `landlock` + `seccompiler` (Linux), Job Objects + restricted token (Windows)
- **Signing**: Apple Developer ID + Azure Code Signing; signed GPG/SSH release tags
