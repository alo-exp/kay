# Phase 3: Tool Registry + KIRA Core Tools — Research

**Researched:** 2026-04-20
**Domain:** Rust async tool-trait layer + shell harness (KIRA marker protocol) + schema hardening wrap + provider integration
**Confidence:** HIGH on in-tree delegation surfaces (verified against source); HIGH on portable-pty API (Context7 verified); MEDIUM on KIRA-specific parameter choices (upper bound of SC ranges chosen by CONTEXT.md D-07, no independent KIRA writeup re-validation in this session — assumption carried from Phase 3 CONTEXT).

## Executive Summary

Phase 3 lifts Kay from "streaming text + tool-call frames" (Phase 2) to "agent can actually invoke tools, run shells with KIRA marker protocol, and verify completion." All 12 decisions in `03-CONTEXT.md` are locked; this research fills the gap between those decisions and a plannable build.

**What's known (and verified in-tree on 2026-04-20):**
- `forge_app::utils::enforce_strict_schema` at `crates/forge_app/src/utils.rs:351` is the canonical hardener — operates on `&mut serde_json::Value` (not `schemars::Schema`). Round-trip `Schema → Value → enforce → Value → Schema` is required. `[VERIFIED: source read]`
- `forge_app::ToolExecutor::execute(tool_input: ToolCatalog, context: &ToolCallContext) -> anyhow::Result<ToolOutput>` is the byte-parity delegation target — `crates/forge_app/src/tool_executor.rs:334`. `[VERIFIED: source read]`
- `ToolCatalog` variants needed by Phase 3: `Read(FSRead)`, `Write(FSWrite)`, `FsSearch(FSSearch)`, `Shell(Shell)`, `Fetch(NetFetch)`. Task/Plan/Todo/Skill/Patch/Remove/Undo/SemSearch/Followup deferred. `[VERIFIED: crates/forge_domain/src/tools/catalog.rs:41-61]`
- Workspace is 31 crates post-Phase-2.5; `crates/kay-tools/` does NOT exist yet. `[VERIFIED: crates/ listing]`
- `ToolCallContext { sender, metrics }` is ready for reuse; no rebuild needed. `[VERIFIED: crates/forge_domain/src/tools/call/context.rs]`
- `ForgeConfig.tool_timeout_secs` (u64, default 300) and `max_image_size_bytes` (u64) exist. `[VERIFIED: crates/forge_config/src/config.rs:147,151]`
- `AgentEvent` is already `#[non_exhaustive]` at `crates/kay-provider-openrouter/src/event.rs:15` — additive extensions are safe. `[VERIFIED: source read]`
- `ContentPart::ImageUrl { image_url: ImageUrl { url, detail } }` exists in `forge_app::dto::openai::request.rs:99-112` — Phase 2's request-body serializer (`translator.rs` Option-B OrderedObject build) does NOT currently emit image parts; a small extension is required. `[VERIFIED: source read]`
- `portable-pty = "0.8"` API is synchronous: `ChildKiller::kill`, `Child::wait`, `Child::try_wait`, master `try_clone_reader` → `Box<dyn Read + Send>`. Async integration uses `tokio::task::spawn_blocking` or `tokio::io::unix::AsyncFd` on the reader's raw FD. `[CITED: docs.rs/portable-pty/latest — Context7]`
- License gate: MIT/Apache-2.0 are already in `deny.toml` allow list; `subtle`, `portable-pty`, `nix`, `windows-sys` all qualify. `[VERIFIED: deny.toml]`

**What's new (Phase 3 must add):**
- A new `kay-tools` workspace crate (D-01).
- The marker protocol (D-03): `__CMDEND_<nonce>_<seq>__EXITCODE=N`.
- PTY fallback (D-04), timeout cascade (D-05), sandbox DI (D-12), verifier DI (D-06).
- Two `AgentEvent` variants (D-08): `ToolOutput { call_id, chunk }`, `TaskComplete { call_id, verified, outcome }`.
- One new workspace dep: `portable-pty` (subtle and rand are already in workspace — `rand = "0.10.0"` is newer than CONTEXT.md's speculative `"0.8"` reference; use workspace pin).

**Open planner decisions (flagged in §15):** image-content request-body extension shape, process-group semantics on Unix (setsid vs. rely on `kill_on_drop`), test-harness split, kay.toml namespace for new keys, exact ToolError variant coverage for `Pending` verifier flow.

**Primary recommendation:** Split Phase 3 into 5 plans (Wave 0 → Wave 4). Wave 1+2 can parallelize (crate skeleton + Tool trait independent of AgentEvent extensions). Wave 3 (marker shell) is the longest; Wave 4 (remaining tools + registry wiring + image-path extension) depends on it.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| `Tool` trait + `ToolRegistry` (object-safe, Arc<dyn Tool>) | `kay-tools` crate | — | New per D-01; keeps provider-agnostic tool abstraction separate from `forge_domain`'s closed-world `ToolCatalog` enum. |
| Built-in tool implementations (7 tools) | `kay-tools` crate | `forge_app::ToolExecutor` (delegation) | Parity preservation per D-10; thin adapters that construct `ToolCatalog::*` variants and call `ToolExecutor::execute`. |
| Schema hardening | `forge_app::utils::enforce_strict_schema` (unchanged) | `kay-tools::schema` (thin wrapper) | Reuse canonical impl per D-02 / TOOL-05. |
| Marker protocol shell wrap | `kay-tools::execute_commands` | `tokio::process` + `portable-pty` | Load-bearing for TB 2.0 score per PROJECT.md non-negotiable; cannot delegate to `forge_services::ForgeShell` because that uses `execute_command` (collect-then-return, no streaming). |
| Native provider `tools` emission | `kay-provider-openrouter` (consumer) | `kay-tools::ToolRegistry::tool_definitions()` | TOOL-06: provider owns the request-body, registry supplies `&[ToolDefinition]`. |
| `AgentEvent::ToolOutput` / `TaskComplete` emission | `kay-provider-openrouter` (owner of the enum) | `kay-tools` (producer via mpsc sender) | Phase 2 froze `AgentEvent` location; additive extensions live there. |
| Sandbox enforcement | `kay-tools::sandbox` trait (DI); `NoOpSandbox` impl in `kay-tools` | Phase 4 swaps in `kay-sandbox-*` impls | D-12: seam, not enforcement. |
| Task verification | `kay-tools::verifier::TaskVerifier` trait (DI); `NoOpVerifier` in `kay-tools` | Phase 8 swaps in real impl | D-06. |
| Image request-body extension | `kay-provider-openrouter::translator` (request-body builder) | `kay-tools::image_read` (buffers images into a turn-scoped context) | Phase 2 emits text-only; Phase 3 must extend the OrderedObject body builder to emit `content: [{type: image_url, ...}]` when images are pending. See §10. |

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** New workspace crate `kay-tools` owns the `Tool` trait (object-safe, async, `#[async_trait]`, `Arc<dyn Tool>`), `ToolRegistry` (`HashMap<ToolName, Arc<dyn Tool>>`), built-in impls, and `default_tool_set(...)` factory. Built-ins delegate to `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)` to preserve byte-identical parity.
- **D-02:** Reuse `forge_app::utils::enforce_strict_schema` unchanged. Kay wrapper `harden_tool_schema` appends truncation-reminder description fields only (no algorithmic change).
- **D-03:** Marker `__CMDEND_<128-bit-hex-nonce>_<seq>__EXITCODE=N` via `rand::rngs::OsRng` nonce + per-session `AtomicU64` seq. Match via `subtle::ConstantTimeEq`.
- **D-04:** Default `tokio::process`; `portable-pty = "0.8"` fallback on heuristic denylist (ssh, sudo, docker -it, less, more, vim, nvim, nano, top, htop, python -i, node -i, watch) OR explicit `tty: true` in tool args.
- **D-05:** Reuse `ForgeConfig.tool_timeout_secs`. Unix: SIGTERM → 2s grace → SIGKILL → `child.wait().await`. Windows: `TerminateProcess` (Phase 4 Job Objects supersede).
- **D-06:** `NoOpVerifier` returns `VerificationOutcome::Pending`. Phase 8 swaps via `TaskVerifier` trait DI.
- **D-07:** `image_read` per-turn cap = 2, per-session cap = 20 (toml + env overridable).
- **D-08:** Additive `AgentEvent::ToolOutput { call_id, chunk: ToolOutputChunk }` + `AgentEvent::TaskComplete { call_id, verified, outcome }` (safe under existing `#[non_exhaustive]`).
- **D-09:** Dedicated `ToolError` enum in `kay-tools`, separate from `ProviderError`.
- **D-10:** 7 tools: KIRA trio (`execute_commands`, `task_complete`, `image_read`) + parity minimum (`fs_read`, `fs_write`, `fs_search`, `net_fetch`). All other parity tools deferred to Phase 5+.
- **D-11:** `kay-cli` builds immutable registry at startup via `default_tool_set(...)`. No runtime mutation (HashMap, not DashMap).
- **D-12:** `Arc<dyn Sandbox>` DI seam; Phase 3 ships `NoOpSandbox` (in `kay-tools::sandbox` for Phase 3; Phase 4 may move to `kay-sandbox-core`).

### Claude's Discretion
- Module layout inside `kay-tools` (sub-module per tool vs. single `tools.rs`).
- Whether `harden_tool_schema` mutates in place or returns owned value.
- Test harness for marker-race scenarios (mock stdout streams vs. fork-a-subshell).
- `ToolCallContext` Kay-owned vs. re-export of `forge_domain::ToolCallContext`.
- Whether `execute_commands` emits a distinct `AgentEvent::ToolCallStart`-analog frame at invocation time.
- Unit-test split between `kay-tools/src/` and `kay-tools/tests/`.
- `subtle` import path (direct vs. transitive).
- Default `tool_timeout_secs` override for Kay.

### Deferred Ideas (OUT OF SCOPE)
- MCP tool integration (no ROADMAP phase yet — file before v0.2.0).
- Runtime end-user tool registration / plugin system (Phase 12+).
- Agent delegation `task` tool (requires Phase 5 agent loop).
- `sem_search` (requires Phase 7 context engine).
- `plan_create`, `todo_write`, `todo_read` (Phase 5 orchestration).
- Per-model tool schema variants (Gemini/Anthropic — v2).
- Cost-aware tool throttling.
- Screen-coverage tooling (full-screen PTY app capture).
- Marker protocol for `net_fetch` long polls.
- Multi-perspective verifier impl (Phase 8).
- Session-level tool output persistence (Phase 6).
- Full process-group SIGTERM propagation on Windows (Phase 4 Job Objects).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|---|---|---|
| TOOL-01 | `Tool` trait object-safe, async, `Arc<dyn Tool>` map | §1 crate skeleton; §2 delegation surface (ToolCatalog mapping) |
| TOOL-02 | `execute_commands` executes shell in project-root sandbox | §4 marker protocol wrap; §6 timeout cascade; §9 Sandbox trait |
| TOOL-03 | `task_complete` triggers multi-perspective verification before signaling completion | §8 TaskVerifier trait + NoOpVerifier; §7 AgentEvent::TaskComplete emission |
| TOOL-04 | `image_read` reads base64 terminal screenshots (multimodal) | §10 image caps + request-body extension; §11 deps |
| TOOL-05 | Tool schemas via ForgeCode hardening post-process | §3 schema hardening wrap |
| TOOL-06 | Tool calls flow through provider native `tools` parameter (no ICL parsing) | §1 registry `tool_definitions()`; §2 consumer integration |
| SHELL-01 | `__CMDEND__<seq>__` marker polling; agent advances as soon as marker observed | §4 marker protocol wrap (load-bearing) |
| SHELL-02 | `tokio::process` default; `portable-pty` fallback | §5 PTY fallback heuristic |
| SHELL-03 | Output streamed as `AgentEvent::ToolOutput` frames (no blocking reads) | §4 marker protocol + §7 AgentEvent::ToolOutput frames |
| SHELL-04 | Hard timeout configurable; clean termination (signal propagation + zombie reap) | §6 timeout cascade |
| SHELL-05 | Marker races detected and recovered without loop corruption | §4 marker protocol threat model; §Threat Model §1 |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

| Directive | Phase 3 Implication |
|---|---|
| Forked ForgeCode parity gate (EVAL-01) — byte-parity required on TB 2.0 | Built-in tools delegate through `forge_app::ToolExecutor` unchanged per D-10; no reimplementation of service logic in `kay-tools`. |
| DCO `Signed-off-by` on every commit | Every plan commit includes `git commit -s`. |
| Single merged Rust binary — no `externalBin` sidecar | `kay-tools` is a library crate linked into `kay-cli` (main binary); no separate binary. |
| Strict OpenRouter model allowlist | N/A directly; confirmed Phase 2 allowlist already in place. |
| ForgeCode's JSON-schema hardening is load-bearing | §3: reuse `enforce_strict_schema` unchanged (D-02). |
| Never commit directly to `main`; branches + PRs | Per standing workflow. |
| `rm -rf`, `git reset --hard`, `git clean` require explicit authorization | N/A for research; planner/executor must observe. |

## Standard Stack

### Core (new workspace dependencies added in Phase 3)

| Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `portable-pty` | 0.8 | Cross-platform PTY for TTY-requiring commands (SHELL-02) | De-facto Rust PTY crate; used by wezterm; MIT license; Context7 rep 90.9. [VERIFIED: Context7 /websites/rs_portable-pty]. Versions: 0.8.x is current stable (April 2026 — verify `cargo search portable-pty` before pinning). |
| `subtle` | 2.x (workspace pinned) | Constant-time comparison for marker nonce (SHELL-05) | Canonical in Rust crypto ecosystem (dalek-cryptography); avoids timing-channel leakage. |

### Supporting (workspace deps already present — verify)

| Library | Version | Purpose | When to Use |
|---|---|---|---|
| `rand` | 0.10.0 [VERIFIED: Cargo.toml:162] | `OsRng` nonce generation | Per-tool-call marker nonce. NOTE: CONTEXT.md D-03 references `rand = "0.8"`; the workspace already pins `0.10.0` — use workspace pin. |
| `async-trait` | 0.1 [VERIFIED: Cargo.toml:81] | Object-safe async `Tool` trait | TOOL-01 object-safety is load-bearing; without it, `Arc<dyn Tool>` is impossible. |
| `tokio` | 1.51 [VERIFIED: Cargo.toml:53] | async runtime, `tokio::process`, `io-util`, `signal`, `sync`, `time` | Default process execution + timeouts. Already feature-enabled. |
| `schemars` | 1.2 [VERIFIED: Cargo.toml:59] | JSON Schema generation for tool inputs | `schemars::schema_for!(T)` — unchanged from ForgeCode. |
| `serde_json` | 1 [VERIFIED: Cargo.toml:58] | JSON argument parsing + schema mutation | `enforce_strict_schema` operates on `serde_json::Value`. |
| `anyhow` | 1 [VERIFIED: Cargo.toml:63] | Downstream error bridging | `ToolExecutor::execute` returns `anyhow::Result<ToolOutput>`; Kay wraps into `ToolError::ExecutionFailed { source }`. |
| `thiserror` | 2 [VERIFIED: Cargo.toml:64] | `ToolError` typed enum | D-09. |
| `tracing` | 0.1 [VERIFIED: Cargo.toml:62] | Structured logging | Per Phase 2 convention. |
| `futures` | 0.3 [VERIFIED: Cargo.toml:82] | Stream combinators for stdout/stderr merge | Marker-protocol streaming path. |
| `hex` | 0.4 [VERIFIED: Cargo.toml:146] | Nonce hex encoding | 128-bit → 32-char hex. |

### Platform-Specific (conditional deps)

| Library | Target | Purpose |
|---|---|---|
| `nix = "0.29"` (NEW — not yet in workspace) | Unix | `signal::kill(pid, Signal::SIGTERM)` — `tokio::process::Child::start_kill` sends SIGKILL on Unix by default, NOT SIGTERM. SIGTERM with grace requires explicit `nix::sys::signal::kill`. [VERIFIED: tokio docs + source inspection]. Alternative: use raw `libc::kill` (already a transitive dep via `portable-pty`) to avoid adding nix. Planner decides. |
| `windows-sys = "0.61"` [VERIFIED: Cargo.toml:187] | Windows | `TerminateProcess` is already reachable via `windows-sys` which is in workspace with `Win32_System_Console` — Phase 3 also needs `Win32_System_Threading`. Small feature-flag addition. |

### Alternatives Considered (rejected by CONTEXT.md)

| Instead of | Could Use | Tradeoff |
|---|---|---|
| `enforce_strict_schema` reuse (D-02) | Roll Kay-side hardener | Drift from ForgeCode parity across upstream syncs. Rejected. |
| `__CMDEND_<nonce>_<seq>__` (D-03) | Static `__CMDEND__` | Trivially forgeable by prompt injection. Rejected. |
| `portable-pty` (D-04) | `pty-process` | Smaller API surface but lower ecosystem adoption; less cross-platform. Rejected. |
| Dedicated trait in `kay-tools` (D-01) | Extend `ToolCatalog` enum | Breaks object-safety; forces kay-tool compilation into `forge_domain`. Rejected. |

**Installation (new workspace deps to add in `Cargo.toml [workspace.dependencies]`):**
```toml
portable-pty = "0.8"
subtle = "2"
# nix = "0.29"  # Unix only — planner decides vs. raw libc
```

**Version verification:** Planner must run `cargo search portable-pty`, `cargo search subtle`, `cargo search nix` at plan-write time to confirm latest 0.8.x / 2.x / 0.29.x point releases. `[ASSUMED]` 0.8 is current major for portable-pty — if 0.9+ landed, planner adjusts.

## Architecture Patterns

### System Architecture Diagram

```
                    ┌─────────────────────────────┐
                    │    kay-cli (main binary)    │
                    │  - build ForgeConfig + Svc  │
                    │  - default_tool_set(...)    │──────┐
                    └──────────────┬──────────────┘      │
                                   │                     │
                                   ▼                     │
    ┌──────────────────────────────────────────────────┐ │
    │             kay-tools::ToolRegistry               │ │
    │  HashMap<ToolName, Arc<dyn Tool>> (immutable)     │ │
    │  fn tool_definitions() -> Vec<ToolDefinition>  ───┼─┼──► passed to kay-provider-openrouter::
    │  fn get(name) -> Option<&Arc<dyn Tool>>           │ │        OpenRouterProvider::chat(req)
    └──────┬───────────────────────────────────────────┘ │
           │                                             │
           ▼                                             │
    ┌──────────────────────────────────────────────────┐ │
    │  Arc<dyn Tool> (7 built-ins) — trait object      │ │
    │  async fn invoke(args, ctx) -> Result<…, ToolErr>│ │
    └──────┬───────────────────────────────────────────┘ │
           │                                             │
           ├── FsReadTool/FsWriteTool/FsSearchTool/      │
           │   NetFetchTool ──► ToolExecutor::execute(   │
           │                     ToolCatalog::Read(…))   │
           │                                             │
           ├── ExecuteCommandsTool (marker protocol)     │
           │      │                                      │
           │      ├─► Sandbox::check_shell(cmd,cwd)      │
           │      ├─► PickShell: tokio::process   OR     │
           │      │              portable-pty (D-04)     │
           │      ├─► Spawn with wrapped command +       │
           │      │   __CMDEND_<nonce>_<seq>__ tail      │
           │      ├─► Two BufReader tasks (stdout+err)   │
           │      │     line-by-line, no buffering       │
           │      │         │                            │
           │      │         ▼                            │
           │      │   subtle::ConstantTimeEq check       │
           │      │   on marker prefix per line          │
           │      │         │                            │
           │      │         ▼                            │
           │      │   mpsc::Sender<AgentEvent> ─────────►┼─ to translator / event pipe
           │      │     ToolOutput{call_id, Stdout(l)}   │   (back into AgentEvent stream
           │      │     ToolOutput{call_id, Stderr(l)}   │    shared with Phase 2)
           │      │     ToolOutput{call_id, Closed{…}}   │
           │      └─► Timeout cascade (D-05):            │
           │             SIGTERM → 2s wait → SIGKILL     │
           │                                             │
           ├── ImageReadTool                             │
           │      ├─► check per-turn/session quota       │
           │      ├─► forge_services::ForgeImageRead     │
           │      └─► attach base64 payload to           │
           │          ImageContext (turn-scoped) ───────►┼─► extend request body next turn
           │                                             │    (§10)
           └── TaskCompleteTool                          │
                  ├─► NoOpVerifier::verify(...)          │
                  └─► emit AgentEvent::TaskComplete{...} ┘
                         (Phase 5 agent loop reads this)
```

### Recommended Project Structure

```
crates/kay-tools/
├── Cargo.toml
├── NOTICE                     # ForgeCode delegation attribution (Phase 2.5 precedent)
├── src/
│   ├── lib.rs                 # crate root, #![deny(clippy::unwrap_used, clippy::expect_used)]
│   ├── tool.rs                # #[async_trait] pub trait Tool
│   ├── registry.rs            # ToolRegistry + tool_definitions()
│   ├── error.rs               # #[non_exhaustive] pub enum ToolError
│   ├── schema.rs              # harden_tool_schema() + hardened_description()
│   ├── sandbox.rs             # trait Sandbox + NoOpSandbox + SandboxDenial
│   ├── verifier.rs            # trait TaskVerifier + NoOpVerifier + VerificationOutcome
│   ├── default_set.rs         # default_tool_set(services, sandbox, verifier) -> ToolRegistry
│   ├── quota.rs               # ImageQuota (per-turn + per-session atomic counters)
│   ├── markers/
│   │   ├── mod.rs             # wrap_command + scan_line (+ subtle compare)
│   │   └── shells.rs          # per-OS wrap templates (bash/zsh/sh/powershell/cmd)
│   └── builtins/
│       ├── mod.rs
│       ├── fs_read.rs         # FsReadTool — delegates to ToolExecutor::execute(Read)
│       ├── fs_write.rs        # FsWriteTool — delegates to ToolExecutor::execute(Write)
│       ├── fs_search.rs       # FsSearchTool — delegates to ToolExecutor::execute(FsSearch)
│       ├── net_fetch.rs       # NetFetchTool — delegates to ToolExecutor::execute(Fetch)
│       ├── execute_commands.rs # ExecuteCommandsTool (NEW: marker + PTY + timeout)
│       ├── image_read.rs      # ImageReadTool — delegates + quota + ImageContext attach
│       └── task_complete.rs   # TaskCompleteTool — NoOpVerifier hook
└── tests/
    ├── registry_integration.rs  # all 7 tools round-trip harden → emit ToolDefinition
    ├── marker_streaming.rs      # subshell fork, assert Stdout frames + Closed{exit,true}
    ├── marker_race.rs           # fake marker in command output, assert no early close
    ├── timeout_cascade.rs       # sleeping subshell, assert SIGTERM→SIGKILL→reap
    ├── image_quota.rs           # 3rd call in turn returns ImageCapExceeded
    └── schema_hardening_property.rs  # proptest: every tool schema has sorted `required`
```

Naming: matches `kay-*` workspace convention (Phase 2.5 precedent — `kay-provider-openrouter`). Crate is hyphenated workspace name → underscored `kay_tools` Rust identifier.

### Pattern 1: Object-Safe Async Trait with Delegation
**What:** Trait returns `Result<ToolOutput, ToolError>`; built-in impls hold `Arc<Services>` and call `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)`; `anyhow::Error` from delegate wraps into `ToolError::ExecutionFailed { source }`.
**When to use:** All 4 parity tools (`fs_read`, `fs_write`, `fs_search`, `net_fetch`).

```rust
// Source: synthesized from CONTEXT.md D-01 + forge_app::ToolExecutor inspection
#[async_trait::async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &ToolName;
    fn description(&self) -> &str;
    fn input_schema(&self) -> &schemars::Schema;
    async fn invoke(&self, args: serde_json::Value, ctx: &ToolCallContext)
        -> Result<ToolOutput, ToolError>;
}

pub struct FsReadTool<S> {
    services: Arc<S>,
    sandbox: Arc<dyn Sandbox>,
    name: ToolName,
    description: String,
    schema: schemars::Schema,    // cached, harden_tool_schema applied at construction
}

#[async_trait::async_trait]
impl<S: /* ToolExecutor bounds */> Tool for FsReadTool<S> {
    fn name(&self) -> &ToolName { &self.name }
    fn description(&self) -> &str { &self.description }
    fn input_schema(&self) -> &schemars::Schema { &self.schema }

    async fn invoke(&self, args: Value, ctx: &ToolCallContext)
        -> Result<ToolOutput, ToolError>
    {
        let input: forge_domain::FSRead = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() })?;

        // sandbox check (NoOp in Phase 3)
        self.sandbox.check_fs_read(Path::new(&input.file_path)).await
            .map_err(|d| ToolError::SandboxDenied { tool: self.name.clone(), reason: d.reason })?;

        let executor = forge_app::ToolExecutor::new(self.services.clone());
        executor.execute(ToolCatalog::Read(input), ctx).await
            .map_err(|e| ToolError::ExecutionFailed { tool: self.name.clone(), source: e })
    }
}
```

### Pattern 2: Streaming Shell with Marker Detection
**What:** ExecuteCommandsTool owns the child process; spawns two `BufReader::lines()` tasks, emits every non-marker line as `AgentEvent::ToolOutput::Stdout/Stderr` on an mpsc sender, detects the marker via `subtle::ConstantTimeEq`, emits `Closed{exit_code, marker_detected}` and returns from `invoke`.
**When to use:** `execute_commands` only (and any future streaming tool).

See §4 for full shell wrapping template and Rust sketch.

### Pattern 3: Dependency Injection Seams
**What:** `Arc<dyn Sandbox>`, `Arc<dyn TaskVerifier>` constructor params on tools that need them; Phase 3 ships NoOp impls; Phase 4/8 swap without touching tool code.
**When to use:** `execute_commands` (sandbox), `task_complete` (verifier), `fs_read/fs_write/net_fetch` (sandbox).

### Anti-Patterns to Avoid
- **Using `unwrap()` / `expect()` in marker scan or child-kill paths:** wedges agent loop silently. Enforce via crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` (Phase 2 precedent).
- **Walking `serde_json::Value` twice for hardening:** `enforce_strict_schema` already walks the tree; Kay's truncation-reminder injection should append to `description` at the TOP level only (shallow mutation), not recurse.
- **Using `rand::thread_rng()` for nonce:** PRNG, not cryptographic. Always `OsRng`.
- **Using `std::string::String::eq` for marker compare:** timing-channel leak. Always `subtle::ConstantTimeEq`.
- **Buffering shell output:** violates SHELL-03. Emit per-line immediately.
- **Re-implementing `ForgeShell::execute`:** breaks parity. But for marker-protocol execution, there's no existing parity behavior — this is genuinely new (see §4).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| JSON schema hardening | Bespoke walker for `required`/`additionalProperties`/`allOf` | `forge_app::utils::enforce_strict_schema` (D-02) | Canonical ForgeCode impl; parity gate EVAL-01. [VERIFIED: crates/forge_app/src/utils.rs:351] |
| fs_read/fs_write/fs_search/net_fetch logic | Duplicate filesystem + HTTP logic | `forge_app::ToolExecutor::execute` (D-10) | Parity preservation; existing tests cover edge cases. [VERIFIED: crates/forge_app/src/tool_executor.rs] |
| Image decode + MIME handling | base64/MIME classification | `forge_services::tool_services::image_read::ForgeImageRead` | Already supports JPEG/PNG/WebP/GIF with size checks. [VERIFIED: image_read.rs:16-24] |
| Tolerant tool-call JSON parsing | Custom recovery | Already shipped in Phase 2 via `forge_json_repair` fallback in translator | Phase 3 consumes `AgentEvent::ToolCallComplete { arguments: Value }` — translator did repair. |
| Tool-call reassembly across SSE deltas | Custom buffer | Phase 2 `translator.rs` ToolCallBuilder | Already working. |
| PTY cross-platform abstractions | ConPTY / openpty ifdef mess | `portable-pty = "0.8"` (D-04) | De-facto Rust PTY crate. [CITED: docs.rs/portable-pty — Context7 verified] |
| Constant-time string compare | Hand-rolled eq loops | `subtle::ConstantTimeEq` (D-03) | Timing-channel safety, audited. |
| Random nonce | /dev/urandom by hand | `rand::rngs::OsRng` (D-03) | Cross-platform cryptographic source. |
| Child-process timeout + grace | ad-hoc sleep loops | `tokio::time::timeout` + `tokio::process::Child::{start_kill, wait}` + explicit `nix::sys::signal::kill(SIGTERM)` first | Tokio's `start_kill` sends SIGKILL directly on Unix — SIGTERM needs nix or libc. |

**Key insight:** Phase 3 is a composition problem, not a greenfield implementation problem. The only genuinely-new code is (a) the trait + registry + errors, (b) the marker protocol (which legitimately replaces `ForgeShell::execute` for streaming semantics, not for tool dispatch), and (c) the DI shims (sandbox + verifier). Everything else wraps existing parity code.

## Runtime State Inventory

_(Greenfield phase — no renames, no data migrations. This section is not required per researcher template.)_

## Common Pitfalls

### Pitfall 1: `tokio::process::Child::start_kill` sends SIGKILL on Unix, not SIGTERM
**What goes wrong:** Applying D-05 naively with `child.start_kill()?; tokio::time::sleep(2s).await; child.kill().await?;` skips the 2-second grace period because `start_kill` already killed the child.
**Why it happens:** Tokio's docs for `Child::start_kill` — "Attempts to forcefully kill the child … On Unix this is equivalent to `libc::kill(pid, SIGKILL)`."
**How to avoid:** Use `nix::sys::signal::kill(Pid::from_raw(child.id().unwrap() as i32), Signal::SIGTERM)` first; only after the 2s wait times out call `child.start_kill()` (SIGKILL); then `child.wait().await` for reap.
**Warning signs:** Long-running commands in tests finish in <2s on timeout rather than ~2s; no observable SIGTERM signal received by test child.

### Pitfall 2: Schema round-trip loss through `schemars::Schema → Value → Schema`
**What goes wrong:** `Schema::try_from(value)` can re-order fields or lose metadata if the JSON doesn't match expected shape exactly.
**Why it happens:** `enforce_strict_schema` takes `&mut Value`; it's not schema-aware.
**How to avoid:** Keep the final hardened form as `serde_json::Value` inside the tool struct; expose via `Tool::input_schema() -> &Value` OR keep a `Schema` adapter that internally holds `Value` and delegates Serialize through it. CONTEXT.md D-02 sketch `*schema = Schema::try_from(v).expect("re-roundtrip")` uses `.expect()` — planner MUST propagate through `ToolError::SchemaHardeningFailed` (new variant) or panic-on-construction only (`new()` returns `Result`, registry is immutable post-build).
**Warning signs:** Unit test `assert_eq!(original_schema, harden(original_schema))` round-trip fails for tools with unusual schema shapes (nested `oneOf`, `$ref`).

### Pitfall 3: Marker detection under partial-line stdout
**What goes wrong:** If the child writes the marker mid-line (no trailing newline before exit), `BufReader::lines()` may not yield it.
**Why it happens:** `lines()` is newline-terminated; the final unterminated line is yielded only on EOF.
**How to avoid:** D-03's wrap template prepends `\n` before the marker (`printf '\n__CMDEND_%s_%d__EXITCODE=%d\n' ...`). This guarantees a newline-terminated marker line. Additionally, on EOF, the line reader should emit any residual buffer as Stdout(chunk) with a trailing `Closed` event even if the marker wasn't matched — that's the "marker race" recovery path (SHELL-05).
**Warning signs:** Tests that don't include leading `\n` in the wrapper template intermittently fail.

### Pitfall 4: PTY + marker — `portable-pty` is synchronous
**What goes wrong:** `portable-pty`'s `Child::wait` and `MasterPty::try_clone_reader` return `IoResult` / `Box<dyn Read + Send>` — blocking. Using them directly in an async function blocks the runtime.
**Why it happens:** The crate intentionally avoids tying to any runtime.
**How to avoid:** Wrap reader in `tokio::task::spawn_blocking` that pushes lines onto a `tokio::sync::mpsc::Sender<String>`; on `await` side read from the receiver. Similarly, `child.wait()` goes in `spawn_blocking`. Alternative: `tokio::io::unix::AsyncFd` on `reader.as_raw_fd()` — only on Unix, adds complexity; planner decides.
**Warning signs:** Agent loop hangs; `cargo test` with `--test-threads=1` deadlocks; PTY test times out.

### Pitfall 5: `arguments` JSON may be `null` or empty
**What goes wrong:** Phase 2's `AgentEvent::ToolCallComplete { arguments: Value }` can be `Value::Null` for tools declared with `"arguments": null` shape (some models emit this).
**Why it happens:** OpenRouter variance + `forge_json_repair` tolerance.
**How to avoid:** Every `Tool::invoke` impl deserializes args defensively: `if args.is_null() { args = Value::Object(Map::new()); }` before `serde_json::from_value::<InputType>(args)`. For tools with optional fields (all Phase 3 tools), this makes no-arg calls work.
**Warning signs:** Integration tests with `{}` or null argument shape fail with "missing field" errors.

### Pitfall 6: `#[non_exhaustive]` on `AgentEvent` is at the enum level, not per-variant
**What goes wrong:** Adding a new variant with named fields is safe, but adding a new field to an EXISTING variant requires `#[non_exhaustive]` at the variant level.
**Why it happens:** Rust's `#[non_exhaustive]` semantics.
**How to avoid:** D-08 is purely additive (new variants only). `ToolOutputChunk` is a NEW enum and must be marked `#[non_exhaustive]` itself so Phase 5 can add chunks (e.g., `ResizeHint`) later. Same for `VerificationOutcome`.
**Warning signs:** Phase 5/8 needs to extend `ToolOutputChunk` or `VerificationOutcome` and hits breakage.

### Pitfall 7: `execute_commands` cwd normalization vs. absolute-path expectations
**What goes wrong:** `forge_app::ToolExecutor::execute(ToolCatalog::Shell(...))` normalizes cwd via `services.get_environment().cwd`. Kay's `ExecuteCommandsTool` bypasses `ToolExecutor` (it needs streaming, not collect) — so cwd normalization must be replicated manually.
**Why it happens:** The marker-protocol path is genuinely new; it doesn't go through `ToolExecutor::execute`.
**How to avoid:** Before spawning, the tool calls `services.get_environment().cwd` and joins relative `input.cwd` the same way `ToolExecutor::normalize_path` does. Planner: extract `normalize_path` into a shared helper OR copy the 6-line logic.
**Warning signs:** Relative-cwd tests fail on one OS but not another.

### Pitfall 8: schemars version pin + `Schema::try_from(Value)`
**What goes wrong:** schemars 1.2.x introduced `Schema::try_from<Value>` — earlier 0.8.x had `Schema::Object(SchemaObject)`. If any in-tree code pins 0.8 transitively, `Schema::try_from` won't compile.
**Why it happens:** Parity-imported `forge_domain` uses `schemars = 1.2` per workspace pin. [VERIFIED: Cargo.toml:59]. Should be safe but verify before implementation.
**How to avoid:** Write harden wrapper against the workspace pin; planner runs `cargo tree | grep schemars` to confirm no duplicate versions.
**Warning signs:** `cargo check` reports `error[E0277]: the trait bound \`Schema: TryFrom<Value>\` is not satisfied`.

## Code Examples

### Harden wrapper (D-02 realization)

```rust
// crates/kay-tools/src/schema.rs
// Source: synthesized from CONTEXT.md D-02 + forge_app::utils::enforce_strict_schema inspection
use schemars::Schema;
use serde_json::Value;

/// Append KIRA-style truncation reminder to the top-level description.
/// Does NOT recurse — recursion is owned by enforce_strict_schema.
pub fn harden_tool_schema(
    schema: &mut Value,
    hints: &TruncationHints,
) {
    forge_app::utils::enforce_strict_schema(schema, true);

    if let Some(obj) = schema.as_object_mut() {
        if let Some(note) = hints.output_truncation_note.as_ref() {
            let desc_key = "description".to_string();
            match obj.get_mut(&desc_key) {
                Some(Value::String(existing)) => {
                    existing.push_str("\n\n");
                    existing.push_str(note);
                }
                _ => {
                    obj.insert(desc_key, Value::String(note.clone()));
                }
            }
        }
    }
}

#[derive(Default)]
pub struct TruncationHints {
    /// e.g., "Long outputs are truncated — narrow the command or grep."
    pub output_truncation_note: Option<String>,
}
```

Cost: 3 extra lines of code above `enforce_strict_schema` — zero drift risk.

### Marker wrap template (per D-03)

```rust
// crates/kay-tools/src/markers/shells.rs
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MarkerContext {
    pub nonce_hex: String,    // 32 hex chars (128-bit)
    pub seq: u64,
    pub line_prefix: String,  // precomputed "__CMDEND_{nonce_hex}_{seq}__"
}

impl MarkerContext {
    pub fn new(counter: &AtomicU64) -> Self {
        let mut nonce_bytes = [0u8; 16];
        OsRng.try_fill_bytes(&mut nonce_bytes).expect("OsRng never fails");
        let nonce_hex = hex::encode(nonce_bytes);
        let seq = counter.fetch_add(1, Ordering::Relaxed);
        let line_prefix = format!("__CMDEND_{nonce_hex}_{seq}__");
        Self { nonce_hex, seq, line_prefix }
    }

    /// EXITCODE tail format — stable, used for scanning.
    pub fn exit_tail(&self) -> String {
        format!("{}EXITCODE=", self.line_prefix)
    }
}

pub fn wrap_unix_sh(user_cmd: &str, m: &MarkerContext) -> String {
    // Subshell guarantees exit-code propagation even for multi-command scripts.
    // Leading newline guarantees marker is a fresh line.
    format!(
        "( {user_cmd}\n) ; __KAY_EXIT=$? ; printf '\\n__CMDEND_%s_%d__EXITCODE=%d\\n' '{}' {} \"$__KAY_EXIT\"",
        m.nonce_hex, m.seq
    )
}

pub fn wrap_windows_ps(user_cmd: &str, m: &MarkerContext) -> String {
    // PowerShell equivalent; $LASTEXITCODE replaces $?
    format!(
        "& {{ {user_cmd} ; $kay_exit = $LASTEXITCODE }} ; \
         Write-Host \"`n__CMDEND_{}_{}__EXITCODE=$kay_exit\"",
        m.nonce_hex, m.seq
    )
}
```

### Marker scan with constant-time compare

```rust
// crates/kay-tools/src/markers/mod.rs
use subtle::ConstantTimeEq;

pub fn scan_line(line: &str, m: &MarkerContext) -> ScanResult {
    // Look for the prefix anywhere at start of line
    if !line.starts_with("__CMDEND_") {
        return ScanResult::NotMarker;
    }
    // Expected prefix len = "__CMDEND_" + 32 hex + "_" + seq_digits + "__"
    let expected_prefix = m.line_prefix.as_bytes();
    let actual_head = &line.as_bytes()[..expected_prefix.len().min(line.len())];
    if actual_head.ct_eq(expected_prefix).unwrap_u8() == 0 {
        // Looks like a CMDEND but wrong nonce — SHELL-05: potential injection
        return ScanResult::ForgedMarker;
    }
    // Parse EXITCODE=N
    let tail = &line[expected_prefix.len()..];
    if let Some(num_str) = tail.strip_prefix("EXITCODE=") {
        if let Ok(n) = num_str.trim_end().parse::<i32>() {
            return ScanResult::Marker { exit_code: n };
        }
    }
    ScanResult::ForgedMarker
}

pub enum ScanResult {
    NotMarker,
    Marker { exit_code: i32 },
    ForgedMarker,  // SHELL-05: surface via trace; continue stream
}
```

### Timeout cascade (D-05, Unix with nix)

```rust
// crates/kay-tools/src/builtins/execute_commands.rs (Unix section)
use std::time::Duration;
use tokio::process::Child;
#[cfg(unix)]
use nix::sys::signal::{kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

async fn terminate_with_grace(
    child: &mut Child,
    timeout_secs: u64,
) -> Result<std::process::ExitStatus, ToolError> {
    #[cfg(unix)]
    {
        if let Some(pid_u32) = child.id() {
            let _ = kill(Pid::from_raw(pid_u32 as i32), Signal::SIGTERM);
        }
        match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
            Ok(Ok(status)) => return Ok(status),
            Ok(Err(e)) => return Err(ToolError::Io(e)),
            Err(_) => { /* grace expired — fall through to SIGKILL */ }
        }
        child.start_kill().map_err(ToolError::Io)?;  // SIGKILL on Unix
        child.wait().await.map_err(ToolError::Io)
    }
    #[cfg(windows)]
    {
        child.start_kill().map_err(ToolError::Io)?;  // TerminateProcess
        child.wait().await.map_err(ToolError::Io)
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| ICL-style "parse model's free text for tool calls" | Native provider `tools` parameter (OpenAI tool-calling spec) | OpenAI 2023; OpenRouter proxy since | Kay MUST use native per TOOL-06 — already implemented in Phase 2. |
| Closed enum for tool catalog (`ToolCatalog`) | Object-safe `Arc<dyn Tool>` trait-object map | KIRA pattern (2024-2025) | D-01: Kay splits — keeps `ToolCatalog` for parity dispatch, adds `Arc<dyn Tool>` for extension. |
| Blocking `Command::output()` with fixed timeout | Streaming `tokio::process` + marker protocol for long-running commands | Terminus-KIRA (2024) writeup | Load-bearing for TB 2.0 score; D-03 unforgeable marker. |
| Static `__CMDEND__` string | Crypto-random per-call nonce + constant-time compare | KIRA hardening | D-03: prompt-injection resistant. |
| `rand::thread_rng()` | `rand::rngs::OsRng` | Security best practice | PRNGs aren't cryptographic random; OsRng delegates to `getrandom()` / BCryptGenRandom. |

**Deprecated/outdated (not applicable to Phase 3):**
- schemars 0.8 API (we're on 1.2; `Schema::try_from` works).

---

## Section 1 — `kay-tools` Crate Skeleton

**Cargo.toml path-deps (Phase 2.5 Appendix-A realigned):**

```toml
[package]
name = "kay-tools"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true

[dependencies]
forge_domain   = { path = "../forge_domain" }
forge_app      = { path = "../forge_app" }
forge_services = { path = "../forge_services" }
forge_config   = { path = "../forge_config" }
kay-provider-openrouter = { path = "../kay-provider-openrouter" }  # for AgentEvent, ToolOutputChunk

async-trait = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
schemars    = { workspace = true }
anyhow      = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
rand        = { workspace = true }
hex         = { workspace = true }
futures     = { workspace = true }

# NEW workspace deps (add to top-level Cargo.toml [workspace.dependencies])
portable-pty = { workspace = true }
subtle       = { workspace = true }

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"], optional = false }

[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true, features = ["Win32_System_Threading"] }

[dev-dependencies]
pretty_assertions = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt-multi-thread", "test-util"] }
tempfile = { workspace = true }
```

**Crate-root directives (per Phase 2 precedent):**

```rust
// crates/kay-tools/src/lib.rs
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![doc = include_str!("../../../README.md")]  // optional

pub mod tool;
pub mod registry;
pub mod error;
pub mod schema;
pub mod sandbox;
pub mod verifier;
pub mod quota;
pub mod markers;
pub mod builtins;
mod default_set;

pub use tool::Tool;
pub use registry::ToolRegistry;
pub use error::{ToolError, CapScope};
pub use sandbox::{Sandbox, NoOpSandbox, SandboxDenial};
pub use verifier::{TaskVerifier, NoOpVerifier, VerificationOutcome};
pub use default_set::default_tool_set;
```

**Gotcha:** adding `kay-provider-openrouter` as a dep means Phase 5's agent loop will need to be careful about the dep graph — `kay-core` aggregator depends on both, and `kay-cli` pulls kay-core. The circular risk is zero because kay-provider-openrouter is a leaf (Phase 2.5 layout).

## Section 2 — Delegation Surface

For each of the 7 tools:

| Kay Tool | ToolCatalog variant | Input struct (in forge_domain) | Notes |
|---|---|---|---|
| `fs_read` | `ToolCatalog::Read(FSRead)` | `FSRead { file_path, start_line, show_line_numbers, end_line }` [VERIFIED: catalog.rs:193-212] | Thin adapter; `serde_json::from_value::<FSRead>` then wrap. |
| `fs_write` | `ToolCatalog::Write(FSWrite)` | `FSWrite { file_path, content, overwrite }` [VERIFIED: catalog.rs:214-230] | Thin adapter; `ToolExecutor` enforces read-before-overwrite internally. |
| `fs_search` | `ToolCatalog::FsSearch(FSSearch)` | `FSSearch { pattern, path, glob, output_mode, before_context, ... }` [VERIFIED: catalog.rs:232-260+] | Thin adapter. |
| `net_fetch` | `ToolCatalog::Fetch(NetFetch)` | `NetFetch { url, raw }` [VERIFIED: catalog.rs:622-630] | Thin adapter + sandbox `check_net`. |
| `execute_commands` | ❌ **bypasses ToolExecutor** | Custom `ExecuteCommandsInput { command, cwd: Option<PathBuf>, tty: bool, env: Option<Vec<String>>, description: Option<String> }` | `ToolCatalog::Shell(Shell)` exists [VERIFIED: catalog.rs:584-620] but uses collect-then-return semantics via `ForgeShell::execute`; Phase 3 needs streaming. The Kay tool impls its own spawn + marker path. See §4. |
| `image_read` | ❌ **partial delegation** | Custom `ImageReadInput { path: String }` | `ForgeImageRead::read_image(path)` is reusable for base64 decode; quota is Kay-side. See §10. |
| `task_complete` | ❌ **new tool, no parity equivalent** | `TaskCompleteInput { summary: String }` | Calls `verifier.verify(&summary, &transcript)`; emits `AgentEvent::TaskComplete`. No ToolCatalog variant. |

**Mapping JSON → named struct:** all input types derive `Deserialize` + `JsonSchema`. In `Tool::invoke`, pattern is:
```rust
let input: FSRead = serde_json::from_value(args)
    .map_err(|e| ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() })?;
```

`schemars::schema_for!(FSRead)` yields the unhardened schema; Kay runs `harden_tool_schema` once at construction and caches.

## Section 3 — Schema Hardening Wrap

`enforce_strict_schema` currently:
1. Walks recursively (Objects + Arrays).
2. Flattens `allOf` when `strict_mode=true`.
3. Removes `propertyNames` (unsupported by OpenAI).
4. Normalizes string `format` keywords.
5. Sets `type: object` when schema is object-shaped but missing type.
6. Adds `properties: {}` when missing.
7. Sets `additionalProperties: false`.
8. Computes sorted `required` from all property keys.
9. Converts `nullable: true` → `anyOf: [{orig}, {type: null}]`.

**What hardening does NOT currently do:** append truncation reminders or append any Kay-specific text. This is the ONE slot D-02 carves out.

**Proposed wrap signature:**

```rust
pub fn harden_tool_schema(schema: &mut Value, hints: &TruncationHints);

pub struct TruncationHints {
    pub output_truncation_note: Option<String>,
    // possible future: pub arg_size_note, pub deprecation_note
}
```

**Where it's applied:** exactly once per tool, at tool-struct construction time (cached in `schema: Value` field). The cached Value is exposed via `Tool::input_schema()` — planner decides whether to expose as `&Value` (honest) or convert to `&Schema` (matches `ToolDefinition.input_schema: Schema`).

**Recommendation:** expose `&Value`, add `ToolRegistry::tool_definitions()` that constructs `ToolDefinition` with `input_schema: Schema::try_from(value.clone()).expect_or(…)` — but do that conversion behind a `?` operator so a degenerate schema fails construction, not runtime. If that's a concern, keep `input_schema: Schema` internal and `from_value` at registry-build time.

**Truncation reminder text (draft from KIRA convention):**
- `execute_commands`: "Long outputs are truncated — head + tail shown. Narrow command or grep for specific lines."
- `fs_read`: "For large files, use `start_line`/`end_line` range reads rather than full reads."
- `fs_search`: "Search results are capped by `max_search_lines` and `max_search_result_bytes`."
- `net_fetch`: "Fetched content may be truncated at `max_fetch_chars`; set `raw=true` to skip markdown conversion."
- `image_read`: "Per-turn cap: 2 images. Per-session cap: 20 images."
- `task_complete`: "Verification is mandatory before success; outcome may be `Pending` until Phase 8."

Planner finalizes exact text.

## Section 4 — Marker Protocol Wrap

**Per-OS shell-invocation table:**

| Platform | Shell picker | Wrap template |
|---|---|---|
| Unix | `$SHELL` falls back to `/bin/sh` | `sh -c '( USER_CMD\n) ; __KAY_EXIT=$? ; printf "\n__CMDEND_%s_%d__EXITCODE=%d\n" "NONCE" SEQ "$__KAY_EXIT"'` |
| macOS | same as Unix | same |
| Linux | same as Unix | same |
| Windows (PowerShell) | `powershell -NoProfile -Command` | `& { USER_CMD ; $kay_exit = $LASTEXITCODE } ; Write-Host "`n__CMDEND_NONCE_SEQ__EXITCODE=$kay_exit"` |
| Windows (cmd) | `cmd /c` (fallback only) | `USER_CMD & echo __CMDEND_NONCE_SEQ__EXITCODE=%ERRORLEVEL%` |

CONTEXT.md calls out "bash/zsh/sh on Unix" — for Phase 3, hardcode `sh -c` (POSIX, universal) unless the user's `$SHELL` is set to a known interactive shell (bash/zsh). Planner decides; `sh -c` is the safest default.

**Streaming + detection (recap):**

```
spawn child (wrapped command) -> Stdio::piped() for stdout+stderr
tokio::spawn: BufReader::new(stdout).lines() -> for each line:
   if scan_line(line, &marker) == ScanResult::Marker { emit Closed; break; }
   else { emit Stdout(line); }
tokio::spawn: BufReader::new(stderr).lines() -> emit Stderr(line)
join tasks; child.wait() for reap
```

**Nonce generation (per-tool-call):**
```rust
let marker = MarkerContext::new(&self.seq_counter);  // AtomicU64 on the tool struct
// marker.nonce_hex: 32 hex chars, cryptographically random
// marker.seq: monotonic per-session (tool struct lives for session duration)
```

**Marker-inspection threat (SHELL-05): user-injected prompt containing a fake marker.**
- **Where it enters:** (a) model outputs a forged marker string in the command itself (`echo "__CMDEND_abc_1__EXITCODE=0"`), OR (b) adversarial file content contains a marker pattern that the child subsequently prints.
- **Why it doesn't break us:** marker scan uses `subtle::ConstantTimeEq` against the per-call nonce. Attacker cannot guess 128-bit random → forgery attempt surfaces as `ScanResult::ForgedMarker` → streamed as a normal Stdout line (so model sees it and can react) but does NOT close the stream.
- **Pre-execution arg validation:** Phase 3 tool should reject commands whose `command` field CONTAINS `__CMDEND_` substring — this is a weaker check but catches the naïve case where the model literally writes a marker in the command. `ToolError::InvalidArgs { reason: "command contains reserved marker substring" }`.

**Recovery path on race:** if child exits before emitting a marker (SIGKILL, crash, orphaned shell), emit `AgentEvent::ToolOutput { call_id, chunk: Closed { exit_code: child.wait(), marker_detected: false } }`. Agent loop (Phase 5) reads `marker_detected: false` and can flag the command as abnormally terminated.

**KIRA writeup re-validation:** `[ASSUMED]` — the KIRA public writeup was cited in PROJECT.md but not re-fetched during this research session. CONTEXT.md D-03 claims "Terminus-KIRA's writeup mentions 'cryptographically random per-command'" — this claim carried forward. If a closer reading finds a different marker shape (e.g., a pre-shared secret per session rather than per-command), planner should re-check before plan-write. At 128-bit per-call nonce, we're above whatever KIRA uses either way.

## Section 5 — PTY Fallback Heuristic

**Denylist (config-overridable via `kay.toml [tools.execute_commands] pty_first_tokens = [...]`):**

```rust
const PTY_REQUIRING_FIRST_TOKENS: &[&str] = &[
    "ssh", "sudo", "docker", "less", "more", "vim", "nvim", "nano",
    "top", "htop", "watch", "python", "python3", "node", "irb", "psql",
    "mysql", "sqlite3",
];

fn should_use_pty(command: &str, tty_flag: bool) -> bool {
    if tty_flag { return true; }
    let Some(first) = command.split_whitespace().next() else { return false; };
    // Handle `/usr/bin/ssh` → basename
    let first_basename = std::path::Path::new(first)
        .file_stem().and_then(|s| s.to_str()).unwrap_or(first);
    if !PTY_REQUIRING_FIRST_TOKENS.contains(&first_basename) { return false; }
    // Additional gating: `python` alone isn't interactive; `python -i` is.
    match first_basename {
        "python" | "python3" | "node" | "irb" => {
            command.contains(" -i") || command.split_whitespace().nth(1).is_none()
        }
        "docker" => {
            command.contains(" run") && (command.contains(" -it") || command.contains(" -ti"))
        }
        "ssh" | "sudo" => true,  // almost always needs TTY
        _ => true,
    }
}
```

Planner may simplify ("always PTY for first-token in set") in plan.

**`portable-pty 0.8` API surface (relevant subset):** [VERIFIED: Context7]
- `native_pty_system() -> Box<dyn PtySystem + Send>` — factory.
- `PtySystem::openpty(PtySize) -> Result<PtyPair, …>` — returns `{ master, slave }`.
- `PtyPair { master: Box<dyn MasterPty + Send>, slave: Box<dyn SlavePty + Send> }`.
- `MasterPty::try_clone_reader() -> Result<Box<dyn Read + Send>>` — blocking reader.
- `SlavePty::spawn_command(CommandBuilder) -> Result<Box<dyn Child + Send + Sync>>`.
- `Child::try_wait() -> IoResult<Option<ExitStatus>>`, `Child::wait() -> IoResult<ExitStatus>` (blocking).
- `Child::clone_killer() -> Box<dyn ChildKiller + Send + Sync>` — gives you a handle to kill from another thread.
- `ChildKiller::kill() -> IoResult<()>` — sends SIGHUP first (Unix) with 5×50ms grace, then SIGKILL.

**Async integration pattern:**
```rust
let pair = pty_system.openpty(PtySize { rows: 24, cols: 80, .. })?;
let child = pair.slave.spawn_command(cmd)?;
let killer = child.clone_killer();
let mut reader = pair.master.try_clone_reader()?;

// Blocking reader → mpsc
let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
let read_task = tokio::task::spawn_blocking(move || {
    let mut buf = std::io::BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        match std::io::BufRead::read_line(&mut buf, &mut line) {
            Ok(0) => return,   // EOF
            Ok(_) => { if tx.blocking_send(line.clone()).is_err() { return; } }
            Err(_) => return,
        }
    }
});

// Blocking wait → oneshot
let (status_tx, status_rx) = tokio::sync::oneshot::channel();
let wait_task = tokio::task::spawn_blocking(move || {
    let status = child.wait();
    let _ = status_tx.send(status);
});

// Async side
while let Some(line) = rx.recv().await {
    match scan_line(&line, &marker) {
        ScanResult::Marker { exit_code } => {
            emit_closed(exit_code, marker_detected: true);
            killer.kill().ok();  // clean up child
            break;
        }
        _ => emit_stdout(line),
    }
}
```

Note: PTY doesn't distinguish stdout vs stderr (single stream). Plan emits all PTY output as `ToolOutputChunk::Stdout`.

**License:** portable-pty is MIT — on `deny.toml` allow list. [VERIFIED: deny.toml]

## Section 6 — Timeout Cascade

Signal crate decision:
- **Option A (recommended):** add `nix = "0.29"` (Unix-only, feature `signal`). Clean API: `kill(Pid, Signal)`. ~35 KB compiled.
- **Option B:** use `libc::kill` directly (already transitively present via portable-pty). Zero new deps; uses `unsafe`. Planner decides; Phase 2 precedent allows `#[deny(unsafe_code)]` at crate root which would require nix.

**Process group considerations (Unix):** without `setsid`, SIGTERM to `child.id()` only hits the `sh -c` process, NOT its descendants (e.g., `sh -c 'sleep 1000'` — SIGTERM hits sh, sleep is orphaned). Phase 3 options:
1. **Accept the limitation:** deferred to Phase 4 Job Objects (Windows) + setsid process-group (Unix). The 2s grace + SIGKILL usually is enough because SIGKILL on the parent sh also leaves orphans but at least fast-fails the tool.
2. **Use `Command::process_group(0)`** on Unix (stable since Rust 1.64 via `std::os::unix::process::CommandExt`) — the child becomes group leader; then `kill(-pid, SIGTERM)` kills the whole group.

CONTEXT.md D-05 mentions "tokio's `kill_on_drop(true)` + `process_group(0)` pair (set at Command build time)" — this IS the right approach. Tokio's `Command::kill_on_drop(true)` guarantees SIGKILL on drop; `process_group(0)` establishes a new pgroup. Then `kill(-child.id() as i32, SIGTERM)` hits the group. Planner confirms.

**Composition with `tokio::process::Child::kill` vs. `start_kill`:**
- `start_kill()` — synchronous SIGKILL send (non-blocking). Preferred for the "grace expired" branch because we already awaited `child.wait()` in the timeout.
- `kill().await` — SIGKILL send + wait. Equivalent but blocks.
- We use `nix::sys::signal::kill(SIGTERM)` first (tokio has no SIGTERM shortcut), then `start_kill()` for the SIGKILL.

## Section 7 — `AgentEvent` Extensions

Exact Rust type definitions (per D-08):

```rust
// crates/kay-provider-openrouter/src/event.rs (additions)
#[non_exhaustive]
#[derive(Debug)]
pub enum AgentEvent {
    // ... existing Phase 2 variants ...

    /// Streamed output chunk from a running tool. Phase 3 SHELL-03.
    ToolOutput {
        call_id: String,          // from AgentEvent::ToolCallComplete
        chunk: ToolOutputChunk,
    },

    /// Task-completion signal with verification outcome. Phase 3 TOOL-03 +
    /// Phase 8 TaskVerifier integration.
    TaskComplete {
        call_id: String,
        verified: bool,
        outcome: VerificationOutcome,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ToolOutputChunk {
    Stdout(String),
    Stderr(String),
    Closed {
        exit_code: Option<i32>,
        marker_detected: bool,
    },
}
```

`VerificationOutcome` lives in `kay-tools::verifier` (not in event.rs) — that's the trait crate. To reference from event.rs, re-export via path. Actually — to avoid a circular dep (event.rs in kay-provider-openrouter referencing kay-tools::VerificationOutcome, and kay-tools depending on kay-provider-openrouter for AgentEvent), put `VerificationOutcome` in `kay-provider-openrouter::event` alongside the variant:

```rust
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum VerificationOutcome {
    Pass    { rationale: String },
    Fail    { reasons: Vec<String> },
    Pending { reason: String },
}
```

**Planner decision:** `VerificationOutcome` definition site. Options:
- (a) In `kay-provider-openrouter::event` — resolves cycle but couples verifier to provider crate.
- (b) New crate `kay-events` (another split) — cleaner but adds a crate. Probably overkill for Phase 3.
- (c) In `kay-tools::verifier` + kay-provider-openrouter adds kay-tools as a dep — forms a cycle (kay-tools → kay-provider-openrouter → kay-tools). Rejected.

**Recommendation:** option (a). `VerificationOutcome` is really an event-shape thing; the trait `TaskVerifier` can still live in `kay-tools` and return the event-crate type.

**Clone note:** Phase 2 dropped `Clone` on `AgentEvent` because `ProviderError` contains non-Clone types (reqwest::Error, serde_json::Error). `ToolOutputChunk` must also be `Clone` safe — `String` and primitives only, so OK. `VerificationOutcome` only holds `String`s — OK.

**Back-pressure pattern (Phase 5 anticipation):** the event mpsc channel is bounded. Phase 3 `execute_commands` should use `Sender::send().await` (not `try_send`) so a backed-up consumer slows stdout reading — prevents unbounded memory growth on noisy commands.

## Section 8 — `TaskVerifier` Trait + `NoOpVerifier`

```rust
// crates/kay-tools/src/verifier.rs
use kay_provider_openrouter::event::VerificationOutcome;

#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    async fn verify(
        &self,
        task_summary: &str,
        transcript: &Transcript,
    ) -> VerificationOutcome;
}

pub struct NoOpVerifier;

#[async_trait::async_trait]
impl TaskVerifier for NoOpVerifier {
    async fn verify(&self, _: &str, _: &Transcript) -> VerificationOutcome {
        VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".into(),
        }
    }
}
```

**`Transcript` type:** Phase 6 owns the canonical transcript type (SESS-*). Phase 3 has none to consume. Options:
- (a) **Minimal stub:** `pub struct Transcript(pub Vec<AgentEvent>);` — re-export. Works but pulls NON-Clone events; `&Transcript` by ref is fine.
- (b) **Opaque placeholder trait:** `pub trait TranscriptView: Send + Sync {}` and `NoOpVerifier` accepts any impl. Phase 6 wires concrete type.
- (c) **Skip entirely in Phase 3:** `verify(task_summary: &str) -> VerificationOutcome` — no transcript arg. Phase 8 adds it.

**Recommendation:** option (c) for Phase 3 since NoOpVerifier doesn't use it anyway; add `transcript` arg at Phase 8 when it becomes concrete. The `#[async_trait]` signature change IS a breaking change, but since Phase 3 is the only impl site, it's cheap. Alternative: option (a) with `Transcript = &[AgentEvent]` as a type alias — then Phase 6 swaps the alias and downstream code keeps compiling.

**Phase 5 loop integration:** Phase 3 emits `AgentEvent::TaskComplete { verified: false, outcome: Pending { ... } }`. Phase 5 agent loop reads it and does NOT exit the turn (Pending means "can't conclude"). Phase 8 wires the real verifier — same event surface, verified=true, Pass/Fail outcome. No Phase 3 code changes required.

## Section 9 — `Sandbox` Trait Surface

Minimum API for Phase 3 (Phase 4 swaps impl):

```rust
// crates/kay-tools/src/sandbox.rs
use std::path::Path;
use url::Url;
use std::ffi::OsStr;

#[derive(Debug)]
pub struct SandboxDenial {
    pub reason: String,
    pub resource: String,  // path or URL or command for logging
}

#[async_trait::async_trait]
pub trait Sandbox: Send + Sync {
    async fn check_shell(&self, command: &str, cwd: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_read(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_write(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_search(&self, path: &Path) -> Result<(), SandboxDenial> {
        self.check_fs_read(path).await  // default impl: search is a read
    }
    async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial>;
}

pub struct NoOpSandbox;

#[async_trait::async_trait]
impl Sandbox for NoOpSandbox {
    async fn check_shell(&self, _: &str, _: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_fs_read(&self, _: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_fs_write(&self, _: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_net(&self, _: &Url) -> Result<(), SandboxDenial> { Ok(()) }
}
```

CONTEXT.md D-12 sketch aligns. `check_command_exec(&[OsStr])` was suggested in instructions — prefer `check_shell(&str, &Path)` because Phase 3 passes the command as a string (pre-split), and the sandbox layer needs both command and cwd for path-scope checks.

**Phase 4 boundary:** Phase 4 can MOVE the trait to `kay-sandbox-core` (new crate) or keep it in `kay-tools` and add impl crates. Planner DOES NOT need to decide that here — trait lives in kay-tools for Phase 3; Phase 4 plan decides migration.

## Section 10 — Image Caps & Request-Body Plumbing

**Per-turn / per-session bookkeeping:**

```rust
// crates/kay-tools/src/quota.rs
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct ImageQuota {
    pub max_per_turn: u32,
    pub max_per_session: u32,
    per_turn: AtomicU32,
    per_session: AtomicU32,
}

impl ImageQuota {
    pub fn new(max_per_turn: u32, max_per_session: u32) -> Self { /* ... */ }

    pub fn try_consume(&self) -> Result<(), CapScope> {
        if self.per_turn.load(Ordering::Relaxed) >= self.max_per_turn {
            return Err(CapScope::PerTurn);
        }
        if self.per_session.load(Ordering::Relaxed) >= self.max_per_session {
            return Err(CapScope::PerSession);
        }
        self.per_turn.fetch_add(1, Ordering::Relaxed);
        self.per_session.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Phase 5 agent loop calls this on turn boundary.
    pub fn reset_turn(&self) {
        self.per_turn.store(0, Ordering::Relaxed);
    }
}

pub enum CapScope { PerTurn, PerSession }
```

**ImageContext (turn-scoped buffer for pending images):**

Phase 2 doesn't have this today. Where should pending image payloads live so the NEXT provider turn embeds them?

**Phase 2 inspection finding:** `OpenRouterProvider::chat(request) -> Stream<AgentEvent>` takes a request (Phase 2 plan 02-08 built `build_request_body` in translator.rs using Option B / OrderedObject). The request includes `messages: Vec<Message>`. `ContentPart::ImageUrl { image_url: ImageUrl { url, detail } }` is a DTO field. The OrderedObject body builder in Phase 2's translator doesn't currently emit `ContentPart` array bodies — it serializes text-only.

**Gap for Phase 3:** `image_read` runs, produces a base64 blob — but there's no channel from tool → next `chat()` call to embed it. Options:

- **Option α (minimal seam in Phase 3):** `image_read` returns a `ToolOutput` whose content is an image-URL string (data URI `data:image/png;base64,<…>`), and Phase 5's agent-loop-to-provider bridge is responsible for converting certain `ToolOutput` text content into `ContentPart::ImageUrl` before next turn. Phase 3 just produces the data URI; no request-body extension in Phase 3.
- **Option β (extend request body now):** Phase 3 extends `kay-provider-openrouter::translator::build_request_body` to accept an `images: Vec<(String, String)>` field on the request struct and emit `content: [{type:text,text:"..."},{type:image_url, image_url:{url:"data:...",detail:null}},...]`. `image_read` enqueues onto `Arc<Mutex<Vec<Image>>>` shared with provider.

**Recommendation: Option α.** Phase 5 owns the agent loop and the transcript shape; that's the right layer. Phase 3 just produces the data URI in the `ToolOutput`. This keeps Phase 3 from touching Phase 2's locked request-body builder. Document explicitly in plan: "image_read emits data URI; Phase 5 plan converts to ContentPart::ImageUrl in next-turn message construction." Planner's call.

**ForgeCode's behavior for comparison:** `forge_services::tool_services::image_read::ForgeImageRead::read_image` returns `Image` (bytes + mime) — a `forge_app::domain::Image` type [VERIFIED: image_read.rs:5,93]. That type likely has `.url()` returning a data URI. Confirm by reading `forge_app::domain::Image` impl — planner verifies.

**Config keys (D-07):**

```toml
# kay.toml (new file — Kay workspace root; CONTEXT.md mentions but no existing file)
[tools.image_read]
max_per_turn = 2
max_per_session = 20
```

Env override: `KAY_IMAGE_MAX_PER_TURN=…`, `KAY_IMAGE_MAX_PER_SESSION=…`.

**Extension to `ForgeConfig`?** D-07 says `toml + env overridable`. Cleanest path: add `image_read: ImageReadConfig { max_per_turn: u32, max_per_session: u32 }` field to `ForgeConfig`. Planner decides whether to extend `ForgeConfig` (parity-imported crate) or add a Kay-side `KayConfig` overlay. The parity concern is low here — adding a defaulted field with `#[serde(default)]` is additive and doesn't regress ForgeCode's config behavior. Recommend extending `ForgeConfig` with one small struct.

## Section 11 — Workspace Dependencies to Add

Additions to top-level `Cargo.toml [workspace.dependencies]`:

```toml
portable-pty = "0.8"
subtle       = "2"
# Unix-only — add under crates/kay-tools/Cargo.toml [target.'cfg(unix)'.dependencies]
# nix = { version = "0.29", features = ["signal"] }  # if Option A chosen in §6
```

Workspace deps already present and reused (no additions):
- `async-trait`, `tokio`, `serde_json`, `schemars`, `anyhow`, `thiserror`, `tracing`, `rand`, `hex`, `futures`, `windows-sys`.

**License verification (via `deny.toml`):**
- portable-pty: MIT-only. ✓ on allow list.
- subtle: BSD-3-Clause. ✓ on allow list.
- nix: MIT. ✓ on allow list.

**`cargo-deny` will pass.** Run `cargo deny check` before merging Phase 3 to confirm.

**Version currency check (planner runs at plan-write):**
```bash
cargo search portable-pty  # expect 0.8.x latest
cargo search subtle        # expect 2.x latest
cargo search nix           # expect 0.29.x or 0.30.x
```

## Section 12 — Validation Architecture

Phase 3 has 11 requirement IDs (TOOL-01..06, SHELL-01..05) and 5 ROADMAP success criteria. Each maps to concrete tests.

### Test Framework

| Property | Value |
|---|---|
| Framework | `cargo test` (Rust builtin) + `pretty_assertions` 1.x + `tokio::test` (macros feature) |
| Config file | none — per-crate test organization |
| Quick run command | `cargo test -p kay-tools --lib` |
| Full suite command | `cargo test --workspace --deny warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| TOOL-01 | Arc<dyn Tool> is object-safe; registry stores and retrieves | unit | `cargo test -p kay-tools --lib registry::` | ❌ Wave 0 |
| TOOL-02 | execute_commands runs shell in sandbox and returns stream | integration | `cargo test -p kay-tools --test execute_commands_e2e -- --nocapture` | ❌ Wave 0 |
| TOOL-03 | task_complete invokes verifier and emits TaskComplete event | unit | `cargo test -p kay-tools --lib builtins::task_complete::` | ❌ Wave 0 |
| TOOL-04 | image_read accepts base64; caps enforced | unit + integration | `cargo test -p kay-tools image_quota` | ❌ Wave 0 |
| TOOL-05 | Tool schemas pass through enforce_strict_schema; `required` sorted containing all keys; `additionalProperties: false` at every object level | property | `cargo test -p kay-tools --test schema_hardening_property` | ❌ Wave 0 |
| TOOL-06 | Registry.tool_definitions() emits schemas compatible with OpenAI `tools` format | unit | `cargo test -p kay-tools tool_definitions_emit` | ❌ Wave 0 |
| SHELL-01 | Marker pattern matches; exit code parsed | unit | `cargo test -p kay-tools markers::scan_line_marker_match` | ❌ Wave 0 |
| SHELL-02 | PTY path engages on denylist hit; non-PTY path default | unit (heuristic) + integration (real PTY spawn behind `#[cfg(not(target_os = "windows"))]` in CI) | `cargo test -p kay-tools should_use_pty && cargo test -p kay-tools --test pty_integration` | ❌ Wave 0 |
| SHELL-03 | Stdout streams as frames BEFORE process exits (no blocking collection) | integration (timed) | `cargo test -p kay-tools --test streaming_latency` | ❌ Wave 0 |
| SHELL-04 | Timeout cascade: SIGTERM → 2s → SIGKILL → reap | integration | `cargo test -p kay-tools --test timeout_cascade -- --test-threads=1` | ❌ Wave 0 |
| SHELL-05 | Forged marker in output doesn't close stream; recovery on child crash | unit (forged) + integration (crash) | `cargo test -p kay-tools markers::scan_line_forged && cargo test --test marker_race` | ❌ Wave 0 |

### Phase Success Criteria → Test Map

| SC # | Criterion | Test Type | Command |
|---|---|---|---|
| 1 | Developer registers `Arc<dyn Tool>` and schema is emitted hardened | unit | `cargo test register_dyn_tool_hardens_schema` |
| 2 | `execute_commands` runs command, streams output, signals via marker | integration | `cargo test --test execute_commands_e2e -- --nocapture` |
| 3 | Long-running command cleanly terminated by timeout on all 3 OSes | integration | runs on CI for Linux+macOS+Windows matrix; signal assertion on Unix via `strace -e signal` wrapper (optional) |
| 4 | `image_read` accepts base64, bounded by turn/session caps | unit + integration | `cargo test image_quota` |
| 5 | Forged marker rejected; `task_complete` Pending until Phase 8 | unit | `cargo test scan_line_forged && cargo test task_complete_pending` |

### Sampling Rate

- **Per task commit:** `cargo test -p kay-tools --lib` (Rust unit tests only, ~5-15s).
- **Per wave merge:** `cargo test -p kay-tools --all-targets` (lib + integration, ~1-3 min including PTY tests).
- **Phase gate (before `/gsd-verify-work`):** `cargo test --workspace --deny warnings` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo deny check`.

### Wave 0 Gaps

- [ ] `crates/kay-tools/Cargo.toml` — new crate manifest
- [ ] `crates/kay-tools/src/lib.rs` + module skeleton (§1)
- [ ] `crates/kay-tools/tests/registry_integration.rs` — covers TOOL-01, TOOL-05, TOOL-06
- [ ] `crates/kay-tools/tests/marker_streaming.rs` — covers SHELL-01, SHELL-03
- [ ] `crates/kay-tools/tests/marker_race.rs` — covers SHELL-05
- [ ] `crates/kay-tools/tests/timeout_cascade.rs` — covers SHELL-04
- [ ] `crates/kay-tools/tests/pty_integration.rs` — covers SHELL-02 (gated on unix)
- [ ] `crates/kay-tools/tests/image_quota.rs` — covers TOOL-04
- [ ] `crates/kay-tools/tests/schema_hardening_property.rs` — covers TOOL-05 (property)
- [ ] Workspace `Cargo.toml` — append `"crates/kay-tools"` member + new workspace deps (§11)

**No framework install needed** — cargo test + tokio::test + pretty_assertions already configured by Phase 2.

## Section 13 — Plan Partitioning Hint

Proposed split (granularity = fine, parallelization = true per config.json):

| Wave | Plan | Scope | Parallelizable with |
|---|---|---|---|
| **Wave 0** | 03-01 Crate scaffold + Wave-0 tests | `crates/kay-tools/` skeleton, `Cargo.toml`, empty module tree with `todo!()` stubs, workspace member addition, NOTICE. Does NOT implement logic. Validates: `cargo check -p kay-tools`, `cargo test -p kay-tools --lib` (with todo stubs skipped via `#[ignore]`). | — (foundation) |
| **Wave 1** | 03-02 Core abstractions | `Tool` trait + `ToolRegistry` + `ToolError` + `Sandbox` trait + `NoOpSandbox` + `TaskVerifier` + `NoOpVerifier` + `VerificationOutcome` in event.rs + `CapScope`. Unit tests only. | 03-03 (no shared files except Cargo.toml) |
| **Wave 2** | 03-03 Schema + AgentEvent extensions | `harden_tool_schema` wrapper + `TruncationHints` + `AgentEvent::ToolOutput` + `AgentEvent::TaskComplete` + `ToolOutputChunk` variant added to `kay-provider-openrouter::event`. Property test for hardening. | 03-02 (different files) |
| **Wave 3** | 03-04 Marker shell tool (the big one) | `ExecuteCommandsTool` + `markers/` module + `MarkerContext` + `scan_line` + `wrap_unix_sh` + `wrap_windows_ps` + PTY fallback + timeout cascade + all SHELL-* tests. Longest plan (~60-90 min estimated). | SERIAL — depends on 03-02 and 03-03 |
| **Wave 4** | 03-05 Remaining tools + wiring | `FsReadTool` / `FsWriteTool` / `FsSearchTool` / `NetFetchTool` / `ImageReadTool` / `TaskCompleteTool` + `ImageQuota` + `default_tool_set` + `kay-cli` wiring + E2E integration test (full flow: start registry → feed tool call → stream output → complete). Config extensions (`tool_timeout_secs` reuse, image-cap keys). | SERIAL — depends on 03-04 (uses ExecuteCommandsTool via registry) |

**Parallelization summary:**
- Plans 03-02 and 03-03 can run in parallel (different files; both depend only on 03-01).
- Plans 03-04 and 03-05 are serial.

**Estimated durations (based on Phase 2 velocity ~18 min/plan average, with Wave 3 as outlier):**
- 03-01: ~15 min (scaffold)
- 03-02: ~25 min (several types + trait + tests)
- 03-03: ~20 min (schema wrapper + event additions)
- 03-04: ~60-90 min (marker + PTY + timeout — largest plan)
- 03-05: ~40 min (6 tools + wiring + e2e)

Total Phase 3: ~160-200 min. Comparable to Phase 2 (~227 min of direct execution).

## Section 14 — Parity Preservation Audit

| Tool | Delegation Site | Parity Risk | Mitigation |
|---|---|---|---|
| fs_read | `ToolExecutor::execute(ToolCatalog::Read)` | None — pure passthrough | Re-serialize output as-is. |
| fs_write | `ToolExecutor::execute(ToolCatalog::Write)` | Read-before-overwrite enforcement is inside ToolExecutor, preserved | No bypass. |
| fs_search | `ToolExecutor::execute(ToolCatalog::FsSearch)` | None | Pure passthrough. |
| net_fetch | `ToolExecutor::execute(ToolCatalog::Fetch)` | None | Pure passthrough. |
| execute_commands | **BYPASSES** — marker protocol is new | HIGH — marker wrap changes command semantics. NOT what ForgeCode does. | Parity gate EVAL-01a runs unmodified fork via `forge_main`, NOT Kay's tool path. Phase 1 baseline was forge_main, not kay-cli. Post-Phase-5 kay-cli runs TB 2.0 — if marker protocol regresses score, harden further. Risk is accepted by PROJECT.md (KIRA techniques are the improvement vector). |
| image_read | Thin wrap over `ForgeImageRead::read_image` + quota | None on decode; quota is Kay-new | ForgeCode has NO per-turn / per-session cap — adding them can ONLY reject MORE calls than parity. Could theoretically reject a call ForgeCode would have accepted. Mitigation: defaults (2/20) are generous enough for TB 2.0 (screenshots rare). Monitor TB 2.0 runs; increase if blocker. |
| task_complete | NEW — no ForgeCode equivalent | None — additive | ForgeCode does not have `task_complete` as a registered tool in its catalog. This tool does not exist in the parity-imported catalog. Adding it is additive and required by TOOL-03 + LOOP-05. |

**Flags for code review:**
1. **execute_commands must not regress `cargo test` / `pytest` invocations** — the common case. Test fixture: run `pytest` in a sample repo via execute_commands, confirm marker closes correctly, stdout streams, exit code parsed.
2. **fs_write overwrite semantics must preserve the read-before-edit guard** — ToolExecutor delegation covers this. Test: call fs_write with `overwrite: true` without a prior fs_read → expect error. Covered by existing forge_app tests; Kay layer only adds sandbox check.
3. **net_fetch URL scheme must match ForgeCode behavior** — `NetFetch { url, raw }`, with `url: String`. ToolExecutor handles. Sandbox `check_net(&Url)` requires URL parsing — use `url::Url::parse()` with explicit error handling.

## Section 15 — Open Questions for Planner

1. **`ToolCallContext` ownership:** re-export `forge_domain::ToolCallContext` (simpler) vs. wrap in a Kay-owned type (future-proofing)? Recommended: re-export. Rename only if Kay adds fields.
2. **`VerificationOutcome` location:** kay-provider-openrouter::event (breaks clean layering but avoids cycle) vs. new `kay-events` crate. Recommended: former.
3. **nix vs. libc for SIGTERM:** Option A (nix 0.29) vs. Option B (libc direct + `unsafe`). Recommended: nix unless crate-root `#[deny(unsafe_code)]` is desired — which it is per Phase 2 precedent. Go nix.
4. **sh vs. bash default on Unix:** plain `sh -c` (POSIX, universal) vs. `$SHELL` detection. Recommended: `sh -c` — avoids user-shell variance hurting reproducibility; agents can `bash -c '...'` inside if needed.
5. **Image data URI pass-through (§10 Option α) vs. request-body extension (Option β):** Recommended: Option α. Defer body extension to Phase 5.
6. **ForgeConfig extension vs. KayConfig overlay for image caps:** Recommended: extend ForgeConfig with one defaulted sub-struct. Smallest diff, additive.
7. **PTY denylist config key name:** `[tools.execute_commands] pty_first_tokens = [...]`? Planner finalizes toml shape.
8. **Shell-wrap leading-newline convention:** always prepend `\n` before `__CMDEND_…` tail (recommended) vs. rely on subshell output already being newline-terminated (fragile). Recommended: always prepend.
9. **Clone_killer hold-point for PTY cascade:** where does the `Box<dyn ChildKiller>` live in the async tool impl so SIGTERM-equivalent (SIGHUP) can reach the PTY child before SIGKILL? Planner documents in 03-04.
10. **Per-tool TruncationHints text:** exact strings (draft in §3). Finalized during 03-03.
11. **`ExecuteCommandsInput` vs. reuse of `forge_domain::Shell`:** Kay's input needs an added `tty: Option<bool>` field; can't reuse `Shell` struct directly. Define Kay-side input struct; converter to `forge_domain::Shell` for parity-tools. Planner decides naming.
12. **ToolCallMalformed handling at registry level:** Phase 2 emits `ToolCallMalformed` through the translator for bad argument JSON. Does Phase 3's registry also handle malformed-at-invoke (when the typed Deserialize fails)? Recommended: yes — emit `ToolError::InvalidArgs` and surface as a `ToolOutput::text("...invalid args...")` so the agent sees the error and can retry. Path: 03-05 wiring.
13. **Seq counter scope:** per-tool-struct AtomicU64 (session lifetime) vs. per-tool-call fresh. Recommended: per-struct (session scope) so debuggability across calls is trivial.
14. **Testing PTY on Windows CI:** portable-pty supports Windows via ConPTY, but GitHub Actions Windows runners are flaky for PTY tests. Recommended: gate PTY integration tests with `#[cfg(unix)]` in Phase 3; re-enable Windows in Phase 11.
15. **Should `task_complete` accept an already-malformed summary?** Recommend requiring non-empty `summary: String` (`ToolError::InvalidArgs` on empty). Planner decides.
16. **Tool call id propagation:** every `AgentEvent::ToolOutput` carries `call_id` — who owns and threads it? Phase 2 emits `call_id` in `ToolCallStart`/`Complete`. Phase 3 `execute_commands::invoke` must receive it (add to `ToolCallContext`? add arg to `Tool::invoke`?). Critical open question. **Recommended:** add `call_id: &str` as a third arg to `Tool::invoke` — cleaner than stashing in ToolCallContext. This is a small trait-signature decision but must be locked in 03-02.

## Threat Model

Enumerated per requirement; STRIDE columns.

| # | Threat | STRIDE | Requirement | Standard Mitigation |
|---|---|---|---|---|
| 1 | **Marker injection** — model writes forged marker in shell command (echo "__CMDEND_...") OR attacker file content contains a marker-shaped substring | Spoofing | SHELL-05, TOOL-02 | 128-bit random nonce per call (D-03); `subtle::ConstantTimeEq` compare; pre-execution arg validation rejecting commands containing `__CMDEND_` substring |
| 2 | **Path traversal** in fs_read/fs_write (`../../../etc/passwd`) | Tampering, Info Disclosure | TOOL-02, TOOL-04 | NoOpSandbox is a stub in Phase 3. ForgeCode's `ToolExecutor::normalize_path` makes paths absolute relative to cwd (not sandbox-scoped). Phase 4 Sandbox impl will enforce project-root confinement. Phase 3 ships with documented known-limitation: paths are unrestricted. |
| 3 | **SSRF via net_fetch** — model requests http://169.254.169.254/ (AWS IMDS), http://localhost:5432 (internal DB), file:// URLs | Info Disclosure | TOOL-02 | Sandbox `check_net(&Url)` exists but NoOp in Phase 3. Phase 4 implements allowlist. Phase 3 should at minimum reject `file://` scheme in net_fetch (add to NoOpSandbox). ForgeCode's `net_fetch` uses reqwest which rejects `file://` by default. Document as limitation. |
| 4 | **Image exfiltration via size overflow** — model requests 10 GB PNG | Resource Exhaustion (Tampering in STRIDE) | TOOL-04 | `ForgeImageRead` uses `max_image_size_bytes` check via `assert_file_size`. Kay layer adds turn/session quota. Path traversal in image-path is Phase 4 concern. |
| 5 | **Timeout bypass via signal ignore** — `trap "" TERM` in user command ignores SIGTERM | Tampering | SHELL-04 | 2s grace → SIGKILL cascade. SIGKILL cannot be trapped. Process-group kill (via `process_group(0)` + `kill(-pid, SIGTERM)`) reaches descendants. |
| 6 | **Prompt injection via tool description in emitted schema** — attacker-controlled data ends up in a tool description, model reads schema and follows instructions | Spoofing | TOOL-05 | Descriptions are static, embedded in source (`tool_description_file` macro) or from `TruncationHints` (Kay-owned string literals). No runtime user-input reaches description. If Phase 3 ever takes user-supplied tool-description text, apply sanitization. |
| 7 | **AgentEvent::TaskComplete spoofing** — model emits a tool call to a non-existent `task_complete` argument shape to trick loop into exiting | Spoofing | TOOL-03, SC#5 | `NoOpVerifier` returns `Pending`, never `Pass`. Phase 5 loop reads `verified == true` only — Phase 3's NoOp can't produce that. Locked by construction. |
| 8 | **ToolCallComplete args are already-bad JSON after repair** | Tampering / DoS | TOOL-01 via Tool::invoke | `serde_json::from_value::<InputType>` returns `ToolError::InvalidArgs`; surface as ToolOutput text. Model sees the error and can retry. No panic (crate-root deny on unwrap/expect enforces). |
| 9 | **DoS via unbounded shell output** | Resource Exhaustion | SHELL-03 | Bounded mpsc channel (`Sender::send().await` back-pressure); 1 MiB cap on tool-argument buffers already enforced by Phase 2 translator. Phase 3 adds stdout byte-counting if runaway output becomes an issue — initial default is unlimited streaming but per-line truncation at `max_stdout_line_chars` (already in ForgeConfig). |
| 10 | **PTY child zombie on early exit** — spawn failed, reader still waiting | Resource Exhaustion | SHELL-04 | `tokio::process::Command::kill_on_drop(true)`; PTY side uses `Child::clone_killer` → `ChildKiller::kill` in a Drop impl on the tool's state struct. |
| 11 | **Schema validation bypass** — malformed schema passes hardener and provider rejects at chat time | DoS | TOOL-05 | Unit test: every registered tool's schema round-trips through OpenAI's published tool-schema validator (can simulate via jsonschema crate; planner decides). Failure at registry construction, not at chat time. |
| 12 | **Cross-tool state leak** — session-scoped AtomicU64 seq shared across concurrent tool calls | Info Disclosure (low) | SHELL-01 | `fetch_add(1, Relaxed)` is atomic; seq uniqueness preserved. No leak — just monotonicity. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | portable-pty 0.8 is current stable; 0.9+ hasn't landed with breaking API changes | §Standard Stack, §5 | Plan deps need version bump; API usage changes. Planner verifies via `cargo search`. |
| A2 | KIRA writeup uses 128-bit nonce; no smaller/larger recommendation found in time-boxed research | §4 | 128-bit is abundantly safe regardless; worst case we're over-engineering (harmless). |
| A3 | Running `tokio::process::Command::process_group(0)` + `nix::sys::signal::kill(-pid, SIGTERM)` kills the whole group cleanly | §6, §Threat Model #5 | If pgroup semantics fail, descendants orphan. Phase 4 Job Objects supersede. Document known limitation in Phase 3 plan. |
| A4 | ForgeImageRead returns an `Image` type that can be converted to a data URI | §10 Option α | Planner verifies `forge_app::domain::Image` surface at plan-write. If not convertible, Option β becomes required. |
| A5 | Phase 2's translator doesn't currently emit `ContentPart::ImageUrl` in request bodies | §10 | Verified via grep of `crates/kay-provider-openrouter/src/translator.rs` — no image-content handling. Confirmed. |
| A6 | OpenRouter Exacto models in allowlist all support multimodal (for image_read to work) | §10 | If some don't, image_read call with unsupported model returns provider error — surface as ToolError::ExecutionFailed. Acceptable. |
| A7 | The 7 tools chosen in D-10 suffice for typical TB 2.0 tasks | §14 | If a TB 2.0 task requires `fs_patch` or `fs_remove`, score regresses. D-10 explicitly defers these to Phase 5+; planner monitors TB 2.0 dry-runs. |
| A8 | `subtle::ConstantTimeEq` on raw `[u8]` prefixes provides meaningful timing-channel protection in practice | §4, §Threat Model #1 | Mostly defense-in-depth; the primary guarantee is entropy (128-bit random). Harmless if subtle underperforms. |
| A9 | Adding `image_read` sub-config to `ForgeConfig` is additive and does not trigger parity regression | §10, §14 | Verified: `#[serde(default)]` ensures old configs still parse. Low risk. |
| A10 | KIRA public writeup claim of "marker polling as TB 2.0 score load-bearing technique" is not re-verified this session | §4 | PROJECT.md cites it; CONTEXT.md builds on it. Wrong only if PROJECT.md is wrong — outside Phase 3 research scope to revalidate. |

**If this table needs user confirmation:** A1, A3 are tool-verifiable during planning. A2, A10 are KIRA-writeup claims the user might want to sanity-check. A5-A9 are in-tree verifications the planner can do at plan-write time.

## Open Questions

Already enumerated in §15. Highlight the three that most affect plan correctness:

1. **`Tool::invoke` signature — does `call_id` get added?** (Q16) — locked in Wave 1 (03-02). Affects every other wave's adapter shape.
2. **Image data URI pass-through vs. request-body extension** (Q5) — affects whether 03-05 extends kay-provider-openrouter or stays contained in kay-tools.
3. **ForgeConfig extension vs. KayConfig overlay** (Q6) — affects 03-05 config plumbing; also affects whether a ForgeConfig ParityRisk discussion is needed in that plan.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| Rust 1.95 stable | All | ✓ | per rust-toolchain.toml [VERIFIED: Cargo.toml:45] | — |
| cargo | All | ✓ | bundled | — |
| `sh` / POSIX shell (Unix) | execute_commands | ✓ (macOS, Linux) | — | — |
| `powershell.exe` (Windows) | execute_commands | ✓ on win targets | 5.1+ | `cmd /c` (minimal) |
| `cargo-deny` | CI license check | ✓ (per Phase 1) | — | — |
| `cargo-audit` | CI advisory check | ✓ (per Phase 1) | — | — |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none.

No external services (no database, no Docker, no network APIs) required for Phase 3 tests — marker tests spawn local shells; PTY tests are local; quota tests are pure-Rust. Clean greenfield.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | no | Phase 2 owns provider auth; no new auth surface in Phase 3 |
| V3 Session Management | no | Phase 6 owns sessions |
| V4 Access Control | partial | NoOpSandbox is a stub; Phase 4 enforces. Phase 3 documents the known-limitation in plan. |
| V5 Input Validation | **yes** | `serde_json::from_value` via serde + schemars-generated schemas; `enforce_strict_schema` ensures `additionalProperties: false` so extra fields are rejected by provider before reaching us. |
| V6 Cryptography | **yes** | `rand::rngs::OsRng` (never `thread_rng`); `subtle::ConstantTimeEq` — never hand-roll. |
| V7 Error Handling | **yes** | Crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]`; all errors are typed `ToolError` (`#[non_exhaustive]`); no string-formatted errors on hot paths. |
| V8 Data Protection | partial | image_read blob is in-memory only; `Image` type in forge_app should redact Debug but does not by default. Phase 3 adds `Debug` that omits base64 contents. |
| V9 Communication | no | Phase 2 owns TLS via rustls. |
| V10 Malicious Code | no | N/A |
| V11 Business Logic | partial | TOOL-03 prevents premature task_complete via verifier gate. |
| V12 Files & Resources | **yes** | V12.4 "Do not allow control of file path": Phase 3 Sandbox trait is the seam. Phase 4 enforcement. Phase 3 limitation documented. |

### Known Threat Patterns for Rust async agent-harness Stack

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Prompt-injected marker forgery | Spoofing | 128-bit per-call nonce + constant-time compare (§4, Threat #1) |
| Path traversal via unconfined fs_* tools | Tampering / Info Disclosure | Sandbox trait DI; NoOp in Phase 3, enforced in Phase 4 (Threat #2) |
| SSRF via net_fetch | Info Disclosure | Sandbox `check_net`; reject `file://` in Phase 3 NoOp (Threat #3) |
| Unbounded shell output DoS | Resource Exhaustion | Bounded mpsc; per-line char cap; process-group kill on timeout (Threat #5, #9) |
| `unwrap()` panic in marker scan wedges agent | DoS | `#![deny(clippy::unwrap_used)]` at crate root |
| Schema validation bypass via malformed hardened schema | DoS | Registry-construction-time validation; unit tests per registered tool (Threat #11) |

---

## Sources

### Primary (HIGH confidence)
- `crates/forge_app/src/utils.rs:320-440` (enforce_strict_schema) — source inspected 2026-04-20
- `crates/forge_app/src/tool_executor.rs` (ToolExecutor::execute + ToolCatalog dispatch) — source inspected
- `crates/forge_app/src/tool_registry.rs:45-61` (call_with_timeout pattern) — source inspected
- `crates/forge_domain/src/tools/catalog.rs:41-61, 193-212, 214-230, 232-260, 584-620, 622-630` (ToolCatalog + input structs) — source inspected
- `crates/forge_domain/src/tools/definition/tool_definition.rs` (ToolDefinition shape) — source inspected
- `crates/forge_domain/src/tools/call/context.rs` (ToolCallContext) — source inspected
- `crates/forge_services/src/tool_services/shell.rs` (ForgeShell — reference only, non-streaming) — source inspected
- `crates/forge_services/src/tool_services/image_read.rs` (ForgeImageRead) — source inspected
- `crates/forge_config/src/config.rs:140-155` (ForgeConfig.tool_timeout_secs, max_image_size_bytes) — source inspected
- `crates/kay-provider-openrouter/src/event.rs` (AgentEvent enum + #[non_exhaustive]) — source inspected
- `crates/kay-provider-openrouter/src/translator.rs:1-100` (request-body + event emission) — source inspected
- `crates/forge_app/src/dto/openai/request.rs:99-112, 485-700` (ContentPart::ImageUrl) — source inspected
- `Cargo.toml` (workspace deps inventory) — full read
- `deny.toml` (license allow list) — full read
- `.planning/REQUIREMENTS.md:227-237` (TOOL-01..06, SHELL-01..05) — read
- `.planning/ROADMAP.md:109-121` (Phase 3 goal + 5 SCs) — read
- `.planning/phases/03-tool-registry-kira-core-tools/03-CONTEXT.md` (12 decisions) — read full
- Context7: `/websites/rs_portable-pty` — API surface for Child, ChildKiller, MasterPty

### Secondary (MEDIUM confidence)
- `tokio::process::Child::start_kill` Unix SIGKILL semantics — tokio docs (widely known; verified mental model against Pitfall #1)
- schemars 1.2 `Schema::try_from(Value)` API — workspace pin confirmed, specific trait method not directly tested

### Tertiary (LOW confidence — flagged for planner validation)
- Exact KIRA writeup marker-protocol phrasing (CONTEXT.md claim carried forward; PROJECT.md cites KIRA as source but I did NOT re-fetch the writeup this session)
- Exact cpu overhead of `subtle::ConstantTimeEq` on 32-byte slices (assumed negligible)
- Windows PTY reliability on GitHub Actions runners (planner gates integration tests off-Windows for Phase 3)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Context7-verified for portable-pty; workspace pins confirmed in Cargo.toml
- Architecture: HIGH — ToolExecutor + ToolCatalog + AgentEvent source inspected directly
- Pitfalls: HIGH on #1 (tokio start_kill semantics — known); MEDIUM on #4 (PTY async integration pattern — Context7-verified structurally, not executed); MEDIUM on #2 (schema round-trip — depends on schemars 1.2 Value→Schema conversion not tested in this session)
- Security / threats: HIGH on #1, #5, #11; MEDIUM on #2, #3 (depend on Phase 4 sandbox)
- Image plumbing (§10): MEDIUM — path explored (data URI) but not code-tested; hinges on `forge_app::domain::Image` surface which planner re-verifies

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (30 days — stable ecosystem; portable-pty 0.8 is stable; Rust 1.95 stable)

---

## RESEARCH COMPLETE

Research is planner-ready. Five plan partitions recommended (03-01..03-05), 2 pairs parallelizable (03-02 ‖ 03-03), remaining serial. All 11 requirement IDs covered in Validation Architecture. 12 locked decisions respected; 16 open questions flagged for planner decision during plan-write. Three high-priority planner decisions: (Q16) `Tool::invoke` signature re call_id threading, (Q5) image data URI vs. request-body extension, (Q6) ForgeConfig extension vs. KayConfig overlay.
