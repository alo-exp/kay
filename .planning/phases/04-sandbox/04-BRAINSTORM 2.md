---
phase: 4
flow: 3a
brainstorm_date: 2026-04-21
lens: product-management
---

# Phase 4 Brainstorm — Sandbox: All Three Platforms

## §Product-Lens (PM Brainstorm)

### 1. Problem Definition

Kay is a local coding agent that executes shell commands, reads/writes files, and makes network calls on behalf of users — all through the Phase 3 `ToolRegistry`. Today, Phase 3's `NoOpSandbox` is a complete pass-through: a hallucinating model, an injected prompt from a malicious codebase, or a runaway loop can do literally anything the user's process can do.

**The gap:** There is no kernel-enforced barrier between "what Kay's model intends" and "what Kay's process does." The tool registry is powerful; Phase 4 makes it safe.

**This is not optional.** TB 2.0 evaluates agent correctness on real repos. A sandbox violation that corrupts the evaluator's filesystem is an immediate disqualification. Phase 4 is on the critical path to Phase 12 (>81.8% TB 2.0).

---

### 2. User Personas

| Persona | Who | Job To Be Done | Tolerance for Friction |
|---------|-----|----------------|------------------------|
| **Sam (Local Dev)** | Individual developer running Kay daily | "Protect my dotfiles from accidents, don't ask me every time" | Low — will disable sandbox if it breaks normal tool calls |
| **Alex (Security-Conscious Dev)** | Developer reviewing Kay before adopting | "Verify Kay doesn't access my credentials or phone home" | High — willing to read audit logs |
| **Pipeline (CI/CD Runner)** | Automated CI environment | "Sandbox must not produce false-positive failures in CI tasks" | Zero — any sandbox-induced failure fails the build |
| **Benchmark Operator** | TB 2.0 evaluation runner | "Sandbox overhead must be imperceptible; violations must be loud" | Zero — silent violations = wrong score |

---

### 3. User Value

**Without Phase 4:** Every tool call Kay makes runs with full user permissions. A prompt-injected `rm -rf ~/Projects` succeeds silently. Kay reading `~/.aws/credentials` to "understand the environment" is indistinguishable from a credential-exfiltration attack.

**With Phase 4:** The kernel rejects out-of-bounds writes. Network calls to non-OpenRouter hosts fail. The agent trace shows a structured `SandboxViolation` event — the user sees exactly what was attempted and blocked. Trust becomes verifiable, not assumed.

**The value proposition:** "Kay is safe by default. You don't have to trust the model — the kernel enforces the policy."

---

### 4. Success Metrics (PM view)

| Metric | Target | How Measured |
|--------|--------|--------------|
| Entry gate: workspace compiles + all tests pass | Zero failures (excl. pre-existing forge_app 39) | `cargo test --workspace --all-targets` |
| Must-fail escape suite passes | 100% of escape attempts blocked on all 3 OS | CI must-fail test results |
| Performance regression | <2ms per subprocess spawn overhead | Benchmark before/after execute_commands |
| `AgentEvent::SandboxViolation` surfaced in trace | All violations produce structured events | Integration test coverage |
| R-4 Windows timeout cascade closed | Job Objects kill grandchildren on Windows | `tests/timeout_cascade_win.rs` |
| R-5 dispatcher/rng modules populated | No empty TODO stubs | `grep -r "TODO(Wave" crates/kay-tools/` returns 0 |

---

### 5. Scope Boundaries

**IN Phase 4:**
- macOS `sandbox-exec` profile authoring + subprocess wrapping
- Linux Landlock file-access control + seccomp syscall filter
- Windows Job Objects (timeout cascade, R-4) + restricted token
- `AgentEvent::SandboxViolation` variant (additive, `#[non_exhaustive]`)
- `SandboxPolicy` struct (default policy: project-root write, network allowlist)
- Must-fail escape CI test suite
- R-5: Populate `dispatcher.rs` (sandbox routing) and `rng.rs` (OsRng seam)

**OUT of Phase 4:**
- Policy customization UI (Phase 10)
- Container-based isolation / Docker (conflicts NN#5 — single binary)
- Network packet-level filtering (iptables/PF) — too invasive for local dev
- Fine-grained per-tool policy profiles (Phase 10/11)
- Phase 3 residuals R-1, R-2, R-6 (Phase 5/MCP)

---

### 6. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| macOS `sandbox-exec` overhead regresses TB 2.0 | HIGH | Benchmark gate; inline `-p` profile (no file I/O per call) |
| Linux Landlock unavailable on kernel <5.13 | MEDIUM | Graceful degradation to seccomp-only with warning |
| Windows CI not available in GitHub Actions free tier | MEDIUM | Use `windows-latest` runner; Job Objects are standard Win32 |
| Policy too restrictive: breaks legit tool calls (npm install writes to node_modules) | HIGH | Default allows writes within project root **tree** (not just root dir) |
| AgentEvent::SandboxViolation breaks Phase 2 provider serialization | LOW | `#[non_exhaustive]` + additive variant — backwards-safe |

---

### 7. Assumptions

| ID | Assumption | Confidence | Test |
|----|------------|------------|------|
| A1 | macOS `sandbox-exec -p <inline>` avoids per-call file I/O | HIGH | Benchmark |
| A2 | Linux Landlock is available on GitHub Actions `ubuntu-latest` (kernel 5.15+) | HIGH | Check runner version |
| A3 | Windows restricted token + Job Object is sufficient for timeout cascade (R-4) | MEDIUM | CI test |
| A4 | `SandboxPolicy` default (project-root writes, OpenRouter-only net) doesn't break TB 2.0 tasks | HIGH | Escape suite + smoke test |
| A5 | `dispatcher.rs` can be populated with sandbox routing without structural change to Phase 3 tool execution path | HIGH | Existing `Arc<dyn Sandbox>` DI seam in `ToolContext` |

---

### 8. Key Insights from Brainstorm

1. **Spawn-time enforcement beats wrap-time.** Userspace checks (pre-flight `check_shell()`) are necessary but not sufficient — a model can subshell out. Real enforcement is kernel-level: sandbox-exec wraps the subprocess process before exec, Landlock applies before the tool spawns, Job Objects bind at CreateProcess. Phase 3 already has `check_shell()` at the seam; Phase 4 adds the kernel layer underneath.

2. **The escape suite IS the product.** If CI doesn't literally attempt `rm -rf` from inside a sandboxed subprocess and assert it fails, the sandbox isn't tested. The escape tests are the most valuable deliverable in Phase 4 — they're the proof of concept.

3. **Benchmark performance is non-negotiable.** Every decision in Phase 4 must be checked against: "does this add latency to the execute_commands happy path?" The answer for spawn-time enforcement is "yes, once per subprocess spawn" — which is acceptable because subprocess spawning is already the expensive operation.

4. **AgentEvent::SandboxViolation is the user-facing product.** The kernel-level enforcement is invisible to users. The structured event is how users know the sandbox is working. It must carry: attempted resource, policy rule violated, OS-level error code, and the tool call ID that triggered it.

5. **Windows is the hardest platform.** macOS and Linux have mature Rust crates (`sandbox-exec` is a well-understood API; `landlock` crate exists). Windows Job Objects have limited Rust ecosystem support. The `windows-sys` crate provides raw bindings. Budget 40% of Phase 4 implementation time for Windows.

---

### 9. Strongest Direction

**Profile-based spawn-time enforcement** with a `SandboxPolicy` struct:

```
SandboxPolicy {
    project_root: PathBuf,            // writes allowed here (and subtree)
    config_dir: PathBuf,              // read-only: ~/.config/kay/
    network_allowlist: Vec<HostPort>, // default: [openrouter.ai:443]
    allow_reads: Vec<PathBuf>,        // additional read-only paths
    deny_reads: Vec<PathBuf>,         // explicit blocks: [~/.aws, ~/.ssh, ~/.gnupg]
}
```

Each OS backend translates this struct into its native policy format:
- **macOS**: `sandbox-exec -p` inline SBPL profile string
- **Linux**: `landlock_restrict_self` + seccomp filter
- **Windows**: Restricted token ACE + Job Object limits

The same `Arc<dyn Sandbox>` DI seam from Phase 3 receives the new OS implementations. Zero retrofit of existing tool code.

---

### 10. Next Step

**→ Engineering Brainstorm (PATH 3b):** Lock the cross-platform architecture (API shape, Landlock fallback strategy, Windows syscall surface, AgentEvent extension) before writing the discuss-phase questions.

---

## §Engineering-Lens (Engineering Brainstorm)

### E1 — macOS sandbox-exec: Inline profile vs. tempfile?

**Decision: Inline `-p <profile-string>` cached at `KaySandboxMacos::new()` time.**

`sandbox-exec` accepts either `-f <file>` (profile file) or `-p <profile-string>` (inline SBPL). Using `-p` with a cached pre-computed profile string avoids file I/O entirely on the hot path (per-subprocess spawn). The profile string is generated once from `SandboxPolicy` at construction time and stored as `Arc<str>` inside `KaySandboxMacos`.

```
// Cold path (once at startup):
let profile = macos_sbpl_from_policy(&policy);  // generate SBPL string
let sandbox = KaySandboxMacos::new(policy, profile);

// Hot path (per subprocess spawn in execute_commands):
Command::new("sandbox-exec").args(["-p", &self.cached_profile, "--", "sh", "-c", cmd])
```

Pitfall: `sandbox-exec` is deprecated on macOS 15 (Apple private API). The deprecation warning exists but the binary remains functional through macOS 15.x. Phase 11 will need to revisit (monitor `seatbelt` alternative). For Phase 4, proceed with `sandbox-exec`.

---

### E2 — Linux: Landlock + seccomp? Kernel version gating?

**Decision: Landlock + seccomp BPF. Graceful degradation to seccomp-only on kernel < 5.13.**

- **Landlock** (kernel 5.13+): restricts filesystem access paths (read/write/exec) per-process. Use the `landlock` crate (v0.4+) which handles ABI version detection (`LANDLOCK_ACCESS_FS_*` flags vary by ABI).
- **seccomp BPF** (kernel 3.5+): restricts syscall numbers. Useful for blocking raw socket creation (`socket(AF_INET, SOCK_RAW)`) beyond what Landlock covers.
- **Degradation path**: `landlock::Ruleset::new()` returns `Err(landlock::CompatError::Kernel)` on old kernels. Catch with `match`, log `tracing::warn!("Landlock unavailable — falling back to seccomp-only sandbox")`, continue with seccomp-only.

GitHub Actions `ubuntu-latest` runs kernel 5.15+ — Landlock available in CI. The production fallback is a safety net for self-hosted runners or older distros.

Crate: `landlock = "0.4"` (Landlock ABI v3, kernel 5.19+; falls back to ABI v1 on 5.13).

---

### E3 — Windows: Job Objects + restricted token — which Rust API?

**Decision: `windows-sys` raw bindings (already in `[target.'cfg(windows)'.dependencies]` of `kay-tools`).**

The `windows` crate (microsoft/windows-rs) has better type safety but is heavier (proc-macro, codegen). `windows-sys` provides the raw FFI bindings needed:

| Win32 API | Purpose |
|-----------|---------|
| `CreateJobObjectW` | Create Job Object for process containment |
| `AssignProcessToJobObject` | Bind child process to job |
| `SetInformationJobObject` with `JobObjectBasicLimitInformation` | Set `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` |
| `CreateRestrictedToken` | Strip privileges from child process token |
| `DISABLE_MAX_PRIVILEGE` | Drop all privileges except minimum needed |

R-4 closure: `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` guarantees that when the Kay process exits (or the job handle drops), all child and grandchild processes are killed — mirroring Unix `killpg(pgid, SIGKILL)` from Phase 3.

---

### E4 — AgentEvent::SandboxViolation shape

```rust
#[non_exhaustive]
pub enum AgentEvent {
    // ... existing variants ...
    SandboxViolation {
        call_id: String,      // tool call that triggered the violation
        tool_name: String,    // "execute_commands" | "fs_write" | "net_fetch" | …
        resource: String,     // the path/URL/command that was denied
        policy_rule: String,  // human-readable: "writes outside project root"
        os_error: Option<i32>, // errno (Unix) or GetLastError (Windows); None if pre-flight
    },
}
```

- Already `#[non_exhaustive]` from Phase 3 → additive, zero breaking change
- `os_error: None` for pre-flight denials (check_shell returned Err before spawn); `Some(errno)` for kernel-level denials detected post-spawn
- `call_id` links back to the `ToolOutput` stream for the GUI/TUI to correlate

---

### E5 — SandboxPolicy placement: new `kay-sandbox-policy` crate

**Decision: New `kay-sandbox-policy` workspace crate.**

The three stub sandbox crates already exist. Adding a thin policy crate gives the cleanest DAG:

```
kay-sandbox-policy  (SandboxPolicy, SandboxRule, NetAllowEntry — no OS deps, just serde)
       ↑
kay-sandbox-macos   (KaySandboxMacos: Sandbox impl, deps: kay-sandbox-policy + kay-tools)
kay-sandbox-linux   (KaySandboxLinux: Sandbox impl, deps: kay-sandbox-policy + kay-tools + landlock)
kay-sandbox-windows (KaySandboxWindows: Sandbox impl, deps: kay-sandbox-policy + kay-tools + windows-sys)
       ↑
kay-cli             (selects OS backend via cfg at startup, constructs Arc<dyn Sandbox>)
```

`kay-tools/src/seams/sandbox.rs` remains unchanged — just the `Sandbox` trait + `NoOpSandbox`.

```rust
// kay-sandbox-policy/src/lib.rs
pub struct SandboxPolicy {
    pub project_root: PathBuf,
    pub config_dir: PathBuf,               // read-only (e.g., ~/.config/kay/)
    pub network_allowlist: Vec<NetAllow>,  // default: [openrouter.ai:443]
    pub extra_read_paths: Vec<PathBuf>,    // caller-supplied read-only additions
    pub deny_read_paths: Vec<PathBuf>,     // default: [~/.aws, ~/.ssh, ~/.gnupg, ~/.gnupg2]
}
pub struct NetAllow { pub host: String, pub port: u16 }
impl SandboxPolicy {
    pub fn default_for_project(project_root: PathBuf) -> Self { … }
}
```

---

### E6 — dispatcher.rs: Populate with sandbox routing dispatch

**Decision: Implement `dispatch()` in `kay-tools/src/runtime/dispatcher.rs` as the Phase 5 entry point with Phase 4 sandbox pre-flight.**

```rust
pub async fn dispatch(
    registry: &ToolRegistry,
    sandbox: Arc<dyn Sandbox>,
    call: &ToolCall,
) -> AgentEvent {
    // 1. Look up tool
    let tool = match registry.get(&call.name) {
        Some(t) => t,
        None => return AgentEvent::ToolCallMalformed { … },
    };
    // 2. Sandbox pre-flight (tool-type-aware)
    if let Err(denial) = sandbox_preflight(&*sandbox, call).await {
        return AgentEvent::SandboxViolation { call_id: call.id.clone(), … };
    }
    // 3. Execute
    tool.execute(call).await
}
```

The `sandbox_preflight` function maps tool name → `Sandbox` method:
- `execute_commands` → `check_shell`
- `fs_read` / `fs_search` / `image_read` → `check_fs_read`
- `fs_write` → `check_fs_write`
- `net_fetch` → `check_net`

Satisfies R-5 (no empty TODO) while being genuinely Phase 5-ready.

---

### E7 — rng.rs: RngSeam trait

**Decision: Implement `RngSeam` trait + two implementations.**

```rust
// kay-tools/src/seams/rng.rs
pub trait RngSeam: Send + Sync {
    fn fill_bytes(&mut self, buf: &mut [u8]);
}

pub struct OsRngSeam;
impl RngSeam for OsRngSeam {
    fn fill_bytes(&mut self, buf: &mut [u8]) { OsRng.fill_bytes(buf); }
}

#[cfg(test)]
pub struct DeterministicRng(pub Vec<u8>, pub usize);
#[cfg(test)]
impl RngSeam for DeterministicRng {
    fn fill_bytes(&mut self, buf: &mut [u8]) { /* copy from self.0 */ }
}
```

The `MarkerFactory` in `markers/mod.rs` (Phase 3) currently takes `OsRng` directly. In Phase 4 it gains an `Arc<dyn RngSeam>` parameter behind a `#[cfg(test)]` toggle, enabling escape tests to use deterministic nonces. Non-test code is unchanged.

---

### E8 — Cross-platform CI matrix

**Decision: 3-OS matrix.**

```yaml
# .github/workflows/ci.yml addition
strategy:
  matrix:
    os: [macos-14, ubuntu-latest, windows-latest]
runs-on: ${{ matrix.os }}
```

- `macos-14`: arm64 M1 runner — tests `KaySandboxMacos`
- `ubuntu-latest`: kernel 5.15+ — tests `KaySandboxLinux` with Landlock
- `windows-latest`: Win Server 2022 — tests `KaySandboxWindows`

Escape tests are `#[test]` (not `#[tokio::test]`) — they spawn real subprocesses synchronously and assert exit status. Use `#[cfg_attr(not(target_os = "macos"), ignore)]` etc. on OS-specific tests, OR use conditional compilation (`mod tests` inside `#[cfg(target_os = "macos")]`).

---

### E9 — Graceful degradation when Landlock unavailable

**Decision: `tracing::warn!` + continue with seccomp-only.**

```rust
let ruleset_status = landlock::Ruleset::new()
    .handle_access(landlock::AccessFs::from_all(landlock::ABI::V1))?
    .create();
match ruleset_status {
    Ok(rs) => { /* apply Landlock rules */ }
    Err(landlock::RulesetError::Compat(e)) => {
        tracing::warn!(
            "Landlock unavailable ({}); falling back to seccomp-only sandbox",
            e
        );
        // seccomp-only path
    }
}
```

No `AgentEvent::Warning` variant needed (not yet defined). Document the degradation behavior in `04-SECURITY.md`. Add a Nyquist test that asserts `tracing::warn!` fires when Landlock is mocked as unavailable (inject mock error in unit test).

---

### E10 — Escape test architecture (no root required)

**Key insight: Landlock/sandbox-exec/Job Objects apply to child processes without root.** Escape tests run as the CI user.

```
tests/
  escape_suite.rs                     # main integration test file
    escape_write_outside_project()    # spawn child: write to /tmp/outside — assert Err
    escape_read_credentials()         # spawn child: open ~/.aws/credentials — assert Err
    escape_network_non_allowlisted()  # spawn child: connect evil.com:443 — assert Err
    escape_write_within_project()     # sanity: write to project_root/ — assert Ok (must-pass)
```

Each escape test:
1. Constructs `SandboxPolicy::default_for_project(tmpdir)`
2. Wraps a `Command` using the OS sandbox backend
3. The child process attempts the forbidden operation (`fs::write("/tmp/evil", "x")`, etc.)
4. Asserts the child exits with a non-zero code or the operation returns `PermissionDenied`

The tests live in each OS crate's `tests/` dir plus a workspace-level `tests/` for the shared `SandboxViolation` event integration.

---

### Engineering Architecture Summary

```
New crate:    kay-sandbox-policy          (SandboxPolicy struct, serde only)
Modified:     kay-sandbox-macos/linux/windows (full Sandbox impl)
Modified:     kay-tools/src/seams/rng.rs      (RngSeam trait — R-5)
Modified:     kay-tools/src/runtime/dispatcher.rs (dispatch() fn — R-5)
Modified:     kay-tools/src/events/mod.rs     (AgentEvent::SandboxViolation)
Modified:     crates/forge_domain/Cargo.toml  (json feature — already fixed, cherry-pick)
Modified:     .github/workflows/ci.yml        (3-OS matrix)
New tests:    escape_suite.rs × 3 OS crates   (must-fail escape suite)
```

**Approach chosen: A (Thin OS backend crates)** — aligns with existing workspace structure (three stub crates already scaffolded), cleanest DAG, independently testable per OS.

---

### Key Engineering Decisions for Discuss-Phase (locked)

| ID | Decision | Rationale |
|----|----------|-----------|
| D-01 | macOS: `sandbox-exec -p <cached-inline-profile>` | No per-call file I/O; profile generated once at construction |
| D-02 | Linux: Landlock + seccomp BPF; graceful fallback to seccomp-only | Defense-in-depth; fallback ensures CI doesn't fail on old kernels |
| D-03 | Windows: `windows-sys` raw bindings + Job Objects + restricted token | Already in workspace; sufficient for R-4 closure |
| D-04 | New `kay-sandbox-policy` crate for `SandboxPolicy` | Clean DAG; zero OS deps |
| D-05 | `AgentEvent::SandboxViolation` with 5 fields | Additive; call_id links to existing ToolOutput stream |
| D-06 | `dispatcher.rs` becomes Phase 4 sandbox routing + Phase 5 agent-loop entry point | Satisfies R-5; eliminates Phase 5 rework |
| D-07 | `rng.rs` implements `RngSeam` trait + `OsRngSeam` + `DeterministicRng` (cfg(test)) | Satisfies R-5; enables deterministic escape test nonces |
| D-08 | 3-OS CI matrix: macos-14, ubuntu-latest, windows-latest | Verifies all three sandboxes in CI |
| D-09 | Escape tests: user-space safe, no root, `std::process::Command` child | Works on GitHub Actions free tier |
| D-10 | `SandboxPolicy::default_for_project(root)` denies `~/.aws`, `~/.ssh`, `~/.gnupg` by default | Opinionated safe default; user can extend |
