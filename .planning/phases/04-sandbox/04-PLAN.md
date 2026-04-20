---
phase: 4
goal: "Replace NoOpSandbox with real per-OS kernel-enforced sandbox on macOS/Linux/Windows. Close R-4 + R-5. Ship 68-test pyramid with kernel escape suite passing on all 3 CI OSes."
requirements: ["REQ-SEC-01", "REQ-SEC-02", "REQ-SEC-03", "REQ-PERF-01", "REQ-OPS-01"]
created: 2026-04-21
status: active
---

# Phase 4 Plan: Sandbox — All Three Platforms

## Success Criteria (from ROADMAP.md §Phase 4)

- SC#1: `cargo test --workspace --all-targets` passes (entry gate — already green via cherry-picks)
- SC#2: `KaySandboxMacos` / `KaySandboxLinux` / `KaySandboxWindows` each implement `Sandbox` trait
- SC#3: Escape suite (E-01..E-06) passes on macos-14, ubuntu-latest, windows-latest CI
- SC#4: `AgentEvent::SandboxViolation` serializes correctly (snapshot test)
- SC#5: `SandboxPolicy` round-trips through serde (unit test)
- SC#6: R-4 closed — Windows Job Objects kill grandchild on parent exit
- SC#7: R-5 closed — `dispatch()` + `RngSeam` trait populated and tested

## Planning Constraints (from QG-C1..C4)

- **QG-C1:** macOS spawn latency benchmark < 5ms (criterion or proptest micro-bench)
- **QG-C2:** `KaySandboxMacos::new()` returns `Result<_, SandboxError::BackendUnavailable>` — no panic
- **QG-C3:** All OS backends use `RULE_*` constants from `kay-sandbox-policy`
- **QG-C4:** `SandboxViolation` NOT re-injected into model context (document as Phase 5 constraint)

---

## Wave Structure

### Wave 0 — Workspace Setup (no tests yet)

**Goal:** Add `kay-sandbox-policy` crate to workspace; update all Cargo.tomls; `cargo check --workspace` passes.

**Tasks:**
1. Create `crates/kay-sandbox-policy/` with `Cargo.toml` + `src/lib.rs` (empty `pub mod`)
2. Add `"crates/kay-sandbox-policy"` to workspace `members` in root `Cargo.toml`
3. Add `landlock = "0.4"` + `seccompiler = "0.5"` to `[workspace.dependencies]`
4. Expand `windows-sys` workspace dep: add `Win32_Security` + `Win32_System_JobObjects` features
5. Update `crates/kay-sandbox-macos/Cargo.toml` — add deps: `kay-sandbox-policy`, `kay-tools`, `async-trait`, `tokio`, `tracing`, `thiserror`
6. Update `crates/kay-sandbox-linux/Cargo.toml` — add deps (+ `[target.cfg(linux)]` for `landlock` + `seccompiler`)
7. Update `crates/kay-sandbox-windows/Cargo.toml` — add deps (+ `[target.cfg(windows)]` for `windows-sys`)
8. Verify: `cargo check --workspace 2>&1 | tail -5`

**Commit:** `feat(04): scaffold kay-sandbox-policy crate and update workspace deps`

---

### Wave 1 — SandboxPolicy Crate (TDD)

**Goal:** `kay-sandbox-policy` implements `SandboxPolicy`, `NetAllow`, `RULE_*` constants, `SandboxError`. All unit tests green.

**Test files:** `crates/kay-sandbox-policy/src/policy.rs` (inline `#[cfg(test)]` modules)
**Tests:** U-01..U-12 (policy serde, NetAllow matching, rule constants, default_for_project)

**RED tasks (write failing tests first):**
1. `test_policy_default_for_project` — asserts write root, deny-listed paths, net allowlist
2. `test_net_allow_matching` — asserts openrouter.ai:443 allowed; arbitrary host denied
3. `test_policy_serde_roundtrip` — serialize → deserialize → assert eq
4. `test_rule_constants_nonempty` — `RULE_WRITE_OUTSIDE_ROOT.len() > 0` etc.
5. `test_sandbox_error_display` — `SandboxError::BackendUnavailable` has useful message

**GREEN tasks (implement):**
```rust
// crates/kay-sandbox-policy/src/lib.rs
pub mod policy;
pub mod rules;
pub mod error;
```

```rust
// crates/kay-sandbox-policy/src/policy.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SandboxPolicy {
    pub write_roots: Vec<PathBuf>,
    pub read_deny_list: Vec<PathBuf>,
    pub network_allowlist: Vec<NetAllow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NetAllow { pub host: String, pub port: u16 }

impl SandboxPolicy {
    pub fn default_for_project(project_root: PathBuf) -> Self { ... }
    pub fn allows_net(&self, host: &str, port: u16) -> bool { ... }
    pub fn allows_write(&self, path: &Path) -> bool { ... }
    pub fn allows_read(&self, path: &Path) -> bool { ... }
}
```

```rust
// crates/kay-sandbox-policy/src/rules.rs
pub const RULE_WRITE_OUTSIDE_ROOT: &str = "write-outside-project-root";
pub const RULE_READ_DENIED_PATH: &str = "read-denied-path";
pub const RULE_NET_NOT_ALLOWLISTED: &str = "net-not-allowlisted";
pub const RULE_SHELL_DENIED: &str = "shell-exec-denied";
```

```rust
// crates/kay-sandbox-policy/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("sandbox backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("platform not supported")]
    PlatformNotSupported,
    #[error("policy build error: {0}")]
    PolicyBuild(String),
}
```

**REFACTOR:** extract `SandboxPolicy::allows_*` into trait if >3 callers share logic.

**Commit:** `feat(04-W1): implement SandboxPolicy crate with RULE_* constants — RED→GREEN`

---

### Wave 2 — RngSeam + dispatcher.rs in kay-tools (TDD)

**Goal:** Close R-5. `RngSeam` trait + impls in `rng.rs`; `dispatch()` skeleton in `dispatcher.rs`. Tests U-30..U-36 green.

**RED tasks:**
1. `test_deterministic_rng_produces_same_output` — seed 42 → same hex twice
2. `test_os_rng_produces_unique_output` — two calls differ (probabilistic)
3. `test_dispatch_routes_to_registered_tool` — mock `ToolRegistry` entry → `dispatch()` calls it
4. `test_dispatch_unknown_tool_returns_error` — unregistered name → `Err`

**GREEN tasks:**

```rust
// crates/kay-tools/src/seams/rng.rs
pub trait RngSeam: Send + Sync {
    fn hex_nonce(&self, len: usize) -> String;
}

pub struct OsRngSeam;
impl RngSeam for OsRngSeam {
    fn hex_nonce(&self, len: usize) -> String {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        hex::encode(&bytes)
    }
}

pub struct DeterministicRng { pub seed: u64 }
impl RngSeam for DeterministicRng { ... }
```

```rust
// crates/kay-tools/src/runtime/dispatcher.rs
pub async fn dispatch(
    registry: &ToolRegistry,
    sandbox: &Arc<dyn Sandbox>,
    call: ToolCall,
    ctx: &ToolContext,
) -> Result<ToolResult, DispatchError> {
    let tool = registry.get(&call.name).ok_or(DispatchError::UnknownTool(call.name.clone()))?;
    // pre-flight sandbox check happens in each tool's execute() via ctx.sandbox
    tool.execute(call.arguments, ctx).await
}
```

**Commit:** `feat(04-W2): populate RngSeam trait + dispatch() — close R-5`

---

### Wave 3 — AgentEvent::SandboxViolation (TDD)

**Goal:** Add `SandboxViolation` variant to `AgentEvent`. Snapshot test passes.

**RED tasks:**
1. `test_sandbox_violation_serializes` — `AgentEvent::SandboxViolation { ... }` → serde_json roundtrip
2. `test_sandbox_violation_fields` — all 5 fields present in JSON output

**GREEN tasks:**

```rust
// crates/kay-tools/src/events/mod.rs — add to existing AgentEvent enum
#[non_exhaustive]
pub enum AgentEvent {
    // ... existing variants ...
    SandboxViolation {
        call_id: String,
        tool_name: String,
        resource: String,
        policy_rule: String,   // MUST be RULE_* constant value
        os_error: Option<i32>, // None = pre-flight; Some(errno) = kernel
    },
}
```

**Commit:** `feat(04-W3): add AgentEvent::SandboxViolation variant`

---

### Wave 4 — macOS Backend (TDD)

**Goal:** `KaySandboxMacos` implements `Sandbox` trait with inline SBPL profile caching. Unit + integration tests green on macOS; skip-guarded on other OSes.

**Test file:** `crates/kay-sandbox-macos/src/tests.rs`

**RED tasks:**
1. `test_new_returns_ok_when_sandbox_exec_present` — `KaySandboxMacos::new(policy)` on macOS
2. `test_sbpl_profile_cached_not_regenerated` — `Arc::ptr_eq` on two calls
3. `test_check_shell_allows_echo` — `check_shell("echo hi", root)` → `Ok(())`
4. `test_check_fs_write_denied_outside_root` — path outside root → `Err(SandboxDenial)`
5. `test_escape_write_outside_root` — real subprocess writes `/tmp/escape_test_*` → kernel denies (E-01)
6. `test_escape_net_not_allowlisted` — curl to arbitrary host → kernel denies (E-04)

**GREEN tasks:**

```rust
// crates/kay-sandbox-macos/src/lib.rs
pub struct KaySandboxMacos {
    cached_profile: Arc<str>,
    policy: Arc<SandboxPolicy>,
}

impl KaySandboxMacos {
    pub fn new(policy: SandboxPolicy) -> Result<Self, SandboxError> {
        let profile = build_sbpl_profile(&policy);
        // verify sandbox-exec exists
        which_sandbox_exec()?;
        Ok(Self { cached_profile: Arc::from(profile), policy: Arc::new(policy) })
    }
}

fn build_sbpl_profile(policy: &SandboxPolicy) -> String { ... }
fn which_sandbox_exec() -> Result<(), SandboxError> { ... }
```

**QG-C1 benchmark task:** Add `benches/spawn_latency.rs` with criterion benchmark asserting < 5ms per spawn.

**Commit:** `feat(04-W4): implement KaySandboxMacos with SBPL profile caching`

---

### Wave 5 — Linux Backend (TDD)

**Goal:** `KaySandboxLinux` implements Landlock + seccomp BPF with ENOSYS graceful degradation. OS-gated tests green on ubuntu-latest.

**RED → GREEN → REFACTOR same pattern as Wave 4.**

Key implementation:
```rust
pub struct KaySandboxLinux {
    policy: Arc<SandboxPolicy>,
    landlock_available: bool,
}

impl KaySandboxLinux {
    pub fn new(policy: SandboxPolicy) -> Result<Self, SandboxError> {
        let landlock_available = probe_landlock();
        if !landlock_available {
            tracing::warn!("Landlock unavailable (kernel < 5.13) — seccomp-only sandbox active");
        }
        Ok(Self { policy: Arc::new(policy), landlock_available })
    }
}

fn probe_landlock() -> bool {
    use landlock::{ABI, Access, AccessFs, Ruleset, RulesetAttr};
    Ruleset::default().create().is_ok()
}
```

**Escape tests (E-01..E-06):** `#[cfg_attr(not(target_os = "linux"), ignore)]`

**Commit:** `feat(04-W5): implement KaySandboxLinux with Landlock+seccomp`

---

### Wave 6 — Windows Backend + CI Matrix (TDD)

**Goal:** `KaySandboxWindows` implements Job Objects + restricted token. R-4 closed. CI matrix yml added. All 3-OS escape tests defined (Windows-gated).

Key R-4 fix:
```rust
// Job Object with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
unsafe {
    let job = CreateJobObjectW(null(), null());
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    SetInformationJobObject(job, JobObjectExtendedLimitInformation, ...);
    AssignProcessToJobObject(job, child_handle);
}
```

**CI matrix:**
```yaml
# .github/workflows/ci.yml
strategy:
  matrix:
    os: [macos-14, ubuntu-latest, windows-latest]
```

**Commit:** `feat(04-W6): implement KaySandboxWindows + Job Objects R-4 closure + CI matrix`

---

## Verification Plan

After all waves:
1. `cargo test --workspace --all-targets` — must show 68+ tests, 0 failures
2. `cargo check --workspace` — no warnings on any OS
3. `grep -r "RULE_WRITE_OUTSIDE_ROOT\|RULE_READ_DENIED_PATH\|RULE_NET_NOT_ALLOWLISTED" crates/kay-sandbox-{macos,linux,windows}` — constants used (not freeform strings)
4. `grep "SandboxViolation" crates/kay-tools/src/events/mod.rs` — variant present
5. `git log --oneline -10` — DCO `Signed-off-by` on every commit

## Threat Model

| Threat | Mitigation |
|--------|-----------|
| Escape via symlink (Linux) | Documented in 04-SECURITY.md; Landlock follows symlinks (known limitation) |
| sandbox-exec deprecated macOS 15 | Functional through 15.x; Phase 11 will monitor |
| Job Object bypass via inherited handle | `CreateRestrictedToken` drops excess privileges |
| Pre-flight check race (TOCTOU) | Kernel enforcement is authoritative; pre-flight is advisory only |
| SandboxViolation re-injected into model | QG-C4: Phase 5 constraint recorded in CONTEXT.md |
