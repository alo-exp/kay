---
phase: 4
date: 2026-04-21
---

# Phase 4 Dependency Analysis

## New Crate: kay-sandbox-policy

Zero OS deps. Must be `#[no_std]`-friendly at the type level (all std features are fine, just no OS syscalls).

**Required dependencies:**
```toml
[dependencies]
serde = { workspace = true }         # SandboxPolicy serialization
schemars = { workspace = true }      # JSON schema for config
thiserror = { workspace = true }     # SandboxError type
```

**Workspace Cargo.toml addition:** `"crates/kay-sandbox-policy"` → members list.

---

## Crate: kay-sandbox-macos

macOS `sandbox-exec` — no native Rust crate needed; invokes system binary via `std::process::Command`.

**Required dependencies:**
```toml
[dependencies]
kay-sandbox-policy = { path = "../kay-sandbox-policy" }
kay-tools           = { path = "../kay-tools" }          # Sandbox trait
async-trait = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }
thiserror   = { workspace = true }
```

**Target gate:** `[target.'cfg(target_os = "macos")'.dependencies]` — crate only compiles on macOS. On other OSes, all public items can be stubs returning `SandboxError::PlatformNotSupported`.

**Dev dependencies:**
```toml
[dev-dependencies]
tempfile = { workspace = true }
tokio    = { workspace = true, features = ["macros"] }
```

---

## Crate: kay-sandbox-linux

Linux Landlock LSM + seccomp BPF. Two crates needed:

| Crate | Version | Purpose |
|-------|---------|---------|
| `landlock` | `0.4` | Landlock ruleset API (safe wrapper over `landlock_create_ruleset` syscall) |
| `seccompiler` | `0.5` | seccomp BPF JIT compiler (safe wrapper over `prctl(PR_SET_SECCOMP)`) |

Neither is in workspace yet — both must be added.

**Required dependencies:**
```toml
[dependencies]
kay-sandbox-policy = { path = "../kay-sandbox-policy" }
kay-tools           = { path = "../kay-tools" }
async-trait = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }
thiserror   = { workspace = true }

[target.'cfg(target_os = "linux")'.dependencies]
landlock    = "0.4"
seccompiler = "0.5"
```

**Workspace.dependencies additions:**
```toml
landlock    = "0.4"
seccompiler = "0.5"
```

**Kernel version gating (D-02b):** `landlock::ABI::V1` supported from kernel 5.13. On ENOSYS from `landlock_create_ruleset` → `tracing::warn!("Landlock unavailable; falling back to seccomp-only")` → proceed with seccomp BPF only.

**Dev dependencies:**
```toml
[dev-dependencies]
tempfile = { workspace = true }
tokio    = { workspace = true }
```

---

## Crate: kay-sandbox-windows

Windows Job Objects via `windows-sys`. Already in workspace (`windows-sys = "0.61"`). Need to expand feature set.

**Current workspace features:** `["Win32_System_Console", "Win32_System_Threading"]`

**Required additional features:**
- `Win32_Security` — `CreateRestrictedToken`, `RESTRICTED_TOKEN`
- `Win32_System_JobObjects` — `CreateJobObjectW`, `SetInformationJobObject`, `AssignProcessToJobObject`, `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`

**Updated workspace entry:**
```toml
windows-sys = { version = "0.61", features = [
    "Win32_System_Console",
    "Win32_System_Threading",
    "Win32_Security",
    "Win32_System_JobObjects",
] }
```

**Required crate dependencies:**
```toml
[dependencies]
kay-sandbox-policy = { path = "../kay-sandbox-policy" }
kay-tools           = { path = "../kay-tools" }
async-trait = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }
thiserror   = { workspace = true }

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { workspace = true }
```

---

## Crate: kay-tools (modifications)

New items added to existing crate:

1. **`src/events/mod.rs`** — add `AgentEvent::SandboxViolation` variant (no new deps)
2. **`src/seams/rng.rs`** — populate `RngSeam` trait (uses `rand` already in deps)
3. **`src/runtime/dispatcher.rs`** — populate `dispatch()` (uses `ToolRegistry` already in deps)
4. **`src/seams/sandbox.rs`** — add `RULE_*` re-exports from `kay-sandbox-policy`... OR keep constants in `kay-sandbox-policy` only and have OS crates dep on it directly.

**Decision:** `kay-tools` does NOT dep on `kay-sandbox-policy`. The `RULE_*` constants are in `kay-sandbox-policy`; OS crates use them internally. `AgentEvent::SandboxViolation.policy_rule` is `String` — the constant values are embedded at construction time in each OS crate.

**No new deps for kay-tools.**

---

## Dependency Graph

```
kay-sandbox-policy  (new, zero OS deps)
        │
        ├── kay-sandbox-macos   (macOS only)
        ├── kay-sandbox-linux   (Linux only; + landlock + seccompiler)
        └── kay-sandbox-windows (Windows only; + windows-sys expanded)

kay-tools
  ├── src/seams/rng.rs          (uses existing rand dep)
  ├── src/runtime/dispatcher.rs (uses existing ToolRegistry)
  └── src/events/mod.rs         (SandboxViolation variant — no new dep)
```

**No circular dependencies.** `kay-sandbox-*` → `kay-tools` (for `Sandbox` trait); `kay-tools` does NOT dep on `kay-sandbox-*`.

---

## CI Matrix Dependencies

```yaml
# .github/workflows/ci.yml additions:
jobs:
  test:
    strategy:
      matrix:
        os: [macos-14, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --all-targets
        env:
          RUST_LOG: warn
```

`macos-14` = arm64 M1 runner (Landlock N/A, sandbox-exec available).
`ubuntu-latest` = kernel 5.15+ (Landlock V1 available).
`windows-latest` = Windows Server 2022 (Job Objects available).

---

## Summary of Changes Required

| Action | Target |
|--------|--------|
| Add workspace member | `crates/kay-sandbox-policy` |
| Create crate dir | `crates/kay-sandbox-policy/` |
| Add workspace deps | `landlock = "0.4"`, `seccompiler = "0.5"` |
| Expand workspace dep | `windows-sys` add `Win32_Security` + `Win32_System_JobObjects` |
| Update crate deps | kay-sandbox-macos, kay-sandbox-linux, kay-sandbox-windows Cargo.toml |
| No change | kay-tools Cargo.toml |
| Add CI matrix | `.github/workflows/ci.yml` (if exists) |
