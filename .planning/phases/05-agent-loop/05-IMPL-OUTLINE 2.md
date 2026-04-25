# Phase 5 Implementation Outline — Agent Loop + Canonical CLI

> **Date:** 2026-04-21
> **Upstream:** 05-BRAINSTORM.md + 05-TEST-STRATEGY.md
> **Purpose:** TDD-focused wave skeleton to seed `gsd-plan-phase` (Step 6). NOT the executable PLAN.md — that is produced by the GSD planner from this outline.
> **TDD discipline:** every task RED (failing test) → GREEN (minimal impl) → REFACTOR. Commits atomic; DCO-signed.

---

## Wave breakdown (7 waves)

Each wave is self-contained: ends with a green `cargo test --workspace` on 3 OSes (or platform-gated subset). Wave N+1 depends on Wave N's public API. Parallelism opportunities noted.

### Wave 1 — AgentEvent wire layer (LOOP-02 · CLI-05 locks the contract)

**Goal:** `AgentEvent` is JSONL-serializable via `AgentEventWire`; 16 insta snapshots committed; schema frozen via doc + snap.

**Tasks (TDD order):**

1. (RED) Write T-6 contract snapshots — one `.snap` per variant — all FAIL (AgentEventWire doesn't exist yet)
2. (GREEN) Create `crates/kay-tools/src/events_wire.rs` — mirror enum + `From<&AgentEvent> for AgentEventWire` + `serde::Serialize`
3. (GREEN) `impl Display for AgentEventWire` → JSONL line
4. (REFACTOR) Hoist shared serialization helpers; add module doc linking to CONTRACT-AgentEvent.md
5. Add 2 new variants to `AgentEvent`: `Paused` (unit), `Aborted { reason: String }`
6. (RED → GREEN) Add snapshot fixtures for `Paused` + `Aborted`
7. Write `.planning/CONTRACT-AgentEvent.md` — human-readable wire schema

**Exit:** 16 snapshots green on all 3 OSes.

**Parallelizable:** No (every downstream wave imports the wire layer).

### Wave 2 — Control channel + event filter (LOOP-06 + QG-C4)

**Goal:** `ControlMsg` mpsc + `event_filter::for_model_context` at 100% coverage.

**Tasks:**

1. (RED) Write T-4 event_filter unit tests for every `AgentEvent` variant (12 tests)
2. (GREEN) Create `crates/kay-core/src/event_filter.rs` with match-all-variants for_model_context
3. (RED) Write T-4 property test — 10k random sequences never leak SandboxViolation
4. (GREEN) Property test must pass (should be by construction since filter matches on variant kind)
5. (RED) Write T-3 control unit tests — ControlMsg enum shape
6. (GREEN) Create `crates/kay-core/src/control.rs` — mpsc types + Ctrl-C handler installer (no active consumer yet — wired in Wave 3)
7. Measure 100% line + 100% branch coverage on event_filter via `cargo-llvm-cov`

**Exit:** event_filter 100% covered; control module compiles + unit tests green.

**Parallelizable with Wave 1 after step 5** (only needs AgentEvent variants Paused + Aborted from Wave 1).

### Wave 3 — Persona loader (LOOP-03)

**Goal:** YAML personas load + schema-validate; bundled forge/sage/muse YAMLs shipped.

**Tasks:**

1. (RED) Write T-2 unit tests for persona schema (5 cases)
2. (GREEN) Create `crates/kay-core/src/persona.rs` — serde struct + validate_against_registry()
3. Write `crates/kay-core/personas/forge.yaml`, `sage.yaml`, `muse.yaml` (content inherited from ForgeCode's catalog; CLEAN-ROOM NOTE: copy prompts from ForgeCode's public README + documented persona descriptions, not leaked-source copies)
4. (GREEN) `include_str!` bundling + `Persona::load(name)` API
5. (RED → GREEN) `Persona::from_path(p)` for external YAMLs
6. Snapshot-test each bundled persona's deserialized form

**Exit:** 3 bundled personas + external loader green.

**Parallelizable with Wave 2**.

### Wave 4 — Agent loop skeleton (LOOP-01, LOOP-05)

**Goal:** `run_turn` with 4-channel `tokio::select!`; task_complete gates on NoOpVerifier.

**Tasks:**

1. (RED) Write T-1 unit test `run_turn_single_turn_happy_path`
2. (GREEN) Create `crates/kay-core/src/loop.rs::run_turn` — biased select {control, input, tool, model}
3. (RED) Write T-1 integration `loop_plus_dispatcher_invokes_registered_tool`
4. (GREEN) Wire dispatcher + NoOpVerifier into loop
5. (RED) T-1 `task_complete_requires_verification` — assert loop does NOT return success on Pending outcome
6. (GREEN) Implement task_complete gate via verifier seam
7. (RED) T-1 property `select_robust_to_random_close_order`
8. (GREEN) Verify property holds — likely passes by construction; iterate if not
9. Wire ControlMsg consumption (LOOP-06 closes here): Pause → buffer, Resume → replay, Abort → cancel token

**Exit:** loop runs single-turn with mock provider, respects control channel.

**Parallelizable:** No (depends on Waves 1, 2, 3).

### Wave 5 — sage_query sub-tool (LOOP-04)

**Goal:** sage_query tool invokable from forge/muse with nesting_depth guard.

**Tasks:**

1. (RED) Write T-9 unit tests (5 cases)
2. (GREEN) Create `crates/kay-tools/src/builtins/sage_query.rs` — Tool impl
3. (GREEN) Thread `nesting_depth: u8` through `ToolCallContext`
4. (RED) T-9 integration `forge_calls_sage_via_sage_query_end_to_end`
5. (GREEN) Wire sage_query into default_tool_set for forge/muse; exclude from sage's bundled YAML
6. Regression: assert sage persona YAML does NOT list sage_query in tool_filter

**Exit:** forge + muse can call sage_query; sage cannot; depth ≥ 2 rejected.

**Parallelizable with Wave 4 after Wave 4 Step 4** (needs loop API stable).

### Wave 6 — Residuals R-1 + R-2 + trybuild infra

**Goal:** Close Phase 3 residuals; add compile-fail tier.

**Tasks:**

6a. R-1 (PTY tokenizer):
1. (RED) Write T-7 6 regression tests in `crates/kay-tools/tests/execute_commands_r1.rs`
2. (GREEN) Patch `kay-tools/src/builtins/execute_commands.rs::should_use_pty` to tokenize on `[\s;|&]`
3. Verify 6 tests green on all 3 OSes

6b. R-2 (image_read cap):
4. (RED) Write T-8 5 tests in `crates/kay-tools/tests/image_read_r2.rs`
5. (GREEN) Add `max_image_bytes: u64` to `ForgeConfig.image_read`; default 20 MiB
6. (GREEN) `ImageReadTool::new` reads cap; metadata-size-check before read; emit `ToolError::ImageTooLarge` on over-cap
7. Add wire serialization for `ImageTooLarge` variant

6c. trybuild infra:
8. Add `trybuild = "1"` to workspace dev-deps
9. Create `crates/kay-tools/tests/compile_fail.rs` runner
10. Write 3 compile-fail fixtures under `crates/kay-tools/tests/compile_fail/`
11. Run once to generate `.stderr` snapshots; commit

**Exit:** residuals closed + compile-fail tier enforces object-safety.

**Parallelizable with Waves 4 + 5** (completely independent code paths).

### Wave 7 — kay-cli binary + forge_main port (CLI-01/03/04/05/07)

**Goal:** `kay` binary with clap subcommands; `forge_main` entry-point-surface rebranded.

**Tasks:**

1. (RED) Write T-5 E2E tests in `crates/kay-cli/tests/cli_e2e.rs` (9 cases including parity diff)
2. (GREEN) Populate `crates/kay-cli/src/main.rs` — clap derives with `run` subcommand + interactive fallback
3. (GREEN) Wire `kay run` → `kay-core::loop::run_turn` with persona loader + wire-JSONL output
4. (GREEN) Port `forge_main::banner.rs` → `kay-cli::banner.rs` (brand swap only)
5. (GREEN) Port `forge_main::cli.rs` clap help text → kay-cli help strings
6. (GREEN) Port `forge_main::prompt.rs` prompt string: `forge>` → `kay>`
7. Retain `forge_main` crate as re-export shim (per E8 — kept through Phase 10)
8. Exit code mapping (CLI-03): 0/1/2/3/130 per T-5 matrix
9. Signal handler install — Ctrl-C → ControlMsg::Abort
10. Interactive fallback: `kay` (no args) → interactive mode preserving inherited ForgeCode UI
11. Parity fixture diff: interactive banner + prompt must match `forgecode-parity-baseline` tag except on brand-string lines (if fixture exists; else flag per open Q)

**Exit:** all 9 T-5 cases green on all 3 OSes (Ctrl-C platform-gated); `kay --help` has zero "forge" mentions.

**Parallelizable:** No (needs Waves 1-6 all green).

---

## Cross-wave infrastructure (lands early in Wave 1)

- Update `Cargo.toml` workspace to add `trybuild` dev-dep (Wave 6 uses it but declaring early is cheap)
- Add `crates/kay-core/src/lib.rs` module declarations: `pub mod r#loop;`, `pub mod persona;`, `pub mod control;`, `pub mod event_filter;`
- Update `crates/kay-cli/Cargo.toml` to add `kay-core`, `serde_yaml`, `serde_json`, `tokio`, `futures`, `tracing` deps

---

## Wave sequencing + dependency DAG

```
         Wave 1 (AgentEventWire + 2 new variants)
           │
    ┌──────┼──────┐
    │      │      │
  Wave 2 Wave 3  Wave 6 (R-1/R-2/trybuild — fully parallel)
  (ctl+  (persona)
  filter)
    │      │
    └──┬───┘
       │
     Wave 4 (loop skeleton) ──┐
       │                      │
     Wave 5 (sage_query) ─────┘
       │
     Wave 7 (kay-cli + forge_main port — the integration wave)
```

**Parallel waves:** {Wave 2, Wave 3, Wave 6} after Wave 1; {Wave 4, Wave 5} after Waves 2+3.

---

## Commit cadence

- 1 commit per RED (failing test) + 1 commit per GREEN (making it pass) + 1 optional REFACTOR commit
- Every commit DCO-signed (`Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`)
- Every commit passes `cargo test --workspace` locally before push (or is explicitly a RED commit with the failing test noted in the commit message)
- Each wave ends with a PR-compatible commit group; no mid-wave force-pushes

---

## Exit criteria (whole phase)

All checkboxes below green before `silver:validate` pass:

- [ ] 11 REQs mapped to test suites (LOOP-01..06, CLI-01/03/04/05/07)
- [ ] Residuals R-1 + R-2 closed with regression tests
- [ ] trybuild tier added and green
- [ ] QG-C4 enforced in event_filter at 100% coverage
- [ ] 16 AgentEvent wire snapshots committed
- [ ] `kay run --prompt …` works end-to-end on mock provider
- [ ] `kay --help` zero "forge" mentions (entry-point rebrand done)
- [ ] `forge_main` retained as re-export shim (Phase 10 deletes)
- [ ] All 3 CI OSes green
- [ ] `event_filter` coverage = 100% line + 100% branch

---

**Next step:** Step 2.7 `silver:validate` pre-build gate.
