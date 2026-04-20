# Requirements: Kay

**Defined:** 2026-04-19
**Core Value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop UI.

## v1 Requirements

Requirements for initial release. Each maps to a roadmap phase.

### Governance (GOV)

- [x] **GOV-01**: Fork of ForgeCode with upstream attribution in `NOTICE`, `README`, and crate `authors` preserving Apache-2.0 obligations
- [ ] **GOV-02**: Apache-2.0 `LICENSE` present at repo root; `NOTICE` lists all upstream copyright holders
- [ ] **GOV-03**: DCO (`Signed-off-by: Name <email>`) enforced on every commit via a GitHub Action that blocks unsigned PRs
- [ ] **GOV-04**: `CONTRIBUTING.md` documents DCO, clean-room attestation, code-style, and PR process
- [ ] **GOV-05**: All release tags are GPG- or SSH-signed; CI refuses to publish an unsigned tag
- [ ] **GOV-06**: `SECURITY.md` describes vulnerability reporting, private advisory flow, and response SLA
- [ ] **GOV-07**: Clean-room contributor attestation required — contributors confirm no exposure to leaked Claude Code source before first merge

### Workspace (WS)

- [ ] **WS-01**: Rust 2024 edition cargo workspace with `kay-core`, `kay-cli`, `kay-tauri`, `kay-sandbox-*` crates (mirrors codex-rs layout)
- [ ] **WS-02**: Workspace-level pinning of tokio 1.51 LTS, reqwest 0.13, rustls 0.23, serde_json, schemars
- [ ] **WS-03**: `cargo-deny` configured to block GPL/AGPL transitive deps and known-vulnerable crates
- [ ] **WS-04**: `cargo-audit` runs in CI on every PR and nightly
- [ ] **WS-05**: Workspace compiles clean on stable Rust with `--deny warnings` on macOS, Linux, and Windows

### Provider / HAL (PROV)

- [x] **PROV-01**: `Provider` trait supports chat completion + tool calling + streaming SSE with typed `AgentEvent` output
- [x] **PROV-02**: OpenRouter provider implementation using reqwest 0.13 + reqwest-eventsource + backon retry
- [x] **PROV-03**: API key auth via environment variable and config file — no OAuth
- [x] **PROV-04**: Strict model allowlist targeting OpenRouter Exacto endpoints (not "300+ models")
- [ ] **PROV-05**: Tolerant tool-call JSON parser handles OpenRouter provider variance (malformed strings, stringified args, partial tool deltas)
- [ ] **PROV-06**: Streaming token budget enforcement with per-session cost cap and hard abort
- [ ] **PROV-07**: Rate-limit / 429 / 503 retry with exponential backoff + jitter; user-visible retry events
- [ ] **PROV-08**: Provider errors surface typed `ProviderError` (not string) for diagnosis and retry decisions

### Tool Registry & KIRA Techniques (TOOL)

- [ ] **TOOL-01**: `Tool` trait with `name`, `description`, `input_schema`, `invoke` — object-safe, async, `Arc<dyn Tool>` map
- [ ] **TOOL-02**: `execute_commands` tool executes shell commands in the project-root sandbox
- [ ] **TOOL-03**: `task_complete` tool triggers multi-perspective verification before signaling completion
- [ ] **TOOL-04**: `image_read` tool reads base64-encoded terminal-state screenshots (multimodal)
- [ ] **TOOL-05**: Tool schemas emitted via schemars with the ForgeCode JSON-schema hardening post-process (required-before-properties, flattening, explicit truncation reminders)
- [ ] **TOOL-06**: Tool calls flow through the provider's native `tools` parameter — no ICL-style free-form parsing

### Shell & Marker Polling (SHELL)

- [ ] **SHELL-01**: `__CMDEND__<seq>__` marker-based polling; agent advances as soon as the marker is observed
- [ ] **SHELL-02**: `tokio::process` for non-PTY work; `portable-pty` fallback for TTY-requiring commands
- [ ] **SHELL-03**: Output streamed as `AgentEvent::ToolOutput` frames; no blocking reads
- [ ] **SHELL-04**: Hard timeout per command configurable; termination is clean (signal propagation + zombie reap)
- [ ] **SHELL-05**: Marker races with user-injected input are detected and recovered without agent-loop corruption

### Sandbox (SBX)

- [ ] **SBX-01**: `Sandbox` trait with per-OS implementations: macOS `sandbox-exec`, Linux `landlock` + `seccompiler`, Windows `Job Objects` + restricted token
- [ ] **SBX-02**: Default policy restricts writes to the project root; reads allowed within project + home config
- [ ] **SBX-03**: Network access gated by policy (default: allowed for HTTPS to configured provider host only)
- [ ] **SBX-04**: Sandbox escape attempts fail loudly in the agent trace, not silently

### Agent Loop (LOOP)

- [ ] **LOOP-01**: Event-driven loop via `tokio::select!` over input/model/tool/control channels
- [ ] **LOOP-02**: `AgentEvent` enum marked `#[non_exhaustive]`; frozen shape for UI consumers
- [ ] **LOOP-03**: Persona configuration (`forge` = write, `sage` = research, `muse` = plan) is data (YAML per-persona) — same code path, different prompt + tool filter + model. Persona names are inherited from ForgeCode to minimize launch-day drift; Kay is the brand/binary name, not a persona
- [ ] **LOOP-04**: Sage (read-only research) is callable as a sub-tool by `forge` and `muse`
- [ ] **LOOP-05**: Mandatory verification pass runs before `task_complete` returns success
- [ ] **LOOP-06**: Loop can be paused, resumed, and cancelled cleanly via control channel

### Session Store (SESS)

- [ ] **SESS-01**: JSONL transcript as source of truth; SQLite index for lookup/resume
- [ ] **SESS-02**: Pre-edit snapshots enable single-step rewind within a session
- [ ] **SESS-03**: Session resume by ID restores full transcript + cursor position
- [ ] **SESS-04**: `parent_session_id` column reserved in SQLite schema from v1 (for v2 multi-agent orchestration)
- [ ] **SESS-05**: Session export as self-contained JSONL for reproducibility and benchmark submission

### Context Engine (CTX)

- [ ] **CTX-01**: `tree-sitter` parses project files; function signatures and module boundaries extracted to SQLite
- [ ] **CTX-02**: Symbol store maps names, signatures, and file spans — not full file bodies
- [ ] **CTX-03**: Hybrid retrieval (structural lookup + similarity search via sqlite-vec) assembles prompt context
- [ ] **CTX-04**: Context-size budget enforced per turn; truncation is explicit, not silent
- [ ] **CTX-05**: ForgeCode's schema-hardening post-process applied consistently to all tool schemas in-context

### Verifier (VERIFY)

- [ ] **VERIFY-01**: Multi-perspective critics (test engineer + QA engineer + end-user) run before `task_complete` accepts completion
- [ ] **VERIFY-02**: Critic disagreement triggers targeted re-work loop with bounded retries (max N per turn)
- [ ] **VERIFY-03**: Verifier cost budget capped — turns off (with trace event) if the token cost exceeds the policy ceiling
- [ ] **VERIFY-04**: Verifier output surfaced as structured `AgentEvent::Verification` frames

### Tauri Desktop Shell (TAURI)

- [ ] **TAURI-01**: Tauri 2.x app (`kay-tauri` crate) with macOS, Windows, Linux build targets
- [ ] **TAURI-02**: Rust backend is merged into the main binary — no `externalBin` sidecar (notarization blocked by Tauri #11992)
- [ ] **TAURI-03**: `tauri-specta` v2 bindings generate TypeScript types for commands and channel payloads
- [ ] **TAURI-04**: `ipc::Channel<AgentEvent>` streams agent trace frames to the frontend with batching
- [ ] **TAURI-05**: Long-session memory-leak canary test runs 4 hours nightly on macOS and Linux (guards Tauri #12724 / #13133)
- [ ] **TAURI-06**: React 19 + TypeScript + Vite frontend; Monaco or CodeMirror for diff viewer

### Tauri UI Features (UI)

- [ ] **UI-01**: Session view shows live agent trace with tool-call timeline and token/cost meter
- [ ] **UI-02**: Multi-session manager: spawn, pause, resume, and fork sessions from the GUI
- [ ] **UI-03**: Project workspace picker + environment/key management UI
- [ ] **UI-04**: OpenRouter account binding (API key entry + model allowlist picker)
- [ ] **UI-05**: Structured session export (JSONL + metadata manifest) via a one-click action
- [ ] **UI-06**: Command-approval dialog (off by default for clean TB 2.0 runs; opt-in for first-time users)
- [ ] **UI-07**: Settings panel surfaces cost budgets, model allowlist, verifier policy, and sandbox policy

### CLI — Canonical Backend (CLI)

Kay CLI is the **canonical user-facing surface** and the contract GUI/TUI frontends consume. Rebrands + extends ForgeCode's `forge_main` (imported during Phase 1) — its interactive mode inherits completer, editor, banner, stream-renderer, and syntax highlighter from upstream.

- [ ] **CLI-01**: `kay` CLI preserves headless non-interactive mode for CI and TB 2.0 submission
- [ ] **CLI-02**: CLI supports session import/export and replay
- [ ] **CLI-03**: CLI exit codes reflect task success/failure/sandbox-violation cleanly
- [ ] **CLI-04**: `kay-cli` crate rebrands `forge_main`: renames the binary to `kay`, rewrites banners/help text to Kay, preserves all inherited interactive behaviors (completer, editor, stream-renderer, syntax highlighter) without regression against the `forgecode-parity-baseline` tag
- [ ] **CLI-05**: Structured-event output mode (`--events jsonl` or equivalent) streams every agent event to stdout in a stable JSONL contract consumed by both `kay-tui` and `kay-tauri`
- [ ] **CLI-06**: Standalone distribution — `cargo install kay` yields a fully functional agent with zero GUI/TUI dependencies; the CLI works over bare SSH without terminal capabilities beyond ANSI
- [ ] **CLI-07**: Interactive mode is a first-class experience (not just a fallback); new-user onboarding flow guides first agent run without requiring flags

### TUI — Full-Screen Terminal Frontend (TUI)

Kay TUI is a **full-screen ratatui frontend over the CLI contract**, for users who want a richer in-terminal experience than the default interactive CLI (e.g., multi-pane inspectors, sticky status bars, SSH-friendly navigation).

- [ ] **TUI-01**: `kay-tui` crate uses `ratatui` + `crossterm` — multi-pane layout with session list, active transcript, tool-call inspector, cost meter
- [ ] **TUI-02**: Consumes `kay-cli`'s structured-event JSONL stream (CLI-05) as its data source — no parallel agent-loop implementation
- [ ] **TUI-03**: Keyboard-first navigation; works over SSH and in low-bandwidth terminals (no mouse required)
- [ ] **TUI-04**: Installable standalone (`cargo install kay-tui`) or bundled with the desktop release; invocation via `kay tui` subcommand from the main CLI
- [ ] **TUI-05**: Session control parity with GUI — spawn, pause, resume, fork sessions via keyboard shortcuts

### Release & Distribution (REL)

- [ ] **REL-01**: Binary distribution matrix: macOS (arm64 + x64), Windows (x64), Linux (x64 + arm64 musl + glibc)
- [ ] **REL-02**: macOS notarization via Apple Developer ID on every main merge (not only release)
- [ ] **REL-03**: Windows Authenticode code signing via Azure Code Signing or equivalent
- [ ] **REL-04**: Linux builds shipped as AppImage + tar.gz with SHA attestations
- [ ] **REL-05**: `cargo install kay` publishes the headless CLI to crates.io
- [ ] **REL-06**: Tauri bundler produces `.app`, `.msi`, `.AppImage` signed artifacts with reproducible build metadata
- [ ] **REL-07**: `tauri-plugin-updater` uses minisign keypair committed to `tauri.conf.json` before the first release

### Evaluation (EVAL)

- [ ] **EVAL-01**: Parity gate — forked ForgeCode baseline reproduced (≥ 80% on TB 2.0 with a documented reference run) before any harness modifications merge to `main`
- [ ] **EVAL-02**: TB 2.0 harness runner wrapped in a reproducible script with pinned Docker images, seed, and model allowlist
- [ ] **EVAL-03**: Held-out task subset maintained locally (not submitted) to guard against benchmark overfitting
- [ ] **EVAL-04**: Parallel real-repo eval set runs nightly to detect regressions outside TB 2.0's Docker-scored shape
- [ ] **EVAL-05**: v1 acceptance gate — public TB 2.0 submission achieving >81.8% with a documented reference run, model pinned, full transcript archived

## v2 Requirements

Deferred to future release. Tracked but not in v1 roadmap.

### Wedges (WEDGE)

- **WEDGE-01**: Self-improving context (ACE — Agentic Context Engineering; ICLR 2026 paper; OpenCode issue #15456)
- **WEDGE-02**: Dynamic model routing per subtask (plan=Opus / code=Sonnet / verify=Haiku, learned cost-quality tradeoffs)
- **WEDGE-03**: Deep verification-first loop (multi-round critic debate, self-correction budget tuning)
- **WEDGE-04**: Re-enterable hierarchical multi-agent orchestration (addresses OpenCode #11012 + Claude Code Agent Teams)

### Providers (PROV-v2)

- **PROV-v2-01**: Direct Anthropic API with prompt caching and extended thinking
- **PROV-v2-02**: Direct OpenAI API with Responses API and Codex-aligned tool patterns
- **PROV-v2-03**: Direct Google Gemini API
- **PROV-v2-04**: Direct Groq API
- **PROV-v2-05**: Local models via Ollama / llama.cpp / LM Studio

### Surfaces (SURF-v2)

- **SURF-v2-01**: VS Code extension
- **SURF-v2-02**: JetBrains plugin
- **SURF-v2-03**: MCP server surface so Kay exposes its own tools to other agents

### Benchmarks (BENCH-v2)

- **BENCH-v2-01**: SWE-bench Verified submission
- **BENCH-v2-02**: MLE-bench submission
- **BENCH-v2-03**: Custom real-repo long-session benchmark

## Out of Scope

Explicitly excluded from v1 and v2 alike — documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Web dashboard / browser UI | Tauri covers the "not-a-terminal" surface. Web UI fragments effort without unlocking new users. |
| ~~TUI (terminal UI)~~ | **Reversed 2026-04-19** — TUI is now v1 (see §CLI and §TUI). `kay-tui` is a full-screen ratatui frontend over the CLI contract. |
| Plugin marketplace | OpenClaw's supply-chain incident (21K exposed instances) proves marketplaces shift security burden to solo maintainers. |
| Auto-memory across sessions at v1 | Claude Code's feature; without ACE it tends toward silent context drift. Lands with WEDGE-01. |
| Hooks (pre/post tool, session) | Claude Code's feature; hook surface is wide and distracts from the TB 2.0 target. Revisit if/when Kay has a stable harness + community. |
| 75+ model providers like OpenCode | Tool-calling variance destroys benchmark scores. Allowlist of known-good Exacto-grade endpoints is the inverse of OpenCode's bet. |
| Voice input | Codex CLI has this; off-strategy for v1. |
| Cerebras / WSE-3 specific optimizations | Codex CLI's 1000+ tok/sec path; locks us into one hardware vendor. |
| CLA for contributors | Switched to DCO based on pitfalls research (2026-04-19). CLA caused measurable contributor drop-off without meaningful legal gain. |

## Traceability

Every v1 requirement maps to exactly one phase. Populated by the gsd-roadmapper agent on 2026-04-19.

| Requirement | Phase | Status |
|-------------|-------|--------|
| GOV-01 | Phase 1 | Complete |
| GOV-02 | Phase 1 | Pending |
| GOV-03 | Phase 1 | Pending |
| GOV-04 | Phase 1 | Pending |
| GOV-05 | Phase 1 | Pending |
| GOV-06 | Phase 1 | Pending |
| GOV-07 | Phase 1 | Pending |
| WS-01 | Phase 1 | Pending |
| WS-02 | Phase 1 | Pending |
| WS-03 | Phase 1 | Pending |
| WS-04 | Phase 1 | Pending |
| WS-05 | Phase 1 | Pending |
| PROV-01 | Phase 2 (plan 02-08) | Complete |
| PROV-02 | Phase 2 (plan 02-08) | Complete |
| PROV-03 | Phase 2 (plan 02-07) | Complete |
| PROV-04 | Phase 2 (plan 02-07) | Complete |
| PROV-05 | Phase 2 | Pending |
| PROV-06 | Phase 2 | Pending |
| PROV-07 | Phase 2 | Pending |
| PROV-08 | Phase 2 | Pending |
| TOOL-01 | Phase 3 | Pending |
| TOOL-02 | Phase 3 | Pending |
| TOOL-03 | Phase 3 | Pending |
| TOOL-04 | Phase 3 | Pending |
| TOOL-05 | Phase 3 | Pending |
| TOOL-06 | Phase 3 | Pending |
| SHELL-01 | Phase 3 | Pending |
| SHELL-02 | Phase 3 | Pending |
| SHELL-03 | Phase 3 | Pending |
| SHELL-04 | Phase 3 | Pending |
| SHELL-05 | Phase 3 | Pending |
| SBX-01 | Phase 4 | Pending |
| SBX-02 | Phase 4 | Pending |
| SBX-03 | Phase 4 | Pending |
| SBX-04 | Phase 4 | Pending |
| LOOP-01 | Phase 5 | Pending |
| LOOP-02 | Phase 5 | Pending |
| LOOP-03 | Phase 5 | Pending |
| LOOP-04 | Phase 5 | Pending |
| LOOP-05 | Phase 5 | Pending |
| LOOP-06 | Phase 5 | Pending |
| CLI-01 | Phase 5 | Pending |
| CLI-03 | Phase 5 | Pending |
| SESS-01 | Phase 6 | Pending |
| SESS-02 | Phase 6 | Pending |
| SESS-03 | Phase 6 | Pending |
| SESS-04 | Phase 6 | Pending |
| SESS-05 | Phase 6 | Pending |
| CLI-02 | Phase 6 | Pending |
| CTX-01 | Phase 7 | Pending |
| CTX-02 | Phase 7 | Pending |
| CTX-03 | Phase 7 | Pending |
| CTX-04 | Phase 7 | Pending |
| CTX-05 | Phase 7 | Pending |
| VERIFY-01 | Phase 8 | Pending |
| VERIFY-02 | Phase 8 | Pending |
| VERIFY-03 | Phase 8 | Pending |
| VERIFY-04 | Phase 8 | Pending |
| TAURI-01 | Phase 9 | Pending |
| TAURI-02 | Phase 9 | Pending |
| TAURI-03 | Phase 9 | Pending |
| TAURI-04 | Phase 9 | Pending |
| TAURI-05 | Phase 9 | Pending |
| TAURI-06 | Phase 9 | Pending |
| UI-01 | Phase 9 | Pending |
| UI-02 | Phase 10 | Pending |
| UI-03 | Phase 10 | Pending |
| UI-04 | Phase 10 | Pending |
| UI-05 | Phase 10 | Pending |
| UI-06 | Phase 10 | Pending |
| UI-07 | Phase 10 | Pending |
| REL-01 | Phase 11 | Pending |
| REL-02 | Phase 11 | Pending |
| REL-03 | Phase 11 | Pending |
| REL-04 | Phase 11 | Pending |
| REL-05 | Phase 11 | Pending |
| REL-06 | Phase 11 | Pending |
| REL-07 | Phase 11 | Pending |
| EVAL-01 | Phase 1 | Pending |
| EVAL-02 | Phase 12 | Pending |
| EVAL-03 | Phase 12 | Pending |
| EVAL-04 | Phase 12 | Pending |
| EVAL-05 | Phase 12 | Pending |

**Coverage:**
- v1 requirements: 83 total (corrected from 77 after recounting the 15 categories: GOV 7 + WS 5 + PROV 8 + TOOL 6 + SHELL 5 + SBX 4 + LOOP 6 + SESS 5 + CTX 5 + VERIFY 4 + TAURI 6 + UI 7 + CLI 3 + REL 7 + EVAL 5 = 83)
- Mapped to phases: 83 ✓
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-19*
*Last updated: 2026-04-19 — traceability populated by gsd-roadmapper (83 requirements mapped across 12 phases)*
