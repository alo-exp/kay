# Product Requirements Overview

This document captures the product vision and high-level requirements for Kay. It is kept in sync with `.planning/REQUIREMENTS.md` — the authoritative requirements source managed by GSD. Update during the FINALIZATION step of each phase.

## Product Vision

Kay is an open-source Rust + Tauri coding agent — a fork of ForgeCode hardened with Terminus-KIRA's harness techniques and delivered through the first native desktop UI for an agentic coding tool. It targets developers who want the top-tier agentic coding experience (today locked inside Claude Code, Codex, and ForgeCode) as a single permissively-licensed binary on macOS, Windows, and Linux.

## Core Value

**Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.** If the score target fails, Kay has no reason to exist; if the UI fails to ship, Kay is just another ForgeCode fork. Both must hold.

## Requirement Areas

See `.planning/REQUIREMENTS.md` for the authoritative 83 v1 requirements. Top-level categories:

- **Governance (GOV)** — Apache-2.0 + DCO + signed tags + clean-room attestation
- **Workspace (WS)** — Rust 2024 cargo workspace with per-OS sandbox crates
- **Provider / HAL (PROV)** — OpenRouter with strict allowlist + tolerant JSON parser
- **Tool Registry & KIRA Techniques (TOOL / SHELL)** — native tool calling + marker polling + image_read
- **Sandbox (SBX)** — per-OS kernel-enforced sandbox
- **Agent Loop (LOOP)** — event-driven, personas as data
- **Session Store (SESS)** — JSONL + SQLite + pre-edit snapshots
- **Context Engine (CTX)** — tree-sitter symbol store + sqlite-vec hybrid retrieval
- **Verifier (VERIFY)** — multi-perspective critics before task-complete
- **Tauri Desktop Shell + UI (TAURI / UI)** — merged binary, React 19, session view, multi-session manager
- **CLI / Release (CLI / REL)** — headless mode preserved, signed bundles for 5 targets
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
