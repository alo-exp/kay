# Phase 3: Tool Registry + KIRA Core Tools - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `03-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 03-tool-registry-kira-core-tools
**Mode:** Auto-resolved per user standing directive ("proceed autonomously, decide best options, never get stalled, never skip GSD steps")
**Areas discussed:** Tool trait architecture, Schema hardening, Marker protocol, PTY strategy, Timeout + termination, task_complete verifier, image_read caps, AgentEvent extensions, Error taxonomy, Tool set scope, Registration flow, Sandbox seam

---

## Tool Trait Architecture (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| A. New `kay-tools` crate with `pub trait Tool` (Arc\<dyn Tool\>) + Registry; built-ins delegate to forge_app::ToolExecutor | Matches TOOL-01 explicitly; parity-preserving via delegation; clean crate boundary for Phase 5 consumers | ✓ |
| B. Extend `forge_domain::ToolCatalog` enum with KIRA variants | Breaks TOOL-01 object-safety; forces compile-in to parity-imported crate | |
| C. Put trait in `kay-core` | Bloats thin aggregator crate (post-Phase-2.5) | |
| D. Put trait in `kay-provider-openrouter` | Wrong layer — tool execution is provider-agnostic | |

**Selected:** A. New crate `kay-tools`.
**Notes:** TOOL-01 is explicit about `Arc<dyn Tool>` — this overrides parity. Parity is preserved because built-in impls delegate to `forge_app::ToolExecutor::execute` which in turn calls the unchanged `forge_services` tool_services. The trait surface is the *only* Kay-owned layer.

---

## Schema Hardening (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Reuse `forge_app::utils::enforce_strict_schema` unchanged; add Kay-side wrapper for truncation reminders | Canonical impl already exists in-tree; TOOL-05 named "ForgeCode hardening" as the spec | ✓ |
| B. Write a fresh hardener in `kay-tools` | Drifts from ForgeCode behavior across upstream syncs; regresses parity | |
| C. Inline hardening at the provider schema-emit step | Couples provider to tool internals; misses non-OpenRouter consumers (future TUI schema preview, etc.) | |

**Selected:** A.
**Notes:** `enforce_strict_schema` at `crates/forge_app/src/utils.rs:351` already implements sorted `required` covering ALL property keys, `allOf` flattening, `additionalProperties: false`, `propertyNames` stripping, `nullable → anyOf[..., null]`. The KIRA-specific addition (truncation reminders) is a tiny description-string wrap, not a schema mutation.

---

## `execute_commands` Marker Protocol (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| A. `__CMDEND_<128-bit-nonce>_<seq>__EXITCODE=N` with OsRng + constant-time compare | Prompt-injection-resistant; matches KIRA writeup; constant-time defends against timing side-channels | ✓ |
| B. Static `__CMDEND__` marker | Trivially forgeable by attacker-controlled stdout; breaks the harness guarantee | |
| C. Exit-code-via-tempfile (write `$?` to a sandbox-writable path, poll) | Adds sandbox complexity (Phase 4 must grant write); doesn't solve prompt-injection | |
| D. Out-of-band signal (named pipe / unix socket) | Not portable to Windows without separate codepath; more complex than string scanning | |

**Selected:** A.
**Notes:** SHELL-01 + SHELL-05 are hard invariants. 128-bit entropy matches KIRA's writeup; the per-session seq is belt-and-suspenders. `subtle::ConstantTimeEq` is cheap and defensive.

---

## PTY vs Non-PTY (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Default to tokio::process; switch to portable-pty on heuristic denylist OR explicit tty:true flag | Fast common-case; PTY only when needed; matches ForgeCode parity for non-PTY work | ✓ |
| B. Always PTY (Terminus-style) | 10-50× slower startup for headless `cargo test` runs; no parity benefit | |
| C. Never PTY (ForgeCode parity) | SHELL-02 explicitly requires PTY fallback for TTY-requiring commands | |

**Selected:** A.
**Notes:** Heuristic denylist is intentionally small and config-overridable. Adding `portable-pty = "0.8"` as a new workspace dep — license verified (MIT/Apache).

---

## Hard Timeout + Termination (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Reuse `ForgeConfig.tool_timeout_secs`; SIGTERM → 2s grace → SIGKILL → wait for reap | ForgeCode pattern; satisfies SHELL-04 across all three OSes | ✓ |
| B. Separate Kay timeout config | Unnecessary config surface; forge_config is already loaded at startup | |
| C. Linux only: cgroups freezer | Phase 4 sandbox territory; out of scope | |

**Selected:** A.
**Notes:** Windows signal propagation is a stopgap via TerminateProcess; proper process-group kill lands with Phase 4 Job Objects.

---

## `task_complete` + Verifier (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Ship `NoOpVerifier` returning `VerificationOutcome::Pending`; Phase 8 swaps in real impl via trait DI | Honors Success Criterion #5 (no false success); unblocks Phase 5 agent loop | ✓ |
| B. Stub that always returns success | Silently breaks SC #5; makes Phase 8 a no-op at the agent layer | |
| C. Defer the tool entirely to Phase 8 | Blocks Phase 5 (LOOP-05 needs tool existence as a signal) | |

**Selected:** A.
**Notes:** `TaskVerifier` trait is the Phase-8 seam. `NoOpVerifier` is a real implementation (not a panic) — production-safe.

---

## `image_read` Caps (D-07)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Per-turn = 2, Per-session = 20 (upper of spec range), overridable via kay.toml + env | Maximizes useful bandwidth; can tune down at Phase 12 if TB 2.0 runs show regression | ✓ |
| B. Per-turn = 1, Per-session = 10 (lower) | More conservative but may starve legitimate multi-image workflows | |
| C. Dynamic cap based on cost cap | Over-engineered for v1; couples image limits to cost logic | |

**Selected:** A.
**Notes:** ROADMAP SC #4 gives a range (1–2 per turn, 10–20 per session); picking the upper end is the "decide best options" default. Tuneable at Phase 12.

---

## `AgentEvent` Extensions (D-08)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Add `ToolOutput { call_id, chunk: ToolOutputChunk }` + `TaskComplete { call_id, verified, outcome }` | Additive per `#[non_exhaustive]`; no Phase 2 break | ✓ |
| B. Overload existing `ToolCallComplete` | Semantically wrong — `ToolCallComplete` is the provider-side tool-call parse signal, not execution output | |

**Selected:** A.
**Notes:** Both new variants are load-bearing for Phase 5 (agent loop reads `verified` to decide loop continuation) and Phase 9/9.5 (frontends render streaming stdout).

---

## `ToolError` Taxonomy (D-09)

| Option | Description | Selected |
|--------|-------------|----------|
| A. Dedicated `ToolError` enum in `kay-tools`, separate from `ProviderError` | Clean match-arm separation; downstream consumers know which layer failed | ✓ |
| B. Reuse `ProviderError` with new variants | Mixes wire errors with tool errors; makes `ProviderError::SandboxDenied` semantically odd | |
| C. Return `anyhow::Error` everywhere | Loses typed diagnosis; violates Phase 2 pattern | |

**Selected:** A.

---

## Phase 3 Tool Set Scope (D-10)

| Option | Description | Selected |
|--------|-------------|----------|
| A. KIRA trio + parity minimum (fs_read, fs_write, fs_search, net_fetch) — 7 tools | Sufficient for TB 2.0 minimal runs; tight blast radius for first integration | ✓ |
| B. Ship full ForgeCode catalog (17 tools) | Bloats Phase 3 integration; each tool adapter is a potential deviation | |
| C. KIRA trio only (3 tools) | Not enough to run realistic benchmarks; delays Phase 5 validation | |

**Selected:** A.
**Notes:** Deferred tools (fs_patch, plan_create, skill_fetch, etc.) have in-tree implementations; re-activating each is a ~1-plan increment in Phase 5+.

---

## Tool Registration Flow (D-11)

| Option | Description | Selected |
|--------|-------------|----------|
| A. `kay-cli` builds immutable registry at startup via `default_tool_set(...)` | Matches TOOL-01 architecture; no runtime mutation; simpler concurrency | ✓ |
| B. Runtime registration API exposed to agents | v2 plugin feature; out of scope | |
| C. Hot-reload from config | No requirement justifies this; adds security surface | |

**Selected:** A.

---

## Sandbox Integration Seam (D-12)

| Option | Description | Selected |
|--------|-------------|----------|
| A. `Arc<dyn Sandbox>` via DI; Phase 3 ships NoOpSandbox; Phase 4 swaps impl | Defines exact extension point now; saves Phase 4 refactor | ✓ |
| B. No sandbox abstraction in Phase 3; Phase 4 retrofits | Forces Phase 4 to touch tool impls; risks deviation during retrofit | |
| C. Hardcode per-OS sandbox calls in Phase 3 | Out of scope per REQ boundaries (SBX-01..04 are Phase 4) | |

**Selected:** A.

---

## Claude's Discretion (noted in CONTEXT.md)

- `kay-tools` module layout (per-tool modules vs. single file)
- `harden_tool_schema` in-place vs. owned return
- Marker-race test harness (mocks vs. integration)
- `ToolCallContext` Kay-own vs. re-export from `forge_domain`
- ForgeCode `tool_timeout_secs` default override (300s → ?)
- `subtle` vs. transitive constant-time lib
- Test split between unit (`kay-tools/src`) and integration (`kay-tools/tests/`)
- Distinct `ToolCallStart` frame at invocation vs. relying on Phase 2 frames

## Deferred Ideas (noted in CONTEXT.md)

- MCP integration → new ROADMAP phase; blocked on rustls-webpki (issue #3)
- Runtime user tool registration → Phase 12+
- `task`, `sem_search`, `plan_create`, `todo_*`, `followup` tools → Phase 5+
- Gemini / Anthropic schema variants → v2
- Cost-aware tool throttling → v2
- Session tool-output persistence → Phase 6
- Multi-perspective verifier → Phase 8
