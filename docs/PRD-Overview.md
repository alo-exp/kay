# Product Requirements Overview

This document captures the product vision and high-level requirements for Kay. It is kept in sync with `.planning/REQUIREMENTS.md` — the authoritative requirements source managed by GSD. Update during the FINALIZATION step of each phase.

## Product Vision

Kay is an open-source Rust coding agent — a fork of ForgeCode hardened with Terminus-KIRA's harness techniques and delivered through **three user surfaces over one core**: a canonical CLI (`kay-cli` — rebrands ForgeCode's `forge_main`), a full-screen ratatui TUI (`kay-tui`), and a native Tauri desktop GUI (`kay-tauri`). It targets developers who want the top-tier agentic coding experience (today locked inside Claude Code, Codex, and ForgeCode) as a single permissively-licensed binary on macOS, Windows, and Linux, from the SSH shell to the desktop.

## Core Value

**Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with three production-grade frontends.** If the score target fails, Kay has no reason to exist; if the CLI fails to work standalone, Kay is unusable on SSH; if the GUI or TUI fails to ship, Kay is just another ForgeCode fork. All three must hold.

## Requirement Areas

See `.planning/REQUIREMENTS.md` for the authoritative v1 requirements (95 REQ-IDs after 2026-04-19 amendments). Top-level categories:

- **Governance (GOV)** — Apache-2.0 + DCO + signed tags + clean-room attestation
- **Workspace (WS)** — Rust 2024 cargo workspace with 8 crates
- **Provider / HAL (PROV)** — OpenRouter with strict allowlist + tolerant JSON parser
- **Tool Registry & KIRA Techniques (TOOL / SHELL)** — native tool calling + marker polling + image_read
- **Sandbox (SBX)** — per-OS kernel-enforced sandbox
- **Agent Loop (LOOP)** — event-driven, personas as data (`forge`/`sage`/`muse` — inherited from ForgeCode)
- **Session Store (SESS)** — JSONL + SQLite + pre-edit snapshots
- **Context Engine (CTX)** — tree-sitter symbol store + sqlite-vec hybrid retrieval
- **Verifier (VERIFY)** — multi-perspective critics before task-complete
- **CLI — Canonical Backend (CLI)** — standalone headless + interactive; rebrands `forge_main`; structured-event JSONL stream is the contract frontends consume
- **TUI — Full-Screen Terminal Frontend (TUI)** — ratatui multi-pane UI; keyboard-first; SSH-friendly; consumes the CLI contract
- **Tauri Desktop Shell + UI (TAURI / UI)** — merged binary, React 19, session view, multi-session manager; frontends the CLI contract
- **Release & Distribution (REL)** — signed + notarized bundles on all five targets; `cargo install kay` and `cargo install kay-tui` standalone
- **Evaluation (EVAL)** — parity gate + TB 2.0 submission

## Out of Scope

See `.planning/REQUIREMENTS.md ## Out of Scope` and `.planning/PROJECT.md ## Requirements → Out of Scope`. Summary:

- Direct Anthropic / OpenAI / Gemini / Groq APIs (v2)
- Local model support (v2)
- IDE extensions (v3+)
- The four wedge differentiators — ACE, dynamic routing, verification-first depth, multi-agent orchestration (v2)
- Web dashboard / plugin marketplace / CLA for contributors (permanently out)

## North-Star Metric

Public Terminal-Bench 2.0 score with model-pinned, archived reference transcript. The only acceptable v1 score is **>81.8%**.
