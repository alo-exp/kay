# Phase 3: Tool Registry + KIRA Core Tools - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning
**Mode:** Auto-resolved from prior context per user directive ("proceed autonomously, decide best options")

<domain>
## Phase Boundary

Any agent turn can invoke tools through a **native provider `tools` parameter** against an object-safe registry whose JSON-Schemas are hardened in the ForgeCode style, and the **KIRA trio** (`execute_commands` with marker polling, `task_complete`, `image_read`) plus a minimal parity-preserving set of core file operations work end-to-end against Phase 2's `OpenRouterProvider`.

**In scope:**
- New `kay-tools` workspace crate: `pub trait Tool` (object-safe, async, `Arc<dyn Tool>`) + `ToolRegistry` + built-in tool impls.
- Concrete KIRA tools: `execute_commands`, `task_complete`, `image_read`.
- Core file operations (minimal set): `fs_read`, `fs_write`, `fs_search`, `net_fetch` — as thin `Tool` adapters over existing `forge_app::ToolExecutor` dispatch paths.
- Schema hardening reused from `forge_app::utils::enforce_strict_schema` (required-before-properties, allOf flattening, OpenAI compatibility) with a Kay-side wrapper that appends truncation-reminder text to descriptions.
- `AgentEvent::ToolOutput` frame added to Phase 2's `#[non_exhaustive]` enum (additive, no break).
- `__CMDEND_<nonce>_<seq>__` marker protocol for async shell execution, with stdout streaming via `AgentEvent::ToolOutput`.
- Hard timeout per tool call (existing `tool_timeout_secs` config in forge_config) + clean process termination (kill + zombie reap).
- `portable-pty` fallback for TTY-requiring commands (new workspace dep).
- Per-turn / per-session caps on `image_read` (2 / 20 defaults, overridable).
- `kay-cli` startup wiring: construct default registry from `kay-tools::default_tool_set()` and hand it to the provider.

**Out of scope (deferred or owned by other phases):**
- **Sandbox enforcement** — Phase 4 (SBX-01..04). Phase 3 accepts an `Arc<dyn Sandbox>` via dependency injection; a `NoOpSandbox` placeholder is wired so tools function end-to-end and Phase 4 swaps in real enforcement without API changes.
- **Multi-perspective verification for `task_complete`** — Phase 8 (VERIFY-01..04). Phase 3 wires a `NoOpVerifier` that reports `VerificationOutcome::Pending` so `task_complete` functions without faking success; Phase 8 swaps the impl.
- **Agent loop / TurnEnd semantics** — Phase 5 (LOOP-02, LOOP-05). Phase 3 exposes tool output via `AgentEvent::ToolOutput` but does not control turn lifecycle.
- **MCP (Model Context Protocol) tools** — not in REQUIREMENTS.md Phase 3 scope; defer to a dedicated MCP phase (tracked in STATE.md blockers for `rmcp` advisory cleanup).
- **Additional ForgeCode tools** — `fs_patch`, `fs_multi_patch`, `fs_remove`, `fs_undo`, `plan_create`, `skill_fetch`, `todo_write`, `todo_read`, `task` (agent delegation), `sem_search` (needs Phase 7 context engine), `followup` — available in `forge_app`'s catalog but not registered in Kay's v1 registry. Re-evaluate at Phase 5 when agent-loop surface firms up.
- **End-user tool plug-ins / scripting** — `Arc<dyn Tool>` architecture exists to enable future extensibility (Phase 12+), not v1 end-user scripting.
- **Session persistence of tool outputs** — Phase 6 owns transcript storage.

</domain>

<decisions>
## Implementation Decisions

Decisions below are organized load-bearing → derivative. Claude auto-resolved each per user's "proceed autonomously, decide best options" directive, citing prior phase context or explicit REQ-IDs. Any decision can be revisited during `/gsd-plan-phase 3` research/planning.

### Tool trait + registry architecture (load-bearing)

- **D-01:** **New workspace crate `kay-tools` owns the `Tool` trait, registry, built-in impls, and default tool set.**
  - **Trait shape** (object-safe, async, `Arc<dyn Tool>` per TOOL-01):
    ```rust
    #[async_trait::async_trait]
    pub trait Tool: Send + Sync + 'static {
        fn name(&self) -> &ToolName;
        fn description(&self) -> &str;
        fn input_schema(&self) -> &schemars::Schema;

        async fn invoke(
            &self,
            args: serde_json::Value,
            ctx: &ToolCallContext,
        ) -> Result<ToolOutput, ToolError>;
    }
    ```
    - `ToolName`, `ToolOutput`, `ToolCallContext` re-exported from `forge_domain` (already exist).
    - `ToolError` is a new Kay-side enum (non_exhaustive) — see D-09.
  - **Registry shape:**
    ```rust
    pub struct ToolRegistry {
        tools: HashMap<ToolName, Arc<dyn Tool>>,
    }
    impl ToolRegistry {
        pub fn new() -> Self { ... }
        pub fn register(&mut self, tool: Arc<dyn Tool>) { ... }
        pub fn get(&self, name: &ToolName) -> Option<&Arc<dyn Tool>> { ... }
        pub fn tool_definitions(&self) -> Vec<ToolDefinition> { ... }  // for provider.tools emission
    }
    ```
  - **Rationale:** TOOL-01 is explicit ("object-safe, async, `Arc<dyn Tool>` map"). ForgeCode's `ToolCatalog` enum is closed-world — fine for their product but incompatible with TOOL-01. A new crate keeps the module boundary clean (Phase 5 agent loop + Phase 9/9.5 frontends will depend on it); mirrors the Phase 2.5 sub-crate pattern.
  - **Parity preservation:** Built-in tools (`FsReadTool`, `FsWriteTool`, etc.) are thin structs that wrap `Arc<Services>` and delegate to `forge_app::ToolExecutor::execute(ToolCatalog::Read(...), ctx)` (or the equivalent per-tool catalog variant). The behavior on the wire is **byte-identical** to ForgeCode's dispatch — only the outer trait object is new.
  - **Alternatives rejected:**
    - (a) Extend `forge_domain::ToolCatalog` enum with KIRA variants — breaks TOOL-01's object-safety requirement and forces all new tools to be compiled into `forge_domain` (a parity-imported crate we don't want to fork).
    - (b) Put the trait in `kay-core` — `kay-core` is currently a thin aggregator (post-Phase-2.5); bloating it with the tool system runs against the split's intent.
    - (c) Put the trait in `kay-provider-openrouter` — wrong layer (tool execution is provider-agnostic; only tool-*schema emission* is provider-specific).

### Schema hardening (TOOL-05)

- **D-02:** **Reuse `forge_app::utils::enforce_strict_schema(schema, strict_mode=true)` unchanged** as the canonical hardening step, wrapped by a Kay-side helper that also appends truncation-reminder text to descriptions.
  - `enforce_strict_schema` already implements: `required` containing ALL property keys (sorted deterministically), `allOf` flattening, `additionalProperties: false`, `propertyNames` stripping, `nullable` → `anyOf: [..., {"type":"null"}]` conversion, OpenAI-compatible `type: object` insertion. See `crates/forge_app/src/utils.rs:340-440`.
  - Kay wrapper:
    ```rust
    // kay-tools: hardening.rs
    pub fn harden_tool_schema(schema: &mut schemars::Schema) {
        let mut v = schema.to_value().clone();
        forge_app::utils::enforce_strict_schema(&mut v, true);
        *schema = schemars::Schema::try_from(v).expect("re-roundtrip");
    }

    pub fn hardened_description(base: &str, truncation_note: Option<&str>) -> String {
        // Append KIRA-style truncation reminder, e.g. "Long outputs are truncated
        // at the end with an ellipsis. If your task depends on later output,
        // narrow the command or grep for specific lines."
        ...
    }
    ```
  - **Rationale:** TOOL-05 says "ForgeCode JSON-schema hardening post-process" — the canonical impl **already exists** in the parity-imported tree. Rewriting regresses parity. The only Kay-specific addition is truncation-reminder descriptions (a KIRA technique) — that's a tiny string-level wrap.
  - **Alternatives rejected:** writing a second hardening impl in `kay-tools` would drift from ForgeCode behavior across upstream syncs.

### `execute_commands` marker protocol (SHELL-01 + SHELL-05 load-bearing)

- **D-03:** **Marker format: `__CMDEND_<nonce>_<seq>__` + exit-code tail.**
  - `<nonce>`: 128-bit cryptographically random hex string (32 chars), generated per *tool call* using `rand::rngs::OsRng`. Prompt injection cannot forge the marker because the attacker doesn't know the nonce.
  - `<seq>`: per-session monotonic counter (starts at 0), mostly for debuggability and to guarantee uniqueness if two command tool calls happen to collide on a nonce (astronomically unlikely but cheap insurance).
  - **Wrap shape** — the user's command string is re-framed as:
    ```sh
    ( user_command
    ) ; __KAY_EXIT=$? ; printf '\n__CMDEND_%s_%d__EXITCODE=%d\n' "<nonce>" "<seq>" "$__KAY_EXIT"
    ```
    On Windows PowerShell, the equivalent is emitted with `$LASTEXITCODE`. A shell-specific wrap picker lives in `kay-tools::markers`.
  - **Streaming + detection:**
    - stdout is read via `tokio::io::BufReader::lines()` (or PTY equivalent for TTY path).
    - Each non-marker line is immediately emitted as `AgentEvent::ToolOutput { call_id, chunk: Stdout(line) }` — **no buffering** — to satisfy SHELL-03's "no blocking reads."
    - stderr is streamed in parallel via a second `BufReader::lines()` task; emitted as `ToolOutputChunk::Stderr`.
    - The marker match uses **`subtle::ConstantTimeEq`** (new workspace dep `subtle = "2"`) on the line prefix so timing side-channels don't leak "you almost guessed the nonce" to an adversarial tool author. See SHELL-05.
    - On marker match: parse `EXITCODE=N`, emit `AgentEvent::ToolOutput { call_id, chunk: Closed { exit_code: Some(N), marker_detected: true } }`, stop streaming.
    - If the process exits **without** emitting the marker (e.g., SIGKILL or crash): emit `Closed { exit_code: <from wait()>, marker_detected: false }` — the agent loop sees the command ended abnormally. This is SHELL-05's "marker race" recovery path.
  - **Rationale:** PROJECT.md identifies marker-based polling as one of KIRA's four harness techniques directly responsible for ForgeCode's 81.8% TB 2.0 score — not optional. 128-bit nonce is the same entropy tier that Terminus-KIRA reports using (their writeup mentions "cryptographically random per-command"). Constant-time compare is cheap defense-in-depth.
  - **Alternatives rejected:**
    - Static `__CMDEND__` string — trivially forgeable by any user-controlled shell output; breaks the entire security guarantee (prompt-injected tool output could make the agent think a long-running attacker-controlled command has finished).
    - Exit-code-file-on-disk (write `$?` to a tempfile and poll) — adds sandbox complexity (Phase 4 must allow the tempfile write) and doesn't solve prompt-injection.

### `execute_commands` PTY vs non-PTY (SHELL-02)

- **D-04:** **Default to `tokio::process::Command`; switch to `portable-pty` when a heuristic fires OR the caller passes `tty: true`.**
  - **Heuristic denylist** (config-overridable): commands whose *first token* matches `ssh|sudo|docker|less|more|vim|nvim|nano|top|htop|python -i|node -i` trigger PTY automatically. Optional second token checks (`ssh -t`, `docker run -it`) tighten it.
  - **Schema addition:** `execute_commands` accepts an optional `tty: bool` (default false). JSON-Schema description: *"Force a PTY-backed shell. Use when the command requires a TTY (interactive prompts, full-screen UIs)."*
  - **PTY implementation:** `portable-pty = "0.8"` (new workspace dep — MIT/Apache). PTY path wires stdout/stderr of the pty master into the same line-streaming logic as the non-PTY path (marker detection unchanged).
  - **Rationale:** SHELL-02 explicit. Pure-tokio is 10–50× faster startup for the common case; PTY adds visible overhead. Parity-preserving: ForgeCode's `forge_services/tool_services/shell.rs` uses tokio-only; Kay adds PTY but keeps default fast path.
  - **Alternatives rejected:** always-PTY (Terminus does this) — slows every command; not needed for the parity-run majority that are headless `cargo test` / `pytest` invocations.

### Hard timeout + clean termination (SHELL-04)

- **D-05:** **Reuse `forge_config::ForgeConfig.tool_timeout_secs` and `forge_app::ToolRegistry::call_with_timeout` pattern.**
  - Default timeout: 300s (5 min) — matches ForgeCode's existing default in `tool_registry.rs:54`.
  - **Termination sequence on timeout:**
    1. `Child::start_kill()` → SIGTERM (Unix) / TerminateProcess (Windows).
    2. `tokio::time::timeout(Duration::from_secs(2), child.wait())` — 2-second grace period.
    3. If still running: `Child::kill()` → SIGKILL.
    4. `child.wait().await` to reap (prevents zombies).
  - **PTY branch:** `portable_pty::Child::kill()` (synchronous, library-handled).
  - **Signal propagation:** the shell wrapper (D-03) runs the user command in a subshell `( ... )`; SIGTERM to the outer shell on Unix propagates to the child process group via tokio's `kill_on_drop(true)` + `process_group(0)` pair (set at Command build time). Windows uses Job Objects in Phase 4 (sandbox) — Phase 3's stopgap is `TerminateProcess` only.
  - **Rationale:** SHELL-04 is explicit about "clean termination (signal propagation + zombie reap)" across all three OSes. The pattern exists upstream; we just wire it into the new trait surface.

### `task_complete` + verifier integration (TOOL-03 load-bearing)

- **D-06:** **Phase 3 ships `NoOpVerifier` returning `VerificationOutcome::Pending`; Phase 8 swaps in multi-perspective impl.**
  - Trait surface (new, in `kay-tools::verifier`):
    ```rust
    #[async_trait::async_trait]
    pub trait TaskVerifier: Send + Sync {
        async fn verify(
            &self,
            task_summary: &str,
            transcript: &Transcript,  // Phase 6 seeds this; Phase 3 uses `&[AgentEvent]`
        ) -> VerificationOutcome;
    }

    #[non_exhaustive]
    pub enum VerificationOutcome {
        Pass { rationale: String },
        Fail { reasons: Vec<String> },
        Pending { reason: String },   // Phase 3 stub returns this
    }
    ```
  - `task_complete` tool input: `{ summary: String }`. Its `invoke` calls `verifier.verify(...)`. Emits `AgentEvent::TaskComplete { verified, outcome }` — a NEW variant added to `AgentEvent` in Phase 3.
  - **Phase 3 behavior:** `NoOpVerifier` returns `Pending { reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)" }`. The agent loop (Phase 5) reads `verified: false` and does NOT exit the loop — per ROADMAP Success Criterion #5: "`task_complete` does not return success until the Phase 8 verifier has run." In Phase 3, no loop tear-down happens; the tool call simply emits the Pending event and the agent continues (Phase 5 will codify exit semantics).
  - **Rationale:** Dependency injection via trait lets Phase 8 land without touching Phase 3 code. The `NoOpVerifier` is a real implementation, not a panic or TODO — production-safe as a Phase 3 shipping default.
  - **Alternatives rejected:**
    - Stub `task_complete` that always returns success — silently breaks Success Criterion #5 and makes Phase 8 a no-op at the agent-loop level.
    - Defer the tool entirely to Phase 8 — blocks Phase 5 agent-loop work (LOOP-05 needs `task_complete` existence as a signal).

### `image_read` caps (TOOL-04)

- **D-07:** **Per-turn cap: 2. Per-session cap: 20. Overflow emits `ToolError::ImageCapExceeded`.**
  - Defaults: `kay.toml [tools.image_read] max_per_turn = 2, max_per_session = 20`. Env override `KAY_IMAGE_MAX_PER_TURN` / `KAY_IMAGE_MAX_PER_SESSION`.
  - Bookkeeping: `ImageQuota` struct with `per_turn: AtomicU32, per_session: AtomicU32`. Turn boundary (set by Phase 5 agent loop) resets `per_turn`; session boundary (Phase 6) resets `per_session`. Phase 3 seeds per-turn via a fresh `ImageQuota::new_turn()` on each `invoke` — Phase 5 will convert this into a real turn-scoped counter.
  - **Rationale:** ROADMAP Success Criterion #4 specifies "per-turn (1–2) and per-session (10–20) caps." Picking the upper end of each range maximizes useful bandwidth; if TB 2.0 runs benefit from smaller caps we can tune in Phase 12. Hard-coded defaults mean benchmark runs start from a known floor.
  - **Impl:** Reuse `forge_services::tool_services::image_read::ForgeImageRead` for base64 decode + model-input conversion; layer quota check at `Tool::invoke` entry.

### `AgentEvent` extensions (Phase 3 additions — additive per `#[non_exhaustive]`)

- **D-08:** **Add two variants.** No breakage — Phase 2's enum is already `#[non_exhaustive]`.
  ```rust
  // additions to the existing AgentEvent
  ToolOutput {
      call_id: String,
      chunk: ToolOutputChunk,
  },
  TaskComplete {
      call_id: String,
      verified: bool,
      outcome: VerificationOutcome,
  },

  #[non_exhaustive]
  pub enum ToolOutputChunk {
      Stdout(String),
      Stderr(String),
      Closed { exit_code: Option<i32>, marker_detected: bool },
  }
  ```
  - **Rationale:** SHELL-03 requires "Output streamed as `AgentEvent::ToolOutput` frames." `TaskComplete` is needed for D-06. Both are `#[non_exhaustive]` on their container types; Phase 5 will add `TurnEnd` without breakage.

### `ToolError` taxonomy

- **D-09:** **Dedicated `ToolError` enum in `kay-tools`, separate from `ProviderError`.**
  ```rust
  #[non_exhaustive]
  pub enum ToolError {
      InvalidArgs { tool: ToolName, reason: String },
      Timeout { tool: ToolName, elapsed: Duration },
      ExecutionFailed { tool: ToolName, source: anyhow::Error },
      ImageCapExceeded { scope: CapScope, limit: u32 },   // CapScope::PerTurn | PerSession
      SandboxDenied { tool: ToolName, reason: String },   // Phase 4 populates this
      NotFound { tool: ToolName },
      Io(std::io::Error),
  }
  ```
  - **Rationale:** Separating tool errors from provider errors keeps downstream match arms clean. `ProviderError` concerns wire/auth/cost; `ToolError` concerns per-invocation failures. Both are `#[non_exhaustive]`.

### Phase 3 built-in tool set (TOOL-02 / TOOL-04 scope)

- **D-10:** **Register these tools at startup; defer others.**
  - **KIRA core (this phase's headline):** `execute_commands`, `task_complete`, `image_read`.
  - **Parity-preserving minimum:** `fs_read`, `fs_write`, `fs_search`, `net_fetch` — as thin `Tool` impls over `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)`. These are *sufficient* for most TB 2.0 benchmark tasks (read/write/search/fetch cover the canonical agent-file-edit flow).
  - **Deferred to Phase 5 (agent loop) or later:** `fs_patch`, `fs_multi_patch`, `fs_remove`, `fs_undo`, `plan_create`, `skill_fetch`, `todo_write`, `todo_read`, `task` (agent delegation), `sem_search`, `followup`. These have in-tree implementations we can re-activate; keeping the Phase 3 set small tightens the blast radius during Phase 3's first end-to-end integration with the provider.
  - **Rationale:** Phase 3 success criteria name only the KIRA trio + "core file operations" (singular — not "all"). A 7-tool launch set is enough to demonstrate the architecture and run minimal TB 2.0 benchmarks post-Phase-5. Adding the deferred tools is a ~1-plan increment each (trivial trait adapter + registry entry) once the harness is stable.

### Tool registration flow

- **D-11:** **`kay-cli` startup builds the default registry via `kay_tools::default_tool_set(services, sandbox, verifier)`.** Registry is immutable for the duration of a session. No runtime registration API in v1.
  - **Rationale:** TOOL-01 says "register a new `Tool` at runtime via `Arc<dyn Tool>`" — the *architecture* supports it, but v1 exposes no user-facing registration surface. Plugins / MCP / scripting are future phases. Keeping the registry immutable per-session simplifies Phase 5's agent loop and eliminates a class of concurrency bugs.

### Sandbox integration seam (Phase 3 ↔ Phase 4)

- **D-12:** **`kay-tools` accepts `Arc<dyn Sandbox>` via DI; Phase 3 ships `NoOpSandbox` placeholder in `kay-sandbox-linux/macos/windows` (actually: a shared `NoOpSandbox` lives in a new `kay-sandbox-core` or extends one of the existing crates).**
  - Trait surface (new, in the to-be-determined sandbox crate — may be `kay-tools::sandbox` for Phase 3 and move to `kay-sandbox-*` in Phase 4):
    ```rust
    #[async_trait::async_trait]
    pub trait Sandbox: Send + Sync {
        async fn check_shell(&self, command: &str, cwd: &Path) -> Result<(), SandboxDenial>;
        async fn check_fs_read(&self, path: &Path) -> Result<(), SandboxDenial>;
        async fn check_fs_write(&self, path: &Path) -> Result<(), SandboxDenial>;
        async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial>;
    }
    ```
  - Phase 3 `NoOpSandbox` returns `Ok(())` for everything. Phase 4 swaps in per-OS enforcement without touching the tool impls.
  - **Rationale:** Keeps Phase 3 end-to-end runnable (tools actually execute) while defining the exact extension point Phase 4 will hit. Wiring this seam now saves a Phase 4 refactor wave.

### Claude's Discretion

- Module layout inside `kay-tools` (sub-module per tool vs. single `tools.rs`).
- Whether `harden_tool_schema` mutates in place or returns owned value.
- Exact test harness for marker-race scenarios (mock stdout streams vs. fork-a-subshell integration tests).
- Whether `ToolCallContext` is Kay-owned or re-exports `forge_domain::ToolCallContext` directly — planner decides based on which fields are actually needed.
- Whether `execute_commands` emits a distinct `AgentEvent::ToolCallStart`-analog frame at invocation time or relies on Phase 2's existing frames.
- Unit-test split between `kay-tools` (pure-trait tests) and a new `kay-tools/tests/` integration directory (end-to-end tool invocations).
- Whether `subtle` crate is imported directly or pulled in transitively via another hardened string-compare lib.
- Default `tool_timeout_secs` override — ForgeCode's default is 300s; Kay may want to lower for benchmark predictability. Planner decides.

### Folded Todos

None — cross-referencing todos surfaced no matches for Phase 3 scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executor) MUST read these before planning or implementing.**

### Phase & project specs
- `.planning/PROJECT.md` — vision; KIRA four techniques (marker polling is one); non-negotiable #7 (ForgeCode JSON-schema hardening).
- `.planning/REQUIREMENTS.md` §TOOL (TOOL-01..06) + §SHELL (SHELL-01..05) — locked requirements for this phase.
- `.planning/ROADMAP.md` §Phase 3 — goal + 5 success criteria.
- `.planning/STATE.md` — Phase 2 closed 2026-04-20; `rmcp` security advisories are v0.2.0 ship-blockers (issue #3) and don't gate Phase 3 since MCP is deferred.

### Prior phase artifacts Phase 3 inherits from
- `.planning/phases/02-provider-hal-tolerant-json-parser/02-CONTEXT.md` — `AgentEvent` + `ProviderError` enum shapes (D-05, D-06); OpenRouter path; `#[non_exhaustive]` evolution pattern.
- `.planning/phases/02-provider-hal-tolerant-json-parser/02-CONTEXT.md §Appendix A` — post-2.5 substitution rules (sub-crate paths instead of `kay_core::forge_*`); apply to all Phase 3 plans.
- `.planning/phases/02.5-kay-core-sub-crate-split/02.5-CONTEXT.md` — 23-sub-crate DAG; which leaf crate exposes which type.
- `crates/kay-provider-openrouter/src/translator.rs` — Phase 2 tool-call reassembly (`ToolCallStart/Delta/Complete`) that Phase 3's registry must plug into.

### ForgeCode-inherited code (imported; Phase 3 delegates or adapts)
- `crates/forge_domain/src/tools/definition/tool_definition.rs` — `ToolDefinition { name, description, input_schema: schemars::Schema }` — reuse as the emit-target type for `tool_definitions()`.
- `crates/forge_domain/src/tools/catalog.rs` — `ToolCatalog` enum + per-tool input structs (`FSRead`, `FSWrite`, `Shell`, `TaskInput`, `NetFetch`, etc.); adapters in `kay-tools` construct these variants to delegate.
- `crates/forge_domain/src/tools/call/tool_call.rs` — `ToolCallFull` / `ToolCallContext` / `ToolOutput` / `ToolResult` — reuse unchanged.
- `crates/forge_app/src/tool_registry.rs` — ForgeCode's registry pattern (timeout handling, policy checks, MCP dispatch). Phase 3's `ToolRegistry` reuses `call_with_timeout` and `check_tool_permission`; structurally differs (HashMap<Name, Arc<dyn Tool>> vs. generic-over-S struct).
- `crates/forge_app/src/tool_executor.rs` — dispatch-to-services implementation; built-in Kay tools delegate here.
- `crates/forge_app/src/utils.rs:340-440` — `enforce_strict_schema(schema, strict_mode=true)` — TOOL-05 reuse target.
- `crates/forge_services/src/tool_services/shell.rs` — existing non-PTY shell impl (parity reference; Phase 3 reshapes to add marker protocol + PTY fallback).
- `crates/forge_services/src/tool_services/image_read.rs` — base64 decode + multimodal wrapping; Phase 3 adds quota layer on top.
- `crates/forge_services/src/tool_services/{fs_read,fs_write,fs_search,fetch}.rs` — parity tool impls; Phase 3 wraps as `Tool` adapters.
- `crates/forge_app/src/infra.rs:141-165` — `CommandInfra::execute_command` / `execute_command_raw` — low-level exec primitives reused (or: replaced with direct `tokio::process` in Phase 3's marker-wrapping path).

### Sandbox (Phase 4 boundary)
- `crates/kay-sandbox-linux/` / `-macos/` / `-windows/` (crate shells exist from Phase 1) — Phase 3 defines the `Sandbox` trait, ships `NoOpSandbox`, does not touch per-OS enforcement code.
- `.planning/phases/999.1-windows-sandbox-hardening-research/` — existing research notes relevant to Phase 4.

### Rust toolchain + deps
- `Cargo.toml` (workspace) — **Phase 3 adds:** `portable-pty = "0.8"`, `subtle = "2"`, `rand` (already present); may add a sub-dep for shell quoting (`shell-escape` or `shlex`).
- `rust-toolchain.toml` — Rust stable 1.95, 2024 edition.
- `deny.toml` — verify `portable-pty` and `subtle` license compatibility (both MIT/Apache, pre-approved).

### External reference (fetch during planning / research)
- KIRA writeup (Terminus-KIRA harness techniques) — PROJECT.md cites this as the source for marker polling.
- ForgeCode public docs on tool JSON-schema format — informs `enforce_strict_schema` behavior verification.
- `portable-pty` crate: https://docs.rs/portable-pty — cross-platform PTY API.
- `subtle` crate: https://docs.rs/subtle — constant-time comparison primitives.
- OpenRouter tool-calling spec: https://openrouter.ai/docs#tool-calling — provider `tools` parameter format + `tool_choice` options.

### License / governance
- `NOTICE` + `crates/kay-core/NOTICE` + `ATTRIBUTIONS.md` — ForgeCode attribution preserved; no changes required.
- When `kay-tools` crate is created, add its `NOTICE` referencing the forge_* delegations (per Phase 2.5 precedent).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets (delegate, don't rewrite)
- `forge_app::utils::enforce_strict_schema` → OpenAI/OpenRouter schema hardening (D-02).
- `forge_app::ToolExecutor::execute` → dispatches `ToolCatalog` variants to `forge_services` — Phase 3 `Tool` adapters wrap this.
- `forge_app::ToolRegistry::call_with_timeout` → timeout + kill pattern (D-05 reuses).
- `forge_services::tool_services::image_read::ForgeImageRead` → base64 decode + multimodal formatter.
- `forge_services::tool_services::shell::ForgeShell` → existing non-PTY shell; Phase 3 takes the skeleton and bolts on marker protocol + PTY branch (effectively a rewrite but with behavior-parity grab-bag from the original).
- `forge_services::tool_services::fs_read.rs` / `fs_write.rs` / `fs_search.rs` / `fetch.rs` → parity adapters.
- `forge_domain::ToolDefinition` / `ToolName` / `ToolOutput` / `ToolCallContext` → re-export; no rebuild needed.

### Established patterns
- **`schemars::schema_for!(T)`** is the generation path throughout forge_*; Kay's `Tool::input_schema` returns a `&schemars::Schema` cached at construction.
- **Typed errors via `anyhow::Error` in forge_*; typed enums in kay-**. Kay's trait uses `Result<_, ToolError>` (D-09); internal calls to forge_app may return `anyhow::Error` which wraps into `ToolError::ExecutionFailed { source }`.
- **Arc-services pattern in forge_app** — `ToolExecutor<S>` takes `Arc<S>`. Kay's `Tool` impls hold `Arc<Services>` identically; no generic parameter on the trait (keeps it object-safe).
- **Tool timeouts come from `ForgeConfig.tool_timeout_secs`** — Kay reuses this config field rather than introducing a separate `kay.toml` timeout key.

### Integration points (where `kay-tools` connects)
- **Consumed by:** `kay-provider-openrouter` (needs `&[ToolDefinition]` for the provider `tools` parameter — currently Phase 2's translator emits tool calls but doesn't have a tool list to advertise; Phase 3 provides this), `kay-cli` (constructs the registry at boot; to be built in Phase 5), future `kay-core` agent loop (Phase 5).
- **Depends on:** `forge_domain`, `forge_app` (for `ToolExecutor` + `ToolRegistry` reuse), `forge_services` (tool_services), `forge_config`, plus new deps `portable-pty`, `subtle`.
- **NOT depended on by:** `kay-tauri` / `kay-tui` — they consume Phase 5's CLI event stream, not tools directly.

### Blocked by
- Nothing — Phase 2 closed; Phase 2.5 closed. Phase 3 is unblocked on code. Only external dependency is that TB 2.0 parity run (EVAL-01a, still tracked) is *not* a prerequisite; it runs post-Phase-5.

</code_context>

<specifics>
## Specific Ideas

- **KIRA's marker protocol is load-bearing for the TB 2.0 score** — PROJECT.md names it one of four harness techniques. Treat SHELL-01 + SHELL-05 as hard invariants for code review, not "nice to get right."
- **Parity-preserving delegation is the default** — wherever a ForgeCode tool exists and doesn't conflict with KIRA requirements (e.g., `fs_read`, `fs_write`, `fs_search`, `net_fetch`), the Kay `Tool` impl is a thin adapter that calls `forge_app::ToolExecutor::execute(ToolCatalog::Read(...), ctx)` verbatim. DO NOT rewrite service logic in `kay-tools`.
- **Schema hardening is non-negotiable** — every `Tool::input_schema()` emission must pass through `harden_tool_schema` before reaching the provider. Phase 3 planner should add a unit test that round-trips every registered tool's schema through the hardener and verifies `required` is sorted, contains all keys, and `additionalProperties: false`.
- **`#[non_exhaustive]` on every new public enum** — `ToolError`, `ToolOutputChunk`, `VerificationOutcome`, `CapScope`. Phase 5 / Phase 8 will extend these.
- **No `unwrap()` in marker detection or child-kill paths** — process-management bugs silently wedge the agent loop. Crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` per Phase 2 precedent.
- **Random nonce source: `rand::rngs::OsRng`** — cheap, high-entropy, and auditable. NOT `rand::thread_rng()` (PRNG, not cryptographically random).
- **Windows signal propagation is stopgap in Phase 3** — proper process-group killing lands with Job Objects in Phase 4 (sandbox). Phase 3 uses `TerminateProcess`; known limitation; test on Windows CI.

</specifics>

<deferred>
## Deferred Ideas

Items that surfaced during analysis but belong to other phases:

- **MCP (Model Context Protocol) tool integration** → not in Phase 3 REQUIREMENTS. Phase 2 flagged `rmcp` in v0.2.0 ship-blocker (issue #3); the MCP client rewire has no explicit phase yet — file as a new ROADMAP phase before v0.2.0.
- **Runtime end-user tool registration / plugin system** → Phase 12+ (plugins are explicitly v2 per PROJECT.md).
- **Agent delegation tool (`task`)** → requires Phase 5 agent loop first; the agent-executor path it needs (`forge_app::agent_executor`) is parity-imported but needs `AgentLoop` wiring.
- **`sem_search` (semantic codebase search)** → requires Phase 7 context engine (tree-sitter + embeddings).
- **`plan_create` + `todo_write` / `todo_read`** → orchestration-tier tools; cleaner to land in Phase 5 alongside the agent loop.
- **Per-model tool schema variants** — if Gemini or Anthropic lands in v2, `enforce_strict_schema`'s Gemini path (already in forge_app) will be needed; not Phase 3 scope.
- **Cost-aware tool throttling** (stop invoking expensive tools near cost cap) — natural extension but out-of-scope v1.
- **Screen-coverage tooling** (full-screen PTY apps like `less` / `vim` captured as animated output) — PTY support in D-04 is a prerequisite but the capture-to-frame tooling is a separate feature.
- **Marker protocol for `net_fetch` long polls** — `net_fetch` is synchronous in v1; streaming fetch with markers is a v2 idea.
- **Multi-perspective verifier impl** → Phase 8 owns this (D-06 leaves the trait in place).
- **Session-level tool output persistence** → Phase 6.

### Reviewed Todos (not folded)
None — no backlog items matched this phase at discussion time.

</deferred>

---

## Appendix A — Post-Phase-2.5 Realignment Reminder

All three substitution rules from Phase 2 CONTEXT.md Appendix A still apply to Phase 3 plans:
1. Replace `kay-core = { path = "../kay-core" }` with the specific forge_* sub-crates consumed. Expected `kay-tools` Cargo.toml deps:
   ```toml
   forge_domain      = { path = "../forge_domain" }
   forge_app         = { path = "../forge_app" }
   forge_services    = { path = "../forge_services" }
   forge_config      = { path = "../forge_config" }
   ```
   Plus workspace-pinned `async-trait`, `tokio`, `serde`, `serde_json`, `schemars`, `anyhow`, `tracing`, `rand`, plus NEW: `portable-pty`, `subtle`, `futures`.
2. `use kay_core::forge_X::Y;` → `use forge_X::Y;`.
3. `crates/kay-core/src/forge_X/Y.rs` doc refs → `crates/forge_X/src/Y.rs`.

Deviations during execution are recorded in plan SUMMARY.md, not in the plan itself.

---

*Phase: 03-tool-registry-kira-core-tools*
*Context gathered: 2026-04-20*
*Mode: auto-resolved per user "proceed autonomously, decide best options" directive; every decision traceable to prior phase artifacts or explicit REQ-IDs*
