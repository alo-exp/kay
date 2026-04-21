---
phase: 05-agent-loop
status: passed
recommendation: PASS
reviewer: Claude (gsd-code-reviewer)
depth: deep
branch: phase/05-agent-loop
commit_range: 1ae2a7f..8432c87
head: 8432c87d8cdee9775e56808f50c46e56d9059ae5
commits: 68
started: 2026-04-21T15:40:00Z
updated: 2026-04-21T16:29:00Z
findings:
  blocker: 0
  critical: 0
  high: 0
  medium: 0
  low: 2
  info: 3
  total: 5
source:
  crates:
    - crates/kay-core
    - crates/kay-tools
    - crates/kay-cli
  docs_reviewed:
    - .planning/phases/05-agent-loop/05-PLAN.md
    - .planning/phases/05-agent-loop/05-CONTEXT.md
    - .planning/phases/05-agent-loop/05-QUALITY-GATES.md
    - .planning/phases/05-agent-loop/05-VERIFICATION.md
  key_files_read:
    - crates/kay-core/src/loop.rs
    - crates/kay-core/src/event_filter.rs
    - crates/kay-core/src/control.rs
    - crates/kay-core/src/persona.rs
    - crates/kay-tools/src/events.rs
    - crates/kay-tools/src/events_wire.rs
    - crates/kay-tools/src/error.rs
    - crates/kay-tools/src/default_set.rs
    - crates/kay-tools/src/runtime/context.rs
    - crates/kay-tools/src/builtins/sage_query.rs
    - crates/kay-tools/src/builtins/execute_commands.rs
    - crates/kay-tools/src/builtins/image_read.rs
    - crates/kay-cli/src/main.rs
    - crates/kay-cli/src/run.rs
    - crates/kay-cli/src/exit.rs
    - crates/kay-cli/src/interactive.rs
    - crates/kay-cli/src/prompt.rs
    - crates/kay-cli/src/banner.rs
    - crates/kay-cli/src/boot.rs
    - crates/kay-cli/tests/cli_e2e.rs
---

# Phase 5 — Agent Loop + Canonical CLI — Code Review

## Summary

Phase 5 delivers the first end-to-end turn loop and kay-branded headless CLI in
68 DCO-signed, GPG-signed commits between `1ae2a7f..8432c87`. The review
examined the three owned crates (`kay-core`, `kay-tools`, `kay-cli`) at depth,
cross-referenced the 10 non-negotiable invariants from `05-VERIFICATION.md`,
ran `cargo clippy --workspace --all-targets -- -D warnings` (clean, exit 0),
`cargo test -p kay-core -p kay-tools -p kay-cli --no-fail-fast` (**227 passed /
0 failed**), and confirmed `cargo fmt --all -- --check` is a no-op (exit 0).
Security-critical gates — QG-C4 event filter, persona `deny_unknown_fields`,
R-1 quote-aware PTY tokenizer, R-2 metadata-first image size cap, MAX_NESTING_DEPTH=2
recursion guard — are all present, tested, and documented with module-level
rationale that reads like the engineer was writing for a future auditor. The
biased `tokio::select!` loop orders control > input > tool > model per
BRAINSTORM's locked table and is exercised by both happy-path
integration tests and a 256-case close-order proptest. No BLOCKER, CRITICAL,
HIGH, or MEDIUM-severity issues were found. Five observations recorded: two
LOW (wildcard-arm forward-compat and SIGINT task is detached) and three INFO
(pre-existing `forge_app` test failures unrelated to Phase 5, `let _ =
dispatch(..)` silently drops `ToolError` by design, TTY-fallback error arm
bypasses `classify_error`). **Recommendation: PASS. Advance to Step 10
`/silver:security`.**

## Findings

| ID        | File:Line                                               | Severity | Category            | Summary                                                                                                                                                                                                                            | Recommendation                                                                                                                                                       | Status   |
| --------- | ------------------------------------------------------- | -------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| L-05-01   | `crates/kay-tools/src/events_wire.rs:226,245`           | LOW      | Forward-compat      | `retry_reason_tag` and `provider_error_kind` use wildcard `_ => "unknown"` arms against `kay_provider_errors::{RetryReason, ProviderError}` — both `#[non_exhaustive]`. A future variant would silently emit `"unknown"` in the JSONL wire stream instead of forcing a compile-time schema review. | Add a Phase-9-ish hardening task to (a) exhaustively match all current variants with `match` blocks that drop the wildcard by moving the match inside the same crate (via a trait impl on `kay-provider-errors`), or (b) emit a distinct tag (e.g., `"unknown_retry_reason_future_variant"`) plus a `tracing::warn!` so gaps are observable in logs. Not gating for Phase 5. | Open     |
| L-05-02   | `crates/kay-core/src/control.rs:189-198`                | LOW      | Resource management | `install_ctrl_c_handler` uses `tokio::spawn(async move { … })` and discards the `JoinHandle` — the listener task is fully detached and cannot be awaited or shut down from outside. On a normal `kay run` this is fine (process exits; OS reaps), but a future library-embedded use (Tauri host, long-lived daemon) would leak one task per `install_ctrl_c_handler` call. | Keep the MVP as-is; file a backlog note for Phase 10+ Tauri integration to either return the `JoinHandle` or wrap registration in an `Arc<AbortHandle>` so embedders can tear down the listener between turns. | Open     |
| I-05-01   | `cargo test --workspace` (any commit)                   | INFO     | Test hygiene        | `cargo test --workspace` surfaces 39 failing tests in `forge_app` (all `orch_spec::*` and `compact::*` — missing `forge-custom-agent-template.md` partial). `git log 1ae2a7f..HEAD -- crates/forge_app/` returns zero commits; the failures trace to the Phase 2.5 wholesale import (commit `7e22cc5`) and predate Phase 5 entirely. | Not a Phase 5 issue; should be tracked as a separate upstream-template-vendoring task. Until then, CI should invoke `cargo test -p kay-core -p kay-tools -p kay-cli` (scoped) rather than `--workspace` to avoid the noise. | Deferred |
| I-05-02   | `crates/kay-core/src/loop.rs` (dispatch call sites)     | INFO     | Error handling      | The T4.11 refactor (`5801a0b`) dispatches tool calls through `let _ = dispatcher.dispatch(...)` — `ToolError` variants from the dispatcher are silently dropped rather than surfaced as a `ToolCallComplete { outcome: Err(..) }` event. The module-level doc acknowledges this as Wave-4-later scope. | Fine for this phase (scope-gated + documented). Phase 6 tool-call plumbing should wrap the dispatch in `match` and emit the `Err` arm onto the event channel so the JSONL wire surface carries tool failures to GUI consumers. | Open     |
| I-05-03   | `crates/kay-cli/src/interactive.rs:194-205`             | INFO     | Error handling      | The reedline error arm in the interactive REPL logs to stderr and `break`s, causing `run()` to return `Ok(())` → `ExitCode::Success` (0). A raw-mode failure is therefore indistinguishable at the shell level from a clean Ctrl-D exit, and never routes through `classify_error` to `RuntimeError` (1). | The comment at lines 198-202 explicitly documents the choice — the REPL has already "done its job" by the time this fires. Revisit in Phase 10+ when the REPL carries real user work; a raw-mode crash mid-turn should arguably propagate as non-zero. | Open     |

**No BLOCKER, CRITICAL, HIGH, or MEDIUM findings.**

## Invariants Verified

### Non-negotiables (10)

- [x] **QG-C4 — SandboxViolation never re-injected into model context.**
  Single-gate implementation at `crates/kay-core/src/event_filter.rs` line 81:
  `!matches!(event, AgentEvent::SandboxViolation { .. })`. 100% line + 100%
  region CI gate via the `coverage-event-filter` job (commit `0e25274`). 15
  per-variant tests (2 deny) plus 10k-case proptest using a Clone-safe
  `EventSpec` proxy (`tests/event_filter_property.rs`).
- [x] **NoOpVerifier Pass-gate on `TaskComplete`.** `crates/kay-core/src/loop.rs`
  short-circuits only when both `verified: true` AND `outcome: Pass { .. }`
  match (`matches!(&ev, TaskComplete { verified: true, outcome: Pass { .. },
  .. })`). The `Pending` case does NOT close the loop. Locked by
  `tests/loop.rs::task_complete_gated_on_verified_pass`.
- [x] **Biased `tokio::select!` priority ordering — control > input > tool >
  model.** `crates/kay-core/src/loop.rs` uses `biased;` at the `select!` open
  and orders control before model. `tests/loop_property.rs` exercises random
  close-order over 256 cases without deadlock or lost events.
- [x] **Persona YAML strictness.** `#[serde(deny_unknown_fields)]` at
  `crates/kay-core/src/persona.rs:84` on `Persona`; equivalently on
  `SageQueryArgs` (`crates/kay-tools/src/builtins/sage_query.rs:122`),
  `ImageReadArgs` (`image_read.rs:54`), and `TaskCompleteArgs`
  (`task_complete.rs:25`). Any unknown field in a bundled or external YAML
  produces a structured `PersonaError::Yaml` / tool-arg decode error.
- [x] **R-1 — Quote-aware PTY tokenizer.** `crates/kay-tools/src/builtins/
  execute_commands.rs::split_shell_tokens` walks ALL shell tokens on
  `[\s;|&]+` separators (not just the first). 6 regression fixtures
  (`tests/execute_commands_r1.rs`) cover quoted-pipe-in-arg, semicolon-chain,
  backgrounded-chain, escaped-quote, and leading-denylisted-helper-before-
  `vim` patterns.
- [x] **R-2 — Metadata-first image size cap.** `image_read.rs` gates on
  `tokio::fs::metadata().len()` BEFORE reading bytes. Inclusive `20 *
  1024 * 1024` cap via `DEFAULT_MAX_IMAGE_BYTES` + optional `with_size_cap`
  constructor. Quota credits released on every error path (Io, TooLarge,
  DecodeFailed). Insta snapshot locks the `ToolError::ImageTooLarge` Display
  form. 5 boundary tests in `tests/image_read_r2.rs`.
- [x] **`#[non_exhaustive]` on all wire-surface enums.** Confirmed on
  `AgentEvent` (`events.rs:27`), `ToolOutputChunk` (`events.rs:161`),
  `ToolError` (`error.rs:8`), `VerificationOutcome` (`seams/verifier.rs:11`),
  `CapScope` (`error.rs:66`), `LoopError` (`loop.rs:495`), `ToolCallContext`
  (`runtime/context.rs:60`). Wave 6c trybuild object-safety canaries
  (`tests/object_safety_canaries.rs`) lock the shape.
- [x] **Clean-room provenance.** All bundled persona YAMLs
  (`crates/kay-core/personas/{forge,sage,muse}.yaml`) declare "clean-room
  from public ForgeCode docs" in commit `dbf53dc`. No TypeScript-derived
  structure from the 2026-03-31 leak.
- [x] **OpenRouter model allowlist.** `persona.rs` rejects unknown models via
  `PersonaError::ModelNotAllowed` (tested in `tests/persona.rs`). Phase 5 does
  not expand the allowlist — persona load validates against the Phase 4
  locked set.
- [x] **JSON schema hardening.** All tool arg structs
  (`SageQueryArgs`/`ImageReadArgs`/`TaskCompleteArgs`) use
  `rename_all = "snake_case"` + `deny_unknown_fields`. The
  required-before-properties flattening pattern inherited from Phase 3 is
  preserved (no regressions).

### Depth-gated sub-loop safety

- [x] **`MAX_NESTING_DEPTH = 2`.** `crates/kay-tools/src/builtins/sage_query.rs`
  line 1 const. Depth check fires BEFORE args parse to avoid leaking YAML
  injection errors from a disallowed recursion level. Inner `ToolCallContext`
  clones all seams and bumps only `nesting_depth + 1`. The bundled `sage.yaml`
  persona explicitly excludes `sage_query` from its tool allowlist, giving
  belt-and-suspenders coverage against a sage-invokes-sage loop (regression
  tested by `tests/persona.rs::sage_tool_filter`).
- [x] **NoOpInnerAgent returns `ExecutionFailed`.** `crates/kay-cli/src/boot.rs`
  stub never returns `Ok` — it emits a pointed `ToolError::ExecutionFailed`
  with a clear "sage_query not wired at CLI boundary yet" message. No silent
  success path.

### Exit-code contract (CLI-03)

- [x] **`#[repr(u8)]` on `ExitCode`.** `crates/kay-cli/src/exit.rs` locks
  discriminants 0 (Success), 1 (RuntimeError), 2 (SandboxViolation), 3
  (UsageError), 130 (UserAbort) with `#[repr(u8)]` for ABI stability.
- [x] **Precedence: UserAbort > SandboxViolation > Success.** `run.rs` builds
  `aborted_seen` and `sandbox_violation_seen` sticky booleans during event
  drain; the final `match (aborted_seen, sandbox_violation_seen)` returns
  `UserAbort` whenever aborted, `SandboxViolation` otherwise if a violation
  was seen, else `Success`. Exercised by `tests/cli_e2e.rs::exit_code_2/3/130`.

## Clippy & Lint

Ran: `cargo clippy --workspace --all-targets -- -D warnings` (2026-04-21,
cache-warm, exit 0, 1.62s).

- `kay-core` — **clean** (0 warnings)
- `kay-tools` — **clean** (0 warnings; `clippy::identity_op` on the
  `image_too_large` fixture was explicitly fixed by commit `63e2b27` to use
  `20 * 1024 * 1024` with an inline `// 20 MiB` comment)
- `kay-cli` — **clean** (0 warnings)
- `forge_main` / other forge_* crates — **clean** (0 warnings; DL-3 retention
  contract with forward-facing binary-preservation through Phase 10)

Ran: `cargo fmt --all -- --check` (exit 0). Five nightly-only
`unstable_features` warnings are expected (workspace `rustfmt.toml` targets
nightly sugar but degrades cleanly on 1.95-stable).

## Test Quality

**Scoped test run:** `cargo test -p kay-core -p kay-tools -p kay-cli --no-fail-fast`
→ **227 passed / 0 failed / 1 ignored** across 33 test binaries.

Coverage high points:

- **Unit + integration parity.** Every Wave 4 loop branch is covered by at
  least one `tests/loop.rs` case (happy path, verify gate, pause, abort,
  idempotent-abort, close-order).
- **Property tests for security-sensitive gates.** `event_filter_property.rs`
  drives 10,000 randomised `AgentEvent`s through the filter and asserts
  SandboxViolation is never emitted downstream. `loop_property.rs` runs 256
  close-order permutations to exercise the `biased select!` under adversarial
  channel-close sequences.
- **Insta snapshots for wire-shape locks.** 21 `AgentEventWire` cases
  (`events_wire_snapshots.rs`), 3 persona cases (`persona.rs`), 1
  `ImageTooLarge` case (`tool_error_display_snapshots.rs`), and Wave 6c
  trybuild stderrs all versioned under `tests/snapshots/`. Any wire-schema
  drift would flip a snapshot diff, not silently change the JSONL output.
- **End-to-end parity.** `crates/kay-cli/tests/cli_e2e.rs` spawns `kay` as
  a separate process for 9 scenarios including the SIGINT → exit 130 loop
  (1500ms settle window), help string brand-swap (no `forge` mentions), JSONL
  stream emission, and interactive-fallback parity diff against a captured
  ForgeCode baseline.
- **Object-safety canaries.** Wave 6c `trybuild` fail-fixtures lock
  `Arc<dyn Tool>` object safety at compile time; any future trait change that
  breaks the object-safe surface flips a compile error with a reviewer-friendly
  stderr diff.

Test-hygiene observations (not findings):

- Dispatch-related assertions in `loop_sage_query_integration.rs` use
  `NullServices` stubs duplicated across `kay-core` and `kay-cli`. This is
  intentional per DL-3 (no cross-crate test-type coupling); the duplication
  is narrow (a zero-field empty struct with trait no-ops) and keeps the
  dependency arrow uni-directional.
- One `#[ignore]` marker exists (the SIGINT nix test on Windows). The
  platform-gate (`#[cfg(unix)]`) makes this a no-op on CI targets; scored
  as expected behavior, not test debt.

## Security

Phase 5 introduces no new cryptographic surface but adds three new
security-sensitive gates, each examined:

- **Persona YAML injection.** `deny_unknown_fields` is the first line of
  defence; the second is `allowed_tools: Vec<String>` validation against the
  registered tool-name set (rejects unknown references with
  `PersonaError::UnknownTool`). External loader (`Persona::from_path`)
  surfaces `Io` vs. `Yaml` errors distinctly so an attacker-controlled YAML
  path cannot masquerade as a parse-level failure.
- **R-1 PTY bypass.** The quote-aware tokenizer closes the "leading helper
  command + pipe + vim" bypass reported during Phase 4 review. Fixture
  matrix covers single-quoted, double-quoted, escaped, semicolon-chained,
  and backgrounded variants.
- **R-2 image exhaustion.** Metadata-first size gate plus quota-credit
  release on every error path ensures a malicious / corrupted image file
  cannot consume the process's image budget even when the byte-level decode
  would later fail. Inclusive 20 MiB cap matches the Phase 3 spec.
- **Ctrl-C handler arm order.** `install_ctrl_c_handler` calls the
  synchronous registrar (`unix::signal(SignalKind::interrupt())` /
  `windows::ctrl_c()`) BEFORE spawning the listener task. The module docs
  explicitly call this out as load-bearing: deferred-arming via bare
  `tokio::signal::ctrl_c()` would race the 1500ms E2E settle window.
- **`kay-cli` arg parsing.** clap derives with explicit `--offline`,
  `--max-turns`, and `--persona` flags; no `arg_required_else_help` (per
  CLAUDE.md UX contract that plain `kay` drops into the interactive REPL).
  Max-turns short-circuits to success before the tokio runtime spins up, so
  a `--max-turns 0` invocation cannot trigger any tool registry or persona
  side effects.

No secrets in source. `grep -rn -E "(password|api[_-]?key|secret)\\s*=\\s*['\"]" crates/`
returns zero matches across `kay-core`, `kay-tools`, `kay-cli` source.

## DCO + Signing

- **DCO sign-off**: 68 / 68 commits carry `Signed-off-by: Shafqat Ullah
  <shafqat@sourcevo.com>` (one of the `Non-Negotiables` from CLAUDE.md).
- **GPG / SSH signing**: 68 / 68 commits report `%G?` = `G` (good signature).
  No `N` (no sig), `U` (unknown-signer), `B` (bad), or `X` (expired)
  entries.

## Recommendation

**PASS — advance to Step 10 `/silver:security`.**

Phase 5 ships the first turn loop and kay-branded CLI with zero BLOCKER /
CRITICAL / HIGH / MEDIUM findings. The 227-test in-scope suite is green,
clippy is clean under `-D warnings`, all 10 non-negotiables are either
preserved or actively hardened (QG-C4, R-1, R-2 all add new tests), and
every locked decision from `05-CONTEXT.md` (DL-1..DL-7) is implemented as
specified. The two LOW items and three INFO items are documented scope
choices or deferred Tauri-integration concerns — none gate shipping v0.3.0.

---

_Reviewed: 2026-04-21T16:29:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep (cross-file call-chain trace + wire-schema audit + invariant sweep)_
