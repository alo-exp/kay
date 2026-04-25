---
phase: 4
flow: 6
gate_date: 2026-04-21
mode: design-time
verdict: PASS
---

# Phase 4 Quality Gates — Design-time

**Mode:** design-time (no PLAN.md exists yet)
**Overall verdict:** PASS — proceed to discuss-phase and planning

---

## Quality Gates Report

| Dimension | Result | Notes |
|-----------|--------|-------|
| Modularity | ✅ PASS | 4-crate split (kay-sandbox-policy + 3 OS crates); single responsibility per crate; change locality ≤5 files per OS impl; dispatcher.rs + rng.rs additions are <50 lines each |
| Reusability | ✅ PASS | `SandboxPolicy` is single source of truth; `Sandbox` trait composition (no inheritance); `dispatch()` reusable Phase 5 entry point; `RngSeam` reusable for test determinism |
| Scalability | ⚠️ CONDITIONAL | macOS `sandbox-exec` adds one process fork per subprocess spawn. At 100 tool calls/task (TB 2.0 realistic), overhead must stay <5ms/spawn. **CI benchmark gate required.** |
| Security | ✅ PASS | Defense-in-depth: pre-flight userspace check (`check_shell/fs/net`) + kernel enforcement (sandbox-exec/Landlock/Job Objects). SBPL profile built from `SandboxPolicy` struct (no user-controlled injection). Advisory: document Landlock symlink behavior (Linux follows symlinks — denied subtree is denied even via symlink). |
| Reliability | ⚠️ CONDITIONAL | `KaySandboxMacos::new()` must return `Result<_, SandboxError>` where `BackendUnavailable` covers missing `sandbox-exec` binary (macOS 15 deprecation risk). Must NOT `panic!` on init failure. Landlock degradation to seccomp-only is correctly handled via `tracing::warn!`. |
| Usability | ⚠️ CONDITIONAL | `AgentEvent::SandboxViolation.policy_rule: String` must use pre-defined constants (not freeform per-OS strings that diverge). `kay-sandbox-policy` must export `RULE_WRITE_OUTSIDE_ROOT`, `RULE_READ_DENIED_PATH`, `RULE_NET_NOT_ALLOWLISTED` constants. Otherwise violation messages will be inconsistent across macOS/Linux/Windows. |
| Testability | ✅ PASS | `Arc<dyn Sandbox>` DI in `ToolContext`; `Arc<dyn RngSeam>` for nonces; `NoOpSandbox` for unit tests; 68-test pyramid with 6-test kernel-level escape suite; OS-gated tests via `#[cfg_attr]`; `tempfile` isolation for escape tests |
| Extensibility | ✅ PASS | New OS = new crate implementing `Sandbox` trait (no existing code modified); `Sandbox` trait stable since Phase 3 D-12; `AgentEvent` is `#[non_exhaustive]` — additive `SandboxViolation` variant is safe |
| AI/LLM Safety | ⚠️ CONDITIONAL | `AgentEvent::SandboxViolation` is a user-facing event, not a tool result. **Phase 5 agent loop MUST NOT re-inject `SandboxViolation` into the model's context as a tool response.** It must be surfaced to the CLI/TUI/GUI event stream only. Re-injecting it would teach the model to route around the policy (meta-level prompt injection risk). Document as Phase 5 planning constraint. |

---

## Failures Requiring Redesign

None. All dimensions pass. The 4 conditionals below are planning-phase advisories — they must appear as constraints in PLAN.md but do not block the gate.

---

## Planning Constraints (must land in PLAN.md)

| ID | Constraint | Dimension |
|----|-----------|-----------|
| QG-C1 | CI adds macOS benchmark test: `execute_commands("echo x", project_root)` latency <5ms vs. Phase 3 baseline (proptest or criterion micro-benchmark) | Scalability |
| QG-C2 | `KaySandboxMacos::new()` returns `Result<Self, SandboxError>` where `SandboxError::BackendUnavailable` covers `sandbox-exec` not found; construction failure is logged + propagates, never panics | Reliability |
| QG-C3 | `kay-sandbox-policy` crate exports `RULE_*` string constants for all policy rule messages; all OS backend `SandboxDenial.reason` strings MUST use these constants | Usability |
| QG-C4 | Phase 5 agent loop planning constraint: `SandboxViolation` events are routed to the UI/user event stream; they MUST NOT be serialized back into the model's message history as tool call results | AI/LLM Safety |

---

## Non-Negotiables Compliance (design-time)

| NN | Status |
|----|--------|
| NN#1 ForgeCode parity gate | N/A — no parity-sensitive changes in Phase 4 |
| NN#2 ED25519 signed release tags | Phase 4 tag will be v0.2.0, signed at ship time |
| NN#3 DCO on every commit | Standing directive; all Phase 4 commits must carry `Signed-off-by` |
| NN#4 Clean-room (no TS-derived structure) | Phase 4 is pure Rust OS syscall work; no TypeScript in scope |
| NN#5 Single binary, no externalBin | `sandbox-exec` is a system binary (not Kay's binary); no `externalBin` entry added |
| NN#6 OpenRouter allowlist | `SandboxPolicy.network_allowlist` default: `[{host: "openrouter.ai", port: 443}]` — enforces NN#6 at kernel level |
| NN#7 Schema hardening | N/A — no new tool schemas in Phase 4 |

---

Quality gates passed (design-time). Proceed to `/gsd-discuss-phase 4`.
