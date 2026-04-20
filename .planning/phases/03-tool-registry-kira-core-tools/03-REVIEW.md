---
phase: 03-tool-registry-kira-core-tools
reviewed: 2026-04-21T00:00:00Z
depth: deep
branch: phase/03-tool-registry
head: 925cfae
diff_base: 1bb792d..8a9a0d5
files_reviewed: 26
files_reviewed_list:
  - crates/kay-tools/src/lib.rs
  - crates/kay-tools/src/contract.rs
  - crates/kay-tools/src/registry.rs
  - crates/kay-tools/src/error.rs
  - crates/kay-tools/src/events.rs
  - crates/kay-tools/src/schema.rs
  - crates/kay-tools/src/forge_bridge.rs
  - crates/kay-tools/src/quota.rs
  - crates/kay-tools/src/default_set.rs
  - crates/kay-tools/src/runtime/mod.rs
  - crates/kay-tools/src/runtime/context.rs
  - crates/kay-tools/src/runtime/dispatcher.rs
  - crates/kay-tools/src/seams/mod.rs
  - crates/kay-tools/src/seams/rng.rs
  - crates/kay-tools/src/seams/sandbox.rs
  - crates/kay-tools/src/seams/verifier.rs
  - crates/kay-tools/src/markers/mod.rs
  - crates/kay-tools/src/markers/shells.rs
  - crates/kay-tools/src/builtins/mod.rs
  - crates/kay-tools/src/builtins/execute_commands.rs
  - crates/kay-tools/src/builtins/image_read.rs
  - crates/kay-tools/src/builtins/task_complete.rs
  - crates/kay-tools/src/builtins/fs_read.rs
  - crates/kay-tools/src/builtins/fs_write.rs
  - crates/kay-tools/src/builtins/fs_search.rs
  - crates/kay-tools/src/builtins/net_fetch.rs
  - crates/kay-tools/tests/**/*.rs (11 files)
  - crates/kay-cli/src/main.rs
  - crates/kay-cli/src/boot.rs
  - crates/kay-provider-errors/src/lib.rs
  - crates/kay-provider-openrouter/src/event.rs (delta)
  - crates/kay-provider-openrouter/src/error.rs (delta)
findings:
  critical: 0
  high: 1
  medium: 5
  low: 7
  info: 3
  total: 16
status: issues_found
---

# Phase 3: Code Review Report — Tool Registry + KIRA Core Tools

**Reviewed:** 2026-04-21
**Depth:** deep (adversarial, per-axis phase-3 review)
**Branch / HEAD:** `phase/03-tool-registry` @ `925cfae`
**Diff range:** `1bb792d..8a9a0d5` (Waves 0-4)
**Files Reviewed:** 26 source + 11 integration tests
**Status:** issues_found (0 CRITICAL, 1 HIGH, 5 MEDIUM, 7 LOW, 3 INFO)

## Summary

Phase 3 lands a well-factored tool registry with strong object-safety discipline, a load-bearing marker protocol with constant-time nonce compare, and clean schema hardening that delegates to ForgeCode verbatim. Testing is dense (174 tests incl. proptest + parity gates) and the 10 documented Rule-3 reconciliations all check out. DCO is clean across all 16 Wave 0-4 commits.

**Gate for security review:** 0 CRITICAL findings. **One HIGH finding** (H-01: SIGKILL escalation does not target the process group) must be resolved or explicitly accepted before declaring Phase 3 security-clean, because it weakens SHELL-04 (orphan reaping is the entire point of process-group termination). The MEDIUM findings cluster around three themes: (a) marker RNG failure silently produces a predictable nonce (M-01 — attacker-relevant for SHELL-05), (b) quota reservation leaks on IO failure (M-02 — contradicts docstring + enables DoS of per-session cap), (c) PTY path has zero SIGTERM grace and drops stdin eagerly (M-03, M-04).

None of the MEDIUM issues block the Phase 3 verify gate but they should all be tracked into Phase 4/5 hardening.

**Strong points worth calling out:**
- `subtle::ConstantTimeEq` correctly used for the 32-byte nonce compare (markers/mod.rs:87).
- `Arc<dyn Tool>` object-safety is locked at three tiers (trait shape, integration test coercion, trybuild fixture).
- Parity test (`tests/parity_delegation.rs`) proves byte-identical output via two independent execution paths with a shared `CallLog` — strong enough to catch argument-reordering regressions.
- `ImageQuota::try_consume` is a genuinely atomic check (single call, both dimensions, correct rollback on breach).
- Zero placeholder strings (`todo!()`, `unimplemented!()`, `unreachable!()`, `unimplemented_at_planning_time_*`) on any reachable production path. Scanned clean.
- Zero `.unwrap()` / `.expect()` in production code paths (all occurrences are under `#[cfg(test)]` and the crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` enforces this).

---

## HIGH Issues

### H-01: `escalate_sigkill_and_reap` sends SIGKILL only to shell leader, not the process group

**File:** `crates/kay-tools/src/builtins/execute_commands.rs:370-375`
**REQ/NN:** SHELL-04 (timeout cascade completeness)

**Issue:**
The SIGTERM path correctly uses `killpg(pgid, SIGTERM)` to hit every process in the spawned process group (line 363). When the grace window expires and the cascade escalates, `escalate_sigkill_and_reap` calls `child.start_kill()` (which delivers SIGKILL to the child PID only) instead of `killpg(pgid, SIGKILL)`. Any grandchild in the same pgid that ignored or mishandled SIGTERM survives — the shell leader dies, the survivors get reparented to PID 1, and Kay has no handle on them.

This contradicts the SHELL-04 plan text ("SIGTERM-grace-SIGKILL cascade to the whole process group") and the module comment on line 335-337 which asserts "killpg (positive pgid) to target the process group ... shell-spawned descendants inherit the pgid and receive SIGTERM" — but not SIGKILL. A misbehaving benchmark command (e.g., `trap '' TERM; sleep 3600 &`) leaves a zombie-candidate behind after Kay reports timeout.

The `tests/timeout_cascade.rs::timeout_sigterm_then_sigkill` test only verifies the ELAPSED time of the outer `invoke` call, not that the pgid is empty after cascade — so this regression is not test-covered.

**Fix:**
```rust
#[cfg(unix)]
async fn escalate_sigkill_and_reap(child: &mut tokio::process::Child) {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid;
    if let Some(pid_u32) = child.id() {
        let _ = killpg(Pid::from_raw(pid_u32 as i32), Signal::SIGKILL);
    }
    // Still await child.wait() to reap the leader; descendants are now PID-1-owned zombies.
    let _ = child.wait().await;
}
```

Recommend also extending `tests/timeout_cascade.rs` with a case that forks a grandchild (e.g., `command: "( trap '' TERM; sleep 60 ) &"`) and asserts the grandchild's PID is gone after the cascade via a post-timeout `kill -0 <pid>` probe.

---

## MEDIUM Issues

### M-01: Marker RNG failure silently produces predictable zero nonce

**File:** `crates/kay-tools/src/markers/mod.rs:34-45`
**REQ/NN:** SHELL-01, SHELL-05, NN#7 (marker protocol load-bearing for TB 2.0)

**Issue:**
`MarkerContext::new` calls `SysRng.try_fill_bytes(&mut nonce_bytes)` and ignores the result (`let _ = ...`). The docstring explicitly rationalizes: "A failure maps to zero bytes — scan_line still works, just reduces entropy of THIS call." That rationalization understates the impact — zero entropy for one call means a prompt-injection attacker can pre-compute the valid marker `__CMDEND_00000000000000000000000000000000_<seq>__EXITCODE=0` (seq is a monotonic counter trivially guessable by the attacker across calls) and force early stream closure with an attacker-supplied exit code.

Yes, getrandom() effectively never fails on modern OSes — but the silent zero-fallback is exactly the kind of assumption that turns into a CVE when a sandboxed/seccomp/chroot environment blocks `getrandom`. The lib.rs `#![deny(clippy::unwrap_used, clippy::expect_used)]` forced the pattern, but a silent zero is worse than a panic here.

**Fix:**
Two acceptable paths:
1. Propagate the failure — make `MarkerContext::new` return `Result<Self, io::Error>` and have `ExecuteCommandsTool::invoke` map it to `ToolError::Io`.
2. Fall back to a mixed entropy source (e.g., `(seq << 32) | nanos_since_epoch` XORed across 16 bytes) which is still forgeable but not zero.

Path 1 is cleaner and adds ~3 lines at the call-site (line 185).

---

### M-02: ImageQuota reservation leaks on filesystem IO failure (docstring contradiction)

**Files:**
- `crates/kay-tools/src/builtins/image_read.rs:117-137`
- `crates/kay-tools/tests/image_quota.rs:141-171` (pins the contradictory behavior)

**REQ:** TOOL-04, T-3-07

**Issue:**
The image_read.rs docstring lines 117-118 claims "Reserve a quota slot BEFORE touching the filesystem — a failed read shouldn't consume quota." The implementation does exactly the opposite: `try_consume()` runs first, then `tokio::fs::read`, and there is **no rollback** on `ToolError::Io`. The integration test `missing_file_returns_io_error_and_does_not_leak_quota` explicitly asserts a POST-condition of `per_turn_count == 1` after a failed call — the test name contradicts its own assertion.

Impact: a malicious/confused prompt that supplies 20 non-existent `.png` paths drains the per-session cap without ever reading a single byte. The model is then unable to read legitimate images for the rest of the session — a low-effort DoS against IMG-01.

**Fix (pick one):**
1. Move `self.quota.try_consume()` AFTER the `tokio::fs::read` call, so only successful reads charge quota. Aligns impl with the existing docstring.
2. Keep the current ordering but catch `ToolError::Io` and issue a matching `release()` on the quota (requires adding `ImageQuota::release()` as a public API — small addition).
3. Update the docstring + test name to reflect current behavior as intentional and rename the test to `missing_file_returns_io_error_and_holds_quota_reservation`.

Option 1 is the minimal-change fix and matches the stated design intent.

---

### M-03: PTY timeout path skips SIGTERM grace window entirely

**File:** `crates/kay-tools/src/builtins/execute_commands.rs:489-502`
**REQ:** SHELL-04 (parity between piped and PTY paths)

**Issue:**
The `run_piped` path correctly implements SIGTERM → 2s grace → SIGKILL cascade via `terminate_with_grace`. The `run_pty` path on timeout calls `killer.kill()` (portable-pty's `ChildKiller`) — this delivers an immediate SIGKILL with zero grace. PTY-run commands (`ssh`, `vim`, `psql`) are exactly the commands most likely to benefit from a graceful shutdown (state flush, tty restore); the asymmetry is surprising and undocumented.

This is not a correctness bug (it DOES terminate), but it's a parity divergence vs. the plan's SHELL-04 spec. No test covers the PTY timeout grace.

**Fix:**
Factor the cascade into a helper that operates on a `portable_pty::ChildKiller`-compatible interface (or accept the asymmetry explicitly and document it with a forward-reference to a Phase 4 ticket). Minimal patch: send SIGTERM via `killer` (portable-pty v0.8+ exposes this), sleep `SIGTERM_GRACE_SECS`, then `killer.kill()` for SIGKILL.

---

### M-04: PTY `pair.master` dropped immediately — comment claims "after the command finishes"

**File:** `crates/kay-tools/src/builtins/execute_commands.rs:432-433`

**Issue:**
```rust
// Drop master to signal EOF on slave stdin after the command finishes.
drop(pair.master);
```
The comment is misleading. `drop(pair.master)` runs immediately after spawn — BEFORE the command runs, not "after the command finishes". The effect is that the slave's stdin sees EOF immediately. For `ssh -V` this is fine (no stdin read), but for any genuinely interactive command (`sudo`, `psql`, `vim`) this breaks the "PTY" promise — the child sees an instantly-closed stdin. The test suite only covers `ssh -V` and `echo` (both stdin-free).

This is a phase-scoped deviation (Phase 3 has no user-facing interactive agent), but:
1. The comment lies about when the drop occurs → confuses future maintainers.
2. The behavior means "engaging PTY" doesn't actually enable interactivity; it only gets the PTY's line-buffering / terminal-emulation quirks.
3. An agent that invokes `sudo some-cmd` will hang or fail in real TB 2.0 benchmarks (sudo wants a password on stdin).

**Fix:**
- Correct the comment to "Drop master to signal EOF on slave stdin; the command runs with a closed stdin. Interactive input is out of scope for the agent-driven executor."
- Document this as a known limitation in the `run_pty` function docstring (or promote to a module-level note).
- File a Phase 5 ticket for actual-interactive PTY support if real agent use-cases require it.

---

### M-05: `ImageReadTool` reads arbitrary filesystem paths without sandbox

**File:** `crates/kay-tools/src/builtins/image_read.rs:126-137`
**REQ:** TOOL-04 (sandbox seam present but unused by image_read)

**Issue:**
`ImageReadTool::invoke` accepts any path string, detects MIME from the extension, and calls `tokio::fs::read(&path_buf)` directly. There is no `ctx.sandbox.check_fs_read(&path_buf)` call. An agent can read ANY file whose extension happens to be jpg/jpeg/png/webp/gif — including symlinks into `/etc/shadow.png`, `/root/.ssh/id_rsa.png`, etc. NoOpSandbox wouldn't have blocked this anyway, but the omission means even Phase 4's real sandbox won't be consulted.

The docstring notes this deliberately: "Direct `tokio::fs::read` instead of going through the sandbox seam or ImageReadService ... any size-limit / auth check Phase 5 adds will happen at the sandbox layer." Fair as a scope note, but the risk exists today: Phase 3 agents running against a real filesystem CAN exfiltrate arbitrary files by renaming. `net_fetch` bothered to call `ctx.sandbox.check_net(&url)` — the asymmetry is notable.

**Fix:**
Add `ctx.sandbox.check_fs_read(&path_buf).await?` between line 133 (mime detection) and line 135 (the read). With NoOpSandbox this is a no-op; Phase 4 gains real enforcement without further changes to image_read. Four-line patch.

---

## LOW Issues

### L-01: `terminate_with_grace` does not abort `stdout_task` / `stderr_task` on timeout

**File:** `crates/kay-tools/src/builtins/execute_commands.rs:315-328`

**Issue:**
On the timeout branch, the two reader tasks (lines 255, 275) are not joined or aborted. They will eventually complete when the child dies and the pipes close — no leak — but if the SIGKILL path hangs (unlikely, bugged portable-pty could), the tasks leak for the duration of the process. Current code is "correct under normal conditions."

**Fix:**
Optional: store `JoinHandle`s and call `.abort()` in the timeout branch. Or rely on `kill_on_drop(true)` which covers the happy-case drop.

### L-02: `should_use_pty` bypassed by metacharacters in first token

**File:** `crates/kay-tools/src/builtins/execute_commands.rs:133-145`

**Issue:**
A command like `ssh;echo owned` has `first = "ssh;echo"`, `file_stem = Some("ssh;echo")`, no denylist match → piped path. Not a security issue (piped path is safer), but means the intended "engage PTY for ssh" heuristic silently fails when the agent composes a compound command. The resulting `ssh` invocation then fails with "no tty" and the agent wastes a turn.

**Fix:**
Tokenize more carefully (split on `[\s;|&]`), or accept the cosmetic degradation and document the compound-command limitation.

### L-03: `runtime::dispatcher` module is empty public surface

**File:** `crates/kay-tools/src/runtime/dispatcher.rs`

**Issue:**
File contains only a single-line TODO comment. Module is `pub mod dispatcher` (lib.rs:15) — exposes an empty namespace to external crates. If Wave 5/Phase 5 intends to populate it, the TODO should reference the planning doc; if not, make it `#[cfg(test)] mod dispatcher` or remove until needed.

**Fix:**
Either (a) delete the file and `pub mod dispatcher` until it has a body, or (b) expand the TODO to cite `.planning/phases/03-tool-registry-kira-core-tools/03-02-PLAN.md` so the deferral is traceable.

### L-04: `seams::rng` module is empty public surface

**File:** `crates/kay-tools/src/seams/rng.rs`

Same rationale as L-03 — single TODO line, module is publicly re-exported via `seams/mod.rs`. Wave 3 (03-04) moved the RNG work directly into `MarkerContext::new` using `rand::rngs::SysRng`, obsoleting this module's original purpose. Recommend delete or fill with a thin wrapper for Phase 4 testability.

### L-05: Unbounded ImageRead event payload size

**File:** `crates/kay-tools/src/builtins/image_read.rs:142-145`

**Issue:**
`AgentEvent::ImageRead { bytes: bytes.clone() }` materializes a full copy of the file's contents alongside the base64-encoded output. A 500 MB `.png` would spike RSS by ~1.7 GB (raw + clone + base64). No size cap.

**Fix:**
Add a `max_image_bytes` cap (default e.g. 20 MiB — parity with ForgeCode's `max_file_size`) to `ImageReadTool` construction. Reject with `ToolError::InvalidArgs` before the read, or truncate with a warning. Small change; fits with the Phase 5 `ForgeConfig` integration.

### L-06: `TaskCompleteTool` docstring contradicts code behavior re: `verified` flag

**File:** `crates/kay-tools/src/builtins/task_complete.rs:7-9`

**Issue:**
The docstring says "Phase 3's NoOpVerifier must never produce a Pass outcome. This tool therefore always emits verified: false". In code, line 98 correctly computes `verified = matches!(outcome, Pass { .. })` — which WILL become `true` as soon as a real verifier (Phase 8) returns Pass. The docstring reads like an invariant of the tool; it's actually an invariant of the CURRENT verifier impl. Misleading for future readers.

**Fix:**
Reword: "Phase 3's NoOpVerifier always returns Pending → `verified` is always false today. When Phase 8 swaps in a real verifier returning Pass, this tool's `verified` flag will track it."

### L-07: `harden_tool_schema` non-idempotent when `output_truncation_note` is set (sharp edge)

**File:** `crates/kay-tools/src/schema.rs:32-57`

**Issue:**
The module docstring (lines 32-35) explicitly warns that calling twice with a non-None hint appends the note twice. No runtime guard. Today all callers build schema in `new()` once, but a future refactor that rebuilds schema (e.g., per-invocation for dynamic hints) would silently double-append.

**Fix:**
Guard by checking if the existing description already ends with the note (`existing.ends_with(note)`) before appending. O(1) additional work; preserves idempotency without a stateful marker.

---

## INFO

### I-01: DCO clean on all Wave 0-4 commits

Verified via `git log 1bb792d..8a9a0d5`: every commit carries `Signed-off-by:`. Two commits (`1362b99`, `9dffe1a`, `ef439af`) have a duplicated trailer (`Signed-off-by: Shafqat` and `Signed-off-by: Shafqat Ullah`) — technically valid, cosmetically odd. Consider a local git hook that dedupes.

### I-02: Object-safety defense is multi-layered and sound

Three independent tiers keep `Arc<dyn Tool>` from regressing:
1. `Tool` trait has no generic methods, no associated types, no `Self: Sized` bounds (contract.rs).
2. `tests/registry_integration.rs::arc_dyn_tool_is_object_safe` — coercion fails at compile time if the trait loses dyn-compat.
3. `tests/compile_fail/tool_not_object_safe.fail.rs` — reviewer-visible trybuild fixture (currently `#[ignore]`d due to forge_tool_macros CWD issue; documented deferral).

The same triangulation covers the owned-`serde_json::Value` return of `input_schema` (A1 resolution). Strong guarantees; no recommendation.

### I-03: Parity test runs independent service instances (as required)

`tests/parity_delegation.rs` satisfies the review contract's item 5: both paths (facade-via-ToolInvoke vs. direct service call + `format_*` helpers) use independent `FakeFs*` constructions per test, and assert byte-for-byte equality via `pretty_assertions::assert_eq`. The `CallLog` further verifies argument-forwarding to catch argument-reordering regressions. Strong — no concerns.

---

_Reviewed: 2026-04-21_
_Reviewer: gsd-code-reviewer (Claude Opus 4.7)_
_Depth: deep (adversarial, Phase-3 ten-axis review)_
_Next action: owner triage of H-01 before security review; M-01..M-05 folded into Phase 4 hardening ticket set._
