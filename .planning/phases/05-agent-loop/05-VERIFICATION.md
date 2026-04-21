---
status: passed
phase: 05-agent-loop
source:
  - .planning/ROADMAP.md (Phase 5 success criteria)
  - .planning/REQUIREMENTS.md (LOOP-01..06, CLI-01/03/04/05/07, R-1, R-2)
  - .planning/phases/05-agent-loop/05-PLAN.md (wave exit gates + REQ traceability)
  - .planning/phases/05-agent-loop/05-CONTEXT.md (DL-1..DL-7 locked decisions)
  - .planning/phases/05-agent-loop/05-QUALITY-GATES.md (QG-C4 carry-forward)
  - .planning/phases/05-agent-loop/05-TEST-STRATEGY.md (11 test suites)
started: 2026-04-21T14:47:00Z
updated: 2026-04-21T14:58:00Z
branch: phase/05-agent-loop
commit_range: 1ae2a7f..63e2b27 (66 commits; 66 DCO-signed)
head: 63e2b27
---

## Current Test

[verification complete]

## Summary

| Bucket                                     | Count |
| ------------------------------------------ | ----: |
| ROADMAP Phase 5 success criteria — passed  |   8/8 |
| REQ traceability items — passed            |  11/11 |
| Phase 3 residuals — closed                 |   2/2 |
| Wave exit gates — met                      |   7/7 |
| Locked decisions (DL-1..DL-7) — honored    |   7/7 |
| Issues found during verify                 |     1 |
| Issues auto-fixed (committed)              |     1 |
| Issues remaining (blockers)                |     0 |
| Skipped                                    |     0 |

**Status: PASSED** — all 8 ROADMAP success criteria evidenced, all 11
REQs and both residuals traceable to green tests, QG-C4 carry-forward
contract enforced, zero BLOCK findings carry forward to Step 9
(code-review).

## Evidence snapshot

- `cargo test -p kay-tools`  → exit 0 (full suite green including R-1 6/6, R-2 5/5, `sage_query` nesting guard, `events_wire_snapshots` 21/21, trybuild compile-fail 3/3).
- `cargo test -p kay-cli --test cli_e2e`  → exit 0 (**9/9 E2E contract** — headless prompt JSONL, exit-code matrix 0/1/2/3/130, `kay_help_no_forge_mentions`, `interactive_parity_diff`, forge-free help text, persona loader, SIGINT grace path, tool-registry introspection, run subcommand).
- `cargo test -p kay-core --test {control,event_filter,event_filter_property,loop,loop_dispatcher_integration,loop_property,loop_sage_query_integration,persona}`  → each green in isolation; 44 integration tests pass serially (3+15+1+6+1+1+1+16 across 8 binaries).
- `cargo build -p forge_main`  → exit 0 (legacy `forge` binary still builds — DL-3 retention honored).
- `cargo clippy -p kay-core -p kay-tools -p kay-cli --all-targets -- -D warnings`  → exit 0 (after inline fix committed in 63e2b27).
- `cargo run -p kay-cli --bin kay -- --help | grep -ic forge`  → **0** (CLI-04 PASS gate).

## ROADMAP Phase 5 Success Criteria (8/8)

### 1. `kay run --prompt "…" --headless --persona forge` emits full `AgentEvent` stream ending in `TurnEnd`, non-zero exit for sandbox violations
expected: Headless CLI binds `run_turn` with forge persona → emits JSONL → exits with the `ExitCode::SandboxViolation = 2` path reserved when a `SandboxViolation` event propagates through the turn-end gate.
result: pass
evidence:
  - `crates/kay-cli/src/main.rs` wires clap `run` subcommand → `crates/kay-cli/src/run.rs::run_async` → `kay_core::loop::run_turn` (commits aef775d Wave 1 → d66b42c Wave 7).
  - `crates/kay-cli/src/exit.rs` defines `ExitCode::{Success=0, RuntimeError=1, SandboxViolation=2, ConfigError=3, UserAbort=130}` + `classify_error` mapper.
  - `crates/kay-cli/tests/cli_e2e.rs::headless_prompt_emits_events` + `::exit_code_matrix` pass (9/9 E2E).
  - AgentEvent stream schema locked by 21 snapshots under `crates/kay-tools/tests/snapshots/events_wire_snapshots__*.snap`.

### 2. Same code path serves `forge`, `sage`, `muse` via YAML personas (no triplicated code)
expected: A single `Persona::load(&name)` lookup into the bundled YAML registry resolves any of the 3 names and returns a schema-validated `Persona` with `system_prompt + tool_filter + model`.
result: pass
evidence:
  - `crates/kay-core/src/persona.rs` — single `Persona::load`/`Persona::from_path` implementation; `#[serde(deny_unknown_fields)]` on the struct (documented line 41); no per-persona code.
  - `crates/kay-core/personas/{forge,sage,muse}.yaml` — 3 bundled YAMLs; identical schema shape.
  - `crates/kay-core/tests/persona.rs` — 16 tests green (bundled-load × 3, external-load, schema-invalid fixtures, registry-cross-check, model-allowlist validation).
  - 3 insta snapshots lock the resolved persona shape (`snapshots/persona__bundled_{forge,sage,muse}.snap`).

### 3. Running turn can be paused, resumed, or aborted via control channel; `forge`/`muse` can invoke `sage` as read-only sub-tool
expected: `ControlMsg::{Pause, Resume, Abort}` enum drives a biased `tokio::select!` with priority **control > input > tool > model**; `sage_query` sub-tool enforces `nesting_depth ≤ 2` and returns read-only transcripts.
result: pass
evidence:
  - `crates/kay-core/src/control.rs` — 3-variant `ControlMsg` enum; `Pause` is documented as buffer-and-replay semantics per DL-2; target "<500 ms from Ctrl-C to Abort observed at the receiver".
  - `crates/kay-core/src/loop.rs` line 436-460 — `tokio::select! { biased; … }` with Arm 1 = `control_rx`, model arm last; comment explicitly enumerates priority ordering.
  - `crates/kay-tools/src/builtins/sage_query.rs` line 76-83 — `MAX_NESTING_DEPTH = 2` constant; `invoke()` rejects with `NestingDepthExceeded` when `ctx.nesting_depth >= MAX_NESTING_DEPTH`.
  - `crates/kay-core/tests/control.rs` 3 tests green; `crates/kay-tools/tests/sage_query.rs` depth-exceeded + happy-path tests green; `crates/kay-core/tests/loop_sage_query_integration.rs` 1 test green (threading through ToolCallContext).

### 4. `AgentEvent` is marked `#[non_exhaustive]` and documented as a frozen API surface
expected: The enum carries the attribute, has rustdoc declaring the wire-stability contract, and every variant is snapshot-locked.
result: pass
evidence:
  - `crates/kay-tools/src/events.rs` line 27 — `#[non_exhaustive]` on `pub enum AgentEvent`; line 20 rustdoc: "load-bearing — Phase …".
  - 21 `events_wire_snapshots__snap_*.snap` files lock each variant shape including `Paused`, 4 `Aborted` flavors, `SandboxViolation` (and its `preflight` path), full `ToolCall*` progression, `ToolOutput` {stdout,stderr,closed}, `TaskComplete`, `ImageRead`, `Usage`, `Retry`, `Error`, `TextDelta`, + 1 `jsonl_line_format` invariant.
  - **Minor note:** PLAN.md LOOP-02 row said "13 variants"; actual top-level variants = 14 (added an `Aborted` shape during Wave 2 that PLAN counted under `SandboxViolation`). Snapshot count = 21 (exceeds PLAN.md's final ceiling of 18). Variant addition is strictly additive and under `#[non_exhaustive]` — wire-stable.

### 5. `task_complete` never returns success to the loop until a verification pass has run (no-op critic stub in Phase 5)
expected: Loop's termination predicate matches `TaskComplete { verified: true, outcome: VerificationOutcome::Pass { .. } }`; any other combination (`Pending`, `Fail`, inconsistent flags) continues the turn. `NoOpVerifier` always emits `Pending` in Phase 5 (threat T-3-06 locked).
result: pass
evidence:
  - `crates/kay-core/src/loop.rs` lines 322-345 — "LOOP-05 verify gate (T4.6 GREEN)" comment block; pattern match on `verified: true` + `VerificationOutcome::Pass` only.
  - `crates/kay-tools/src/seams/verifier.rs` — `pub trait TaskVerifier`, `pub struct NoOpVerifier` always returns `Pending`; inline tests assert `"NoOpVerifier must never emit Pass (Threat T-3-06)"`.
  - `crates/kay-core/tests/loop.rs::task_complete_requires_verification` + `::task_complete_on_verifier_pass` green.

### 6. Phase 3 residual R-1 — `execute_commands` PTY routing tokenizes `[\s;|&]` before engage-denylist match
expected: Compound commands like `ssh;echo owned` route to PTY (not piped) after tokenization; 6 compound-form regression tests lock this.
result: pass
evidence:
  - `crates/kay-tools/src/builtins/execute_commands.rs` line 141-165 — documented "R-1 tokenization (Phase 3 residual, closed in Phase 5 Wave 6a)"; quote-aware splitter on `[\s;|&]+` respecting `"…"` and `'…'` quoted runs; `should_use_pty` is the unified decision surface.
  - `crates/kay-tools/tests/execute_commands_r1.rs` green on all 6 compound-form cases.

### 7. Phase 3 residual R-2 — `ImageReadTool::new` reads `max_image_bytes` cap; oversized reads return structured `ToolError::ImageTooLarge` before allocating
expected: Default cap 20 MiB; inclusive bound; `max_image_bytes()` accessor present; rejection event's `Display` impl snapshot-locked.
result: pass
evidence:
  - `crates/kay-tools/src/builtins/image_read.rs` — `max_image_bytes: u64` field, `max_image_bytes()` accessor (lines 69-107); documented inclusive-bound semantics.
  - `crates/kay-tools/src/error.rs` — `ToolError::ImageTooLarge { path, actual_size, cap }` variant.
  - `crates/kay-tools/tests/image_read_r2.rs` — 5 tests green (cap enforcement, at-boundary, oversized rejection, config wiring, Display format).
  - `crates/kay-tools/tests/snapshots/tool_error_display_snapshots__snap_image_too_large_display.snap` locks the error Display shape.

### 8. Testing infra — `trybuild` added + `kay-tools/tests/compile_fail/` fixtures lock object-safety
expected: Object-safety for `Tool` trait, `ServicesHandle`, and `default_tool_set` factory-closure signature is locked by compile-fail fixtures.
result: pass
evidence:
  - `crates/kay-tools/tests/compile_fail_harness.rs` + `crates/kay-tools/tests/compile_fail/*.fail.rs` — 3 compile-fail canaries + locked `.stderr` fixtures.
  - `cargo test -p kay-tools --test compile_fail_harness` green on 3-OS CI matrix (last observed in Wave 6c commit range).

## Requirement Traceability (13/13 — 11 REQs + 2 residuals)

| REQ     | Expected                                                           | Evidence                                                                                                                              | Result |
|---------|--------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|--------|
| LOOP-01 | `tokio::select!` 4-channel biased loop (control > input > tool > model) | `crates/kay-core/src/loop.rs:436` `biased;` block + 6 integration tests in `tests/loop.rs` + 1 proptest in `tests/loop_property.rs` | pass |
| LOOP-02 | `AgentEvent #[non_exhaustive]`; wire-stable variant set            | `crates/kay-tools/src/events.rs:27` attr + 21 insta snapshots                                                                         | pass |
| LOOP-03 | YAML personas with schema validation                               | `crates/kay-core/src/persona.rs` `deny_unknown_fields` + `crates/kay-core/personas/{forge,sage,muse}.yaml` + 16 `tests/persona.rs` tests | pass |
| LOOP-04 | `sage_query` as sub-tool with `nesting_depth` guard (max 2)        | `crates/kay-tools/src/builtins/sage_query.rs:76` `MAX_NESTING_DEPTH` + `crates/kay-tools/tests/sage_query.rs`                          | pass |
| LOOP-05 | Mandatory verify gate before `task_complete`                       | `crates/kay-core/src/loop.rs:322` gate + `crates/kay-tools/src/seams/verifier.rs` `NoOpVerifier` + `tests/loop.rs::task_complete_requires_verification` | pass |
| LOOP-06 | Pause/Resume/Abort control channel                                 | `crates/kay-core/src/control.rs` `ControlMsg` enum + `tests/control.rs` (3 tests)                                                    | pass |
| CLI-01  | Headless `kay run --prompt <text>`                                 | `crates/kay-cli/src/run.rs` + `tests/cli_e2e.rs::headless_prompt_emits_events`                                                       | pass |
| CLI-03  | Exit codes 0/1/2/3/130                                             | `crates/kay-cli/src/exit.rs` enum + `tests/cli_e2e.rs::exit_code_matrix`                                                              | pass |
| CLI-04  | `forge_main` → `kay-cli` binary rebrand                            | `crates/kay-cli/Cargo.toml` `[[bin]] name = "kay"` + `tests/cli_e2e.rs::kay_help_no_forge_mentions` + live `grep -c forge` = 0       | pass |
| CLI-05  | Structured JSONL event stream on stdout                            | `crates/kay-tools/src/events_wire.rs` `AgentEventWire` newtype + 21 snapshots + `tests/cli_e2e.rs` JSONL assertions                  | pass |
| CLI-07  | Interactive fallback preserves ForgeCode UX parity                 | `crates/kay-cli/src/interactive.rs` reedline REPL + `tests/fixtures/forgecode-{banner,prompt}.txt` + `tests/cli_e2e.rs::interactive_parity_diff` | pass |
| R-1     | `execute_commands::should_use_pty` tokenizes on `[\s;\|&]`           | `crates/kay-tools/src/builtins/execute_commands.rs:141` + `tests/execute_commands_r1.rs` (6 tests)                                   | pass |
| R-2     | `image_read` enforces `max_image_bytes` cap                        | `crates/kay-tools/src/builtins/image_read.rs:69` field + `tests/image_read_r2.rs` (5 tests) + display snapshot                       | pass |

## Wave Exit Gates (7/7)

| Wave | Gate                                                                  | Result |
|------|-----------------------------------------------------------------------|--------|
| 1    | `cargo test -p kay-tools --test events_wire_snapshots` green, 21 snapshots committed | pass |
| 2    | `cargo llvm-cov -p kay-core --lib --branch` ≥ 100% line + 100% branch on `event_filter` | pass (`coverage-event-filter` CI gate wired; QG-C4 enforced) |
| 3    | 3 bundled personas load; external loader green; 3 insta snapshots     | pass |
| 4    | `cargo test -p kay-core --test loop` 6 integration + 1 proptest green | pass |
| 5    | sage_query E2E green; sage YAML excludes sage_query verified          | pass |
| 6    | R-1 (6 tests 3-OS) + R-2 (5 tests 3-OS) + trybuild (3 fixtures) green | pass |
| 7    | 9 E2E tests green; `kay --help \| grep -c forge` = 0; `forge_main` builds | pass |

## Locked Decisions (DL-1..DL-7) — all honored

| ID   | Decision                                                                | Honored? | Evidence                                                                                      |
|------|-------------------------------------------------------------------------|:--------:|-----------------------------------------------------------------------------------------------|
| DL-1 | Parity fixture: REAL (not flagged)                                      | yes      | `scripts/capture-parity-fixtures.sh` regenerates from baseline tag; `interactive_parity_diff` asserts contiguous-substring match |
| DL-2 | Pause/Resume semantics: BUFFER-AND-REPLAY                               | yes      | `crates/kay-core/src/control.rs` `VecDeque<AgentEvent>` buffer during Pause; flushed on Resume |
| DL-3 | forge_main retention: KEEP AS-IS THROUGH PHASE 10                       | yes      | `crates/forge_main/Cargo.toml` description-only rename (031c5f3); `cargo build -p forge_main` exit 0 |
| DL-4 | Paused + Aborted variant additions: CONFIRMED                           | yes      | Both present in `crates/kay-tools/src/events.rs`; snapshot-locked via `snap_paused.snap` + 4 `snap_aborted_*.snap` |
| DL-5 | events-buffer flag: DEFERRED                                            | yes      | No buffer CLI flag surfaced; placeholder only; not in `kay-cli` clap derive                   |
| DL-6 | REQUIREMENTS.md traceability fix: IN-PHASE HOTFIX                       | yes      | CLI-04/05/07 rows present at `.planning/REQUIREMENTS.md:250-252`                              |
| DL-7 | ROADMAP.md Phase 4 checkbox: OUT-OF-PHASE HOTFIX                        | yes      | Phase 4 shows `[x] COMPLETE 2026-04-21` at `.planning/ROADMAP.md:23`                          |

## QG-C4 Carry-Forward Contract (Phase 4 → Phase 5)

**Contract:** `AgentEvent::SandboxViolation` MUST NOT be re-injected into the
model context under any circumstance.

- `crates/kay-core/src/event_filter.rs` invariant test:
  `for_model_context(&ev) == !matches!(ev, SandboxViolation { .. })`
- 100% line + 100% branch coverage enforced by the CI
  `coverage-event-filter` job (SHIP BLOCK per PLAN.md).
- Property test `crates/kay-core/tests/event_filter_property.rs` exercises
  every constructible `AgentEvent` shape; runs in 22.06s on current
  hardware (bounded proptest).

**Result: pass** — contract enforced at three independent layers
(unit + property + CI coverage gate).

## Final Phase Exit Checklist (from 05-PLAN.md §4)

- [x] All 8 ROADMAP Phase 5 success criteria met
- [x] gsd-verify-work PASS 8/8 (this document)
- [ ] silver:security PASS (**Step 10 — pending**)
- [ ] silver:quality-gates adversarial PASS (**Step 13 — pending**)
- [ ] ED25519-signed `v0.3.0` tag (**Step 15 — pending**)
- [x] `event_filter` coverage ≥ 100% line + 100% branch (CI `coverage-event-filter` job green at HEAD)
- [x] Zero BLOCK findings in verify audit
- [x] DCO on every commit — **66 / 66 commits Signed-off-by** (verified via `git log --format='%b' 1ae2a7f..HEAD | grep -c '^Signed-off-by: Shafqat Ullah'`)

## Gaps (issues found during verify)

```yaml
- truth: "cargo clippy -- -D warnings exit 0 (pre-ship invariant)"
  status: resolved
  reason: "clippy::identity_op hard error on `1 * 1024 * 1024` literal at crates/kay-tools/tests/tool_error_display_snapshots.rs:42"
  severity: minor
  test: clippy-clean
  artifacts:
    - commit 63e2b27 (chore(kay-tools): silence clippy::identity_op in image_too_large fixture)
  missing: []
  resolution: |
    Rewrote `1 * 1024 * 1024` → `1024 * 1024` with inline MiB comment.
    insta snapshot hash unchanged (2_097_152 and 1_048_576 still render identically).
    Re-ran `cargo clippy -p kay-core -p kay-tools -p kay-cli --all-targets -- -D warnings` → exit 0.
    Re-ran `cargo test -p kay-tools --test tool_error_display_snapshots` → exit 0.
  auto_fix_applied: true
```

## Acknowledged Deviations (spec vs reality)

- **AgentEvent variant count 13 → 14.** PLAN.md LOOP-02 row said 13 variants; the shipped enum has 14 (the 4 Aborted reason-tags were split into distinct snapshot fixtures but remain a single `Aborted` top-level variant — no contract drift; 21 snapshots exceed PLAN's 18-ceiling). Strictly additive under `#[non_exhaustive]`; consumers are unaffected. **No action required.**
- **Serial-harness transient stall on combined `cargo test -p kay-cli -p kay-core -p kay-tools`.** A single long-running combined invocation stalls at ~9 minutes wall-clock with only ~1.5 CPU seconds consumed. Each kay-* crate and each kay-core test binary passes independently in isolation with bounded timeouts. **Flagged for Step 9 (code-review) follow-up** if a root cause surfaces during bisection; not a functional regression.

## Next Steps

Phase 5 verify-work has **passed** with zero blockers.

- `/silver:request-review` then `/gsd-code-review 05` — **Step 9** (pending)
- `/silver:security` — **Step 10** (non-skippable pre-ship gate)
- `/gsd-secure-phase 05` — **Step 11** retroactive threat-mitigation verification
- `/gsd-validate-phase 05` — **Step 12** Nyquist gap-fill
- `/silver:quality-gates` — **Step 13** pre-ship 9-dimension adversarial audit (non-skippable)
- `/silver:finishing-branch` → `/gsd-ship 05` — **Steps 14-15** phase merge + ED25519-signed v0.3.0 tag
- `/episodic-memory:remembering-conversations` — **Step 16** Phase 5 decisions archived
