# Phase 4: Sandbox — All Three Platforms — Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 swaps `NoOpSandbox` (pass-through) with real per-OS kernel-enforced sandbox backends for every shell, file, and network action dispatched through the `ToolRegistry`. The default policy confines writes to the project root and permits network only to the configured OpenRouter host. All three target OSes — macOS, Linux, Windows — are exercised in CI with a must-fail kernel-level escape suite.

**Also closes:** Phase 3 residuals R-4 (Windows Job Objects timeout cascade) and R-5 (populate `dispatcher.rs` + `rng.rs` stubs).

**Phase 4 entry gate (SC#5):** `cargo test --workspace --all-targets` must pass cleanly. Fixed via cherry-picks 27a4dd4 + 7626d0b (forge_domain json-feature flag + missing conversation.json fixture). Confirmed: 580 forge_domain tests pass.

**Not in Phase 4:**
- Policy customization UI (Phase 10)
- Container-based isolation / Docker (conflicts NN#5 — single binary)
- Network packet-level filtering (iptables/PF)
- Fine-grained per-tool policies
- Phase 3 residuals R-1, R-2, R-6 (Phase 5/MCP)

</domain>

<decisions>
## Implementation Decisions

### macOS Sandbox Backend

- **D-01:** Use `sandbox-exec -p <profile-string>` with the SBPL profile cached as `Arc<str>` at `KaySandboxMacos::new()` construction time. No per-call file I/O. Profile string generated once from `SandboxPolicy`. Hot path: `Command::new("sandbox-exec").args(["-p", &self.cached_profile, "--", "sh", "-c", cmd])`.
- **D-01a:** `sandbox-exec` deprecation on macOS 15 is acknowledged; it remains functional through 15.x. No action in Phase 4. Phase 11 will monitor.
- **D-01b (QG-C2):** `KaySandboxMacos::new()` returns `Result<Self, SandboxError>` where `SandboxError::BackendUnavailable` is the variant for missing `sandbox-exec` binary. Must NOT `panic!` on init failure.

### Linux Sandbox Backend

- **D-02:** Use `landlock` crate (v0.4+) + `seccomp` BPF filter for defense-in-depth. Landlock restricts filesystem path access; seccomp restricts syscall surface.
- **D-02a:** Graceful degradation: if `landlock::Ruleset::create()` returns `CompatError` (kernel < 5.13), emit `tracing::warn!("Landlock unavailable — falling back to seccomp-only sandbox")` and continue with seccomp-only. No `AgentEvent::Warning` needed in Phase 4; documented in 04-SECURITY.md.
- **D-02b:** GitHub Actions `ubuntu-latest` runs kernel 5.15+ — Landlock available in CI. Degradation path is a production safety net for self-hosted runners.
- **D-02c:** Advisory: document Landlock symlink behavior (Linux Landlock is path-based — denied subtree is denied even via symlink) in 04-SECURITY.md.

### Windows Sandbox Backend

- **D-03:** Use `windows-sys` raw bindings (already in workspace `[target.'cfg(windows)'.dependencies]` of `kay-tools/Cargo.toml`). No new crate dependency needed.
- **D-03a:** Job Object: `CreateJobObjectW` + `AssignProcessToJobObject` + `SetInformationJobObject(JobObjectBasicLimitInformation)` with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. This closes **R-4**: grandchild processes are killed when Kay exits (symmetric to Unix `killpg(pgid, SIGKILL)` locked in Phase 3).
- **D-03b:** Restricted token: `CreateRestrictedToken` + `DISABLE_MAX_PRIVILEGE` to drop excess privileges from child process.

### SandboxPolicy Crate

- **D-04:** New `kay-sandbox-policy` workspace crate at `crates/kay-sandbox-policy/`. Contains `SandboxPolicy`, `SandboxRule`, `NetAllow`, and **policy_rule string constants** (QG-C3). Zero OS deps — only `serde` + `std`.
- **D-04a (QG-C3):** All OS backend implementations MUST use `kay_sandbox_policy::rules::RULE_WRITE_OUTSIDE_ROOT`, `RULE_READ_DENIED_PATH`, `RULE_NET_NOT_ALLOWLISTED` constants for `SandboxDenial.reason` strings. Free-form strings are forbidden.
- **D-04b:** Default policy: `SandboxPolicy::default_for_project(project_root: PathBuf)` produces: writes allowed in `<project_root>` subtree; `~/.aws`, `~/.ssh`, `~/.gnupg`, `~/.gnupg2` deny-listed for reads; network allowlist: `[{host: "openrouter.ai", port: 443}]`; config dir `~/.config/kay/` read-only.
- **D-04c:** DAG: `kay-sandbox-policy` ← `kay-sandbox-macos/linux/windows` ← `kay-cli`. `kay-tools` has the `Sandbox` trait + `NoOpSandbox` (unchanged from Phase 3).

### AgentEvent Extension

- **D-05:** Add `AgentEvent::SandboxViolation` variant to `kay-tools/src/events/mod.rs` (or wherever `AgentEvent` is defined — Phase 3 placed it in `kay-tools`). Shape:
  ```rust
  SandboxViolation {
      call_id: String,
      tool_name: String,
      resource: String,
      policy_rule: String,   // MUST use RULE_* constants from kay-sandbox-policy
      os_error: Option<i32>, // None for pre-flight; Some(errno) for kernel-level
  }
  ```
- **D-05a:** `AgentEvent` is already `#[non_exhaustive]` — this is an additive variant. No breaking change.
- **D-05b (QG-C4):** Phase 5 planning constraint (record now): `SandboxViolation` events must be surfaced to the user/UI event stream ONLY — they must NOT be serialized back into the model's message history as tool call results. Re-injection would teach the model to route around policy (meta-level prompt injection).

### dispatcher.rs (R-5 closure)

- **D-06:** Populate `crates/kay-tools/src/runtime/dispatcher.rs` with `pub async fn dispatch(registry: &ToolRegistry, sandbox: Arc<dyn Sandbox>, call: &ToolCall) -> AgentEvent`. This is also the Phase 5 agent-loop entry point (no rework needed).
- **D-06a:** `dispatch()` performs tool-type-aware sandbox pre-flight: `execute_commands` → `check_shell`; `fs_read/search/image_read` → `check_fs_read`; `fs_write` → `check_fs_write`; `net_fetch` → `check_net`. On denial: returns `AgentEvent::SandboxViolation` without calling `tool.execute()`.
- **D-06b:** Unknown tool name returns `AgentEvent::ToolCallMalformed` (existing variant from Phase 3).

### rng.rs (R-5 closure)

- **D-07:** Populate `crates/kay-tools/src/seams/rng.rs` with `RngSeam` trait + `OsRngSeam` (prod) + `DeterministicRng` (cfg(test) only). `MarkerFactory` gains `Arc<dyn RngSeam>` constructor parameter; default constructor uses `OsRngSeam` (non-test path unchanged).
- **D-07a:** This enables deterministic marker nonce testing in Phase 4 + Phase 5 escape tests.

### CI Matrix

- **D-08:** 3-OS matrix added to `.github/workflows/ci.yml`: `macos-14` (arm64 M1), `ubuntu-latest` (kernel 5.15+), `windows-latest` (Win Server 2022 x64). Each matrix leg runs `cargo test --workspace --all-targets`.
- **D-08a:** OS-specific escape tests use `#[cfg_attr(not(target_os = "macos"), ignore)]` pattern for platform gating.
- **D-08b (QG-C1):** macOS benchmark gate: `execute_commands("echo x", project_root)` latency must be <5ms vs. Phase 3 baseline. Implemented as a criterion micro-benchmark or proptest timing assertion in `kay-sandbox-macos/benches/`.

### Landlock Degradation

- **D-09:** `tracing::warn!` + continue with seccomp-only on ENOSYS. No panic; no hard failure. Document in `04-SECURITY.md`.

### Escape Test Architecture

- **D-10:** Escape tests use `std::process::Command` to spawn real child processes attempting forbidden operations. No root required. Child process exits non-zero or returns `PermissionDenied` — parent asserts this. 6 canonical escape tests per OS: write-outside-root, read-aws-credentials, read-ssh-dir, write-within-root (must-pass sanity), net-non-allowlisted, net-allowlisted.

### Claude's Discretion

- SBPL profile string formatting (whitespace, comment style) — planner decides
- seccomp BPF filter specific syscall list (minimum viable) — planner + researcher decide
- Exact Cargo.toml dep versions for `landlock` crate — researcher verifies current stable

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 4 Design Artifacts
- `.planning/phases/04-sandbox/04-BRAINSTORM.md` — Full PM + engineering brainstorm (D-01..D-10, E1..E10)
- `.planning/phases/04-sandbox/04-TEST-STRATEGY.md` — 68-test pyramid, escape suite, CI matrix requirements
- `.planning/phases/04-sandbox/04-QUALITY-GATES.md` — 9-dimension gate (design-time PASS); 4 planning constraints QG-C1..C4

### Phase 3 Artifacts (carry-forward)
- `.planning/phases/03-tool-registry-kira-core-tools/03-SECURITY.md` — R-4 + R-5 residual definitions (§4)
- `.planning/phases/03-tool-registry-kira-core-tools/03-CONTEXT.md` — D-12 (Sandbox DI seam, `Arc<dyn Sandbox>`)

### Codebase Seam Files
- `crates/kay-tools/src/seams/sandbox.rs` — Sandbox trait + SandboxDenial + NoOpSandbox (Phase 4 extends this)
- `crates/kay-tools/src/runtime/dispatcher.rs` — Empty stub (R-5, Phase 4 populates)
- `crates/kay-tools/src/seams/rng.rs` — Empty stub (R-5, Phase 4 populates)
- `crates/kay-tools/Cargo.toml` — `[target.'cfg(windows)'.dependencies]` already has `windows-sys`

### Stub Sandbox Crates (to be populated in Phase 4)
- `crates/kay-sandbox-macos/` — Empty lib stub
- `crates/kay-sandbox-linux/` — Empty lib stub
- `crates/kay-sandbox-windows/` — Empty lib stub

### Project Governance
- `.planning/ROADMAP.md` §Phase 4 — 7 success criteria (SC#1..SC#7)
- `CLAUDE.md` — Non-Negotiables NN#1..NN#7
- `.planning/PROJECT.md` — Core value, architectural constraints

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/kay-tools/src/seams/sandbox.rs`: `Sandbox` trait with `async_trait`, `SandboxDenial` struct — Phase 4 implements this interface, does not modify it
- `crates/kay-tools/src/runtime/context.rs`: `ToolContext` holds `Arc<dyn Sandbox>` DI seam — already wired, no change needed
- `crates/kay-tools/Cargo.toml` `[target.'cfg(windows)'.dependencies]`: `windows-sys` already present — add Job Object bindings without new dep
- `crates/forge_domain/Cargo.toml`: json feature gate fixed (cherry-pick 27a4dd4) — entry gate clear

### Established Patterns
- Phase 3 used `async_trait::async_trait` for object-safe async traits — use same pattern in OS backends
- Phase 3 used `subtle::ConstantTimeEq` for constant-time compare — available in workspace
- Phase 3 `timeout_cascade.rs` pattern: `tokio::time::timeout` + `killpg` on Unix — mirror for Windows with Job Objects in R-4 regression test
- Phase 3 commit pattern: DCO `Signed-off-by` on every commit, atomic wave commits

### Integration Points
- `kay-cli/src/main.rs` — startup creates `Arc<dyn Sandbox>`; Phase 4 replaces `Arc::new(NoOpSandbox)` with `Arc::new(KaySandboxMacos::new(policy)?)` (or Linux/Windows) via `#[cfg(target_os)]`
- `crates/kay-tools/src/runtime/dispatcher.rs` — Phase 4 populates this; Phase 5 uses it as its entry point

</code_context>

<specifics>
## Specific Ideas

- The must-fail escape test suite is the most valuable Phase 4 deliverable — it is the only proof that kernel enforcement actually works. CI must treat it as a required gate (not an optional test run).
- macOS `sandbox-exec` inline profile cached at construction allows zero per-call overhead — performance goal achievable.
- The `kay-sandbox-policy` string constants (RULE_*) are both a usability fix (QG-C3) and a security documentation mechanism — they create a canonical vocabulary for sandbox violations across the codebase and UI.

</specifics>

<deferred>
## Deferred Ideas

- Fine-grained per-tool sandbox profiles (e.g., `net_fetch` has broader network access than `execute_commands`) — Phase 10/11
- Policy customization UI — Phase 10
- Container-based isolation (Docker) — Conflicts NN#5 (single binary), post-v1 if ever
- Network packet-level filtering (iptables/PF) — too invasive for local dev tool, post-v1
- Phase 3 residuals R-1 (PTY metacharacter heuristic) + R-2 (ImageRead size cap) — Phase 5
- Phase 3 residual R-6 (rmcp advisory) — MCP phase
- Phase 5 agent loop filter for SandboxViolation (QG-C4) — must be documented in Phase 5 CONTEXT.md planning constraint

</deferred>

---

*Phase: 04-sandbox*
*Context gathered: 2026-04-21*
