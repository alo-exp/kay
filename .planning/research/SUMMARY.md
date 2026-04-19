# Project Research Summary

**Project:** Kay
**Domain:** Open-source Rust + Tauri 2.x desktop coding agent (ForgeCode fork + KIRA harness + OpenRouter)
**Researched:** 2026-04-19
**Confidence:** HIGH

---

## Executive Summary

Kay is a benchmark-first product: if it does not beat ForgeCode (>81.8%) on Terminal-Bench 2.0, there is no reason for it to exist in a crowded field. The research confirms the wedge is real and achievable. ForgeCode is Apache-2.0, its context engine and JSON schema hardening are the single largest contributors to the 30-point gap over OpenCode, and KIRA's four harness techniques (native tool calling, marker-based polling, multi-perspective critics, multimodal `image_read`) are individually low-to-medium complexity but have never been stacked on top of ForgeCode's base on a public codebase. Codex CLI (codex-rs) is the closest architectural blueprint: a ~70-crate Cargo workspace where a pure `kay-core` library is consumed by both a headless CLI and a Tauri desktop shell. Kay should mirror that split exactly.

Two non-negotiables must be resolved before a single line of agent code is written. First, Phase 1 carries an unusually heavy governance load: fork attribution, NOTICE preservation, DCO enforcement CI, Apple Developer ID and Azure Code Signing enrollment (both take weeks), and signed release tags from v0.0.1. Deferring any of these is the single most common way OSS agent forks get their momentum killed — either by an attribution complaint, a DMCA adjacent to the claw-code ecosystem, or a day-one "can't open because Apple can't check" failure. Second, Phase 1 must also establish a parity gate: fork ForgeCode, run it unmodified on Terminal-Bench 2.0, and confirm it still hits its published baseline before any changes are made. Silent regressions introduced by a naive fork are the most common first-month failure mode.

The Tauri sidecar notarization bug (issue #11992) is a hard constraint that overrides an otherwise natural architecture choice: the ForgeCode harness must be compiled into the main Rust binary, not registered as an `externalBin` sidecar. The OpenRouter tool-calling variance problem is equally hard: Kay must maintain an explicit model allowlist and target OpenRouter's Exacto endpoints for benchmark submissions, not expose all 300+ models as equivalent. Both constraints are architecture-level decisions that must be locked before Phase 3 begins.

---

## Key Findings

### Recommended Stack

The stack is fully inherited from the ForgeCode fork plus Tauri 2.x layered on top. Every top-10 Terminal-Bench 2.0 entrant runs Rust; the single-binary distribution requirement eliminates all alternatives. The key additions over a vanilla ForgeCode fork are: `tauri` 2.10.x + React 19 + TypeScript 5 for the desktop shell; `tauri-specta` v2 for typed IPC; `tauri::ipc::Channel<T>` (not events) for token streaming; `portable-pty` 0.9.x for PTY-backed shell execution; `tree-sitter` 0.24.x for the context engine; and a thin in-tree OpenRouter client over `reqwest` 0.13 + `reqwest-eventsource` 0.6 (the `openrouter_api` crate is pinned to reqwest 0.11 and must not be used). The `schemars` 0.8.x + manual post-processing for `required`-before-`properties` and flattening is load-bearing for the benchmark score — this is ForgeCode's JSON schema hardening and must be preserved verbatim.

**Core technologies:**
- Rust stable >= 1.85 (MSRV 1.82): language — every top-10 TB2 entrant, single binary, parity with ForgeCode
- `tokio` 1.51.x LTS: async runtime — LTS until March 2027; pin tightly, avoid `full` feature
- `tauri` 2.10.x: desktop shell — Kay's primary differentiator; no viable alternative for the first-party GUI wedge
- `reqwest` 0.13 + `reqwest-eventsource` 0.6: HTTP + SSE — thin in-tree OpenRouter client only; no third-party OR crate
- `schemars` 0.8.x + manual post-processing: JSON schema hardening — directly responsible for ForgeCode's jump from ~74% to 81.8% on TB2
- `portable-pty` 0.9.x: cross-platform PTY — WezTerm battle-tested; required for KIRA marker polling
- `tree-sitter` 0.24.x + `rusqlite` 0.32 + `sqlite-vec`: context engine — function signature indexing is the 30-point moat over OpenCode
- `tauri-specta` v2: typed Rust <-> TypeScript bridge — eliminates IPC drift at compile time
- React 19 + TypeScript 5 + Vite 6: UI layer — not Leptos/Yew/Dioxus; ecosystem mass for agent-UI components matters

### Expected Features

Research confirms four feature clusters. The first (ForgeCode parity) is fully inherited from the fork and costs near-zero implementation effort. The second (KIRA techniques) has each technique at LOW-MEDIUM complexity individually; the score gain comes from stacking all four. The third (Tauri UI) is HIGH complexity but is the only unoccupied competitive surface in the OSS agent ecosystem. The fourth (governance) is LOW complexity but HIGH timeline-risk due to external dependencies.

**Must have — table stakes (v1, inherited from ForgeCode):**
- Agent loop (plan -> act -> observe -> repeat) — inherited, keep intact
- Three specialized agents: `kay` (write), `sage` (read-only research), `muse` (planning) — YAML-driven personas, not triplicated code
- Semantic context engine (tree-sitter -> symbol extraction -> hybrid retrieval) — the 30-point moat; do not rewrite
- JSON schema hardening (required-before-properties, flattened nested required, truncation reminders) — the reliability moat; ForgeCode's key differentiator
- Bash tool with sandboxed subprocess isolation (macOS sandbox-exec / Linux Landlock+seccomp / Windows Job Objects)
- Mandatory verification pass before task-complete signal
- OpenRouter provider with API-key auth, model allowlist, and Exacto endpoint targeting
- Session resume, export (JSON + Markdown + HTML), headless/CI mode
- Signed release binaries from v0.0.1; Apache-2.0 + DCO enforcement

**Must have — KIRA harness techniques (v1, new work):**
- Native LLM tool calling via `tools` parameter — eliminates ICL parsing; KIRA documented +4.6pp from this alone
- Marker-based command-completion polling (`__CMDEND__<seq>__` with cryptographically random per-command sentinels)
- Multi-perspective completion verification (test engineer + QA engineer + end-user critics) — gated on confidence signal, not always-on
- Multimodal `image_read` tool (base64 terminal screenshots) — triggered only on ANSI-soup/TUI output, not every command

**Must have — Tauri desktop UI (v1, new work):**
- Native Tauri 2.x desktop app (macOS arm64/x64, Windows x64, Linux x64/arm64) bundled with Kay binary
- Session view: live agent trace, tool-call timeline, token/cost meter
- Multi-session manager: spawn, pause, resume, fork sessions
- Project workspace: directory picker, env/key management, OpenRouter account binding
- Log/export panel: structured session export for TB 2.0 submission
- Headless CLI mode preserved (required for benchmark harness; Tauri is additive)

**Should have — v1.x (after v1 is validated):**
- Session fork from GUI (explore two paths in parallel)
- Verification-critics panel (surface test/QA/end-user votes in UI)
- Syntax-highlighted diff viewer in tool-call inspector
- File/working-tree panel per session
- Session timeline/checkpoint scrubber
- Command approval dialog (safe mode, off by default for benchmark)
- MCP client support (P2 — not in PROJECT.md Active requirements, but table-stakes integration surface by 2026)

**Defer — v2+ (explicitly out of scope per PROJECT.md):**
- ACE (self-improving context), dynamic model routing, deep verification-first loop, re-enterable hierarchical multi-agent orchestration
- Direct Anthropic/OpenAI/Gemini/Groq APIs; local model support (Ollama/llama.cpp)
- IDE extensions; hooks; auto-memory (ClawMem-style); plugin marketplace/MCP catalog

### Architecture Approach

The architecture follows Codex CLI's workspace split: a pure `kay-core` library (no I/O, defines traits and the agent event loop) consumed by two thin shells — `kay-cli` (headless, `clap`-driven) and `kay-desktop` (Tauri app). Per-capability crates (`kay-tools`, `kay-context`, `kay-providers`, `kay-verify`, `kay-sandbox`, `kay-shell`, `kay-session-store`) enforce boundary hygiene and enable incremental compilation. The agent loop is event-driven (`tokio::select!` over model stream, tool results, and control channel), never a blocking REPL. All persona behavior (kay/sage/muse) is data-driven via YAML, not triplicated code. The single most important constraint is that the ForgeCode harness binary must be compiled into the main Rust binary — not registered as a Tauri `externalBin` sidecar — because macOS notarization is broken for sidecars (tauri#11992).

**Major components:**
1. `kay-core` — pure library: agent loop (event-driven), session manager (Thread/Turn/Item state machine), agent switch (YAML personas)
2. `kay-tools` — tool registry (`trait Tool`), built-in tools: `execute_commands` (KIRA bash), `task_complete` (triggers verifier), `image_read` (multimodal), `read_file`, `edit_file`, `search`, `fetch`
3. `kay-context` — semantic context engine: tree-sitter parsers -> symbol extraction -> sled index -> hybrid BM25+embedding retrieval -> prompt assembly (ForgeCode's 30-point moat)
4. `kay-providers` — `trait ChatProvider`: OpenRouter impl (reqwest + SSE + tool-call reassembly + tolerant JSON parser); error taxonomy; model allowlist enforcement
5. `kay-verify` — multi-perspective critics (test engineer + QA + end-user); verdict aggregation; confidence-signal gating
6. `kay-sandbox` — per-OS `trait Sandbox`: macOS sandbox-exec profiles, Linux Landlock+seccomp, Windows Job Objects + RestrictedToken
7. `kay-shell` — PTY-backed execution via `portable-pty` + `__CMDEND__<seq>__` marker polling
8. `kay-session-store` — SQLite index + JSONL append-only transcript (Claude Code model); pre-edit snapshots; session fork via `parent_session_id` column (reserved for v2 multi-agent)
9. `kay-cli` — headless CLI surface (clap-driven; preserves TB 2.0 submission path)
10. `kay-desktop` — Tauri app: `#[tauri::command]` for control, `tauri::ipc::Channel<AgentEvent>` for streaming (not events), React 19 frontend with specta-generated TypeScript types

**Critical data flow:** User input -> Tauri invoke -> Session Manager -> Agent Loop -> Context Engine (symbol retrieval) -> Provider HAL (SSE stream) -> Tool Registry (sandboxed execution) -> Verifier (on task_complete) -> AgentEvent channel -> Tauri -> React render. All agent-loop work runs on `tauri::async_runtime`; the webview never blocks.

### Critical Pitfalls

1. **Tauri sidecar notarization is broken (tauri#11992)** — Never register the Kay harness as `externalBin`. Compile it into the main binary. Catch this in Phase 1 architecture; failing here blocks all macOS distribution.

2. **Apple Developer ID and Windows code-signing lead time is 2-4 weeks** — Apply in Phase 1, not Phase 5. Shipping unnotarized on day one burns trust that is nearly impossible to recover. Run signed+notarized builds on every merge to main, not only on release tags, so stalls surface early.

3. **OpenRouter tool-calling variance is severe** — Maintain an explicit model allowlist from the first provider integration. Never silently fall back to ICL parsing. Target Exacto endpoints for benchmark submissions. Run weekly per-model smoke tests in CI. Flag off-list models as "compatibility unknown" in the UI.

4. **ForgeCode JSON schema hardening is load-bearing for the score** — The `required`-before-`properties` reordering and nested-required flattening in `schemars` post-processing is directly responsible for ForgeCode's jump to 81.8%. `.unwrap()` on tool-call JSON parsing is never acceptable; implement a two-pass tolerant parser.

5. **Multi-perspective verification token cost will triple per-task cost** — Gate the critic trio on a confidence/complexity signal from the start. Default: 1 critic in interactive mode, 3 in benchmark mode. Implement `--max-usd` budget cap in the harness before any eval runs. Cost-per-task regression gate in CI.

6. **DMCA exposure from claw-code proximity** — Kay's contributors must not reference the March 2026 Claude Code leak or any claw-code derivative. CONTRIBUTING.md must spell this out; DCO signoff attests it. Maintain a clean-room log for any prompt or pattern that resembles Claude Code.

7. **Benchmark overfitting to TB 2.0's 89-task shape** — Establish a parallel eval set of real repos (Rails app, React+TS, Rust crate, Python package, monorepo >10k files) and require both to pass. Hold out a subset of TB tasks and never look at it until submission. Set a hard retry-budget cap matching the official Harbor submission settings.

---

## Implications for Roadmap

Research produced a 12-phase dependency-ordered structure with a heavy Phase 1 and a distinct parity-gate at the end of Phase 1. The roadmap should follow this structure closely because the dependencies are real: the agent loop requires the provider HAL, tools require sandboxing before any file writes, the Tauri UI requires stable AgentEvent types, and TB 2.0 submission requires the full stack.

### Phase 1: Fork, Governance, and Infrastructure

**Rationale:** Cannot build on an unsigned, unattributed fork. Governance mistakes are essentially irreversible at scale. Code-signing enrollment has a 2-4 week external lead time that gates every release. This is the most loading Phase 1 of any project — it must be planned for, not rushed.

**Delivers:**
- Rust workspace forked from ForgeCode; crate skeleton split (kay-core, kay-cli, kay-desktop, kay-tools, etc.) following Codex CLI workspace pattern
- Apache-2.0 `NOTICE` preserved verbatim; `ATTRIBUTIONS.md` with upstream commit hash; file-level copyright headers intact
- DCO enforcement via GitHub Actions (`Signed-off-by` required on every commit); CONTRIBUTING.md clean-room policy (no claw-code, no Claude Code leak)
- Apple Developer ID enrollment + Azure Code Signing (or DigiCert KeyLocker) enrollment started immediately
- Signed release tag pipeline from v0.0.1; `tauri-plugin-updater` keypair generated and public key pinned
- `cargo-deny` license gate in CI; `cargo-audit` in CI
- **Parity gate:** run ForgeCode unmodified on TB 2.0 Harbor harness, confirm baseline score. This is a go/no-go for Phase 2.

**Avoids:** Pitfall 2 (NOTICE hygiene), Pitfall 8 (signing lead time), Pitfall 11 (DCO vs CLA — DCO already chosen per PROJECT.md), Pitfall 12 (DMCA exposure), Pitfall 15 (scope creep — Out of Scope contract set)

**Research flag:** Standard patterns for Apache-2.0 fork hygiene and DCO. No deeper research needed. Code-signing enrollment process is well-documented by Tauri.

---

### Phase 2: Provider HAL + Tolerant JSON Parser

**Rationale:** Everything downstream (agent loop, tools, context engine, verification) requires model calls. The provider trait must exist before the agent loop can be built, and the tolerant JSON parser must exist before any eval work. This is the phase where the OpenRouter allowlist and Exacto endpoint strategy gets locked in.

**Delivers:**
- `trait ChatProvider` with streaming `BoxStream<ChatDelta>` interface
- OpenRouter impl: `reqwest` 0.13 + `reqwest-eventsource` 0.6 + SSE parsing + tool-call fragment reassembly
- Tolerant two-pass JSON parser for tool-call arguments (handles `arguments: null`, `arguments: "{}"`, unterminated strings)
- Error taxonomy: `RateLimited`, `ContextLengthExceeded`, `ToolCallMalformed`, `ProviderUnavailable`
- Model allowlist (Tier-1: Claude Opus/Sonnet 4.x, GPT-5/5-mini, Gemini 2.5 Pro); Exacto endpoint targeting for benchmark runs
- `tower` + `backon` retry with jittered exponential backoff for 429/5xx
- Weekly per-model tool-calling smoke test in CI
- Mock `ChatProvider` for agent loop unit testing (no network required)

**Avoids:** Pitfall 4 (non-spec tool-call JSON), Pitfall 10 (OpenRouter coverage gaps), Pitfall 13 (benchmark cost — budget cap built into client)

**Research flag:** SSE retry semantics around OpenRouter's 429 behavior need validation with real traces. Flag for `/gsd-research-phase` if unexpected behavior surfaces.

---

### Phase 3: Tool Registry + KIRA Core Tools

**Rationale:** Smallest valid tool surface that proves the native-tool-calling path end-to-end. KIRA's exact three tools (`execute_commands`, `task_complete`, `image_read`) plus ForgeCode's file ops and search. Must lock in the cryptographically random marker scheme and the `image_read` firing policy before the agent loop is built on top.

**Delivers:**
- `trait Tool` + `ToolRegistry` (HashMap-based, explicit runtime `register()`, no linker-order fragility)
- `execute_commands` tool: PTY-backed bash + `portable-pty` + `__CMDEND__<random-128bit>__` sentinel with `(sentinel, exit_code)` validation; user-input lane separated from agent PTY
- `task_complete` tool: triggers verifier; the gate between "agent says done" and "Kay says done"
- `image_read` tool: base64 terminal screenshot; firing policy: only on ANSI-soup/TUI output or garbled bytes, not every command; per-turn cap (1-2 images), per-session cap (10-20)
- `read_file`, `edit_file` (unified diff patch), `search` (ripgrep wrapper), `fetch` (HTTP)
- Permission gating layer wrapping the registry (claw-code pattern)
- `schemars` 0.8.x + manual post-processing: `required`-before-`properties`, nested-required flattening, truncation reminders

**Avoids:** Pitfall 4 (tool-call JSON parsing), Pitfall 5 (marker race), Pitfall 6 (verification cost — gating designed here), Pitfall 7 (image_read bloat)

**Research flag:** Standard patterns. No deeper research needed for the tool registry itself. `image_read` firing heuristics may need tuning after first TB2 eval run.

---

### Phase 4: Sandbox (All Three Platforms)

**Rationale:** The tool registry's `execute_commands` must not write outside the project root before Phase 5 builds the agent loop. Sandbox must exist before any agent writes files. macOS first (primary dev machine), then Linux, then Windows.

**Delivers:**
- `trait Sandbox` + per-OS impls in `kay-sandbox`
- macOS: `sandbox-exec` (Seatbelt profile) confining writes to project root + /tmp
- Linux: `rust-landlock` + `seccompiler` (kernel >= 5.13 required; fallback to `PR_SET_NO_NEW_PRIVS` on older kernels)
- Windows: Job Objects + `RestrictedToken` + `GenerateConsoleCtrlEvent` for Ctrl+C
- `SandboxPolicy` struct: `writable_roots`, `readable_roots`, `allow_network`
- Full Windows CI on `windows-latest` runner including interactive PTY tests and modern ConPTY flags

**Avoids:** Pitfall 14 (cross-platform PTY + Windows ConPTY), security mistakes (credential exfiltration, prompt injection)

**Research flag:** Windows ConPTY flag set (`PSEUDOCONSOLE_RESIZE_QUIRK` etc.) needs hands-on validation; flag for Phase research if the portable-pty fork is needed.

---

### Phase 5: Agent Loop (Event-Driven Core)

**Rationale:** With provider, tools, and sandbox in place, the core brain can be built. This is the first point where Kay can execute one-shot tasks headlessly. Persona YAML and the event stream must be locked before the Tauri UI subscribes to them.

**Delivers:**
- Event-driven agent loop: `tokio::select!` over model stream, tool results, control channel (pause/abort/resume)
- `AgentEvent` enum (`#[non_exhaustive]`): `ModelText`, `ToolCall`, `ToolResult`, `VerifyResult`, `TurnEnd`, `Aborted`
- `kay/sage/muse` personas as YAML data (`agents.yaml`), not triplicated code; sage callable as a sub-tool from kay and muse
- Two-mode harness: `--benchmark` mode (3 critics, Exacto endpoints, explicit retry budget) vs interactive mode (1 critic, standard routing)
- Context compaction / auto-summarization at 50% context window fill
- Headless `kay-cli` wiring (`kay run --prompt "..." --headless`)
- `--max-usd` budget cap enforced at harness level

**Avoids:** Pitfall 1 (benchmark overfitting — two-mode separation), Pitfall 3 (ForgeCode over-specialization — separate benchmark/interactive prompt sets), Pitfall 15 (scope creep)

**Research flag:** Standard patterns (Codex CLI event-driven loop is the blueprint). No deeper research needed.

---

### Phase 6: Session Store + Transcript

**Rationale:** First point where state survives process restart. Required before Tauri UI can demo anything meaningful and before multi-session management is possible. `parent_session_id` column reserved now for v2 multi-agent (zero-cost future-proofing).

**Delivers:**
- SQLite index (`kay.db`): `sessions`, `turns`, `tool_invocations`, `compaction_history`, `parent_session_id` (NULL in v1)
- JSONL append-only transcript per session (source of truth for replay and export)
- Resume by session ID; session fork (creates new session with `parent_session_id` set)
- Pre-edit file snapshots for rewind under `~/.kay/snapshots/<session-id>/<turn>/`
- JSON + Markdown + HTML export (required for TB 2.0 submission transcript)
- Async write-through: in-memory first -> JSONL append -> SQLite index update; WAL mode

**Avoids:** Pitfall 9 (Tauri memory leak — session lifetime aligns with Channel lifecycle management)

**Research flag:** Standard patterns (Claude Code SQLite+JSONL model is well-documented). No deeper research needed.

---

### Phase 7: Context Engine

**Rationale:** ForgeCode's 30-point moat over OpenCode. Must be inherited intact from the fork; this phase confirms the inheritance works and the tree-sitter grammars cover Kay's target language set. Biggest single phase by implementation complexity.

**Delivers:**
- `tree-sitter` 0.24.x parsers for Rust, TypeScript, JavaScript, Python, Go, Java (6 default grammars)
- Symbol extraction: function signatures, class boundaries, module structure (not raw file dumps)
- `sled`-backed symbol index with `sqlite-vec` extension for local vector search (on-disk, zero-daemon)
- Hybrid BM25 + embedding retrieval; top-K snippet assembly into prompt
- Lazy / on-demand indexing (only files touched in the session) — avoids monorepo re-index overhead
- Incremental parsing with file-watch invalidation
- Context engine trait: `retrieve(turn_context) -> Vec<Snippet>` + `update_on_verdict(trace, verdict)` no-op in v1 (ACE hook reserved)

**Avoids:** Pitfall 1 (benchmark overfitting — context engine must work on real monorepos), Anti-Pattern 2 (raw file dumps)

**Research flag:** SQLite schema for function signatures + vector embeddings is an open design question. Flag for `/gsd-research-phase` at this phase. Context engine quality is the #1 lever on TB2 score after harness techniques.

---

### Phase 8: Multi-Perspective Verification (KIRA Critics)

**Rationale:** The last of the four KIRA techniques. Builds on the stable agent loop and context engine. This phase completes Kay's harness superiority over ForgeCode. Score wedge: stacking all four KIRA techniques on ForgeCode's context engine is the predicted path to >81.8%.

**Delivers:**
- `trait Critic` + three impls: `TestEngineerCritic`, `QAEngineerCritic`, `EndUserCritic`
- Confidence/complexity gating: critic trio fires only when primary agent self-score is low, >N files touched, or tests failed
- Verdict aggregation: PASS -> `TurnEnd`; FAIL+feedback -> inject critic feedback as user message -> continue loop
- Interactive default: 1 critic; benchmark default: 3 critics (configurable)
- `--max-usd` budget integration: verification passes counted against session budget
- Cost-per-task regression gate in CI: >30% token increase vs baseline = build failure

**Avoids:** Pitfall 6 (verification token cost), Pitfall 1 (benchmark overfitting — verification must be parameterized)

**Research flag:** Critic prompt quality is the main tuning lever. Expect iteration after first TB2 run. Meta-Harness (76.4%) shows ceiling risk from over-verification — gating is non-negotiable.

---

### Phase 9: Tauri Desktop Shell

**Rationale:** Cannot ship without the UI; this is Kay's distinguishing surface. Build after AgentEvent types and Session Store are stable so the UI does not chase moving APIs. The Tauri sidecar architecture decision must be confirmed: harness in the main binary, not externalBin.

**Delivers:**
- `kay-desktop` crate: `tauri::Builder`, `#[tauri::command]` handlers, `tauri::ipc::Channel<AgentEvent>` bridge
- React 19 + TypeScript 5 + Vite 6 frontend with `specta`-generated types (zero IPC drift)
- Session view: live agent trace, tool-call timeline cards (each tool call = structured card with args + output + diff)
- Token/cost meter: live per-turn and per-session USD burn from OpenRouter usage headers
- Export panel: MD + JSON + HTML formats, copy-to-clipboard, open-folder buttons
- Agent type selector (kay/sage/muse dropdown); model selector with tiered OpenRouter catalog (Recommended / Experimental / All)
- Benchmark/interactive mode toggle with distinct visual treatment
- Memory hygiene: Channels with explicit lifecycle management; event batching at 100ms for low-priority telemetry; 4-hour canary session with memory delta regression gate

**Avoids:** Pitfall 8 (notarization — no `externalBin` sidecar), Pitfall 9 (Tauri memory leak — channels + batching from day one), Anti-Pattern 3 (Tauri events for streaming)

**Research flag:** Tauri IPC memory leak mitigation — check current upstream status of issues #12724/#13133 before building session view. Flag for `/gsd-research-phase` if unresolved.

---

### Phase 10: Multi-Session Manager + Project Settings

**Rationale:** Completes the UI surface required by PROJECT.md Active requirements. Depends on Session Store (Phase 6) and Tauri Shell (Phase 9). This phase transforms Kay from a single-session demo into a usable product.

**Delivers:**
- Multi-session tab manager: spawn, pause, resume, archive, delete sessions from GUI
- Session fork button (GUI one-click: creates child session with `parent_session_id` set)
- Project workspace panel: directory picker, `.kay/config.toml` editor, environment variable management, OpenRouter API key binding to OS keychain
- `tauri-plugin-store` for UI preferences; `keyring` crate for secrets (never localStorage)
- Config hot-reload with visible reload banner and atomic schema validation on save
- OpenRouter model selector: tiered list (Recommended / Experimental / All behind warning)
- Command approval dialog (safe mode) — off by default for benchmark, on by default for first-time users

**Avoids:** Pitfall 3 (ForgeCode over-specialization — interactive mode config distinct from benchmark config), Pitfall 13 (cost visibility — live cost meter at all times)

**Research flag:** Standard Tauri plugin patterns. No deeper research needed.

---

### Phase 11: Cross-Platform Hardening + Release Pipeline

**Rationale:** Binary matrix (macOS arm64/x64, Windows x64, Linux x64/arm64) and the full signing pipeline must be verified end-to-end before TB2 submission. Windows ConPTY full test suite must be green.

**Delivers:**
- Tauri bundler pipeline: macOS `.dmg` (signed + notarized), Windows `.msi`/`.exe` (Authenticode via Azure Code Signing), Linux `.deb` + `.AppImage`
- `cargo-dist` for headless CLI tarballs adjunct to the Tauri bundle
- `cargo install kay` published to crates.io (headless only; desktop app downloaded from releases)
- `tauri-plugin-updater` signature verification with pinned public key; GitHub Releases-based update channel
- Full Windows CI: ConPTY flags validated, interactive PTY test suite green, `Ctrl+C` interrupt confirmed
- `cargo-deny` + `cargo-audit` + `cargo-machete` all green
- Release checklist: signed+notarized build on every merge to main; notarization timeout 2 hours with retry

**Avoids:** Pitfall 8 (code signing), Pitfall 14 (Windows ConPTY)

**Research flag:** Windows EV certificate automation validated in Phase 1. No new research needed here.

---

### Phase 12: Terminal-Bench 2.0 Submission + v1 Hardening

**Rationale:** The v1 acceptance gate. No release without a clean reference run on the public leaderboard at >81.8%. This phase is primarily evaluation, iteration on weak tasks, and final score confirmation — not new feature development.

**Delivers:**
- Local Harbor harness run matching official submission settings exactly (same container, same retry budget, same model config)
- Held-out task subset never touched during development; revealed now for final validation
- Parallel eval on real-repo set (Rails, React+TS, Rust crate, Python package, monorepo >10k files) — both must pass
- TB2 `--replay-single-task` mode to iterate on weak tasks without re-running passing ones
- Budget cap `--max-usd` verified; cost-per-run documented and within roadmap budget
- Official TB2 submission with documented reference run transcript (required for leaderboard)
- v1.0 release: signed binaries on all 5 platforms, changelog, attribution acknowledgment to ForgeCode and KIRA

**Avoids:** Pitfall 1 (benchmark overfitting — real-repo eval is the final gate), Pitfall 13 (cost overrun — single-task replay avoids re-running all 89 tasks)

**Research flag:** TB2 official Harbor submission procedure should be confirmed at Phase 1. No new research needed at Phase 12 if Harbor validation happened earlier.

---

### Phase Ordering Rationale

The ordering follows three hard dependency chains:

**Chain A (Score):** Phase 1 (fork) -> Phase 2 (provider + JSON parser) -> Phase 3 (tools + KIRA) -> Phase 4 (sandbox) -> Phase 5 (agent loop) -> Phase 7 (context engine) -> Phase 8 (verification) -> Phase 12 (TB2 submission). No phase can be skipped without breaking the downstream one.

**Chain B (UI):** Phase 5 (stable AgentEvent types) -> Phase 6 (session store) -> Phase 9 (Tauri shell) -> Phase 10 (multi-session). The UI is additive but cannot start until AgentEvent types are frozen.

**Chain C (Distribution):** Phase 1 (signing enrollment) -> Phase 9 (signed builds on merge) -> Phase 11 (release pipeline) -> Phase 12 (v1 release). Signing lead time from Phase 1 is the critical path.

**Parallel opportunities:** Phase 7 (Context Engine) and Phase 6 (Session Store) share no code and can run in parallel if staffing allows. Phase 9 (Tauri Shell) scaffolding can begin during Phase 6 once AgentEvent types are frozen from Phase 5.

**Non-negotiable sequencing:** Phase 1's parity gate (confirm ForgeCode hits its baseline unmodified) must complete before Phase 2 begins. Any silent regression introduced by the fork setup must be identified at this point, not discovered in Phase 12.

### Research Flags

Phases likely needing `/gsd-research-phase` during planning:
- **Phase 2 (Provider HAL):** OpenRouter SSE retry semantics + Exacto endpoint behavior under rate limits — needs real-trace validation
- **Phase 4 (Sandbox):** Windows ConPTY flag set in `portable-pty` — may need a Kay-specific fork; validation needed before committing
- **Phase 7 (Context Engine):** SQLite schema for function signatures + vector embeddings is an open design question; audit actual ForgeCode indexer before reimplementing
- **Phase 9 (Tauri Shell):** Tauri IPC memory leak status (issues #12724, #13133) — check current upstream status before building session view

Phases with well-documented patterns (skip research-phase):
- **Phase 1 (Fork + Governance):** Apache-2.0 attribution, DCO setup, and Tauri code signing are well-documented
- **Phase 3 (Tool Registry):** Standard Rust trait pattern; KIRA's tool shapes are public
- **Phase 5 (Agent Loop):** Codex CLI event-driven loop is publicly documented; ForgeCode's loop is in the fork
- **Phase 6 (Session Store):** Claude Code's SQLite+JSONL hybrid is well-documented community synthesis
- **Phase 10 (Multi-Session):** Tauri plugin patterns well-documented; Claudia is a reference
- **Phase 11 (Cross-Platform):** Tauri bundler + cargo-dist pipelines are standard

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core technologies verified against ForgeCode (fork source), Codex CLI (blueprint), and Tauri official docs. One MEDIUM gap: SSE crate selection — minor, resolvable at Phase 2. |
| Features | HIGH | Reference implementations are all public (ForgeCode, KIRA, Claudia, OpenCode, claw-code) with authoritative writeups. Feature table derives from observable behavior, not speculation. |
| Architecture | HIGH | Codex CLI architecture is publicly documented; ForgeCode is in the fork; KIRA has a published technical writeup. Tauri IPC patterns from official docs. |
| Pitfalls | HIGH (Tauri, OpenRouter, claw-code/DMCA) / MEDIUM (ForgeCode internals, KIRA score ratios) | Tauri notarization bug is confirmed open (tauri#11992). OpenRouter variance documented in their Exacto announcement. KIRA ratios (+4.6pp from native tool calling) are self-reported — MEDIUM confidence. |

**Overall confidence:** HIGH

### Gaps to Address

- **KIRA technique score contribution ratios:** Self-reported by KIRA. Measure each technique's individual contribution independently in Phase 8 to confirm stacking is additive, not subadditive.
- **ForgeCode's exact context engine schema:** Inferred from public code and blog posts. Phase 7 must audit the actual fork implementation before reimplementing or extending.
- **Tauri IPC memory leak fix status:** Issues #12724 and #13133 open as of April 2026. Workarounds help but may not fully resolve at scale. Phase 9 research should check current upstream status.
- **Windows EV certificate automation path:** Azure Code Signing vs DigiCert KeyLocker exact enrollment and CI automation path needs Phase 1 validation.
- **TB2 official Harbor submission procedure:** Confirm held-out task subset strategy and "match official submission env exactly" requirement against current Harbor docs at Phase 1 or early Phase 2.

---

## Sources

### Primary (HIGH confidence — official docs or primary source code)
- [ForgeCode GitHub](https://github.com/antinomyhq/forgecode) — fork base; context engine; JSON schema hardening
- [ForgeCode blog: "Benchmarks Don't Matter — Until They Do (Part 2)"](https://forgecode.dev/blog/gpt-5-4-agent-improvements/) — JSON hardening detail (required-before-properties)
- [Terminus-KIRA repository](https://github.com/krafton-ai/KIRA) — four harness techniques; tool shapes
- [KRAFTON AI: How We Reached 74.8% on terminal-bench with Terminus-KIRA](https://krafton-ai.github.io/blog/terminus_kira_en/) — native tool calling, marker polling, multi-perspective verification
- [codex-rs architecture writeup](https://codex.danielvaughan.com/2026/03/28/codex-rs-rust-rewrite-architecture/) — workspace split, per-crate design, sandbox modules
- [Tauri v2 official docs](https://v2.tauri.app/) — IPC channels vs events, code signing, sidecar notarization, plugin APIs
- [Tauri issue #11992](https://github.com/tauri-apps/tauri/issues/11992) — sidecar/externalBin notarization breakage (confirmed open)
- [Tauri issues #12724, #13133, #852](https://github.com/tauri-apps/tauri/issues/) — IPC memory leak documentation
- [OpenRouter Exacto announcement](https://openrouter.ai/announcements/provider-variance-introducing-exacto) — tool-call variance telemetry; Exacto endpoints
- [OpenRouter tool calling docs](https://openrouter.ai/docs/guides/features/tool-calling) — streaming coercion, missing-arguments handling
- [Terminal-Bench 2.0 / Harbor](https://www.tbench.ai/news/announcement-2-0) — 89-task structure, Docker+pytest scoring, 5-run submission average

### Secondary (MEDIUM confidence — community synthesis, blog posts)
- [Claude Code session management (DeepWiki)](https://deepwiki.com/anthropics-claude/claude-code/2.3-session-management) — SQLite+JSONL hybrid pattern
- [Claudia GitHub](https://github.com/getAsterisk/claudia) — Tauri 2 + React reference implementation; session timeline pattern
- [OpenCovibe GitHub](https://github.com/AnyiWang/OpenCovibe) — tool-call inspector card pattern; file tracking panel
- [Claw Code crate architecture (DeepWiki)](https://deepwiki.com/instructkr/claw-code/2-rust-crate-architecture) — 9-crate workspace layout reference
- [Meta-Harness (Stanford IRIS Lab)](https://github.com/stanford-iris-lab/meta-harness-tbench2-artifact) — 76.4% ceiling reference; over-verification risk
- [Bean Kinney Korman legal analysis](https://www.beankinney.com/512000-lines-one-night-zero-permission-the-claude-code-leak-and-the-legal-crisis-of-ai-clean-rooms/) — claw-code clean-room legal risk
- [OpenRouter AI SDK tool calls (DeepWiki)](https://deepwiki.com/OpenRouterTeam/ai-sdk-provider/4.3-tool-calls-and-function-calling) — Gemini finish_reason: 'stop' with pending tool calls

### Tertiary (LOW confidence — single source, needs validation)
- SSE client comparison blog posts — `reqwest-eventsource` vs `eventsource-stream` choice; validate at Phase 2
- Cursor agent-sandbox blog / Pierce Freeman sandbox deep-dive — sandboxing approach lineage
- Windows ConPTY flag documentation in wezterm source — needs hands-on validation at Phase 4

---
*Research completed: 2026-04-19*
*Ready for roadmap: yes*
