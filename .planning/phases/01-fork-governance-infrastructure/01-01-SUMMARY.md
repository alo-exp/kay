---
phase: 01-fork-governance-infrastructure
plan: 01
status: complete-with-deviation
executed: 2026-04-19
wave: 1
host_os: aarch64-apple-darwin
rustc_version: 1.95.0
requirements_satisfied: [WS-01, WS-02, WS-05]
commits:
  - 7006cae  # Task 1 — workspace root manifest + toolchain + rustfmt
  - 4f8e535  # Task 2 — 7 crate skeletons + Cargo.lock (final; see Deviations for the race-and-recovery story)
---

# Plan 01-01 Summary — Workspace Scaffold

## One-liner

Rust 2024 cargo workspace with 7 crate skeletons (kay-core, kay-cli, kay-tauri, kay-provider-openrouter, kay-sandbox-{macos,linux,windows}), pinned to Rust 1.95 stable and ForgeCode-aligned [workspace.dependencies] — all 7 crates compile clean on the host OS.

## Objective Status

- [x] `Cargo.toml`, `rust-toolchain.toml`, `.rustfmt.toml` exist at repo root with all required fields
- [x] 7 crate skeletons under `crates/` each have `Cargo.toml` + src entry point
- [x] `cargo check --workspace --all-features` succeeds on host OS (macOS arm64)
- [x] `cargo metadata` reports exactly 7 workspace members with correct names
- [x] `Cargo.lock` is committed
- [x] `rust-toolchain.toml` pins `channel = "1.95"` with the divergence comment preserved

## Files Created

### Task 1 — workspace root configuration (committed `7006cae`)

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root: `[workspace]` with `resolver = "2"` + `members = ["crates/*"]`; `[workspace.package]` locking edition 2024, rust-version 1.95, Apache-2.0, authors per D-03, repository `https://github.com/alo-exp/kay`; `[workspace.dependencies]` pinning tokio 1.51 LTS, reqwest 0.13, reqwest-eventsource 0.6, rustls 0.23, serde 1 (derive), serde_json 1, schemars 0.8, clap 4.5 (derive+env), tracing 0.1, tracing-subscriber 0.3 (env-filter+json), anyhow 1, thiserror 2; `[profile.release]` with lto=true, codegen-units=1, opt-level=3, strip=true. |
| `rust-toolchain.toml` | Pins `channel = "1.95"` (deliberate divergence from ForgeCode's 1.92, per 01-RESEARCH.md §Pitfall 3). Comment `Kay pins current stable, not ForgeCode's 1.92. Bump via explicit PR.` preserved. |
| `.rustfmt.toml` | ForgeCode-aligned style: `unstable_features = true`, `struct_lit_width = 60`, `imports_granularity = "Module"`, `group_imports = "StdExternalCrate"`, `wrap_comments = true`, `comment_width = 80`. |

### Task 2 — 7 crate skeletons (committed `4f8e535`)

| Crate | Type | Files | Description (per D-05) |
|-------|------|-------|------------------------|
| `kay-core` | lib | `crates/kay-core/Cargo.toml`, `crates/kay-core/src/lib.rs` | Kay's core harness — imports ForgeCode as parity baseline (plan 03) |
| `kay-cli` | bin | `crates/kay-cli/Cargo.toml`, `crates/kay-cli/src/main.rs` | Kay headless CLI for benchmark + CI use |
| `kay-tauri` | lib | `crates/kay-tauri/Cargo.toml`, `crates/kay-tauri/src/lib.rs` | Kay Tauri 2.x desktop shell (Phase 9) |
| `kay-provider-openrouter` | lib | `crates/kay-provider-openrouter/Cargo.toml`, `crates/kay-provider-openrouter/src/lib.rs` | OpenRouter provider HAL (Phase 2) |
| `kay-sandbox-macos` | lib | `crates/kay-sandbox-macos/Cargo.toml`, `crates/kay-sandbox-macos/src/lib.rs` | macOS sandbox-exec sandbox backend (Phase 4) |
| `kay-sandbox-linux` | lib | `crates/kay-sandbox-linux/Cargo.toml`, `crates/kay-sandbox-linux/src/lib.rs` | Linux Landlock + seccomp sandbox backend (Phase 4) |
| `kay-sandbox-windows` | lib | `crates/kay-sandbox-windows/Cargo.toml`, `crates/kay-sandbox-windows/src/lib.rs` | Windows Job Objects + restricted token sandbox backend (Phase 4) |

Plus `Cargo.lock` at repo root (committed per D-07 pinning doctrine and Pitfall 4).

Every crate `Cargo.toml` inherits `version`, `edition`, `rust-version`, `authors`, `license`, `repository` via `field.workspace = true` (6 inheritance lines per crate, 42 total — verified by `grep -c 'workspace = true' crates/*/Cargo.toml`).

## Verification Evidence

### `cargo metadata --format-version 1 --no-deps`

```
workspace_members: 7
packages (sorted): [kay-cli, kay-core, kay-provider-openrouter,
                    kay-sandbox-linux, kay-sandbox-macos,
                    kay-sandbox-windows, kay-tauri]
```

### `cargo check --workspace --all-features`

```
info: syncing channel updates for 1.95-aarch64-apple-darwin
info: latest update on 2026-04-16 for version 1.95.0 (59807616e 2026-04-14)
info: downloading 6 components
    Checking kay-sandbox-linux v0.1.0 (.../crates/kay-sandbox-linux)
    Checking kay-provider-openrouter v0.1.0 (.../crates/kay-provider-openrouter)
    Checking kay-sandbox-macos v0.1.0 (.../crates/kay-sandbox-macos)
    Checking kay-sandbox-windows v0.1.0 (.../crates/kay-sandbox-windows)
    Checking kay-tauri v0.1.0 (.../crates/kay-tauri)
    Checking kay-cli v0.1.0 (.../crates/kay-cli)
    Checking kay-core v0.1.0 (.../crates/kay-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.87s
```

Exit code 0. No warnings. First-run rebuild after rustup bootstrapped 1.95 into the empty toolchain cache.

### Workspace-inheritance check

```
crates/kay-cli/Cargo.toml:6
crates/kay-core/Cargo.toml:6
crates/kay-provider-openrouter/Cargo.toml:6
crates/kay-sandbox-linux/Cargo.toml:6
crates/kay-sandbox-macos/Cargo.toml:6
crates/kay-sandbox-windows/Cargo.toml:6
crates/kay-tauri/Cargo.toml:6
```

Six `workspace = true` lines per crate (version, edition, rust-version, authors, license, repository) — consistent with D-03 / D-07.

## Commits

| Plan task | Commit | Scope |
|-----------|--------|-------|
| Task 1 | `7006cae` | `Cargo.toml`, `rust-toolchain.toml`, `.rustfmt.toml` — clean, DCO + Co-Authored-By trailers present |
| Task 2 | `4f8e535` | 7 `crates/*/Cargo.toml`, 7 `crates/*/src/{lib,main}.rs`, `Cargo.lock` — final corrected commit after parallel-execution race (see Deviations) |

## Deviations from Plan

### [Rule 3 — Blocking dependency] Rust toolchain installation

**Found during:** Task 2 pre-verification.
**Issue:** `cargo` was not present on PATH; Task 2's verification (`cargo check`, `cargo metadata`) cannot run without it.
**Fix:** Installed rustup via the official bootstrap (`curl sh.rustup.rs | sh -s -- -y --default-toolchain none --profile minimal`). With `--default-toolchain none`, no toolchain is installed until a `cargo` invocation triggers `rust-toolchain.toml` resolution — so the first `cargo check` cleanly pulled `1.95.0` into the local cache, matching our pin exactly.
**Side effects:** None in the repo. `$HOME/.cargo/` and `$HOME/.rustup/` populated on the executor host; nothing added to `.gitignore` because Cargo's user-level caches were already covered by the existing patterns.

### [Rule 4 — Architectural: parallel-execution race on shared git index] Task 2 re-committed after 01-02 executor reset HEAD

**Found during:** Task 2 commit step and its aftermath.

**Issue:** Plans 01-01 and 01-02 ran concurrently in the same working directory (wave 1 parallelism as scheduled by the orchestrator). Both executors share a single git index, with no mutex. The race played out across three HEAD states before settling:

```
reflog (read bottom-up):
  7006cae HEAD@{6}: commit (plan 01-01 Task 1) — mine, clean
  3bf2c78 HEAD@{5}: commit (plan 01-02 attempt 1) — had the correct governance files
  7006cae HEAD@{4}: reset  (plan 01-02 undid its own commit)
  5d6f06c HEAD@{3}: commit (plan 01-02 attempt 2) — accidentally consumed my staged crate files
  7006cae HEAD@{2}: reset  (plan 01-02 undid that too, wiping my Task 2 work from history)
  a1895d6 HEAD@{1}: commit (plan 01-02 attempt 3) — finally clean governance-only commit
  6ef8f7f HEAD@{0}: commit (plan 01-02 CONTRIBUTING + SECURITY)
```

My original Task 2 commit (`5d6f06c` at HEAD@{3}) was lost when the 01-02 executor did `git reset --soft HEAD~1` to clean up its own mis-composed commit at HEAD@{2}. Files remained on disk (untracked) but vanished from history.

**Recovery:** Once the 01-02 executor's activity stabilised (two clean governance commits landed as `a1895d6` and `6ef8f7f`), I re-staged my 15 Task 2 files (`crates/**` + `Cargo.lock`) and committed them cleanly as `4f8e535 feat(01-01): seven crate skeletons …`. This commit is final, correctly attributed, carries `Signed-off-by` + `Co-Authored-By: Claude Sonnet 4.6`, and passes all plan-level verification commands.

**Impact on plan-level success criteria:** None. Every success criterion is met in the final HEAD state:
- All 18 deliverable files are present in HEAD (`git ls-tree -r HEAD` confirms: 3 repo-root config + 14 crate files + Cargo.lock).
- `cargo check --workspace --all-features` exits 0.
- `cargo metadata` reports 7 members with correct names.
- `Cargo.lock` is committed.
- `rust-toolchain.toml` pins 1.95 with divergence comment.

**Impact on traceability / DCO:** Both final plan 01-01 commits (`7006cae` and `4f8e535`) carry `Signed-off-by: Shafqat Ullah` and `Co-Authored-By: Claude Sonnet 4.6` trailers. DCO compliance is preserved. The lost intermediate commit `5d6f06c` is visible only in the reflog and carries no semantic weight.

**Root-cause prevention (for the executor workflow, not actioned in this plan):** Wave-parallel executors should either (a) operate in separate git worktrees, or (b) hold a per-repo commit mutex, or (c) serialize the commit step via a lock file. None of those is scaffolded in the current GSD workflow; this is a latent issue that will recur on any future wave with >1 concurrent executor that both touch the shared index. Recommend filing this as a GSD infrastructure backlog item.

## Auth Gates

None. No secrets, API keys, or external services were required for this plan.

## Known Stubs

None inherent to plan 01-01. Each crate's `src/lib.rs` / `src/main.rs` is an intentional empty stub for compilation purposes only; real code lands in later plans (03, 06) and later phases (2, 4, 9) per the per-crate descriptions above. These are documented architecturally, not stubs that leak to user-visible UI.

## Deferred Issues

None. The plan executed with one blocking install (Rust toolchain — Rule 3, resolved) and one parallel-execution collision (Rule 4 — surfaced for orchestrator, does not block plan-level success).

## Self-Check

- [x] `Cargo.toml` — FOUND (in HEAD, commit `7006cae`)
- [x] `rust-toolchain.toml` — FOUND (in HEAD, commit `7006cae`)
- [x] `.rustfmt.toml` — FOUND (in HEAD, commit `7006cae`)
- [x] `crates/kay-core/Cargo.toml` — FOUND (in HEAD, commit `4f8e535`)
- [x] `crates/kay-core/src/lib.rs` — FOUND
- [x] `crates/kay-cli/Cargo.toml` — FOUND
- [x] `crates/kay-cli/src/main.rs` — FOUND
- [x] `crates/kay-tauri/Cargo.toml` — FOUND
- [x] `crates/kay-tauri/src/lib.rs` — FOUND
- [x] `crates/kay-provider-openrouter/Cargo.toml` — FOUND
- [x] `crates/kay-provider-openrouter/src/lib.rs` — FOUND
- [x] `crates/kay-sandbox-macos/Cargo.toml` — FOUND
- [x] `crates/kay-sandbox-macos/src/lib.rs` — FOUND
- [x] `crates/kay-sandbox-linux/Cargo.toml` — FOUND
- [x] `crates/kay-sandbox-linux/src/lib.rs` — FOUND
- [x] `crates/kay-sandbox-windows/Cargo.toml` — FOUND
- [x] `crates/kay-sandbox-windows/src/lib.rs` — FOUND
- [x] `Cargo.lock` — FOUND
- [x] Commit `7006cae` — FOUND (git log confirms; DCO + Co-Authored-By: Claude Sonnet 4.6 trailers present)
- [x] Commit `4f8e535` — FOUND (Task 2 final; DCO + Co-Authored-By: Claude Sonnet 4.6 trailers present; 15 files changed, 139 insertions)

**Self-Check: PASSED.** All 18 deliverable files are present on disk and in HEAD. All verification commands (cargo check, cargo metadata, cargo tree, grep workspace = true) return the expected results. Plan 01-01 is complete. A parallel-execution race is documented in Deviations for workflow improvement.

## Metrics

| Metric | Value |
|--------|-------|
| Tasks completed | 2/2 |
| Commits created (plan 01-01) | 2 (Task 1 = `7006cae`; Task 2 = `4f8e535`) |
| Commits lost in race (recovered) | 1 (`5d6f06c` — reset away by parallel executor; re-committed as `4f8e535`) |
| Files created | 17 (3 repo-root config + 14 crate files) + 1 generated (`Cargo.lock`) |
| cargo check --workspace | 3.87s on warm toolchain cache |
| Host OS | macOS arm64 (aarch64-apple-darwin) |
| Rust version installed | 1.95.0 (2026-04-14 / released 2026-04-16) |

## Host-OS Quirks

- First `cargo` invocation downloaded 6 components (rustc, cargo, rust-std, rust-docs, rustfmt, clippy) into `$HOME/.rustup/toolchains/1.95-aarch64-apple-darwin/` — ~400 MB one-time cost. Subsequent runs hit the cache and complete in <100 ms.
- `rust-toolchain.toml` auto-resolution worked as expected: without a default toolchain installed, the first `cargo` call in the repo pulled exactly `1.95` per the pin. This is the intended reproducibility mechanism for CI's tri-OS matrix (plan 04).

## Next Plan in Wave

- `01-02-PLAN.md` (governance: LICENSE, NOTICE, README, ATTRIBUTIONS, CODE_OF_CONDUCT, PR template, docs/signing-keys placeholder + CONTRIBUTING + SECURITY) — executed in parallel with this plan and committed its files cleanly in the final two commits of this wave (`a1895d6`, `6ef8f7f`). No outstanding governance files remain untracked as of plan 01-01 completion.

---

**Executor:** gsd-executor (Claude Sonnet 4.6)
**Completed:** 2026-04-19
