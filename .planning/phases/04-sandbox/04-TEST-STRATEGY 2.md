---
phase: 4
flow: 5a
strategy_date: 2026-04-21
coverage_target: 90% line coverage on kay-sandbox-* and kay-sandbox-policy
---

# Phase 4 Test Strategy — Sandbox: All Three Platforms

## Testing Pyramid

```
         /  Smoke + Live E2E  \     3 tests — Kay + real sandbox end-to-end
        /   Integration Tests  \   18 tests — OS backend + escape suite
       /    Property Tests      \   6 tests — SandboxPolicy rule coverage
      /     Unit Tests          \  41 tests — policy, dispatch, rng, events
```

**Total planned: 68 tests**

---

## Tooling

| Tool | Purpose |
|------|---------|
| `cargo test` | All test levels |
| `proptest` (already in workspace) | Property tests for SandboxPolicy path classification |
| `tokio::test` | Async Sandbox trait integration tests |
| `tempfile` (already in workspace) | Isolated project roots for escape tests |
| `insta` (via forge_test_kit) | Snapshot for SandboxViolation serialization |
| `std::process::Command` | Real subprocess spawn in escape tests |
| GitHub Actions matrix | `macos-14` + `ubuntu-latest` + `windows-latest` |

---

## Level 1 — Unit Tests (41 tests)

### 1a. `kay-sandbox-policy` (12 tests)

| # | Test | What it proves |
|---|------|----------------|
| U-01 | `policy_default_allows_project_root_write` | Default policy: write in project root is allowed |
| U-02 | `policy_default_denies_outside_project_write` | Default policy: write to `/tmp/` is denied |
| U-03 | `policy_default_denies_aws_credentials_read` | `~/.aws/credentials` blocked by default deny list |
| U-04 | `policy_default_denies_ssh_dir` | `~/.ssh/` blocked by default deny list |
| U-05 | `policy_default_allows_config_dir_read` | `~/.config/kay/` allowed (read-only) |
| U-06 | `policy_default_net_openrouter_allowed` | `openrouter.ai:443` in default allowlist |
| U-07 | `policy_default_net_arbitrary_host_denied` | `evil.com:443` not in default allowlist |
| U-08 | `policy_serde_round_trip` | `SandboxPolicy` serializes + deserializes via serde_json identically |
| U-09 | `policy_extra_read_paths_extend_default` | `extra_read_paths` adds to allow set without dropping defaults |
| U-10 | `policy_project_subtree_write_allowed` | Write to `<project_root>/src/lib.rs` is allowed (subtree, not just exact root) |
| U-11 | `net_allow_port_mismatch_denied` | `openrouter.ai:80` denied (port mismatch from allowlist entry `:443`) |
| U-12 | `net_allow_subdomain_matching` | `api.openrouter.ai:443` allowed if policy uses `*.openrouter.ai` |

### 1b. `kay-tools` seams: `rng.rs` (5 tests)

| # | Test | What it proves |
|---|------|----------------|
| U-13 | `os_rng_seam_fills_bytes` | `OsRngSeam::fill_bytes` produces non-zero bytes of correct length |
| U-14 | `deterministic_rng_repeats_bytes` | `DeterministicRng` yields same bytes every call (test determinism) |
| U-15 | `deterministic_rng_wraps_on_exhaustion` | `DeterministicRng` wraps at end of its byte slice, not panics |
| U-16 | `rng_seam_is_send_sync` | `Arc<dyn RngSeam>` compiles (object-safety regression lock) |
| U-17 | `marker_nonce_uses_rng_seam` | `MarkerFactory::new(rng)` accepts `Arc<dyn RngSeam>` — injection confirmed |

### 1c. `kay-tools` runtime: `dispatcher.rs` (8 tests)

| # | Test | What it proves |
|---|------|----------------|
| U-18 | `dispatch_allows_execute_commands` | `check_shell` passes → tool.execute() called |
| U-19 | `dispatch_denies_execute_commands_sandbox_violation` | `check_shell` returns `Err(denial)` → `AgentEvent::SandboxViolation` |
| U-20 | `dispatch_allows_fs_write` | `check_fs_write` passes → tool.execute() called |
| U-21 | `dispatch_denies_fs_write_sandbox_violation` | `check_fs_write` Err → `SandboxViolation` with `tool_name = "fs_write"` |
| U-22 | `dispatch_allows_net_fetch` | `check_net` passes → tool.execute() called |
| U-23 | `dispatch_denies_net_fetch_sandbox_violation` | `check_net` Err → `SandboxViolation` with resource = URL |
| U-24 | `dispatch_unknown_tool_returns_malformed` | Unknown tool name → `AgentEvent::ToolCallMalformed` |
| U-25 | `dispatch_sandbox_violation_carries_call_id` | `SandboxViolation.call_id` matches the input `ToolCall.id` |

### 1d. `AgentEvent::SandboxViolation` (6 tests)

| # | Test | What it proves |
|---|------|----------------|
| U-26 | `sandbox_violation_all_fields_present` | Struct contains call_id, tool_name, resource, policy_rule, os_error |
| U-27 | `sandbox_violation_serde_json_round_trip` | Serializes/deserializes without field loss |
| U-28 | `sandbox_violation_non_exhaustive_compiles` | Pattern match with `..` compiles (non_exhaustive regression lock) |
| U-29 | `sandbox_violation_os_error_none_for_preflight` | Pre-flight denial sets `os_error: None` |
| U-30 | `sandbox_violation_os_error_some_for_kernel` | Kernel denial sets `os_error: Some(libc::EPERM)` |
| U-31 | `sandbox_violation_insta_snapshot` | `insta::assert_json_snapshot!` locks the JSON shape |

### 1e. macOS SBPL profile generation (5 tests — `cfg(target_os = "macos")`)

| # | Test | What it proves |
|---|------|----------------|
| U-32 | `sbpl_profile_contains_project_root` | Generated SBPL includes `(subpath "<project_root>")` |
| U-33 | `sbpl_profile_denies_aws_path` | Generated SBPL denies `~/.aws` subtree |
| U-34 | `sbpl_profile_denies_ssh_path` | Generated SBPL denies `~/.ssh` subtree |
| U-35 | `sbpl_profile_net_allows_openrouter` | Generated SBPL has `(allow network*)` scoped to openrouter.ai |
| U-36 | `sbpl_profile_is_cached_not_regenrated_per_call` | `KaySandboxMacos::profile_str()` returns same `Arc<str>` both calls |

### 1f. Linux policy translation (5 tests — `cfg(target_os = "linux")`)

| # | Test | What it proves |
|---|------|----------------|
| U-37 | `landlock_ruleset_created_for_policy` | `KaySandboxLinux::new()` produces a `landlock::Ruleset` without error |
| U-38 | `landlock_allows_project_root_read_write` | Read+write rule added for project_root path |
| U-39 | `landlock_denies_aws_path_rule_absent` | `~/.aws` has no read rule in ruleset |
| U-40 | `seccomp_filter_generated` | BPF filter bytes non-empty |
| U-41 | `linux_degradation_logs_warn_on_enosys` | Mock `Ruleset::create()` returning ENOSYS → `tracing::warn!` emitted |

---

## Level 2 — Property Tests (6 tests)

| # | Test | Invariant | Runs |
|---|------|-----------|------|
| P-01 | `proptest_project_subtree_always_write_allowed` | Any path under `project_root` passes `check_fs_write` | 10,000 |
| P-02 | `proptest_deny_list_paths_always_denied` | Any path starting with `~/.aws`, `~/.ssh`, `~/.gnupg` fails `check_fs_read` | 10,000 |
| P-03 | `proptest_non_allowlist_host_always_denied` | Any `host:port` not in allowlist fails `check_net` | 10,000 |
| P-04 | `proptest_allowlisted_host_always_passes` | Any URL with host matching allowlist passes `check_net` | 10,000 |
| P-05 | `proptest_sbpl_never_contains_null_bytes` | Generated SBPL profile string has no embedded null bytes | 1,000 |
| P-06 | `proptest_policy_round_trip_serde` | Any `SandboxPolicy` value serde-round-trips to identical value | 5,000 |

---

## Level 3 — Integration Tests (18 tests)

### 3a. OS backend integration — macOS (4 tests, `cfg(target_os = "macos")`)

| # | Test | Method |
|---|------|--------|
| I-01 | `macos_check_shell_allows_echo` | `KaySandboxMacos.check_shell("echo hi", &project_root)` → Ok |
| I-02 | `macos_check_fs_write_allows_project_file` | `check_fs_write(&project_root.join("x.rs"))` → Ok |
| I-03 | `macos_check_fs_read_denies_aws_creds` | `check_fs_read(~/.aws/credentials)` → Err(SandboxDenial) |
| I-04 | `macos_check_net_denies_non_allowlisted` | `check_net(evil.com:443)` → Err(SandboxDenial) |

### 3b. OS backend integration — Linux (4 tests, `cfg(target_os = "linux")`)

| # | Test | Method |
|---|------|--------|
| I-05 | `linux_check_shell_allows_echo` | `KaySandboxLinux.check_shell("echo hi", &project_root)` → Ok |
| I-06 | `linux_check_fs_write_allows_project_file` | `check_fs_write(&project_root.join("x.rs"))` → Ok |
| I-07 | `linux_check_fs_read_denies_aws_creds` | `check_fs_read(~/.aws/credentials)` → Err(SandboxDenial) |
| I-08 | `linux_check_net_denies_non_allowlisted` | `check_net(evil.com:443)` → Err(SandboxDenial) |

### 3c. OS backend integration — Windows (4 tests, `cfg(target_os = "windows")`)

| # | Test | Method |
|---|------|--------|
| I-09 | `windows_check_shell_allows_echo` | `KaySandboxWindows.check_shell("echo hi", &project_root)` → Ok |
| I-10 | `windows_check_fs_write_allows_project_file` | `check_fs_write(&project_root.join("x.rs"))` → Ok |
| I-11 | `windows_job_object_created_on_init` | `KaySandboxWindows::new()` opens a valid Job Object handle |
| I-12 | `windows_timeout_cascade_r4_regression` | Grandchild that ignores SIGTERM is killed when Job Object closes (R-4 closure) |

### 3d. Escape suite — CRITICAL (6 tests, all OSes via `std::process::Command`)

These spawn a **real child process** that attempts the forbidden operation. Each asserts the child exits non-zero or `PermissionDenied`.

| # | Test | Child operation attempted | Expected |
|---|------|--------------------------|----------|
| E-01 | `escape_write_outside_project_root` | `fs::write("/tmp/kay-escape-test", "x")` | Child non-zero / EPERM |
| E-02 | `escape_read_aws_credentials` | `fs::read("~/.aws/credentials")` | Child non-zero / EACCES |
| E-03 | `escape_read_ssh_dir` | `fs::read_dir("~/.ssh/")` | Child non-zero / EACCES |
| E-04 | `escape_write_within_project_root` | `fs::write("<project>/out.txt", "x")` | Child exits 0 (must-pass sanity) |
| E-05 | `escape_net_non_allowlisted` | TCP connect to `1.2.3.4:9999` | Child non-zero / ECONNREFUSED or EPERM |
| E-06 | `escape_net_allowlisted` | TCP connect to `openrouter.ai:443` (DNS lookup only, no data) | Ok or ECONNRESET (not EPERM) |

**Note on E-05/E-06:** Network sandboxing at the kernel level (seccomp `socket` filter) may be OS-specific. E-05 verifies the filter fires; E-06 verifies it's scoped (not a blanket block). On macOS, sandbox-exec network filtering uses `(deny network*)` + `(allow network (remote ip "openrouter.ai:443"))` — TLS verification not required for the test, only kernel accept/reject.

---

## Level 4 — Smoke Tests (3 tests)

| # | Test | What it proves |
|---|------|----------------|
| S-01 | `smoke_kay_cli_with_macos_sandbox_boots` | `kay-cli --sandbox` starts without panic on macOS; emits no SandboxViolation on idle |
| S-02 | `smoke_execute_commands_ls_under_sandbox` | `execute_commands("ls .")` runs successfully with real OS sandbox enabled |
| S-03 | `smoke_sandbox_violation_event_emitted` | Intentional policy-violating tool call produces `AgentEvent::SandboxViolation` in event stream |

---

## Coverage Targets

| Crate | Target | Notes |
|-------|--------|-------|
| `kay-sandbox-policy` | 95% | Pure logic, no OS calls |
| `kay-sandbox-macos` | 85% | macOS-only — runs in CI on macos-14 only |
| `kay-sandbox-linux` | 85% | Linux-only — runs in CI on ubuntu-latest only |
| `kay-sandbox-windows` | 80% | Windows-only — runs in CI on windows-latest only |
| `kay-tools` (sandbox/rng/dispatch additions) | 90% | All platforms |

---

## Test File Locations

```
crates/
  kay-sandbox-policy/
    src/lib.rs              # U-01..U-12, P-01..P-06 (inline #[cfg(test)])
  kay-sandbox-macos/
    src/lib.rs              # U-32..U-36 (inline cfg(target_os = "macos"))
    tests/
      integration.rs        # I-01..I-04
      escape_suite.rs       # E-01..E-06 (spawns real child via sandbox-exec)
  kay-sandbox-linux/
    src/lib.rs              # U-37..U-41 (inline cfg(target_os = "linux"))
    tests/
      integration.rs        # I-05..I-08
      escape_suite.rs       # E-01..E-06 (spawns real child via Landlock+seccomp)
  kay-sandbox-windows/
    src/lib.rs              # (Windows unit tests)
    tests/
      integration.rs        # I-09..I-12 (incl. R-4 regression)
      escape_suite.rs       # E-01..E-06 (spawns real child via Job Objects)
  kay-tools/
    src/seams/rng.rs        # U-13..U-17 (inline)
    src/runtime/dispatcher.rs # U-18..U-25 (inline)
    tests/
      sandbox_violation_event.rs  # U-26..U-31
      smoke.rs                    # S-01..S-03
```

---

## CI Matrix Requirements

```yaml
# .github/workflows/ci.yml
strategy:
  matrix:
    os: [macos-14, ubuntu-latest, windows-latest]
  fail-fast: false

steps:
  - name: cargo test (all targets, all features)
    run: cargo test --workspace --all-targets
    # Escape tests use #[cfg_attr] to gate per-OS:
    #   #[cfg_attr(not(target_os = "macos"), ignore)]
    # Integration tests in OS-specific crates naturally only run on their OS
    # (crate only compiles with OS-appropriate deps)

  - name: cargo test --doc
    run: cargo test --workspace --doc

  - name: clippy -D warnings
    run: cargo clippy --workspace --all-targets -- -D warnings

  - name: Escape suite (must-fail gate)
    run: cargo test -p kay-sandbox-${{ runner.os | lowercase }} escape_suite
    # This job is flagged as "required" in branch protection
    # A green escape suite is the definitive proof that kernel enforcement works
```

---

## Test Execution Order (TDD)

Per the standing TDD directive (RED → GREEN → REFACTOR, atomic commits):

**Wave 0 (entry gate):** `cargo test --workspace --all-targets` — must pass before Phase 4 code starts (forge_domain fix already cherry-picked ✓)

**Wave 1 (policy crate):** U-01..U-12, P-01..P-06 — write failing tests first, then `kay-sandbox-policy/src/lib.rs`

**Wave 2 (rng + dispatcher):** U-13..U-25 — failing tests first, then `rng.rs` + `dispatcher.rs` implementation

**Wave 3 (macOS backend):** U-32..U-36, I-01..I-04, E-01..E-06 — failing tests on macos-14, then `KaySandboxMacos`

**Wave 4 (Linux backend):** U-37..U-41, I-05..I-08, E-01..E-06 — failing on ubuntu-latest, then `KaySandboxLinux`

**Wave 5 (Windows backend):** I-09..I-12, E-01..E-06 — failing on windows-latest, then `KaySandboxWindows`

**Wave 6 (events + smoke):** U-26..U-31, S-01..S-03 — `AgentEvent::SandboxViolation` + smoke

---

## Gaps from Phase 3 (Nyquist)

| Gap | Owner | Test |
|-----|-------|------|
| R-3: marker_forgery_property.rs (≥10k-case proptest) | Phase 4 Wave 0 (SC#7 in ROADMAP says R-3 was closed by FLOW 14 — verify) | P-* |
| No kernel-level enforcement tests existed | Phase 4 Waves 3-5 | E-01..E-06 |
| `dispatcher.rs` untested (empty stub) | Phase 4 Wave 2 | U-18..U-25 |
| `rng.rs` untested (empty stub) | Phase 4 Wave 2 | U-13..U-17 |
