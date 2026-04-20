---
phase: 03-tool-registry-kira-core-tools
fixed_at: 2026-04-21T00:00:00Z
review_path: .planning/phases/03-tool-registry-kira-core-tools/03-REVIEW.md
branch: phase/03-tool-registry
base_head: 925cfae
fix_head: e983959
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 3: Code Review Fix Report — Tool Registry + KIRA Core Tools

**Fixed at:** 2026-04-21
**Source review:** `.planning/phases/03-tool-registry-kira-core-tools/03-REVIEW.md`
**Branch:** `phase/03-tool-registry`
**Diff:** `925cfae..e983959` (6 fix commits + 1 RED-test commit)

## Summary

- Findings in scope: **6** (1 HIGH + 5 MEDIUM per the fix brief)
- Fixed: **6**
- Skipped: **0**
- Full suite (`cargo test -p kay-tools -p kay-cli --all-targets`): **green**
- Lint (`cargo clippy -p kay-tools -p kay-cli --all-targets -- -D warnings`): **green**

## Finding → Commit Mapping

| Finding | Type  | Commit(s) | Subject |
|---------|-------|-----------|---------|
| H-01    | RED test | `bd80318` | `test(kay-tools): H-01 RED regression for pgid SIGKILL escalation` |
| H-01    | fix   | `e357f1e` | `fix(kay-tools): H-01 SIGKILL the process group, not just the leader` |
| M-01    | fix+test | `6e170e3` | `fix(kay-tools): M-01 propagate marker RNG failure instead of silent zero nonce` |
| M-02    | fix+test | `e0e053d` | `fix(kay-tools): M-02 release image quota slot when FS read fails` |
| M-05    | fix   | `e7598bd` | `fix(kay-tools): M-05 consult sandbox before reading image file` |
| M-03+M-04 | fix | `e983959` | `fix(kay-tools): M-03 PTY timeout SIGTERM grace; M-04 clarify stdin-EOF comment` |

## Test Counts (before → after)

| Suite                        | Before | After | Δ   | Notes |
|------------------------------|-------:|------:|----:|-------|
| `kay-tools` lib              | 62     | 63    | +1  | New `new_returns_result_and_succeeds_in_test_env` (M-01) |
| `tests/timeout_cascade.rs`   | 1      | 2     | +1  | New `timeout_cascade_kills_grandchild_that_ignores_sigterm` (H-01) |
| `tests/image_quota.rs`       | 5      | 5     | 0   | `missing_file_returns_io_error_and_does_not_leak_quota` rewritten to assert release (M-02) |
| `tests/registry_integration.rs` | 1 (1 ignored) | same | 0 | — |
| `tests/contract_roundtrip.rs`   | 3 | 3 | 0 | — |
| `tests/dispatcher_dispatch.rs`  | 2 | 2 | 0 | — |
| `tests/events_*`, `forge_bridge_delegation`, `parity_delegation`, others | — | — | 0 | — |
| `kay-cli` (all targets)      | unchanged | unchanged | 0 | — |

Total: +2 tests added, 0 deleted, 1 behavioral rewrite.

## H-01 Red → Green Confirmation

The H-01 regression test `timeout_cascade_kills_grandchild_that_ignores_sigterm`
was introduced in **commit `bd80318`** (test-only). Against the pre-fix
tree (parent `925cfae`, or equivalently `HEAD~fix` from the H-01 fix
commit), it fails as expected:

```
thread 'timeout_cascade_kills_grandchild_that_ignores_sigterm'
  panicked at crates/kay-tools/tests/timeout_cascade.rs:141:5:
H-01 regression: grandchild PID 22962 survived the cascade —
SIGKILL must target the process group, not just the shell leader
```

After commit `e357f1e` (H-01 fix — capture pgid, unconditional
killpg(SIGKILL) after SIGTERM grace, plus repair to
`escalate_sigkill_and_reap` itself), the test is GREEN:

```
running 2 tests
test timeout_sigterm_then_sigkill ... ok
test timeout_cascade_kills_grandchild_that_ignores_sigterm ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Additional H-01 nuance (beyond the review's suggested patch)

The review's suggested fix (swap `child.start_kill()` → `killpg(pgid, SIGKILL)`
inside `escalate_sigkill_and_reap`) is necessary but **not sufficient on
its own** — because `escalate_sigkill_and_reap` only runs when
`child.wait()` times out inside the 2s grace window. In the H-01 repro,
the shell LEADER honors SIGTERM and dies quickly, so `wait()` completes
and the escalation path is skipped — leaving the ignoring grandchild
alive.

The committed fix therefore:
1. Captures pgid BEFORE `wait()`.
2. Unconditionally issues `killpg(pgid, SIGKILL)` after the grace window,
   whether or not the leader already exited.
3. Reaps the leader only if it hadn't exited yet.

The `escalate_sigkill_and_reap` helper was also repaired (still committed
as part of the same fix) so either entry point does the right thing if a
future caller re-introduces the original control flow.

## Per-Finding Notes

### H-01 (fixed, `e357f1e`)
`terminate_with_grace` now captures pgid before wait, issues
`killpg(pgid, SIGKILL)` unconditionally after the SIGTERM grace window,
then reaps the leader. Regression test spawns a grandchild with
`trap '' TERM; exec sleep 60` and probes its PID via `kill(pid, None)`
after cascade — asserts ESRCH.

### M-01 (fixed, `6e170e3`)
`MarkerContext::new` now returns `Result<Self, io::Error>`; the old
`let _ = try_fill_bytes(..)` silent-zero fallback is gone. Call site in
`ExecuteCommandsTool::invoke` maps failure → `ToolError::Io`. Test
helpers (`markers::tests::mk_marker`, `markers::shells::tests::mk`)
`.expect()` the Result. New lib test `new_returns_result_and_succeeds_in_test_env`
locks in the Result-returning contract and rejects a zero-filled nonce.

### M-02 (fixed, `e0e053d`)
Added `ImageQuota::release()` (saturating decrement on both counters).
`ImageReadTool` now calls `quota.release()` on `Err` from `tokio::fs::read`.
The integration test `missing_file_returns_io_error_and_does_not_leak_quota`
was rewritten (name was always correct — implementation was not) to assert
`per_turn_count == 0` and `per_session_count == 0` after failure, and to
perform a follow-up successful read proving the released slot is reusable.

### M-03 (fixed, `e983959`)
PTY timeout path now mirrors `run_piped`'s cascade:
`killpg(pgid, SIGTERM)` → `sleep(SIGTERM_GRACE_SECS)` →
`killpg(pgid, SIGKILL)` → `killer.kill()` (portable-pty teardown).
pgid captured from `child.process_id()` up-front; portable-pty calls
`setsid()` so PID == PGID in the child. Windows retains the MVP
`ChildKiller` single-call path (job objects = Phase 4 per D-05).

### M-04 (fixed, `e983959`)
Replaced the misleading "Drop master to signal EOF on slave stdin
after the command finishes" comment with an accurate one: drop happens
immediately after spawn, giving the slave a closed stdin. Adds a
forward-reference to Phase 5 for real-interactive PTY support.

### M-05 (fixed, `e7598bd`)
`ImageReadTool::invoke` now calls `ctx.sandbox.check_fs_read(&path)`
before reading — mirrors `net_fetch`'s pattern. On denial, the quota
slot is released (via the new `release()` from M-02) and
`ToolError::SandboxDenied` is returned. Module docstring updated: the
earlier "sandbox seam only accepts URL inputs" rationale was wrong —
`Sandbox::check_fs_read` has always existed.

## Intentionally Deferred

Per the fix brief, the following findings are out of scope and deferred to
Phase 4 hardening / backlog:

- **L-01** reader-task abort on timeout — current code is correct under
  normal conditions; optional cleanup.
- **L-02** `should_use_pty` compound-command bypass — cosmetic; documented
  limitation.
- **L-03** empty `runtime/dispatcher.rs` — stale one-line TODO, not
  touched by any in-scope fix above. Defer.
- **L-04** empty `seams/rng.rs` — same as L-03. Defer.
- **L-05** unbounded ImageRead payload size — `max_image_bytes` cap is a
  Phase 5 `ForgeConfig` integration item.
- **L-06** `TaskCompleteTool` docstring nuance re: `verified` flag — purely
  cosmetic.
- **L-07** `harden_tool_schema` non-idempotence — purely cosmetic; docstring
  already warns.
- **I-01..I-03** — informational only.

The `todo!()` comment cleanup at `dispatcher.rs:3` and `rng.rs:3` was
explicitly gated on those files being opened for an in-scope fix — neither
was, so the stale comments remain. They are now the only remaining
residue from the review's LOW/INFO bucket.

## Verification Commands

```bash
cargo test -p kay-tools --lib                              # 63 passed
cargo test -p kay-tools -p kay-cli --all-targets           # all green
cargo clippy -p kay-tools -p kay-cli --all-targets -- -D warnings   # clean
```

## Non-Negotiables Preserved

- **B7 / VAL-007:** `ToolCallContext` shape unchanged (6 fields).
- **NN#1 (parity gate):** `tests/parity_delegation.rs` untouched and green.
- **Object-safety:** `Tool` and `ServicesHandle` trait shapes unchanged;
  `Arc<dyn Tool>` coercion still compiles (`registry_integration.rs`).
- **DCO:** every fix commit carries `Signed-off-by: shafqat <shafqat@sourcevo.com>`.

---

_Fixed: 2026-04-21_
_Fixer: Claude Opus 4.7 (gsd-code-fixer)_
_Iteration: 1_
