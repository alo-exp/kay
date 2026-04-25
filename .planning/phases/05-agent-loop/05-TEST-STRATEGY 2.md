# Phase 5 Test Strategy — Agent Loop + Canonical CLI

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop (Event-Driven Core) + Canonical CLI rebrand
> **Upstream:** 05-BRAINSTORM.md §Engineering-Lens (E1-E12)
> **TDD policy:** Superpowers TDD — every wave RED → GREEN → REFACTOR

---

## Testing pyramid

```
               /  Parity + Smoke  \            Few, slow, highest confidence
              / ───────────────── \
             /   E2E CLI (3 OSes)  \           Cross-OS correctness
            / ───────────────────── \
           /  Contract (JSONL schema) \        Wire-format locks
          / ─────────────────────────── \
         /    Integration (per-module)   \     Module-level interop
        / ───────────────────────────────── \
       /           Compile-fail (trybuild)    \  Object-safety locks
      / ─────────────────────────────────────── \
     /              Property (proptest)           \ Invariants over random inputs
    / ─────────────────────────────────────────────\
   /                    Unit                        \  Fastest; most granular
  /─────────────────────────────────────────────────/
```

- **Unit (~60% of tests)** — business logic, parsing, state transitions, pure functions
- **Property (~5%)** — invariants over random inputs (select!-order resilience, event_filter coverage)
- **Compile-fail (~3%, new tier)** — object-safety + factory signatures via `trybuild`
- **Integration (~20%)** — module composition (loop + dispatcher + persona + event filter)
- **Contract (~5%)** — JSONL wire-schema snapshots (insta)
- **E2E (~5%)** — `kay run --prompt …` invoked as subprocess on 3 OSes
- **Parity + Smoke (~2%)** — snapshot against `forgecode-parity-baseline` tag; long-running smoke

---

## Tooling

| Tooling                          | Use                                                                  | Rationale                                                                                |
| -------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `cargo test`                     | All Rust tests                                                       | Stdlib                                                                                   |
| `tokio::test(flavor = "multi_thread")` | Async loop tests                                               | Required for `tokio::select!` with real multi-task scheduling                            |
| `proptest` (workspace dep)       | Property tests                                                       | Already in use from Phase 4                                                              |
| `insta` (workspace dep)          | AgentEvent wire-format snapshots, CLI-output snapshots               | Already in use from Phase 3 / 4                                                          |
| `trybuild` (**new dep**)         | Compile-fail fixtures for object-safety                              | Phase 5 adds this — see 05-BRAINSTORM.md §E11                                            |
| `assert_cmd` + `predicates`      | CLI subprocess invocation (E2E)                                      | Exit code + stdout/stderr assertions against `kay` binary                                |
| `tempfile`                       | Per-test tmp dirs for sandbox + snapshot artifacts                   | Isolation                                                                                |
| `wiremock`                       | Mock OpenRouter provider for loop E2E                                | Already in use from Phase 2                                                              |
| `serial_test`                    | CLI signal-handler tests (Ctrl-C)                                    | Global signal state can't be parallelized                                                |

No new crates except `trybuild`.

---

## Coverage targets

| Scope                                         | Target line coverage | Target branch coverage |
| --------------------------------------------- | -------------------- | ---------------------- |
| `kay-core::loop`                              | ≥ 90%                | ≥ 85%                  |
| `kay-core::persona`                           | ≥ 95%                | ≥ 90%                  |
| `kay-core::event_filter`                      | **100%**             | **100%**               |
| `kay-core::control`                           | ≥ 95%                | ≥ 90%                  |
| `kay-cli` (binary)                            | ≥ 80%                | ≥ 70%                  |
| `kay-tools::builtins::execute_commands` (R-1) | ≥ 95%                | ≥ 90%                  |
| `kay-tools::builtins::image_read` (R-2)       | ≥ 95%                | ≥ 90%                  |
| `kay-tools::builtins::sage_query` (new)       | ≥ 95%                | ≥ 90%                  |

`event_filter` is **100% / 100%** because it is the QG-C4 enforcement seam. Any uncovered branch is a prompt-injection surface.

---

## Test plan by module

### T-1: `kay-core::loop` (LOOP-01, LOOP-05, LOOP-06)

**Unit**
- `run_turn_single_turn_happy_path` — input → model TextDelta → model ToolCallComplete → tool invocation → ToolOutput → TurnEnd. Verify `Stream<AgentEvent>` emits in order.
- `run_turn_max_turns_terminates` — max_turns = 3; assert 3 turns, then TurnEnd (un-verified).
- `run_turn_empty_input_channel_does_not_spin` — input.recv returns None → loop exits gracefully with 0 cycles.
- `task_complete_requires_verification` (LOOP-05) — `task_complete` emits TaskComplete { verified: false } (Phase 5 uses NoOpVerifier → Pending outcome); loop does NOT return success — continues until max_turns.
- `loop_respects_biased_select_order` — inject simultaneous Abort + model chunk; assert Abort wins.

**Property (proptest)**
- `select_robust_to_random_close_order` — 10k runs: randomly close any subset of {input, model, tool, control} channels; assert loop exits within 1s for every ordering.
- `select_bias_holds_under_load` — 10k runs: inject N mixed events + 1 Abort; assert Abort is observed before completion-TurnEnd.

**Integration**
- `loop_plus_dispatcher_invokes_registered_tool` — register EchoTool, send model ToolCallComplete with name="echo"; assert ToolOutput event with echo's return.
- `loop_plus_noop_verifier_emits_pending_outcome` — send model's task_complete call; assert `AgentEvent::TaskComplete { verified: false, outcome: VerificationOutcome::Pending { reason } }`.

### T-2: `kay-core::persona` (LOOP-03, LOOP-04)

**Unit**
- `persona_yaml_deserializes` — happy path for each bundled persona (forge/sage/muse).
- `persona_schema_rejects_unknown_tool` — tool_filter contains `"fs_nuke"` (unknown); `from_str` returns Err with "unknown tool in filter".
- `persona_system_prompt_multiline_preserves` — input YAML has `|` block scalar; loaded `system_prompt` field preserves newlines.
- `persona_load_from_binary_bundled` — `Persona::load("forge")` works with no fs access (include_str!).
- `persona_load_from_path_validates_name` — external YAML path with `name: ""` (empty) rejected.

**Integration**
- `persona_tool_filter_applied_at_dispatch` — persona="sage" excludes fs_write; model issues fs_write call; loop emits `AgentEvent::ToolCallMalformed` OR rejects with ToolError::NotAllowed; assert fs_write NOT invoked.
- `sage_subtool_bounded_nesting_depth` (LOOP-04) — forge calls sage_query; sage_query's nested turn calls sage_query again; assert second nested call returns ToolError with "nesting_depth>=2".

### T-3: `kay-core::control` (LOOP-06)

**Unit**
- `abort_cancels_in_flight_tool` — spawn a long-running tool; send Abort; assert tool's cancel token is set within 100ms.
- `pause_halts_further_output_emission` — send Pause; assert UI sink receives `AgentEvent::Paused` + no further events until Resume.
- `resume_replays_buffered_events` — Pause → queue 5 model TextDeltas in buffer → Resume → assert all 5 emitted in order to UI sink.
- `abort_after_pause_still_aborts` — Pause → Abort; assert Aborted event + loop exits within 2s grace.
- `ctrl_c_converts_to_abort_via_control_channel` — install signal handler; raise SIGINT (via `nix::sys::signal::raise` on Unix / `GenerateConsoleCtrlEvent` on Windows where supported, else mocked handler); assert ControlMsg::Abort received within 100ms.

### T-4: `kay-core::event_filter` (QG-C4 — load-bearing)

**Unit**
- `model_context_filter_blocks_sandbox_violation` — pass SandboxViolation; assert `for_model_context` returns false.
- `model_context_filter_blocks_paused_and_aborted` — same for Paused, Aborted.
- `model_context_filter_blocks_tool_call_malformed` — (per E12)
- `model_context_filter_blocks_retry` — (per E12)
- `model_context_filter_allows_text_delta` — TextDelta → true.
- `model_context_filter_allows_tool_output` — ToolOutput → true (the tool ran successfully, its output goes to model).
- `model_context_filter_allows_usage` — Usage → true.
- `model_context_filter_allows_task_complete` — TaskComplete → true (the model needs to know task_complete ran).
- `model_context_filter_allows_image_read` — ImageRead → true.
- (Every enum variant has one allow/block test — 100% variant coverage.)

**Property**
- `model_context_filter_random_sequences_never_leak_sandbox_violation` — 10k runs: generate random AgentEvent sequence (any variant, any field); apply filter; assert `SandboxViolation` never appears in the filtered output, regardless of position or count.

### T-5: `kay-cli` binary (CLI-01, CLI-03, CLI-04, CLI-05, CLI-07)

**E2E via `assert_cmd`**
- `kay_run_headless_prints_jsonl_and_exits_zero_on_complete` — `kay run --prompt "say done" --headless --persona forge --events jsonl --max-turns 1`; stdout is JSONL; exit = 0; every line parses as AgentEventWire.
- `kay_run_headless_exits_two_on_sandbox_violation` — mock provider issues fs_write outside project root; exit = 2; SandboxViolation event present in stdout.
- `kay_run_headless_exits_one_on_max_turns` — max_turns = 0; exit = 1.
- `kay_run_headless_exits_three_on_missing_persona` — `--persona nonexistent`; exit = 3; stderr has "persona not found".
- `kay_run_no_args_enters_interactive` — no args; reads TTY; banner printed; prompt string = "kay> " (NOT "forge>"). [Interactive mode test may use pty/pty_process for TTY sim.]
- `kay_help_mentions_kay_not_forge` — `kay --help`; stdout does NOT contain "forge" in top-level help (brand swap check).
- `kay_version_matches_cargo_toml` — `kay --version` returns workspace version.
- `kay_ctrl_c_exit_code_130` — `kay run --prompt "loop forever" --headless`; send SIGINT; exit = 130 (or 3221225786 on Windows where `^C` produces that); all events flushed before exit.
- **Parity gate**: `kay_interactive_banner_diffs_only_on_brand_strings` — capture interactive startup output; diff against fixture from `forgecode-parity-baseline` tag; assert diff lines are ONLY brand-string lines (ForgeCode → Kay). If fixture absent (parity tag didn't capture interactive snapshots), raise to discuss-phase for a remediation plan.

### T-6: `AgentEvent` wire contract (CLI-05, LOOP-02)

**Contract snapshots (insta)** — one `.snap` per variant, committed under `crates/kay-cli/tests/snapshots/agent_event_wire/`:
- `text_delta.snap`, `tool_call_start.snap`, `tool_call_delta.snap`, `tool_call_complete.snap`, `tool_call_malformed.snap`, `usage.snap`, `retry.snap`, `error.snap`, `tool_output_stdout.snap`, `tool_output_stderr.snap`, `tool_output_closed.snap`, `task_complete_pending.snap`, `image_read.snap`, `sandbox_violation.snap`, `paused.snap`, `aborted.snap`.

Each snap locks: JSON keys + order + type-tag ("type" field) + field names. Drift = test failure. 16 snap files total (one per AgentEvent + ToolOutputChunk variant).

**Schema doc**: `.planning/CONTRACT-AgentEvent.md` — committed alongside tests; same content as the snapshots in Markdown format for humans reading the contract.

### T-7: `kay-tools::builtins::execute_commands` R-1 residual (6 tests)

**Unit**
- `pty_routes_bare_engaged_cmd` — `"ssh user@host echo hi"` → should_use_pty == true
- `pty_routes_semicolon_engaged` — `"ssh;echo owned"` → true (was the Phase 3 bug)
- `pty_routes_pipe_engaged` — `"false | ssh host"` → true
- `pty_routes_and_engaged` — `"ssh host && echo"` → true
- `pty_routes_background_engaged` — `"echo & ssh host"` → true
- `pty_rejects_prefix_match_bug` — `"ssh_hello_world"` → false (prefix match guard)

### T-8: `kay-tools::builtins::image_read` R-2 residual

**Unit**
- `rejects_oversized_before_allocation` — create 21 MiB file in tmp; `ImageReadTool::new(cfg with max=20MiB)` invoked; returns ToolError::ImageTooLarge { actual_bytes: 22020096, cap_bytes: 20971520 }; assert NO allocation (use a memory accounting wrapper or verify via `fs::read` NOT called — spy via mock filesystem seam).
- `accepts_under_cap` — 1 MiB file; cap 20 MiB; succeeds.
- `accepts_at_cap` — exactly 20 MiB; cap 20 MiB; succeeds (<= not <).
- `image_too_large_error_serializes_to_wire` — error → AgentEventWire::Error; snap-matches fixture.
- `default_cap_is_20_mib` — `ForgeConfig::default().image_read.max_image_bytes == 20 * 1024 * 1024`.

### T-9: `kay-tools::builtins::sage_query` (LOOP-04 sub-tool)

**Unit**
- `sage_query_schema` — input_schema has `query: string`.
- `sage_query_invokes_nested_turn` — mock provider; assert nested turn runs with persona=sage.
- `sage_query_filters_tools_to_read_only` — nested turn's tool_filter == ["fs_read", "fs_search", "net_fetch"].
- `sage_query_bounded_max_turns` — inner max_turns = 5; runs exactly 5 if no task_complete.
- `sage_query_recursion_depth_guard` — nested context has nesting_depth=1; recursive call would be 2 → rejects.

**Integration**
- `forge_calls_sage_via_sage_query_end_to_end` — full loop; forge persona; mock provider issues sage_query; assert outer AgentEvent stream includes ToolCallComplete(sage_query) → nested events (visible OR opaque via ToolOutput) → ToolOutput(sage_summary).

### T-10: Compile-fail tier (trybuild — new)

Fixtures under `crates/kay-tools/tests/compile_fail/`:
- `tool_not_object_safe.rs` — adds a method `fn gimme<T>(&self, t: T);` to Tool; compile-fail expected.
- `services_handle_not_object_safe.rs` — same for ServicesHandle.
- `default_tool_set_sig_change.rs` — calls `default_tool_set()` with wrong factory-closure signature.

Runner: `tests/compile_fail.rs` with `#[test] fn trybuild() { trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs"); }`.

### T-11: Smoke (nightly + pre-ship)

- `smoke_kay_run_toy_task_succeeds` — real OpenRouter endpoint (gated on `KAY_OPENROUTER_API_KEY` env), real loop, real sandbox, prompt = "say hello"; expect TurnEnd, cost < $0.01, no errors.
- `smoke_4h_memory_stability` — run a script that issues 1000 turns over 4 hours; assert no memory growth > 50 MB via RSS sampling. (Parallel to Phase 9's memory canary but at CLI-only level.)

---

## Test-level distribution by REQ

| REQ    | Unit | Integration | Contract | E2E | Property | Compile-fail | Smoke |
| ------ | ---- | ----------- | -------- | --- | -------- | ------------ | ----- |
| LOOP-01 | T-1  | T-1         |          |     | T-1      |              |       |
| LOOP-02 |      |             | T-6      |     |          |              |       |
| LOOP-03 | T-2  | T-2         |          |     |          |              |       |
| LOOP-04 | T-9  | T-9, T-2    |          |     |          |              |       |
| LOOP-05 | T-1  | T-1         |          |     |          |              |       |
| LOOP-06 | T-3  | T-3         |          | T-5 |          |              |       |
| CLI-01  |      |             |          | T-5 |          |              | T-11  |
| CLI-03  |      |             |          | T-5 |          |              |       |
| CLI-04  |      |             |          | T-5 |          |              |       |
| CLI-05  |      |             | T-6      | T-5 |          |              |       |
| CLI-07  |      |             |          | T-5 |          |              |       |
| R-1     | T-7  |             |          |     |          |              |       |
| R-2     | T-8  |             |          |     |          |              |       |
| trybuild |    |             |          |     |          | T-10         |       |
| QG-C4   | T-4  |             |          | T-5 | T-4      |              |       |

**Coverage of 11 REQs + 2 residuals + trybuild + QG-C4 = 15 distinct test targets across 11 test suites.**

---

## CI matrix

| OS              | Runs                                          | Gate |
| --------------- | --------------------------------------------- | ---- |
| ubuntu-latest   | All test suites + smoke (unless costly API)   | Blocking |
| macos-14        | All test suites (no API smoke)                | Blocking |
| windows-latest  | All test suites except signal-handler E2E (SIGINT on Windows is conditional; use `GenerateConsoleCtrlEvent` where possible, else `#[cfg(windows)]` alt test path) | Blocking |
| ubuntu-latest (nightly cron) | T-11 smoke (4h memory) — NON-blocking fail-fast | Nightly only |

All 3 OSes run the 16-variant insta snapshot tier (T-6) — wire schema must be bit-identical across platforms.

---

## Test file locations

```
crates/kay-core/src/loop.rs                     (T-1 unit tests inline with #[cfg(test)] mod)
crates/kay-core/src/persona.rs                  (T-2 unit tests inline)
crates/kay-core/src/control.rs                  (T-3 unit tests inline)
crates/kay-core/src/event_filter.rs             (T-4 unit tests inline + property)
crates/kay-core/tests/loop_integration.rs       (T-1 integration)
crates/kay-core/tests/persona_integration.rs    (T-2 integration)
crates/kay-core/tests/loop_select_properties.rs (T-1 property tests)
crates/kay-core/tests/event_filter_properties.rs (T-4 property)
crates/kay-cli/tests/cli_e2e.rs                 (T-5)
crates/kay-cli/tests/agent_event_wire.rs        (T-6 contract)
crates/kay-cli/tests/snapshots/agent_event_wire/*.snap (T-6 snapshots)
crates/kay-tools/tests/execute_commands_r1.rs   (T-7)
crates/kay-tools/tests/image_read_r2.rs         (T-8)
crates/kay-tools/tests/sage_query.rs            (T-9)
crates/kay-tools/tests/compile_fail.rs          (T-10 runner)
crates/kay-tools/tests/compile_fail/*.rs        (T-10 fixtures)
crates/kay-cli/tests/smoke.rs                   (T-11, #[ignore] unless env gated)
```

---

## Exit criteria — Phase 5 testing done when:

- [ ] All unit tests pass on macOS + Linux + Windows
- [ ] `event_filter` has 100% line + 100% branch coverage (measure via `cargo-llvm-cov`)
- [ ] All 16 AgentEvent insta snapshots committed and green
- [ ] `trybuild` compile-fail suite green
- [ ] `assert_cmd` E2E suite green on all 3 OSes (Ctrl-C test platform-gated)
- [ ] Parity-snapshot fixture exists from `forgecode-parity-baseline` tag OR a remediation plan is committed as a Phase 5 OUT-of-scope item (flagged in discuss-phase)
- [ ] `nightly-smoke.yml` CI workflow exists and at least one dry run passed
- [ ] No test has been marked `#[ignore]` without a GitHub Issue link explaining why

---

## Open questions for discuss-phase (carried from brainstorm)

- **Signal testing**: Can we reliably test SIGINT on Windows in CI, or do we guard with `#[cfg(not(windows))]` and rely on manual verification? → discuss-phase decides.
- **Parity fixtures**: Does the `forgecode-parity-baseline` tag capture interactive mode? If no, what's the remediation plan (pair with Phase 1 retro-add)? → discuss-phase decides.
- **Provider mock**: reuse Phase 2's `wiremock` setup or stand up a new `MockProvider` trait impl specifically for loop tests? → Lean: MockProvider trait; discuss-phase confirms.

---

**Next step:** Step 2.5 writing-plans → skeleton implementation plan keyed to these test suites.
