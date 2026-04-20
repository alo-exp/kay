# Roadmap: Kay

**Created:** 2026-04-19
**Granularity:** fine (13 phases — 12 integer + 1 decimal insertion)
**Core Value:** Beat ForgeCode on Terminal-Bench 2.0 (>81.8%) as the first OSS agent that pairs a top-10 harness with a native desktop GUI, a full-screen TUI, and a first-class standalone CLI — three frontends over one core.

## Overview

Kay is a benchmark-first product — if it does not beat ForgeCode on TB 2.0, it has no reason to exist, and if the three frontends (CLI + TUI + Tauri GUI) do not ship, it is just another ForgeCode fork. The journey starts with an unusually heavy Phase 1 that stacks governance, workspace scaffolding, code-signing enrollment, and a non-negotiable ForgeCode parity gate (the forked harness must hit >=80% on TB 2.0 unmodified before any harness changes merge). From there, the harness is built bottom-up: provider HAL and tolerant JSON parser (Phase 2), tool registry + KIRA tools (Phase 3), per-OS sandbox (Phase 4), agent loop + canonical CLI (Phase 5 — rebrands `forge_main` and establishes the structured-event JSONL contract). Session persistence (Phase 6) and the context engine (Phase 7) run in parallel once the agent loop stabilizes. Multi-perspective verification (Phase 8) closes the KIRA harness. The Tauri desktop shell (Phase 9) and the ratatui TUI frontend (Phase 9.5, inserted 2026-04-19) consume the same CLI event contract — no parallel agent-loop implementations; the multi-session manager (Phase 10) puts project/session/settings control into both GUI and TUI. Cross-platform hardening + release pipeline (Phase 11) produces signed binaries + `cargo install kay` standalone distribution on all five targets. Phase 12 is the acceptance gate: public TB 2.0 submission at >81.8% with a documented reference run, paired with a real-repo eval to guard against benchmark overfitting.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Fork, Governance, Infrastructure** - Fork ForgeCode cleanly; set up Apache-2.0 + DCO; enroll code-signing; workspace scaffold; parity-gate the unmodified fork on TB 2.0.
- [ ] **Phase 2: Provider HAL + Tolerant JSON Parser** - OpenRouter streaming client with tool-call reassembly, typed error taxonomy, and a two-pass tolerant parser for provider variance.
- [ ] **Phase 2.5: kay-core sub-crate split** *(INSERTED 2026-04-20)* - Structural fix for the mono-crate approach discovered during Phase 2 execution. ForgeCode's imported source was 23 separate crates; forcing them into one `kay-core` crate breaks proc-macro self-reference, `include_str!` relative paths, trait object-safety, and visibility semantics (1323 residual errors after plan 02-05's mechanical rewrite). Option (c) from D-01 analysis: promote each `forge_*` subtree to its own workspace member crate preserving ForgeCode's original layout. Unblocks Phase 2 plans 02-06..02-10.
- [ ] **Phase 3: Tool Registry + KIRA Core Tools** - Object-safe `Tool` trait, native tool-calling path, `execute_commands` (marker polling), `task_complete`, `image_read`, with hardened schemas.
- [ ] **Phase 4: Sandbox (All Three Platforms)** - Per-OS sandbox: macOS `sandbox-exec`, Linux Landlock+seccomp, Windows Job Objects + restricted token.
- [ ] **Phase 5: Agent Loop + Canonical CLI** - `tokio::select!` loop, frozen `AgentEvent` shape, YAML personas (forge/sage/muse — inherited from ForgeCode), mandatory verification gate, rebranded `forge_main` → `kay-cli` with structured-event JSONL stream (the contract GUI and TUI frontends consume).
- [ ] **Phase 6: Session Store + Transcript** - JSONL source-of-truth transcripts + SQLite index, resume/fork, pre-edit snapshots, self-contained session export.
- [ ] **Phase 7: Context Engine** - tree-sitter symbol store + SQLite hybrid retrieval, explicit context-size budget, ForgeCode schema hardening applied in-context.
- [ ] **Phase 8: Multi-Perspective Verification (KIRA Critics)** - Test engineer + QA engineer + end-user critics with confidence-gated firing, bounded re-work, and cost ceilings.
- [ ] **Phase 9: Tauri Desktop Shell** - Tauri 2.x app with specta-typed IPC, `Channel<AgentEvent>` streaming, session view with tool-call timeline + token/cost meter; 4h memory canary. Desktop GUI frontends the `kay-cli` event contract.
- [ ] **Phase 9.5: TUI Frontend (ratatui)** *(INSERTED 2026-04-19)* - Full-screen ratatui terminal UI consuming the same `kay-cli` JSONL event stream as the Tauri GUI. Multi-pane layout, keyboard-first, SSH-friendly.
- [ ] **Phase 10: Multi-Session Manager + Project Settings** - Spawn/pause/resume/fork sessions from GUI and TUI, project workspace + keyring binding, model allowlist picker, command-approval dialog, settings panel.
- [ ] **Phase 11: Cross-Platform Hardening + Release Pipeline** - Signed + notarized bundles for macOS (arm64/x64), Windows (x64), Linux (x64/arm64); `cargo install kay` (standalone CLI); `cargo install kay-tui` (TUI); minisign updater for desktop bundle.
- [ ] **Phase 12: Terminal-Bench 2.0 Submission + v1 Hardening** - Reproducible Harbor runner, held-out task subset, parallel real-repo eval, official >81.8% TB 2.0 submission with archived reference run.

## Phase Details

### Phase 1: Fork, Governance, Infrastructure

**Goal**: Kay exists as a clean, Apache-2.0-compliant ForgeCode fork on signed infrastructure with DCO-enforced contribution, and the unmodified fork reproduces ForgeCode's TB 2.0 baseline before any harness change is allowed to merge.
**Depends on**: Nothing (first phase)
**Requirements**: GOV-01, GOV-02, GOV-03, GOV-04, GOV-05, GOV-06, GOV-07, WS-01, WS-02, WS-03, WS-04, WS-05, EVAL-01
**Success Criteria** (what must be TRUE):
  1. Anyone can clone `kay/` and see ForgeCode attribution in `NOTICE`, `README`, and crate `authors`, with Apache-2.0 `LICENSE` at repo root.
  2. A PR without `Signed-off-by` on every commit is automatically blocked by CI; `CONTRIBUTING.md` documents DCO, clean-room attestation, and PR process; `SECURITY.md` publishes the private-advisory flow.
  3. A maintainer cutting a release tag without GPG/SSH signature is rejected by the release workflow.
  4. `cargo check --workspace --deny warnings` passes on macOS, Linux, and Windows; `cargo-deny` and `cargo-audit` run on every PR and reject GPL/AGPL or known-vulnerable transitive deps.
  5. The forked ForgeCode baseline, run unmodified on the Harbor harness, reproduces >=80% on TB 2.0 with a documented, archived reference run, and this gate is enforced in CI before any harness modification merges to `main`.
**Plans**: 6 plans

Plans:
- [x] 01-01-PLAN.md — Workspace scaffold: Cargo.toml, rust-toolchain.toml, 7-crate skeleton (WS-01, WS-02, WS-05)
- [x] 01-02-PLAN.md — Governance files: LICENSE, NOTICE, README Acknowledgments, CONTRIBUTING (DCO + clean-room), SECURITY (GOV-01, GOV-02, GOV-04, GOV-06, GOV-07)
- [x] 01-03-PLAN.md — ForgeCode import: clone at SHA, copy into kay-core, single import commit, tag forgecode-parity-baseline (unsigned per D-OP-04) (GOV-01)
- [ ] 01-04-PLAN.md — Supply-chain gates: deny.toml, nightly audit.yml via rustsec/audit-check@v2.0.0 (WS-03, WS-04, WS-05)
- [ ] 01-05-PLAN.md — DCO + signed-tag gate verification: confirm existing ci.yml jobs; apply Pitfall 6 tag-gate if: hardening; ship governance-invariant checker (GOV-03, GOV-05, WS-05)
- [ ] 01-06-PLAN.md — Parity-gate scaffold: kay-cli eval tb2 --dry-run shim, ci.yml parity-gate job (workflow_dispatch only), PARITY-DEFERRED.md + manifest-schema.json (EVAL-01 scaffold-only per user amendment)
**UI hint**: no

### Phase 2: Provider HAL + Tolerant JSON Parser

**Goal**: Any agent turn can stream chat completions and tool calls through OpenRouter with typed events, tolerate provider JSON variance without panicking, recover from transient rate limits, and enforce a per-session cost cap.
**Depends on**: Phase 1
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07, PROV-08
**Success Criteria** (what must be TRUE):
  1. A caller can stream a chat completion from OpenRouter via a mock and a real client and receive a typed `AgentEvent` stream carrying text, reassembled tool calls, and usage frames.
  2. The user authenticates with an API key supplied via environment variable or config file, and OAuth is deliberately absent.
  3. Requests against models outside the Exacto-leaning allowlist are rejected with a typed `ProviderError::ModelNotAllowlisted` — no silent fallback to ICL parsing.
  4. Feeding fragmented, malformed, or null-`arguments` tool-call deltas into the parser yields a valid reassembled tool call (or a typed `ToolCallMalformed` error), never a panic.
  5. A session that crosses its `--max-usd` budget aborts with a user-visible event; 429/503 responses retry with jittered exponential backoff and surface user-visible retry events.
**Plans**: 10 plans

Plans:
- [x] 02-01-PLAN.md — Wave 0 test scaffolding: dev-deps, MockServer helper, 6 SSE cassettes + allowlist fixture (PROV-01, PROV-04, PROV-05, PROV-07)
- [x] 02-02-PLAN.md — D-01 Step 1: atomic rename 23 forge_*/lib.rs to mod.rs (PROV-01 prereq)
- [x] 02-03-PLAN.md — D-01 Step 2 sub-wave A+B+C: 17 leaf + forge_domain + forge_domain-dependent subtrees path-rewritten (PROV-01 prereq)
- [x] 02-04-PLAN.md — D-01 Step 2 sub-wave forge_app: 212 import rewrites across 83 files, commit 808edcc (PROV-01 prereq)
- [ ] 02-05-PLAN.md — D-01 Step 2 final + CI cleanup: forge_services/infra/repo/api/main + remove --exclude kay-core from ci/docs/STATE (PROV-01 prereq)
- [ ] 02-06-PLAN.md — kay-provider-openrouter scaffolding: Cargo.toml deps, Provider trait, AgentEvent, ProviderError, crate-wide #![deny(clippy::unwrap_used)] (PROV-01, PROV-02, PROV-08)
- [ ] 02-07-PLAN.md — Allowlist gate (PROV-04) + API-key auth (PROV-03) with TM-01 Debug redaction + TM-04 charset validation
- [ ] 02-08-PLAN.md — OpenRouterProvider impl: UpstreamClient + SSE translator + tool-call reassembly (PROV-01, PROV-02, PROV-05 part 1)
- [ ] 02-09-PLAN.md — Tolerant two-pass JSON parser (forge_json_repair fallback) + proptest never-panic + 1MB cap (PROV-05, TM-06)
- [ ] 02-10-PLAN.md — Retry policy (backon + Retry-After) + cost cap turn-boundary + error taxonomy + STATE.md closeout (PROV-06, PROV-07, PROV-08)
**UI hint**: no

> **Phase 2 status note (2026-04-20):** plans 02-01 through 02-04 executed and partial 02-05 executed (5 upper subtree rewrites committed). Mechanical rewrite approach (D-01 Options a/b) hit a structural wall at 1323 residual errors — ForgeCode's source cannot run as a single mono-crate. Blocked on **Phase 2.5** (kay-core sub-crate split, D-01 Option c) before 02-06..02-10 can execute. See Phase 2.5 below for scope.

### Phase 2.5: kay-core sub-crate split *(INSERTED 2026-04-20)*

**Goal**: ForgeCode's imported source compiles cleanly as a workspace of sub-crates, preserving the original 23-crate structure from upstream. This unblocks the remaining Phase 2 plans (02-06..02-10) by making `kay-provider-openrouter` able to depend on specific forge_* sub-crates rather than the broken mono-crate.
**Depends on**: Phase 2 (plans 02-01..02-05 complete; planning and Wave 0 scaffold intact, but mono-crate approach ruled out)
**Requirements**: PROV-01 (unblocked), WS-05 (now reachable)
**Success Criteria** (what must be TRUE):
  1. `cargo check --workspace --deny warnings` passes on macOS, Linux, Windows **without** `--exclude kay-core`.
  2. `forge_tool_macros` is its own proc-macro sub-crate (required by Rust — a proc-macro cannot be used from the same crate that defines it).
  3. Each of the 23 forge_* subtrees lives in its own workspace member at `crates/forge_<name>/`, preserving the original ForgeCode lib.rs as the crate root (not `mod.rs`).
  4. `include_str!` resource files (templates/, shell-plugin/, commands/, vertex.json etc.) are placed at repo root so include_str! relative paths resolve correctly when sub-crates are at `crates/<name>/`.
  5. The forgecode-parity-baseline tag's semantic integrity is preserved: combined sha256 of source files imported in Phase 1 remains unchanged (only module-system packaging differs).
  6. All path rewrites from plans 02-02..02-05 are reverted as part of the sub-crate split (each sub-crate's `use crate::X` is now correct because `crate` refers to that sub-crate itself, not to kay-core).
  7. `kay-provider-openrouter` declares path-dependencies on specific forge_* sub-crates it needs (forge_domain, forge_config, forge_services, forge_repo, forge_json_repair) — rather than the single kay-core.
  8. Existing kay-provider-openrouter Wave 0 test scaffold (plan 02-01 artifacts — MockServer, 6 SSE cassettes) continues to compile unchanged.
**Plans**: 4 plans

Plans:
- [ ] 02.5-01-PLAN.md — Prep and revert: discard 132 uncommitted files, revert 24 source-change commits (02-02..02-05), fetch missing resource files from upstream SHA 022ecd9 (WS-05)
- [ ] 02.5-02-PLAN.md — Wave 0 sub-crates (12 crates, no forge_* deps): forge_tool_macros (proc-macro=true), forge_template, forge_json_repair, forge_stream, forge_test_kit, forge_embed, forge_ci, forge_walker, forge_markdown_stream, forge_select, forge_display, forge_config (WS-05)
- [ ] 02.5-03-PLAN.md — Wave 1-5 sub-crates (9 crates, forge_domain through forge_api): forge_domain, forge_spinner, forge_tracker, forge_fs, forge_snaps, forge_app, forge_services, forge_infra, forge_repo, forge_api (integration gate) (WS-05, PROV-01)
- [ ] 02.5-04-PLAN.md — forge_main + kay-core aggregator + kay-provider-openrouter wiring + CI cleanup: remove --exclude kay-core, cargo fmt --all, final cargo check --workspace --deny warnings gate (WS-05, PROV-01)
**UI hint**: no

### Phase 3: Tool Registry + KIRA Core Tools

**Goal**: Agents can invoke tools through a native provider `tools` parameter against an object-safe registry whose schemas are hardened, and the KIRA trio (`execute_commands` with marker polling, `task_complete`, `image_read`) plus core file operations work end-to-end against Phase 2's provider.
**Depends on**: Phase 2
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05, TOOL-06, SHELL-01, SHELL-02, SHELL-03, SHELL-04, SHELL-05
**Success Criteria** (what must be TRUE):
  1. A developer can register a new `Tool` at runtime via `Arc<dyn Tool>` and see its schema emitted into the provider's native `tools` parameter with ForgeCode-style `required`-before-`properties` hardening.
  2. The `execute_commands` tool runs a shell command inside the project-root sandbox, streams output as `AgentEvent::ToolOutput`, and signals completion via a cryptographically random `__CMDEND__<seq>__` marker with captured exit code.
  3. A long-running command can be cleanly terminated by a configurable hard timeout, with signal propagation and zombie reap verified on all three OSes.
  4. The `image_read` tool accepts a base64 terminal screenshot and feeds it to a multimodal model turn, bounded by per-turn (1-2) and per-session (10-20) caps.
  5. User-injected input containing a fake marker is detected before execution and rejected, and `task_complete` does not return success until the Phase 8 verifier has run.
**Plans**: TBD
**UI hint**: no

### Phase 4: Sandbox (All Three Platforms)

**Goal**: Every shell, file, or network action dispatched through the tool registry runs inside an OS-enforced sandbox whose default policy confines writes to the project root and permits network only to the configured provider host.
**Depends on**: Phase 3
**Requirements**: SBX-01, SBX-02, SBX-03, SBX-04
**Success Criteria** (what must be TRUE):
  1. A malicious tool call attempting `rm -rf ~` or writing outside the project root is denied by the kernel (macOS `sandbox-exec`, Linux Landlock+seccomp, Windows Job Objects + restricted token) and produces a structured `AgentEvent::SandboxViolation`.
  2. Reads from the project tree and the user's Kay config directory succeed; reads from arbitrary parts of `$HOME` (e.g. `~/.aws/credentials`, `~/.ssh/`) are blocked by default policy.
  3. Outbound network traffic is allowed only to the configured OpenRouter host by default; other destinations fail with a visible policy-violation event.
  4. Each platform's sandbox is exercised in CI with a "must-fail" escape suite, and failures appear loudly in the agent trace, not silently.
**Plans**: TBD
**UI hint**: no

### Phase 5: Agent Loop (Event-Driven Core)

**Goal**: A headless run of Kay — `kay run --prompt "..." --headless` — executes a full agent turn cycle (compose -> stream -> tool -> verify -> turn end) through a frozen `AgentEvent` API with swappable forge/sage/muse personas (names inherited from ForgeCode), clean pause/resume/abort semantics, and a first-class standalone CLI that rebrands `forge_main` without regressing inherited interactive features.
**Depends on**: Phase 2, Phase 3, Phase 4
**Requirements**: LOOP-01, LOOP-02, LOOP-03, LOOP-04, LOOP-05, LOOP-06, CLI-01, CLI-03, CLI-04, CLI-05, CLI-07
**Success Criteria** (what must be TRUE):
  1. A user runs `kay run --prompt "..." --headless --persona forge` and receives a full `AgentEvent` stream on stdout ending in `TurnEnd` with a non-zero exit code reserved for sandbox violations.
  2. The same code path serves `forge`, `sage`, and `muse` by loading different YAML persona files (system prompt + tool filter + model) — no triplicated code. Persona names inherited from ForgeCode.
  3. A running turn can be paused, resumed, or aborted cleanly via a control channel, and `muse` or `forge` can invoke `sage` as a read-only sub-tool.
  4. `AgentEvent` is marked `#[non_exhaustive]` and documented as a frozen API surface for downstream Tauri and CLI consumers.
  5. `task_complete` never returns success to the loop until a verification pass has run (no-op critic stub in Phase 5, real critics in Phase 8).
**Plans**: TBD
**UI hint**: no

### Phase 6: Session Store + Transcript

**Goal**: A session's full history survives process restart — users can list, resume, fork, and export sessions through the CLI against a JSONL source-of-truth transcript indexed by SQLite, with pre-edit file snapshots enabling single-step rewind.
**Depends on**: Phase 5
**Requirements**: SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, CLI-02
**Success Criteria** (what must be TRUE):
  1. After `kay` exits mid-session, `kay resume <session-id>` restores the full transcript and cursor position from the JSONL + SQLite pair.
  2. `kay session fork <session-id>` creates a child session with `parent_session_id` populated in SQLite (schema reserved for v2 multi-agent orchestration).
  3. A user can `kay session export <session-id>` and receive a self-contained JSONL bundle suitable for TB 2.0 submission and bug-report reproduction.
  4. After an `edit_file` tool call, a pre-edit snapshot exists under `~/.kay/snapshots/<session>/<turn>/<path>` and `kay rewind` restores it.
  5. `kay session import`/`kay session replay` round-trips a previously exported session and reproduces its transcript events.
**Plans**: TBD
**UI hint**: no

### Phase 7: Context Engine

**Goal**: Kay prompts are built from ForgeCode-grade structured context — function signatures and module boundaries from tree-sitter, retrieved via hybrid structural + vector search with an explicit per-turn budget — not raw file dumps.
**Depends on**: Phase 5 (parallelizable with Phase 6)
**Requirements**: CTX-01, CTX-02, CTX-03, CTX-04, CTX-05
**Success Criteria** (what must be TRUE):
  1. Opening a new project triggers a lazy tree-sitter index that extracts function signatures and module boundaries into the symbol store, not full file bodies.
  2. A turn's prompt assembly calls `retrieve(turn_context)` and returns a bounded set of symbols and snippets via hybrid structural lookup + `sqlite-vec` similarity.
  3. When retrieved context would exceed the per-turn budget, truncation is explicit and surfaced to the user (never silent drop).
  4. All tool schemas inlined into prompts are passed through the same ForgeCode hardening post-process used in Phase 3 — `required` before `properties`, flattened nested required, truncation reminders.
  5. Re-indexing after a file-watch invalidation completes incrementally rather than re-parsing the entire repo, and a monorepo >10k files stays responsive on first use.
**Plans**: TBD
**UI hint**: no

### Phase 8: Multi-Perspective Verification (KIRA Critics)

**Goal**: Before `task_complete` accepts a turn as finished, Kay runs KIRA-style critics (test engineer + QA engineer + end-user) with confidence-gated firing and a bounded re-work loop; verification cost stays inside a configurable ceiling.
**Depends on**: Phase 5, Phase 7
**Requirements**: VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04
**Success Criteria** (what must be TRUE):
  1. A task that `task_complete` would otherwise accept is rejected when any critic returns FAIL, and the critic feedback is injected as a user message to continue the loop (bounded by N retries per turn).
  2. In interactive mode the default is 1 critic; in benchmark mode the trio runs — the mode toggle is visible and configurable.
  3. Verifier cost counts against the session `--max-usd` budget and emits a structured `AgentEvent::Verification` frame for every critic verdict.
  4. A regression in verifier token cost of >30% vs baseline fails the CI cost gate, and when the policy ceiling is breached the verifier turns off with a trace event explaining why.
**Plans**: TBD
**UI hint**: no

### Phase 9: Tauri Desktop Shell

**Goal**: A user double-clicking `Kay.app` / `Kay.exe` / `Kay.AppImage` sees a native Tauri 2.x window streaming agent events — live trace, tool-call timeline, token/cost meter — driven by the merged-into-main-binary `kay-core` (no `externalBin` sidecar), with a 4-hour memory canary proving long-session stability.
**Depends on**: Phase 5, Phase 6
**Requirements**: TAURI-01, TAURI-02, TAURI-03, TAURI-04, TAURI-05, TAURI-06, UI-01
**Success Criteria** (what must be TRUE):
  1. A user launches Kay on macOS, Windows, or Linux and sees a live session view streaming agent-trace frames, tool-call cards, and a per-turn/per-session token/cost meter via `ipc::Channel<AgentEvent>`.
  2. The Tauri bundle is a single merged Rust binary (no `externalBin` sidecar), allowing macOS notarization to succeed.
  3. TypeScript bindings for commands and channel payloads are generated by `tauri-specta` v2 and compile-checked against the Rust types — IPC drift is caught at build time.
  4. A nightly 4-hour scripted session on macOS and Linux passes a memory-delta regression gate, guarding against Tauri IPC memory leaks (#12724/#13133).
  5. The React 19 + TypeScript + Vite frontend renders diffs through Monaco or CodeMirror in the tool-call inspector.
**Plans**: TBD
**UI hint**: yes

### Phase 9.5: TUI Frontend (ratatui)

**Goal**: A user running over SSH, in a low-bandwidth terminal, or by preference types `kay tui` and gets a full-screen ratatui interface with session list, active transcript, tool-call inspector, and cost meter — tailing the same `kay-cli` structured-event stream that `kay-tauri` consumes. No parallel agent-loop implementation; the TUI is a pure frontend over the CLI contract.
**Depends on**: Phase 5 (needs CLI-05 structured-event stream), Phase 6 (needs session store for multi-session list)
**Requirements**: TUI-01, TUI-02, TUI-03, TUI-04, TUI-05
**Success Criteria** (what must be TRUE):
  1. A user runs `kay tui` (or `kay-tui` standalone) and sees a multi-pane ratatui layout rendering a live session's agent trace, tool-call timeline, and token/cost meter.
  2. `kay-tui` reads the JSONL event stream from `kay-cli` (CLI-05) — not from `kay-core` directly — proving the CLI contract is the single source of truth for frontends.
  3. All navigation is keyboard-driven; no mouse required; the TUI works cleanly over SSH with no terminal capabilities beyond ANSI 256-color.
  4. `cargo install kay-tui` installs the TUI standalone, and it can also be invoked as `kay tui` from the main binary.
  5. Session control (spawn / pause / resume / fork) reaches parity with the GUI via keyboard shortcuts documented in the help pane.
**Plans**: TBD
**UI hint**: yes

### Phase 10: Multi-Session Manager + Project Settings

**Goal**: A user manages multiple sessions, projects, API keys, and policy from the GUI — spawning, pausing, resuming, and forking sessions; binding OpenRouter keys into the OS keychain; selecting from a tiered model allowlist; and opting into command approval and sandbox policy from a settings panel.
**Depends on**: Phase 6, Phase 9, Phase 9.5
**Requirements**: UI-02, UI-03, UI-04, UI-05, UI-06, UI-07
**Success Criteria** (what must be TRUE):
  1. A user spawns, pauses, resumes, and forks sessions entirely from the GUI without touching the CLI.
  2. The user picks a project directory, edits per-project env/keys through the UI, and binds an OpenRouter API key that lands in the OS keychain (never localStorage or config plaintext).
  3. The model selector presents a tiered list — Recommended (Exacto allowlist), Experimental (smoke-tested), All (behind a "Compatibility unknown" warning).
  4. A one-click export produces a JSONL + metadata manifest suitable for TB 2.0 submission or bug-report reproduction.
  5. The settings panel surfaces cost budgets, model allowlist, verifier policy, sandbox policy, and a command-approval toggle that is off by default for benchmark runs but on by default for first-time users.
**Plans**: TBD
**UI hint**: yes

### Phase 11: Cross-Platform Hardening + Release Pipeline

**Goal**: A user downloads a signed, notarized, reproducibly-built Kay artifact for their OS from a GitHub release; `cargo install kay` yields the headless CLI from crates.io; the updater verifies signatures against a pre-pinned minisign public key.
**Depends on**: Phase 4, Phase 9
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07, CLI-06
**Success Criteria** (what must be TRUE):
  1. Every push to `main` produces signed + notarized macOS (arm64 + x64), Windows Authenticode-signed, and Linux (x64 + arm64, AppImage + tar.gz with SHA attestations) artifacts — not only on release tags.
  2. A user runs `cargo install kay` on crates.io and gets a working headless CLI matching the release binary.
  3. The Tauri bundler produces `.app`, `.msi`, `.AppImage` bundles with reproducible build metadata and signed artifacts.
  4. `tauri-plugin-updater` ships with a minisign keypair whose public key was pinned in `tauri.conf.json` before the first release — rotations are not possible without a documented migration.
  5. Windows CI runs the full interactive PTY suite (ConPTY flags, `Ctrl+C`, resize) green.
**Plans**: TBD
**UI hint**: no

### Phase 12: Terminal-Bench 2.0 Submission + v1 Hardening

**Goal**: Kay posts a public Terminal-Bench 2.0 score >81.8% with a documented reference run (model pinned, seed pinned, transcript archived), validated against a held-out task subset and a parallel real-repo eval — and the v1.0 release ships with that score in the README.
**Depends on**: Phase 8, Phase 10, Phase 11
**Requirements**: EVAL-02, EVAL-03, EVAL-04, EVAL-05
**Success Criteria** (what must be TRUE):
  1. A single `kay eval tb2` command runs the Harbor harness locally with pinned Docker images, seed, and model allowlist, matching official submission settings exactly.
  2. A held-out task subset (never referenced during development) is revealed for final validation and scores within 2 percentage points of the full-set local score.
  3. Nightly real-repo eval (Rails, React+TS, Rust crate, Python package, monorepo >10k files) passes and its result is published alongside the TB 2.0 score.
  4. The public TB 2.0 leaderboard lists Kay >81.8% with a documented, model-pinned reference run whose full transcript is archived in the repo.
**Plans**: TBD
**UI hint**: no

## Dependencies & Parallelization

**Chain A (Score, serial):** 1 -> 2 -> 3 -> 4 -> 5 -> 7 -> 8 -> 12. No phase in this chain can be skipped without breaking the downstream one.

**Chain B (UI, serial after Phase 5):** 5 -> 6 -> 9 -> 10. The UI cannot start until `AgentEvent` is frozen in Phase 5 and sessions persist in Phase 6.

**Chain C (Distribution, serial after Phase 1):** 1 (signing enrollment) -> 9 (signed dev builds on merge) -> 11 (full release pipeline) -> 12 (v1.0 release). Apple Developer ID + Azure Code Signing lead time is the critical path — it is gated at Phase 1.

**Parallelizable phases** (parallelization enabled in config.json):
- **Phase 6 (Session Store) || Phase 7 (Context Engine):** share no code and can run in parallel once Phase 5 lands.
- **Phase 9 (Tauri Shell) scaffolding can begin during Phase 6:** once `AgentEvent` is frozen from Phase 5, the UI team can scaffold `kay-desktop` while Phase 6 wires persistence.
- **Phase 11 (Release Pipeline) hardening can begin during Phase 10:** signing, bundler, updater, and Windows CI hardening need Phase 4 (sandbox) and Phase 9 (Tauri shell) in place, not Phase 10.

**Non-negotiable sequencing:** Phase 1's parity gate (EVAL-01) must pass before any Phase 2 harness modification merges to `main`. Any silent regression from the fork surfaces now, not in Phase 12.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12 (with Phase 7 allowed to overlap Phase 6 per parallelization policy).

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Fork, Governance, Infrastructure | 6/6   | Complete | 2026-04-19 |
| 2. Provider HAL + Tolerant JSON Parser | 4/10 | Blocked on 2.5 | - |
| 2.5. kay-core sub-crate split *(INSERTED 2026-04-20)* | 0/4 | Planned — ready for execution | - |
| 3. Tool Registry + KIRA Core Tools | 0/TBD | Not started | - |
| 4. Sandbox (All Three Platforms) | 0/TBD | Not started | - |
| 5. Agent Loop (Event-Driven Core) | 0/TBD | Not started | - |
| 6. Session Store + Transcript | 0/TBD | Not started | - |
| 7. Context Engine | 0/TBD | Not started | - |
| 8. Multi-Perspective Verification (KIRA Critics) | 0/TBD | Not started | - |
| 9. Tauri Desktop Shell | 0/TBD | Not started | - |
| 10. Multi-Session Manager + Project Settings | 0/TBD | Not started | - |
| 11. Cross-Platform Hardening + Release Pipeline | 0/TBD | Not started | - |
| 12. Terminal-Bench 2.0 Submission + v1 Hardening | 0/TBD | Not started | - |

## Backlog

### Phase 999.1: Windows sandbox hardening research (BACKLOG)

**Goal:** [Captured for future planning] — Budget deeper research during Phase 4 for Windows sandbox (Job Objects + restricted token + integrity level). Community guidance is weaker than macOS/Linux per `.planning/research/ARCHITECTURE.md` (MEDIUM confidence). Prototype against known-good Windows restricted-mode examples before Phase 4 Windows sub-task.
**Requirements:** TBD (related to SBX-01 Windows implementation)
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd-review-backlog when ready)

### Phase 999.2: Provider-level circuit breaker (BACKLOG)

**Goal:** [Captured 2026-04-20 via /silver-quality-gates Phase 2 design-time review, Reliability dim advisory] — Phase 2 implements per-request retry bounds (backon, 3 attempts, 8s cap, full jitter) but has no cross-request circuit-breaker state. When OpenRouter has sustained outages across many sessions, each session pays the full retry budget. Post-v1, add a shared circuit-breaker layer (open on 5 consecutive failures or >50% in 60s window; half-open test after 30s) that short-circuits HTTP attempts during outages. Likely a small crate (`kay-circuit-breaker`) or a feature of `kay-provider-openrouter`.
**Requirements:** TBD (extends PROV-07 posture)
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd-review-backlog when ready; earliest targeted slot: between Phase 11 hardening and Phase 12 submission, or as part of a post-v1 reliability phase)

### Phase 999.3: Error message "what-to-do" audit (BACKLOG)

**Goal:** [Captured 2026-04-20 via /silver-quality-gates Phase 2 design-time review, Usability dim advisory] — Phase 2's `ProviderError` uses `thiserror` typed variants with structured fields (ModelNotAllowlisted lists requested+allowed, Auth::Missing names env var, CostCapExceeded shows cap+spent), but the `Display` remediation language per variant has not been user-tested. Post-v1, audit each ProviderError variant's Display impl against the usability "what happened / why / what to do" rubric. Also apply to AgentEvent::Error surfacing in CLI/TUI/GUI (Phase 5/9/9.5 consumers).
**Requirements:** TBD (extends PROV-08 UX surface)
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd-review-backlog when ready; natural slot: during Phase 9.5 TUI work or Phase 10 settings UI)

---
*Roadmap created: 2026-04-19*
*Last backlog update: 2026-04-20 — added 999.2 + 999.3 from Phase 2 quality-gates advisory*
*Phase 2.5 plans authored: 2026-04-20 — 4 plans created (02.5-01 through 02.5-04)*
