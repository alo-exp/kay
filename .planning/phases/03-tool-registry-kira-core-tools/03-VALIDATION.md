---
phase: 3
slug: tool-registry-kira-core-tools
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-20
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `03-RESEARCH.md` §12 (Validation Architecture).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust builtin) + `pretty_assertions` 1.x + `tokio::test` (macros feature) |
| **Config file** | none — per-crate test organization |
| **Quick run command** | `cargo test -p kay-tools --lib` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` |
| **Estimated runtime** | ~5–15 s (lib), ~1–3 min (full inc. PTY + integration) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p kay-tools --lib` (≤15 s).
- **After every plan wave:** Run `cargo test -p kay-tools --all-targets` (lib + integration).
- **Before `/gsd-verify-work`:** `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo deny check` must all be green.
- **Max feedback latency:** 15 s per commit; 3 min per wave.

---

## Per-Task Verification Map

> Filled progressively as plans land. Each task row is attached by the executor at task-completion time.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 0 | (scaffold) | — | N/A | unit | `cargo check -p kay-tools` | ❌ W0 | ⬜ pending |
| 3-02-01 | 02 | 1 | TOOL-01 | — | Registry stores/retrieves `Arc<dyn Tool>` by name; object-safe | unit | `cargo test -p kay-tools --lib registry::` | ❌ W0 | ⬜ pending |
| 3-02-02 | 02 | 1 | TOOL-06 | — | `tool_definitions()` returns OpenAI-compatible schemas | unit | `cargo test -p kay-tools tool_definitions_emit` | ❌ W0 | ⬜ pending |
| 3-02-03 | 02 | 1 | TOOL-03 | T-3-06 | `TaskVerifier::verify` default returns `Pending`; `TaskComplete` requires verifier pass | unit | `cargo test -p kay-tools task_complete_pending` | ❌ W0 | ⬜ pending |
| 3-03-01 | 03 | 2 | TOOL-05 | T-3-01 | `enforce_strict_schema` retained: `required` sorted, `additionalProperties: false`, `allOf` flattened | property | `cargo test -p kay-tools --test schema_hardening_property` | ❌ W0 | ⬜ pending |
| 3-03-02 | 03 | 2 | (events) | — | `AgentEvent::ToolOutput` + `AgentEvent::TaskComplete` emitted additively (no parity break) | unit | `cargo test -p kay-provider-openrouter events::phase3_additions` | ❌ W0 | ⬜ pending |
| 3-04-01 | 04 | 3 | SHELL-01 | T-3-02 | Marker pattern `__CMDEND_<hex32>_<seq>__EXITCODE=N` matches; exit code parsed | unit | `cargo test -p kay-tools markers::scan_line_marker_match` | ❌ W0 | ⬜ pending |
| 3-04-02 | 04 | 3 | SHELL-05 | T-3-02 | Forged marker in stdout does NOT close stream; `subtle::ConstantTimeEq` nonce compare | unit | `cargo test -p kay-tools markers::scan_line_forged` | ❌ W0 | ⬜ pending |
| 3-04-03 | 04 | 3 | SHELL-03 | — | Stdout emits as `ToolOutput` frames BEFORE process exit (no blocking collection) | integration (timed) | `cargo test -p kay-tools --test streaming_latency` | ❌ W0 | ⬜ pending |
| 3-04-04 | 04 | 3 | SHELL-02 | T-3-03 | PTY engages on denylist (e.g., `htop`, `vim`) or explicit `tty: true`; non-PTY default | unit + integration | `cargo test -p kay-tools should_use_pty && cargo test -p kay-tools --test pty_integration` | ❌ W0 | ⬜ pending |
| 3-04-05 | 04 | 3 | SHELL-04 | T-3-04 | Timeout cascade: SIGTERM → 2s grace → SIGKILL → wait for reap on Unix; `TerminateProcess` on Windows | integration | `cargo test -p kay-tools --test timeout_cascade -- --test-threads=1` | ❌ W0 | ⬜ pending |
| 3-04-06 | 04 | 3 | TOOL-02 | T-3-05 | `execute_commands` runs inside project-root sandbox; streams output as `AgentEvent::ToolOutput` | integration | `cargo test -p kay-tools --test execute_commands_e2e -- --nocapture` | ❌ W0 | ⬜ pending |
| 3-05-01 | 05 | 4 | TOOL-04 | T-3-07 | `image_read` accepts base64; per-turn cap=2, per-session cap=20 enforced | unit + integration | `cargo test -p kay-tools image_quota` | ❌ W0 | ⬜ pending |
| 3-05-02 | 05 | 4 | (parity) | T-3-08 | `fs_read`/`fs_write`/`fs_search`/`net_fetch` delegate byte-identically to `forge_app::ToolExecutor` | integration | `cargo test -p kay-tools --test parity_delegation` | ❌ W0 | ⬜ pending |
| 3-06-01 | 06 | 5 | (wiring) | — | `default_tool_set(...)` builds immutable 7-tool registry at CLI startup | integration | `cargo test -p kay-cli --test startup_registry` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/kay-tools/Cargo.toml` — new crate manifest (Phase 2.5 Appendix A: direct `forge_*` path deps only)
- [ ] `crates/kay-tools/src/lib.rs` + module skeleton (`tool.rs`, `registry.rs`, `error.rs`, `schema.rs`, `sandbox.rs`, `verifier.rs`, `events.rs`, `builtins/*.rs`, `markers.rs`, `pty.rs`, `timeout.rs`)
- [ ] `crates/kay-tools/tests/registry_integration.rs` — covers TOOL-01, TOOL-05, TOOL-06
- [ ] `crates/kay-tools/tests/marker_streaming.rs` — covers SHELL-01, SHELL-03
- [ ] `crates/kay-tools/tests/marker_race.rs` — covers SHELL-05
- [ ] `crates/kay-tools/tests/timeout_cascade.rs` — covers SHELL-04
- [ ] `crates/kay-tools/tests/pty_integration.rs` — covers SHELL-02 (unix-only `#[cfg(not(windows))]`)
- [ ] `crates/kay-tools/tests/image_quota.rs` — covers TOOL-04
- [ ] `crates/kay-tools/tests/schema_hardening_property.rs` — covers TOOL-05 (property)
- [ ] `crates/kay-tools/tests/parity_delegation.rs` — covers byte-identical delegation to `forge_app::ToolExecutor`
- [ ] `crates/kay-tools/tests/execute_commands_e2e.rs` — covers TOOL-02
- [ ] Workspace `Cargo.toml` — append `"crates/kay-tools"` member + new workspace deps (`portable-pty = "0.8"`, `subtle = "2.5"`, `nix = "0.29"` on unix cfg, `windows-sys = "0.59"` on windows cfg, `hex = "0.4"`)
- [ ] `crates/kay-cli` — wire `default_tool_set(...)` and `ToolRegistry` at startup

*No framework install needed — `cargo test` + `tokio::test` + `pretty_assertions` already configured by Phase 2.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real PTY spawn with `htop` produces colored TTY frames | SHELL-02 | Requires interactive terminal; CI PTY is pseudo but no human observer | Developer runs `cargo run -p kay-cli -- exec -- htop` locally and confirms rendered UI + ability to quit via `q` |
| Cross-OS timeout signal behavior (Linux/macOS/Windows) | SHELL-04 | CI matrix exercises it, but observable signal semantics (SIGTERM reaching child's children) needs manual `strace` on Linux | Developer runs `strace -e signal -f cargo test -p kay-tools --test timeout_cascade` on Linux and confirms SIGTERM→SIGKILL sequence |
| KIRA parity trio under adversarial input (prompt-injected fake `__CMDEND__` in file contents) | SHELL-05 | Requires crafting pathological inputs; covered in property test but adversarial review is manual | Red-team review of `markers/scan_line.rs` + fuzzing run `cargo +nightly fuzz run marker_scanner -- -max_total_time=60` (if cargo-fuzz added later) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15 s per task commit; < 3 min per wave
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending — set to `approved YYYY-MM-DD` once the plan-checker pass is clean and Wave 0 tests are scaffolded.

---

## Cross-Reference

- Source: `.planning/phases/03-tool-registry-kira-core-tools/03-RESEARCH.md` §12
- Threat model: `.planning/phases/03-tool-registry-kira-core-tools/03-RESEARCH.md` §Threat Model (T-3-01..08)
- Decisions: `.planning/phases/03-tool-registry-kira-core-tools/03-CONTEXT.md` D-01..D-12
- Requirements: `.planning/REQUIREMENTS.md` TOOL-01..06 + SHELL-01..05
- Roadmap success criteria: `.planning/ROADMAP.md` Phase 3 (SC #1–#5)
