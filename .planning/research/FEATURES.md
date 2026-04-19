# Feature Research

**Domain:** Open-source terminal + desktop AI coding agent (Kay v1 = ForgeCode parity + KIRA harness + Tauri UI + OpenRouter)
**Researched:** 2026-04-19
**Confidence:** HIGH — reference implementations are public (ForgeCode, KIRA, OpenCode, Claudia, claw-code) and authoritative writeups exist for each.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Any "serious" OSS coding agent in 2026 must ship all of the following at v1. Missing any item in this table means a user will bounce to ForgeCode / Claude Code / Codex CLI within the first 30 minutes. Dependencies are inherited from the reference agents Kay is forking and studying.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Agent loop (plan → act → observe → repeat)** | Core of every agent since 2024; this IS the product | MEDIUM | Inherited from ForgeCode fork; Kay just keeps the existing loop intact. |
| **Native LLM tool calling** | KIRA proved ICL parsing is a 10+ point benchmark regression; OpenRouter, Anthropic, OpenAI all speak it | MEDIUM | v1 MUST use `tools` parameter, not ICL response parsing. Core KIRA technique. |
| **Bash / shell tool** | The terminal agent's defining capability — all TB 2.0 tasks require it | MEDIUM | Needs subprocess sandboxing + the marker-based `__CMDEND__<seq>__` sentinel from KIRA for reliable completion polling. |
| **File ops: read / write / edit / patch** | Can't code without them; ForgeCode, Claude Code, Codex all expose identical shapes | MEDIUM | Unified diff patch is industry standard in 2026 (Codex, Claude Code, ForgeCode all use it). |
| **Ripgrep-backed code search** | Every top-tier agent ships it (WarpGrep v2 lift on Opus 4.6 was +2.1 pp on SWE-Bench Pro) | LOW | Bundle `rg` or shell out. |
| **Web fetch / search tool** | Users expect the agent to grab current docs; ForgeCode, Claude Code, Codex ship it | LOW | Single `fetch(url)` + optional search API is fine for v1. |
| **MCP client support** | Became universal standard in 2026; claw-code, OpenClaw, OpenCode, Claude Code all speak MCP | MEDIUM | Wire MCP client from day 1; even if no MCP servers ship with Kay, the protocol is a table-stakes integration surface. |
| **Semantic context engine (function signatures + module boundaries)** | ForgeCode's defining feature; raw-file dumping is a 30-point benchmark gap (OpenCode 51.7% vs ForgeCode 81.8%) | HIGH | Inherited from ForgeCode fork. DO NOT rewrite. |
| **Context compaction / auto-summarization** | Claude Code, Codex, ForgeCode all auto-compact when window fills | MEDIUM | Already in ForgeCode; preserve it. |
| **Multi-turn session (resume by ID)** | Users expect "close terminal, open later, resume" — Claude Code and Codex both ship this | MEDIUM | JSON-on-disk session store; session ID is the primary key. |
| **OpenRouter provider integration with API-key auth** | v1 constraint; 300+ models via one key | LOW | Uses OpenAI-compatible shape; OpenRouter supports `tools` param. `auto` and `required` tool modes both needed. |
| **Configuration: project / user / env scopes** | Claude Code, OpenCode, ForgeCode all layer configs; users expect per-project overrides | LOW | Config precedence: `.kay/config.toml` > `~/.kay/config.toml` > env. |
| **Error / retry with exponential backoff** | OpenRouter free tier is 20 req/min, 200 req/day — will 429 constantly on long tasks | LOW | Standard HTTP client retries + respect `Retry-After`. |
| **JSON schema hardening (field reordering, flattening, truncation reminders)** | ForgeCode's defining reliability trick; without it, Opus/GPT hallucinate arguments | MEDIUM | Inherited from ForgeCode fork. |
| **Mandatory verification pass before task-complete** | Every top-10 TB 2.0 harness enforces it; without it, false-completion rate spikes | MEDIUM | Required for score. Inherited from ForgeCode + hardened with KIRA's multi-perspective critics. |
| **Session export (JSON / markdown / HTML)** | ForgeCode, Codex, Claude Code all support it; required for benchmark submission reproducibility | LOW | TB 2.0 leaderboard requires a reference run transcript. |
| **Headless / CI mode** | Every OSS agent in 2026 supports it (ForgeCode, OpenCode, Codex CLI, Claude Code); required for CI and benchmark harness invocation | LOW | `kay run --prompt "..." --headless` path must not require the Tauri UI. |
| **Slash commands** | Unified standard across Claude Code, OpenCode, claw-code in 2026 | LOW | Minimum: `/help`, `/clear`, `/compact`, `/resume`, `/export`. |
| **Three built-in agents (write / research / plan)** | ForgeCode's `forge / sage / muse` trio — users coming from ForgeCode expect them | MEDIUM | Inherited. Kay renames to `kay / sage / muse` per PROJECT.md. |
| **Multimodal `image_read` tool** | KIRA's fourth harness technique; TB 2.0 has terminal-screenshot tasks where this matters | LOW | Base64 → model. OpenRouter supports image-capable models. |
| **Multi-perspective completion critics (test engineer + QA + end-user)** | KIRA's flagship technique; addresses the "model says 'yup, done' when it isn't" failure mode | MEDIUM | v1 ships the KIRA baseline. Deeper verification-first architecture is v2. |
| **Signed release binaries (GPG or SSH tags)** | Silver-Bullet#28 filed 2026-04-19 flagged this; OpenClaw's 21K exposed instances burned the community | LOW | Non-negotiable per PROJECT.md constraints. Ship from v0.0.1. |

### Differentiators (Competitive Advantage)

Features that set Kay apart from ForgeCode, OpenCode, Codex CLI, and claw-code. These come from either (a) inheriting the best of the reference implementations as a combined package, or (b) the Tauri desktop UI which no top-tier OSS agent ships.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Native Tauri 2 desktop app bundled with binary** | No top-tier OSS coding agent has a first-party GUI in 2026. OpenCode shipped a desktop app but it's a TUI-wrapper; Claudia is third-party. Kay is the only one shipping a first-party Tauri GUI bundled with the harness | HIGH | See "Desktop UI Features" section below. Single-binary distribution is a PROJECT.md constraint. |
| **KIRA-level harness on an Apache-2.0 OSS codebase** | ForgeCode is Apache-2.0 but KIRA's techniques (native tool calling, marker polling, multi-perspective critics, image_read) aren't merged upstream. Kay combines both for the first time | HIGH | This is the "score" wedge. Each KIRA technique is individually LOW-MEDIUM; the system integration is HIGH. |
| **First OSS agent > 81.8% on TB 2.0** | The v1 quality bar. ForgeCode holds #1; Kay ships the only OSS harness that beats them with a public reference run | HIGH | The measurable wedge. Everything else is narrative around this number. |
| **Single permissively-licensed binary across macOS / Windows / Linux (arm64 + x64)** | Claude Code is proprietary; claw-code's license origins are murky; OpenCode requires Go + Node toolchains; Codex CLI is OpenAI-tied | MEDIUM | Rust + Tauri both support the matrix. Build/release pipeline is the real work. |
| **CLA + signed tags from v0.0.1** | Claw-code hit 100K stars with legal ambiguity; OpenClaw had supply-chain incidents. Kay trades "move fast" for "audit-ready" | LOW | Governance investment, not code. |
| **Session fork from GUI (explore two paths in parallel)** | Claude Code supports `--fork-session` but from CLI only; Claudia has session versioning but no fork-branch exploration | MEDIUM | Tauri surface + reuse ForgeCode session store. High-perceived-value UX moment. |
| **Tool-call inspector with syntax-highlighted diffs** | OpenCovibe and Claudia ship this for Claude Code; Kay's version is first-party and tied to the agent's own session format | MEDIUM | Every tool call → inline card with args, output, diff. High educational value for users new to agent behavior. |
| **Live token / cost meter per session and per model** | agentsview and costgoat-style analytics are post-hoc; Kay shows live burn in the UI | LOW | OpenRouter exposes per-request usage; aggregate in-app. |
| **OpenRouter-only simplicity** | OpenCode's 75-provider matrix creates config complexity and maintenance burden. Kay picks one and does it well for v1 | LOW | Constrains the provider abstraction; trades flexibility for reliability. Revisit in v2. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem obvious but are either (a) explicitly v2, (b) attack-surface traps, or (c) feature-creep that hurts benchmark scores. Keep these OUT of v1.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Direct Anthropic / OpenAI / Gemini / Groq APIs** | "OpenRouter markup is X%" / latency concerns | Multiplies provider abstraction surface; each API has its own tool-call quirks; v1 scope creep | OpenRouter for v1; revisit direct APIs in v2 when harness is stable. (PROJECT.md constraint.) |
| **Local models (Ollama / LM Studio / llama.cpp)** | OpenCode's differentiator; privacy appeal | Irrelevant to beating ForgeCode on TB 2.0 (local models score ~40% max); adds 3 providers' worth of quirks | Defer to v2 "privacy" milestone. (PROJECT.md constraint.) |
| **IDE extensions (VS Code / JetBrains / Xcode)** | Codex and Claude Code ship them; "where my users live" | Four separate extension codebases; not what beats TB 2.0; Cursor/Copilot already own this surface | v3+ concern. Tauri app is the integration surface for v1. (PROJECT.md constraint.) |
| **Dynamic multi-model routing per subtask** | ForgeCode "use thinking model to plan, fast model to code" | One of the four post-v1 wedges; requires subtask-level cost/capability modeling | v1 = single user-selected model per session. v2 = routing wedge. (PROJECT.md constraint.) |
| **Self-improving context / ACE** | Claw-code's ClawMem shows appetite; emerging research area | Requires stable harness foundation; research-phase feature | v2 wedge. (PROJECT.md constraint.) |
| **Re-enterable hierarchical multi-agent orchestration** | Claude Code Agent Teams; claw-code ClawTeam; OpenCode #11012 | Requires session-persistence primitives ForgeCode doesn't have; hot area but early | v2 wedge. (PROJECT.md constraint.) |
| **Deep verification-first loop (beyond KIRA baseline)** | Meta-Harness 76.4% shows upside | KIRA baseline is enough to beat ForgeCode; deeper verification risks score regression from over-thinking | v2 wedge. KIRA-level is the v1 bar. (PROJECT.md constraint.) |
| **Web dashboard / browser UI** | "Tauri app + web UI is even better" | Fragments effort; Tauri already covers non-terminal surface | Tauri only. (PROJECT.md constraint.) |
| **TUI (terminal UI like OpenCode)** | "We're a terminal agent, we should have a TUI" | ForgeCode's CLI is already good; TUI is a third surface to maintain after CLI + Tauri | CLI + Tauri only. TUI revisit if users demand. (PROJECT.md constraint.) |
| **Plugin marketplace / third-party MCP catalog** | OpenClaw shipped this; users love extensibility | OpenClaw's early-2026 supply-chain incident: 21K exposed instances, malicious marketplace exploits. Non-trivial to secure | MCP client only (no catalog); users add servers manually in v1. Catalog is a v2+ trust problem. |
| **Automatic Git commits / push on task complete** | "Agent should commit its own work" | Supply-chain risk (OWASP AI Agent #4 — tool misuse); users lose review step; failure mode = force-pushed bad code | Agent emits diff; user runs `git commit` manually. Desktop UI can show the diff for review. |
| **Auto-install dependencies across languages** | Codex/Cursor do this | Magnifies hallucinated-dependency supply-chain attack surface | Agent proposes `npm install X`; user approves. No auto-pkg-install in v1. |
| **Hooks (pre/post tool)** | Claude Code + claw-code have them; users ask for them | Security risk (arbitrary shell exec on hook triggers); complicates benchmark reference runs (non-deterministic) | v2 feature. v1 = no hooks. Multi-perspective critics cover the "enforcement" use case. |
| **Auto-memory / persistent cross-session memory (ClawMem-style)** | Claw-code ecosystem made this popular | Prompt-injection and memory-poisoning attack surface (OWASP AI Agent 2026 top 10). Requires separate trust model | v2 feature. v1 uses in-session context only. |
| **IDE-style autocomplete / ghost text** | Copilot / Cursor surface | Different product category (inline-completion assistant vs agent); dilutes positioning | Out of scope forever. Kay is an agent, not a completion engine. |
| **Voice input (Codex CLI-style dictation)** | "170+ WPM prompt entry" | Non-core; distracts from score goal; needs OS-specific audio pipeline | v2+ consideration. Not v1. |
| **Supporting 75+ providers (OpenCode-style)** | "More flexibility is better" | Each provider is a maintenance tax; OpenCode's 51.7% TB 2.0 suggests breadth hurts depth | OpenRouter-only. Breadth through one well-tested integration. |
| **Background / long-running agents with web dashboard** | Codex App Server pattern | Needs hosted infrastructure; not a single-binary story; different product | Tauri desktop session view is the "watch it work" surface. No server. |
| **Skills / subagent catalog (Claude Code skills system)** | "Extensibility" | Secondary ecosystem to maintain; pre-v1 scope creep | v2 feature. Three built-in agents (kay/sage/muse) are v1. |
| **Browser / DOM automation** | "Full-stack agent" | Playwright/Puppeteer dependency; not a terminal-agent capability; expands attack surface | Out of scope. Bash + web fetch is enough. |
| **E2B-style remote sandbox execution** | "Safer than local" | Requires hosted infrastructure, non-trivial cost, outside single-binary story | Local subprocess sandbox is enough for v1. Revisit if security becomes a user ask. |

### Desktop UI Features (Kay's Distinguishing Surface)

This is the wedge no top-tier OSS agent occupies. Reference implementations studied: Claudia (AGPL Tauri GUI for Claude Code), OpenCovibe (Tauri v2 + Svelte for Claude Code / Codex), Commander (Tauri v2 multi-CLI orchestrator), agentsview (local session analytics). Each is third-party — Kay is the first first-party Tauri GUI.

| Feature | Why It Matters (What a Terminal Can't Show) | Complexity | Notes |
|---------|---------------------------------------------|------------|-------|
| **Session view with live agent trace** | Terminal shows the last 40 lines; desktop shows the full structured timeline with collapse/expand | MEDIUM | Stream from harness → IPC → renderer. Core surface. |
| **Tool-call inspector (inline cards per tool call)** | Terminal flattens everything into text; desktop renders args / output / diff as structured UI | MEDIUM | OpenCovibe is the reference pattern: each tool call = card with syntax-highlighted body. |
| **Syntax-highlighted diff viewer** | Terminal `+/-` diffs are hard to read for multi-file edits | MEDIUM | Monaco or shiki for highlighting; works for every tool call that mutates files. |
| **Live token / cost meter** | Terminal has no persistent UI surface for burn rate | LOW | Subscribe to OpenRouter usage events; show $/hour and tokens/min. |
| **Multi-session manager** | Terminal users juggle sessions via tmux; desktop shows them as a native tabs / list | MEDIUM | Spawn, pause, resume, archive, delete. Required by PROJECT.md active requirements. |
| **Session fork from GUI** | CLI `--fork-session` is powerful but invisible; GUI turns "try both paths" into a one-click operation | MEDIUM | Depends on multi-session manager. Differentiator vs Claudia (which has checkpointing but not fork exploration). |
| **Session timeline / checkpointing** | Scrub back to any point in a 2-hour agent run; see state at each decision | MEDIUM | Claudia's visual timeline pattern. Depends on ForgeCode session store (already exists). |
| **Project workspace (directory picker + env/key management)** | Terminal users juggle `.env`, `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY` by hand | MEDIUM | UI for binding an OpenRouter account to a project. Required by PROJECT.md active requirements. |
| **Log / export panel** | Terminal export is a command; desktop exposes it as a button with format picker | LOW | Markdown + JSON + HTML formats (HTML per replay.md / cass patterns). Required by PROJECT.md. |
| **File / working-tree view per session** | Shows which files the agent has touched, with jump-to-editor | MEDIUM | OpenCovibe's "file tracking panel" pattern. High educational value. |
| **Agent type selector (kay / sage / muse)** | CLI users pick via flag; GUI users expect a dropdown | LOW | Each session picks one of the three built-in agents. |
| **Model selector per session (OpenRouter catalog)** | Users want to pick the model; OpenRouter has 300+, so a filterable picker beats a flag | LOW | Pull OpenRouter models endpoint; filter by "tools" capability. |
| **Structured session export UI** | Required for TB 2.0 submission + debugging; desktop makes it a guided flow | LOW | Same JSON/MD export as CLI, but with copy-to-clipboard + "open folder" buttons. |
| **Verification-critics panel** | Shows test engineer / QA / user critic votes side-by-side during verification pass | MEDIUM | KIRA critics as first-class UI. Educational + helps users debug false completions. |
| **Terminal output viewer (when `image_read` fires)** | KIRA's `image_read` for terminal screenshots; the desktop surface is the natural place to show them | LOW | Base64 → `<img>`. Differentiator moment — users see what the agent "sees". |
| **Command approval dialog (optional safe mode)** | Terminal agents auto-run bash; desktop can gate destructive ops behind a dialog | MEDIUM | Off by default for benchmark runs; on by default for first-time users. Security posture that matches 2026 supply-chain concerns. |

---

## Feature Dependencies

```
Native LLM tool calling ──required by──> Bash / File ops / Web fetch / Image read
                         └──required by──> All three built-in agents (kay/sage/muse)

Marker-based command polling ──depends on──> Bash tool
                               └──required by──> Reliable multi-step tasks

Session store (ForgeCode, inherited) ──required by──> Multi-turn session / resume
                                      └──required by──> Session export
                                      └──required by──> Session fork
                                      └──required by──> Session timeline / checkpointing
                                      └──required by──> Multi-session manager (GUI)

Session fork ──depends on──> Multi-session manager

Semantic context engine ──required for──> TB 2.0 score target (without it, 30-pt gap)

JSON schema hardening ──required for──> Reliable tool-arg generation → TB 2.0 score

Verification pass (baseline) ──depends on──> Native tool calling
                               └──required for──> TB 2.0 score target
                               └──enhanced by──> Multi-perspective critics (KIRA)

Multi-perspective critics ──depends on──> Verification pass (baseline)
                           └──surfaced by──> Verification-critics panel (GUI)

Tauri desktop app ──depends on──> Harness IPC surface (new work)
                   └──required by──> All "Desktop UI Features" rows

Tool-call inspector ──depends on──> Tauri app + structured session events
Syntax-highlighted diff ──depends on──> Tool-call inspector + file-ops tool output shape
Verification-critics panel ──depends on──> Multi-perspective critics + Tauri app
Token / cost meter ──depends on──> OpenRouter usage events + Tauri app

OpenRouter provider ──required by──> All LLM calls in v1
                    └──required for──> Tool-calling-capable model filter in UI

Headless CLI mode ──preserves──> CI / benchmark harness invocation (TB 2.0 submission path)
                   └──independent of──> Tauri app (additive)

Signed release tags ──depends on──> Release pipeline (not code)
                    └──required by──> v1 launch (PROJECT.md constraint)

CLA ──required by──> First external PR merge
     └──required for──> Apache-2.0 + future relicensing path
```

### Dependency Notes

- **Native tool calling is the choke point.** Everything that "acts" — bash, file ops, image_read, web fetch — passes through it. KIRA's finding that replacing ICL parsing with native tool calling alone lifted their score materially is the single most important harness decision.
- **Session store from ForgeCode is load-bearing.** Five of the desktop UI features depend on it. Don't rewrite; preserve it from the fork.
- **Verification pass has two tiers.** v1 ships the KIRA baseline (test engineer + QA + end-user critics). The "deeper verification-first architecture" is explicitly v2 per PROJECT.md — resisting this scope creep is important because over-verification can *reduce* benchmark scores (Meta-Harness 76.4% was a ceiling, not a floor, and came with a complex setup).
- **Tauri app is additive, not mandatory.** The headless CLI path must remain first-class for TB 2.0 submission (benchmark harnesses don't open windows) and CI use. Every desktop feature row has a terminal equivalent behind it.
- **Semantic context engine is the biggest "don't touch" inheritance.** The 30-point gap between OpenCode (51.7%) and ForgeCode (81.8%) on the same model class is almost entirely context-engine driven. Preserve it from the ForgeCode fork; don't optimize.

---

## MVP Definition

### Launch With (v1)

Exactly the PROJECT.md `Active` scope. Nothing more.

**Core harness (ForgeCode parity):**
- [ ] Rust workspace forked from ForgeCode with attribution — required for legal clarity
- [ ] Three specialized agents (kay / sage / muse) — inherited; table-stakes
- [ ] Semantic context engine — inherited; 30-point benchmark moat
- [ ] OpenRouter provider with API-key auth — v1 provider constraint
- [ ] Bash tool with sandboxed subprocess — core tool; required for TB 2.0
- [ ] Mandatory verification pass — required for score target
- [ ] JSON schema hardening — ForgeCode's reliability moat

**KIRA harness techniques:**
- [ ] Native tool calling via `tools` param — biggest single-technique score lift
- [ ] Marker-based command-completion polling — eliminates stuck commands
- [ ] Multi-perspective critics (test engineer + QA + end-user) — eliminates false completions
- [ ] Multimodal `image_read` — required for terminal-screenshot tasks

**Tauri desktop UI:**
- [ ] Tauri 2.x app bundled with binary (macOS + Windows + Linux) — the distinguishing surface
- [ ] Session view: live trace + tool-call timeline + token/cost meter — core GUI surface
- [ ] Multi-session manager — required for "not just another fork"
- [ ] Project workspace (directory + env/key + OpenRouter binding) — onboarding surface
- [ ] Log / export (MD + JSON + HTML) — required for TB 2.0 submission
- [ ] Headless CLI mode preserved — required for CI / benchmark

**Governance & distribution:**
- [ ] Apache-2.0 + CLA — license constraint
- [ ] Signed tags (GPG or SSH) from v0.0.1 — Silver-Bullet#28 lesson
- [ ] Binaries for macOS arm64/x64, Windows x64, Linux x64/arm64 — distribution constraint
- [ ] `cargo install kay` on crates.io — distribution channel
- [ ] TB 2.0 submission > 81.8% with documented reference run — the acceptance gate

### Add After Validation (v1.x)

Features to add once v1 score is validated and the Tauri UI has real users.

- [ ] **Session fork from GUI** — trigger: users ask "how do I try two approaches." High-perceived-value UX but not required for score or launch.
- [ ] **Verification-critics panel** — trigger: support-load on "why did it say done when it wasn't." Surface critics as UI.
- [ ] **File / working-tree panel per session** — trigger: users ask "what did the agent touch."
- [ ] **Session timeline / checkpoint scrubber** — trigger: users ask to replay long sessions. Claudia pattern.
- [ ] **Command approval dialog (safe mode)** — trigger: first supply-chain scare in the ecosystem. Security posture upgrade.
- [ ] **Syntax-highlighted diff in tool-call inspector** — trigger: once the basic inspector is used; polish.
- [ ] **Slash-command library** — trigger: users asking for more than `/help /clear /compact /resume /export`.

### Future Consideration (v2+)

The four wedges explicitly deferred by PROJECT.md, plus the anti-features that might graduate.

- [ ] **Self-improving context (ACE wedge)** — requires stable v1 harness foundation; research-phase feature.
- [ ] **Dynamic multi-model routing per subtask (routing wedge)** — requires per-subtask cost/capability modeling.
- [ ] **Deep verification-first architecture (verification wedge)** — beyond KIRA baseline; requires care to avoid score regression.
- [ ] **Re-enterable hierarchical multi-agent orchestration (orchestration wedge)** — addresses OpenCode #11012; needs session-persistence primitives.
- [ ] **Direct Anthropic / OpenAI / Gemini / Groq APIs** — revisit once harness is stable and OpenRouter edge cases are known.
- [ ] **Local model support (Ollama / LM Studio / llama.cpp)** — v2 "privacy" milestone per PROJECT.md.
- [ ] **Hooks (pre/post tool)** — once a defensible security story exists.
- [ ] **Auto-memory / ClawMem-style cross-session memory** — once a defensible prompt-injection story exists.
- [ ] **IDE extensions (VS Code / JetBrains / Xcode)** — v3+ consideration per PROJECT.md; not where Kay's wedge is.
- [ ] **MCP server catalog / marketplace** — requires OpenClaw-scale trust model; incremental after MCP client usage is proven.

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Native tool calling | HIGH | MEDIUM | P1 |
| Bash tool + marker polling | HIGH | MEDIUM | P1 |
| File read/write/edit/patch | HIGH | MEDIUM | P1 |
| Semantic context engine (inherit) | HIGH | LOW (inherited) | P1 |
| JSON schema hardening (inherit) | HIGH | LOW (inherited) | P1 |
| OpenRouter provider | HIGH | LOW | P1 |
| Three agents kay/sage/muse (inherit) | HIGH | LOW (inherited) | P1 |
| Verification pass + KIRA critics | HIGH | MEDIUM | P1 |
| Multimodal image_read | MEDIUM | LOW | P1 |
| Session resume + export | HIGH | LOW | P1 |
| Headless CLI mode | HIGH | LOW | P1 |
| Tauri app shell | HIGH | HIGH | P1 |
| Session view (live trace) | HIGH | MEDIUM | P1 |
| Tool-call inspector | HIGH | MEDIUM | P1 |
| Multi-session manager | HIGH | MEDIUM | P1 |
| Project workspace (GUI) | HIGH | MEDIUM | P1 |
| Token / cost meter | MEDIUM | LOW | P1 |
| Export panel (GUI) | MEDIUM | LOW | P1 |
| Signed release tags | HIGH | LOW | P1 |
| Cross-platform binaries | HIGH | MEDIUM | P1 |
| TB 2.0 > 81.8% reference run | HIGH | HIGH | P1 |
| Session fork from GUI | MEDIUM | MEDIUM | P2 |
| Syntax-highlighted diff viewer | MEDIUM | MEDIUM | P2 |
| Verification-critics panel | MEDIUM | MEDIUM | P2 |
| File / working-tree panel | MEDIUM | MEDIUM | P2 |
| Session timeline / checkpointing | MEDIUM | MEDIUM | P2 |
| Command approval dialog | MEDIUM | MEDIUM | P2 |
| Slash-command library | LOW | LOW | P2 |
| MCP client support | MEDIUM | MEDIUM | P2 |
| ACE / self-improving context | HIGH | HIGH | P3 |
| Dynamic model routing | MEDIUM | HIGH | P3 |
| Multi-agent orchestration | MEDIUM | HIGH | P3 |
| Deep verification-first loop | MEDIUM | HIGH | P3 |
| Direct provider APIs | LOW | MEDIUM | P3 |
| Local model support | MEDIUM | MEDIUM | P3 |
| Hooks | LOW | MEDIUM | P3 |
| Auto-memory (ClawMem-style) | MEDIUM | HIGH | P3 |
| IDE extensions | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1 launch
- P2: v1.x — add once v1 is validated
- P3: v2+ — explicitly deferred per PROJECT.md

**Note on MCP:** MCP client at P2 (not P1) is a judgment call. It's a "table stakes integration surface" per the ecosystem but NOT in PROJECT.md's Active requirements. Treat as a v1.x add-on unless the phase-planning process surfaces TB 2.0 tasks that require it.

---

## Competitor Feature Analysis

| Feature | ForgeCode | Claude Code | Codex CLI | OpenCode | claw-code | **Kay v1** |
|---------|-----------|-------------|-----------|----------|-----------|------------|
| Agent loop | Yes | Yes | Yes | Yes | Yes | **Yes (inherit)** |
| Native tool calling | Yes | Yes | Yes | Yes | Yes | **Yes (KIRA upgrade)** |
| Semantic context engine | Yes (flagship) | Partial | Partial | Limited | Partial | **Yes (inherit)** |
| JSON schema hardening | Yes (flagship) | Proprietary | Proprietary | Limited | Partial | **Yes (inherit)** |
| Multi-perspective critics | No | No | No | No | No | **Yes (KIRA)** |
| Marker-based polling | No | No | No | No | No | **Yes (KIRA)** |
| Multimodal image_read | No | Yes | Yes | Partial | Yes | **Yes (KIRA)** |
| Three-agent split | Yes (flagship) | Subagents | No | Specialized agents | ClawTeam subagents | **Yes (inherit)** |
| Hooks | No | Yes (flagship) | No | Limited | Yes | **No (v2)** |
| Auto-memory | No | Yes (`CLAUDE.md`) | Partial | No | Yes (ClawMem) | **No (v2)** |
| Checkpointing | No | Yes (flagship) | No | No | Yes (worktree) | **No (v1.x)** |
| Agent Teams / orchestration | No | Yes (flagship) | Background agents | Limited | Yes (ClawTeam) | **No (v2)** |
| Provider count | 300+ via OpenRouter | Anthropic direct | OpenAI direct | 75+ (models.dev) | Multi | **OpenRouter only** |
| Local models | No | No | No | Yes (flagship) | Some | **No (v2)** |
| MCP client | Yes | Yes | Yes | Yes | Yes | **v1.x (P2)** |
| Slash commands | Yes | Yes (flagship) | Yes | Yes | Yes | **Yes (minimal set)** |
| Session resume | Yes | Yes | Yes | Yes | Yes | **Yes (inherit)** |
| Session fork (CLI) | No | Yes (`--fork-session`) | No | No | Partial | **v1.x (P2)** |
| Session export | Yes | Yes | Yes | Yes (share links) | Yes | **Yes** |
| TUI | Basic CLI | Terminal | Terminal | Yes (flagship — Bubble Tea) | CLI | **No (CLI only)** |
| Native desktop GUI | No | No (Claudia is 3rd-party) | Desktop app (closed-source shell) | Desktop app (TUI wrapper) | No | **Yes (Tauri, first-party)** |
| IDE extensions | No | No (VS Code, via MCP) | Yes (VS Code / JetBrains / Xcode / Eclipse) | VS Code / Cursor / Windsurf | Some | **No (v3+)** |
| Voice input | No | No | Yes | No | No | **No (v2+)** |
| Single binary | Yes | Yes | Yes | Partial (Go + Node) | Yes | **Yes** |
| License | Apache-2.0 | Proprietary | Open source | Open source | Clean-room (AGPL/unclear) | **Apache-2.0 + CLA** |
| Signed releases | Partial | N/A (cloud) | Partial | Partial | Partial | **Yes from v0.0.1** |
| TB 2.0 score | 81.8% (#1) | N/A (not submitted) | N/A | 51.7% (#51) | N/A | **Target > 81.8%** |

**Key observations:**
1. **Desktop GUI is the unclaimed wedge.** Every top-tier agent is terminal-first; Claudia/OpenCovibe/Commander prove appetite for GUIs but are all third-party wrappers.
2. **No agent has ever shipped KIRA's four techniques on top of ForgeCode's harness.** That's Kay's score wedge.
3. **OpenCode (51.7%) shows that provider breadth hurts depth.** OpenRouter-only trades breadth for reliability — the right v1 call.
4. **Hooks / Agent Teams / ACE / Memory are where the frontier is moving.** Kay deliberately ships without them in v1 to keep the score bar achievable; they're the v2 wedges.
5. **License clarity is a differentiator.** Apache-2.0 + CLA + signed tags from v0.0.1 positions Kay as the auditable alternative to claw-code's ambiguous origin and OpenClaw's early security incidents.

---

## Sources

### ForgeCode
- [ForgeCode — World's #1 Coding Harness](https://forgecode.dev/) — flagship positioning, 81.8% TB 2.0
- [GitHub: antinomyhq/forgecode](https://github.com/antinomyhq/forgecode) — three agents, 300+ models via OpenRouter
- [ForgeCode Review — terminal-native AI coding agent](https://aicoolies.com/reviews/forgecode-review) — context engine details
- [ForgeCode vs Claude Code](https://dev.to/liran_baba/forgecode-vs-claude-code-which-ai-coding-agent-actually-wins-36c)

### Terminus-KIRA
- [GitHub: krafton-ai/KIRA](https://github.com/krafton-ai/KIRA) — reference implementation
- [Terminus-KIRA — KRAFTON AI](https://www.krafton.ai/blog/posts/2026-02-20-terminus_kira/terminus-en.html) — 74.8% TB 2.0, four techniques writeup
- [How We Reached 74.8% on terminal-bench with Terminus-KIRA](https://krafton-ai.github.io/blog/terminus_kira_en/) — technical details on native tool calling, multi-perspective verification
- [Meta-Harness (Stanford IRIS Lab) — 76.4% TB 2.0 with Opus 4.6](https://github.com/stanford-iris-lab/meta-harness-tbench2-artifact) — upper-bound reference

### Claude Code
- [Orchestrate teams of Claude Code sessions — Claude Code Docs](https://code.claude.com/docs/en/agent-teams) — Agent Teams v2.1.32+
- [Enabling Claude Code to work more autonomously — Anthropic](https://www.anthropic.com/news/enabling-claude-code-to-work-more-autonomously) — checkpointing announcement
- [Claude Code Hooks: All 12 Events (2026) — Pixelmojo](https://www.pixelmojo.io/blogs/claude-code-hooks-production-quality-ci-cd-patterns) — 12 hook events
- [Session Management: Resume, Fork, and Recovery — Agent Factory](https://agentfactory.panaversity.org/docs/General-Agents-Foundations/claude-code-teams-cicd/session-management-resume-fork-recovery) — `--fork-session` pattern
- [Understanding Claude Code's Full Stack — alexop.dev](https://alexop.dev/posts/understanding-claude-code-full-stack/) — MCP / skills / subagents / hooks

### OpenCode
- [OpenCode — opencode.ai](https://opencode.ai/) — privacy, TUI, 75+ providers
- [OpenCode TUI docs](https://opencode.ai/docs/tui/) — Bubble Tea TUI
- [OpenCode Deep Dive 2026 — sanj.dev](https://sanj.dev/post/opencode-deep-dive-2026) — provider-agnostic positioning

### Codex CLI
- [Codex CLI — OpenAI Developers](https://developers.openai.com/codex/cli) — Rust CLI
- [Codex CLI Features](https://developers.openai.com/codex/cli/features) — tool set
- [GPT-5.2-Codex now in Visual Studio, JetBrains, Xcode, Eclipse](https://github.blog/changelog/2026-01-26-gpt-5-2-codex-is-now-available-in-visual-studio-jetbrains-ides-xcode-and-eclipse/) — IDE breadth
- [OpenAI Codex App Server Architecture — InfoQ](https://www.infoq.com/news/2026/02/opanai-codex-app-server/) — unified surface

### Claw Code / OpenClaw / Clean-room rewrites
- [Claw Code — claw-code.codes](https://claw-code.codes/) — Rust + Python clean-room
- [GitHub: HKUDS/ClawTeam](https://github.com/HKUDS/ClawTeam) — multi-agent orchestration
- [GitHub: yoloshii/ClawMem](https://github.com/yoloshii/ClawMem) — auto-memory reference pattern
- [Hooks — OpenClaw docs](https://docs.openclaw.ai/automation/hooks) — hook chain architecture
- [What Claw Code Reveals About AI Coding Agent — ToLearn Blog](https://tolearn.blog/blog/2026-04-02-claw-code-ai-coding-agent-architecture) — crate architecture

### Desktop UI References
- [Claudia — claudia.so](https://claudia.so) — Tauri 2 + React + Rust GUI for Claude Code (checkpointing, timeline)
- [GitHub: getAsterisk/claudia](https://github.com/getAsterisk/claudia) — session versioning, custom agents, CLAUDE.md editor
- [GitHub: AnyiWang/OpenCovibe](https://github.com/AnyiWang/OpenCovibe) — Tauri v2 + Svelte 5 for Claude Code / Codex, tool activity timeline, file tracking panel
- [GitHub: autohandai/commander](https://github.com/autohandai/commander) — Tauri v2 multi-CLI orchestrator with Git DAG visualization
- [GitHub: wesm/agentsview](https://github.com/wesm/agentsview) — local-first session analytics across 14+ agents
- [Replay MCP — replay.io](https://www.replay.io/) — time-travel debugger; session export pattern
- [Cursor vs GitHub Copilot vs Continue — DEV Community](https://dev.to/synsun/cursor-vs-github-copilot-vs-continue-ai-code-editor-showdown-2026-2h89) — IDE-adjacent UI patterns

### OpenRouter
- [OpenRouter API Rate Limits](https://openrouter.ai/docs/api/reference/limits) — 20 req/min, 200 req/day on free tier
- [OpenRouter API Parameters](https://openrouter.ai/docs/api/reference/parameters) — tool calling, `auto` vs `required` mode, parallel function calling
- [OpenRouter Models](https://openrouter.ai/docs/guides/overview/models) — 300+ models, filter by tool support

### Security / Anti-features research
- [Coding Agents Widen Your Supply Chain Attack Surface — Security Boulevard](https://securityboulevard.com/2026/03/coding-agents-widen-your-supply-chain-attack-surface/) — 2026 attack surface
- [AI Agent Security in 2026: Prompt Injection, Memory Poisoning, and OWASP Top 10](https://swarmsignal.net/ai-agent-security-2026/) — five attack surfaces
- [AI Coding Agents Are Insider Threats — Botmonster Tech](https://botmonster.com/posts/ai-coding-agent-insider-threat-prompt-injection-mcp-exploits/) — MCP / supply-chain risks
- [AI Agent Security Risks 2026: MCP, OpenClaw & Supply Chain](https://blog.cyberdesserts.com/ai-agent-security-risks/) — OpenClaw 21K exposed instances incident

### Benchmark / ecosystem context
- [AI Coding Benchmarks 2026 — Morph](https://www.morphllm.com/ai-coding-benchmarks-2026) — SEAL vs best-agent 11-point gap
- [We Tested 15 AI Coding Agents (2026) — Morph](https://www.morphllm.com/ai-coding-agent) — WarpGrep v2 subagent lift on Opus 4.6
- [The Complete Guide to Agentic Coding Frameworks in 2026 — Ralph Wiggum Blog](https://ralphwiggum.org/blog/agentic-coding-frameworks-guide)

---

*Feature research for: open-source AI coding agent Kay v1 — ForgeCode parity + KIRA harness + Tauri UI + OpenRouter*
*Researched: 2026-04-19*
