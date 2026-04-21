# Phase 5 Plan — Agent Loop + Canonical CLI

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop + Canonical CLI
> **Milestone:** v0.3.0
> **Branch:** `phase/05-agent-loop` (base `1ae2a7f`)
> **Mode:** autonomous (§10e); TDD strict (RED → GREEN → REFACTOR)
> **Skill:** `gsd-plan-phase` (inline executor in autonomous mode)
> **Upstream:** 05-BRAINSTORM.md, 05-TEST-STRATEGY.md, 05-IMPL-OUTLINE.md, 05-VALIDATION.md, 05-QUALITY-GATES.md, 05-CONTEXT.md, 05-DEPENDENCIES.md

---

## 1. Goal

Ship a headless `kay run --prompt <text>` execute-compose-stream-tool-verify-turnEnd cycle with:
- Frozen `AgentEvent` wire schema (13 variants; 16 insta snapshots locking JSONL form)
- YAML-declared forge/sage/muse personas (schema-validated at load)
- Pause/Resume/Abort control channel (Ctrl-C → cooperative abort w/ 2s grace)
- `forge_main` → `kay-cli` entry-point-surface rebrand (banner/prompt/help; internal modules deferred to Phase 10)
- QG-C4 carry-forward: `AgentEvent::SandboxViolation` never re-injected into model context (100%-line + 100%-branch coverage on `kay-core::event_filter`; CI-enforced SHIP BLOCK)
- Phase 3 residuals R-1 (PTY tokenizer) + R-2 (image_read max_bytes) closed with regression tests
- `trybuild` compile-fail tier enforcing object-safety invariants

Success = all 8 ROADMAP Phase 5 success criteria met + all 9 QUALITY-GATES dimensions pass adversarial review + ED25519-signed `v0.3.0` tag pushed.

---

## 2. Requirements mapping

| REQ-ID | Description | Implementation wave | Test suite | Test file |
| ------ | ----------- | ------------------- | ---------- | --------- |
| LOOP-01 | `tokio::select!` 4-channel biased loop (control > input > tool > model) | Wave 4 | T-1 | `crates/kay-core/tests/loop.rs` |
| LOOP-02 | `AgentEvent #[non_exhaustive]`; 13 variants; wire-stable | Wave 1 | T-6 | `crates/kay-tools/tests/events_wire_snapshots.rs` |
| LOOP-03 | YAML personas (forge/sage/muse) with schema validation | Wave 3 | T-2 | `crates/kay-core/tests/persona.rs` |
| LOOP-04 | `sage_query` as sub-tool with nesting_depth guard (max 2) | Wave 5 | T-9 | `crates/kay-tools/tests/sage_query.rs` |
| LOOP-05 | Mandatory verify gate before `task_complete` | Wave 4 | T-1 | `crates/kay-core/tests/loop.rs::task_complete_requires_verification` |
| LOOP-06 | Pause/Resume/Abort control channel | Wave 2 + Wave 4 | T-3 | `crates/kay-core/tests/control.rs` |
| CLI-01 | Headless `kay run --prompt <text>` | Wave 7 | T-5 | `crates/kay-cli/tests/cli_e2e.rs::headless_prompt_emits_events` |
| CLI-03 | Exit codes 0/1/2/3/130 | Wave 7 | T-5 | `crates/kay-cli/tests/cli_e2e.rs::exit_code_matrix` |
| CLI-04 | `forge_main` → `kay-cli` binary rebrand | Wave 7 | T-5 | `crates/kay-cli/tests/cli_e2e.rs::kay_help_no_forge_mentions` |
| CLI-05 | Structured JSONL event stream on stdout | Wave 1 + Wave 7 | T-6 + T-5 | snapshots + E2E |
| CLI-07 | Interactive fallback preserves ForgeCode UX parity | Wave 7 | T-5 | `crates/kay-cli/tests/cli_e2e.rs::interactive_parity_diff` |
| R-1    | `execute_commands::should_use_pty` tokenizes on `[\s;|&]` | Wave 6a | T-7 | `crates/kay-tools/tests/execute_commands_r1.rs` |
| R-2    | `image_read` enforces `max_image_bytes` cap | Wave 6b | T-8 | `crates/kay-tools/tests/image_read_r2.rs` |
| (infra)| trybuild compile-fail tier | Wave 6c | T-10 | `crates/kay-tools/tests/compile_fail.rs` |
| (carry)| QG-C4 event_filter enforcement | Wave 2 | T-4 | `crates/kay-core/tests/event_filter.rs` |

---

## 3. Task breakdown (71 atomic tasks)

Each task = 1 DCO-signed commit. RED = failing test added; GREEN = minimal impl; REFACTOR = cleanup with green tests. Every task specifies files, tests, REQ, dependencies.

### Wave 1 — AgentEvent wire layer (LOOP-02 · CLI-05)

**Goal:** 13-variant `AgentEvent` + `AgentEventWire` + 16 insta snapshots + CONTRACT-AgentEvent.md

**Pre-task commits:**
- **T1.0 INFRA-CARGO** — Update `crates/kay-core/Cargo.toml` with Phase 5 deps (per 05-DEPENDENCIES.md §3.4)
  - Files: `crates/kay-core/Cargo.toml`
  - Tests: `cargo check --workspace` green
  - Commit: `chore(kay-core): add Phase 5 runtime deps (tokio, serde_yml, async-trait, ...)`
  - REQ: infra
  - Depends: none

**TDD pairs:**
- **T1.1 RED** — Add 14 insta snapshot assertions for existing AgentEvent variants (all fail — no AgentEventWire yet)
  - Files: `crates/kay-tools/tests/events_wire_snapshots.rs` (new)
  - Tests: `snap_text_delta`, `snap_tool_call_start`, `snap_tool_call_delta`, `snap_tool_call_complete`, `snap_tool_call_malformed`, `snap_usage`, `snap_retry`, `snap_error`, `snap_tool_output`, `snap_task_complete`, `snap_image_read`, `snap_sandbox_violation` (12 tests)
  - Commit: `test(kay-tools): RED — assert JSONL wire snapshots for 12 existing AgentEvent variants`
  - REQ: LOOP-02, CLI-05
  - Depends: T1.0

- **T1.2 GREEN** — Create `AgentEventWire` newtype + `From<&AgentEvent>` + `serde::Serialize`
  - Files: `crates/kay-tools/src/events_wire.rs` (new), `crates/kay-tools/src/lib.rs` (module decl)
  - Tests: T1.1 tests green; `cargo insta review --accept` used to commit `.snap` files
  - Commit: `feat(kay-tools): GREEN — AgentEventWire newtype + From<&AgentEvent> mirror`
  - REQ: LOOP-02, CLI-05
  - Depends: T1.1

- **T1.3 RED** — Add insta snapshot for `Display` impl as single JSONL line (trailing newline)
  - Files: `crates/kay-tools/tests/events_wire_snapshots.rs` (add test)
  - Tests: `snap_jsonl_line_format`
  - Commit: `test(kay-tools): RED — AgentEventWire Display produces valid JSONL line`
  - REQ: CLI-05
  - Depends: T1.2

- **T1.4 GREEN** — `impl Display for AgentEventWire` → `serde_json::to_string(self)? + "\n"`
  - Files: `crates/kay-tools/src/events_wire.rs`
  - Tests: T1.3 green
  - Commit: `feat(kay-tools): GREEN — AgentEventWire Display emits JSONL line`
  - REQ: CLI-05
  - Depends: T1.3

- **T1.5 RED+GREEN** — Add `Paused` + `Aborted { reason: String }` variants to `AgentEvent` (DL-4)
  - Files: `crates/kay-tools/src/events.rs`
  - Tests: compile-passes + `snap_paused` + `snap_aborted_user_ctrl_c` + `snap_aborted_max_turns` + `snap_aborted_verifier_fail` + `snap_aborted_sandbox_violation_prop` (5 new snapshots, total now 17 — but IMPL-OUTLINE target is 16; decision: drop one redundant Aborted reason snapshot → keep 4 reasons + Paused = 5 new, 12 existing = 17 tests but TEST-STRATEGY budget of 16 snapshots kept by merging 2 "close-sibling" variants into one parametric test)

  **Clarification (locked now, supersedes earlier estimate):** final snapshot count = 17 tests in `events_wire_snapshots.rs`. IMPL-OUTLINE's "16" was an estimate; the accurate count after enumerating reason-tags is 17. Exit criteria updated accordingly.

  - Files (continued): `crates/kay-tools/src/events_wire.rs` (extend From impl)
  - Commit: `feat(kay-tools): add Paused + Aborted variants to AgentEvent (DL-4; variant count 11→13)`
  - REQ: LOOP-02, LOOP-06
  - Depends: T1.4

- **T1.6 DOCS** — Write `.planning/CONTRACT-AgentEvent.md` — human-readable JSONL wire schema
  - Files: `.planning/CONTRACT-AgentEvent.md` (new)
  - Tests: N/A (docs); verified by reviewer
  - Commit: `docs(contract): AgentEvent JSONL wire schema v1 for GUI/TUI consumers`
  - REQ: LOOP-02, CLI-05
  - Depends: T1.5

**Wave 1 exit:** 17 snapshots green on all 3 OSes; CONTRACT-AgentEvent.md committed; variant count = 13; 1 new module; `cargo test -p kay-tools` green.

### Wave 2 — Control channel + event filter (LOOP-06 + QG-C4)

**Parallelizable with Wave 3 + Wave 6 after Wave 1 T1.5.**

- **T2.1 RED** — Unit tests for `event_filter::for_model_context` across all 13 variants
  - Files: `crates/kay-core/tests/event_filter.rs` (new)
  - Tests: 13 tests `filter_allows_text_delta`, `filter_allows_tool_call_start`, ..., `filter_denies_sandbox_violation`, `filter_allows_paused`, `filter_allows_aborted`
  - Commit: `test(kay-core): RED — event_filter per-variant allow/deny across 13 AgentEvent cases`
  - REQ: LOOP-02 + QG-C4
  - Depends: T1.5

- **T2.2 GREEN** — Implement `event_filter::for_model_context(&AgentEvent) -> bool`
  - Files: `crates/kay-core/src/event_filter.rs` (new), `crates/kay-core/src/lib.rs` (module decl)
  - Tests: T2.1 green
  - Commit: `feat(kay-core): GREEN — event_filter::for_model_context denies SandboxViolation only (QG-C4)`
  - REQ: QG-C4 (LOOP-02)
  - Depends: T2.1

- **T2.3 RED** — Proptest: 10k random `AgentEvent` sequences — filter never leaks SandboxViolation
  - Files: `crates/kay-core/tests/event_filter_property.rs` (new)
  - Tests: `proptest! { model_context_filter_random_sequences_never_leak_sandbox_violation }` with Arbitrary impl for AgentEvent
  - Commit: `test(kay-core): RED — proptest event_filter never leaks SandboxViolation (10k cases)`
  - REQ: QG-C4
  - Depends: T2.2

- **T2.4 GREEN** — Property holds by construction (filter matches on variant discriminant). If proptest fails, iterate until green.
  - Files: `crates/kay-core/src/event_filter.rs` (minor adjustments if needed)
  - Tests: T2.3 green
  - Commit: `feat(kay-core): GREEN — proptest proves event_filter sandbox-leak-free`
  - REQ: QG-C4
  - Depends: T2.3

- **T2.5 CI-INFRA** — Add `coverage-event-filter` job to `.github/workflows/ci.yml` enforcing 100% line + 100% branch via `cargo-llvm-cov`
  - Files: `.github/workflows/ci.yml`, optionally `.llvm-cov.toml` if needed
  - Tests: CI job runs and passes (green on `phase/05-agent-loop` push)
  - Commit: `ci: coverage-event-filter job enforces 100%-line + 100%-branch on kay-core::event_filter (QG-C4)`
  - REQ: QG-C4 SHIP BLOCK
  - Depends: T2.4

- **T2.6 RED** — Unit tests for `ControlMsg` enum shape + mpsc wiring helpers
  - Files: `crates/kay-core/tests/control.rs` (new)
  - Tests: `control_msg_variant_shapes`, `control_channel_pair_split`, `control_abort_cooperative_grace`
  - Commit: `test(kay-core): RED — ControlMsg enum + mpsc pair API`
  - REQ: LOOP-06
  - Depends: T1.5

- **T2.7 GREEN** — Create `crates/kay-core/src/control.rs` — `ControlMsg { Pause, Resume, Abort }` + `control_channel() -> (Sender, Receiver)` + `install_ctrl_c_handler(tx: Sender<ControlMsg>) -> Result<()>`
  - Files: `crates/kay-core/src/control.rs`, `crates/kay-core/src/lib.rs` (module decl)
  - Tests: T2.6 green
  - Commit: `feat(kay-core): GREEN — ControlMsg + control_channel + install_ctrl_c_handler`
  - REQ: LOOP-06
  - Depends: T2.6

**Wave 2 exit:** event_filter at 100%-line + 100%-branch (verified in CI); control module compiles + unit-tests green; 2 new modules in kay-core.

### Wave 3 — Persona loader (LOOP-03)

**Parallelizable with Wave 2 + Wave 6 after Wave 1 T1.0.**

- **T3.1 RED** — 5 schema tests covering valid/invalid YAML shapes
  - Files: `crates/kay-core/tests/persona.rs` (new)
  - Tests: `persona_loads_valid_forge_yaml`, `persona_rejects_unknown_field`, `persona_rejects_missing_required_field`, `persona_rejects_bad_tool_filter_entry`, `persona_model_field_validates_against_allowlist`
  - Commit: `test(kay-core): RED — Persona YAML schema validation (5 cases)`
  - REQ: LOOP-03
  - Depends: T1.0

- **T3.2 GREEN** — Implement `Persona` struct + `Persona::validate_against_registry(&ToolRegistry)`
  - Files: `crates/kay-core/src/persona.rs` (new)
  - Tests: T3.1 green
  - Commit: `feat(kay-core): GREEN — Persona serde struct + schema validation (deny_unknown_fields)`
  - REQ: LOOP-03
  - Depends: T3.1

- **T3.3 DATA** — Write `crates/kay-core/personas/{forge,sage,muse}.yaml` bundled persona files (CLEAN-ROOM: prompts from public ForgeCode README + docs only)
  - Files: `crates/kay-core/personas/forge.yaml`, `crates/kay-core/personas/sage.yaml`, `crates/kay-core/personas/muse.yaml`
  - Tests: N/A (data); consumed in T3.5
  - Commit: `feat(kay-core): add bundled forge/sage/muse persona YAMLs (clean-room from public ForgeCode docs)`
  - REQ: LOOP-03
  - Depends: T3.2

- **T3.4 RED** — Test `Persona::load(name)` resolves bundled personas via `include_str!`
  - Files: `crates/kay-core/tests/persona.rs` (extend)
  - Tests: `load_forge_from_bundled`, `load_sage_from_bundled`, `load_muse_from_bundled`, `load_unknown_name_errors`
  - Commit: `test(kay-core): RED — Persona::load(bundled) for forge/sage/muse`
  - REQ: LOOP-03
  - Depends: T3.3

- **T3.5 GREEN** — `Persona::load(name: &str)` uses `include_str!` bundling + `serde_yml::from_str`
  - Files: `crates/kay-core/src/persona.rs`
  - Tests: T3.4 green
  - Commit: `feat(kay-core): GREEN — Persona::load bundles forge/sage/muse via include_str!`
  - REQ: LOOP-03
  - Depends: T3.4

- **T3.6 RED+GREEN** — `Persona::from_path(p)` external YAML loader + test against tempfile
  - Files: `crates/kay-core/src/persona.rs`, `crates/kay-core/tests/persona.rs`
  - Tests: `load_external_yaml_via_tempfile`, `load_external_yaml_rejects_bad_schema`
  - Commit: `feat(kay-core): Persona::from_path for external YAML (extension point for Phase 11+)`
  - REQ: LOOP-03
  - Depends: T3.5

- **T3.7 SNAPSHOT** — insta snapshot test for each bundled persona's deserialized form (3 snapshots)
  - Files: `crates/kay-core/tests/persona.rs` (extend)
  - Tests: `snap_forge_persona_deserialized`, `snap_sage_persona_deserialized`, `snap_muse_persona_deserialized`
  - Commit: `test(kay-core): insta snapshots lock deserialized form of bundled personas`
  - REQ: LOOP-03
  - Depends: T3.6

**Wave 3 exit:** 3 bundled personas load; external loader works; schema strict; 3 insta snapshots locking persona structure.

### Wave 4 — Agent loop skeleton (LOOP-01, LOOP-05, LOOP-06 integration)

**Depends on Waves 1, 2, 3 complete.**

- **T4.1 RED** — Unit test `run_turn_single_turn_happy_path` with mock provider + NoOpVerifier
  - Files: `crates/kay-core/tests/loop.rs` (new)
  - Tests: `run_turn_single_turn_happy_path`
  - Commit: `test(kay-core): RED — run_turn single-turn happy path with mock provider`
  - REQ: LOOP-01
  - Depends: T2.7, T3.5

- **T4.2 GREEN** — Create `crates/kay-core/src/loop.rs` with `run_turn` skeleton using biased `tokio::select!` over 4 channels
  - Files: `crates/kay-core/src/loop.rs` (new), `crates/kay-core/src/lib.rs` (declare `pub mod r#loop;`)
  - Tests: T4.1 green
  - Commit: `feat(kay-core): GREEN — run_turn with biased tokio::select! over control>input>tool>model`
  - REQ: LOOP-01
  - Depends: T4.1

- **T4.3 RED** — Integration test `loop_plus_dispatcher_invokes_registered_tool`
  - Files: `crates/kay-core/tests/loop_dispatcher_integration.rs` (new)
  - Tests: `loop_plus_dispatcher_invokes_registered_tool`
  - Commit: `test(kay-core): RED — loop + dispatcher tool invocation integration`
  - REQ: LOOP-01
  - Depends: T4.2

- **T4.4 GREEN** — Wire dispatcher into `run_turn` (call `kay_tools::runtime::dispatcher::dispatch` on model tool_call)
  - Files: `crates/kay-core/src/loop.rs`
  - Tests: T4.3 green
  - Commit: `feat(kay-core): GREEN — run_turn dispatches tool calls via kay_tools::runtime::dispatcher`
  - REQ: LOOP-01
  - Depends: T4.3

- **T4.5 RED** — Test `task_complete_requires_verification` — assert loop does NOT return success on `Verifier::Pending`
  - Files: `crates/kay-core/tests/loop.rs` (extend)
  - Tests: `task_complete_requires_verification`, `task_complete_on_verifier_pass`
  - Commit: `test(kay-core): RED — task_complete gated on Verifier::Pass (not Pending)`
  - REQ: LOOP-05
  - Depends: T4.4

- **T4.6 GREEN** — Add `Verifier` trait + `NoOpVerifier` default impl + wire into `run_turn`
  - Files: `crates/kay-core/src/loop.rs` (Verifier trait + NoOpVerifier), `crates/kay-core/src/lib.rs` (re-export)
  - Tests: T4.5 green
  - Commit: `feat(kay-core): GREEN — Verifier trait + NoOpVerifier; task_complete gate`
  - REQ: LOOP-05
  - Depends: T4.5

- **T4.7 RED** — Property test `select_robust_to_random_close_order` (proptest: 10k random channel-close orderings)
  - Files: `crates/kay-core/tests/loop_property.rs` (new)
  - Tests: proptest property
  - Commit: `test(kay-core): RED — proptest select! robust to random channel close order`
  - REQ: LOOP-01
  - Depends: T4.6

- **T4.8 GREEN** — Verify property holds (likely by construction; iterate if needed)
  - Files: `crates/kay-core/src/loop.rs` (minor adjustments if needed)
  - Tests: T4.7 green
  - Commit: `feat(kay-core): GREEN — select! property proven robust to close ordering`
  - REQ: LOOP-01
  - Depends: T4.7

- **T4.9 RED** — Test `control_pause_buffers_then_resume_replays` (LOOP-06 integration)
  - Files: `crates/kay-core/tests/loop.rs` (extend)
  - Tests: `control_pause_buffers_then_resume_replays`, `control_abort_emits_aborted_event_and_exits`, `control_ctrl_c_cooperative_then_force`
  - Commit: `test(kay-core): RED — control channel integration (pause/resume/abort DL-2 semantics)`
  - REQ: LOOP-06
  - Depends: T4.8

- **T4.10 GREEN** — Wire `ControlMsg::Pause/Resume/Abort` handling in `run_turn` per DL-2 buffer-and-replay semantics
  - Files: `crates/kay-core/src/loop.rs` (add `paused_buffer: VecDeque<AgentEvent>`, select-arm for control_rx)
  - Tests: T4.9 green
  - Commit: `feat(kay-core): GREEN — run_turn handles Pause (buffer) / Resume (replay) / Abort (cancel_token)`
  - REQ: LOOP-06
  - Depends: T4.9

- **T4.11 REFACTOR** — Extract shared select-arm helpers; add module docs
  - Files: `crates/kay-core/src/loop.rs`
  - Tests: all Wave 4 tests remain green
  - Commit: `refactor(kay-core): extract select-arm helpers + module docs for run_turn`
  - REQ: LOOP-01
  - Depends: T4.10

**Wave 4 exit:** run_turn executes single-turn with mock provider; control channel honored; task_complete gated; 8 integration tests + 1 property test green.

### Wave 5 — sage_query sub-tool (LOOP-04)

**Depends on Wave 4 T4.4 (dispatcher integration) + Wave 3 (personas).**

- **T5.1 RED** — 5 unit tests for sage_query invocation and nesting_depth guard
  - Files: `crates/kay-tools/tests/sage_query.rs` (new)
  - Tests: `sage_query_invokes_inner_agent`, `sage_query_threads_nesting_depth`, `sage_query_rejects_depth_gte_2`, `sage_query_emits_nested_events`, `sage_query_respects_parent_sandbox`
  - Commit: `test(kay-tools): RED — sage_query sub-tool unit coverage (5 cases)`
  - REQ: LOOP-04
  - Depends: T4.4

- **T5.2 GREEN** — Thread `nesting_depth: u8` through `ToolCallContext`
  - Files: `crates/kay-tools/src/runtime/dispatcher.rs` (`ToolCallContext` struct), `crates/kay-tools/src/lib.rs` (re-export)
  - Tests: T5.1 partial green (structural)
  - Commit: `feat(kay-tools): add nesting_depth u8 to ToolCallContext (sage_query recursion guard)`
  - REQ: LOOP-04
  - Depends: T5.1

- **T5.3 GREEN** — Implement `sage_query` tool — spawns inner `run_turn` with `nesting_depth + 1`
  - Files: `crates/kay-tools/src/builtins/sage_query.rs` (new), `crates/kay-tools/src/builtins/mod.rs` (re-export)
  - Tests: T5.1 all green
  - Commit: `feat(kay-tools): GREEN — sage_query sub-tool with nesting_depth ≤ 2 guard`
  - REQ: LOOP-04
  - Depends: T5.2

- **T5.4 RED** — Integration test `forge_calls_sage_via_sage_query_end_to_end`
  - Files: `crates/kay-tools/tests/sage_query_integration.rs` (new)
  - Tests: `forge_calls_sage_via_sage_query_end_to_end`
  - Commit: `test(kay-tools): RED — forge→sage_query E2E integration`
  - REQ: LOOP-04
  - Depends: T5.3

- **T5.5 GREEN** — Wire `sage_query` into `default_tool_set` for forge + muse; exclude from sage's tool_filter
  - Files: `crates/kay-tools/src/runtime/registry.rs` (or default_tool_set location), `crates/kay-core/personas/sage.yaml` (verify tool_filter excludes sage_query)
  - Tests: T5.4 green
  - Commit: `feat(kay-tools): wire sage_query into default_tool_set for forge+muse (sage excluded)`
  - REQ: LOOP-04
  - Depends: T5.4

- **T5.6 REGRESSION** — Assert sage's bundled YAML does NOT list sage_query in tool_filter
  - Files: `crates/kay-tools/tests/sage_query.rs` (extend)
  - Tests: `sage_persona_yaml_excludes_sage_query_from_tools`
  - Commit: `test(kay-tools): regression — sage persona YAML must not list sage_query in tool_filter`
  - REQ: LOOP-04
  - Depends: T5.5

**Wave 5 exit:** forge + muse invoke sage_query; sage cannot (regression-asserted); depth ≥ 2 rejected; 6 tests green.

### Wave 6 — Residuals R-1 + R-2 + trybuild infra

**Parallelizable with Waves 4 + 5 after Wave 1.**

#### Wave 6a — R-1 PTY tokenizer fix

- **T6a.1 RED** — 6 regression tests locking PTY tokenization behavior on `[\s;|&]`
  - Files: `crates/kay-tools/tests/execute_commands_r1.rs` (new)
  - Tests: `pty_needed_for_semicolon_separated_commands`, `pty_needed_for_pipe`, `pty_needed_for_ampersand_bg`, `pty_needed_for_multi_space_separator`, `pty_not_needed_for_simple_command`, `pty_needed_for_quoted_substring_containing_separator`
  - Commit: `test(kay-tools): RED — PTY tokenizer regression for R-1 (6 cases on [\\s;|&])`
  - REQ: R-1
  - Depends: T1.0

- **T6a.2 GREEN** — Patch `should_use_pty` to tokenize on `[\s;|&]` instead of substring match
  - Files: `crates/kay-tools/src/builtins/execute_commands.rs`
  - Tests: T6a.1 all green; existing execute_commands tests remain green (no regression)
  - Commit: `fix(kay-tools): GREEN — R-1 PTY tokenizer uses [\\s;|&] separators`
  - REQ: R-1
  - Depends: T6a.1

- **T6a.3 CI-VERIFY** — 3-OS CI green on the 6 new R-1 tests
  - Files: CI config untouched; verification step only
  - Tests: `cargo test -p kay-tools --test execute_commands_r1` green on macos-14/ubuntu-latest/windows-latest
  - Commit: (no commit — verification beat; rolled into T6a.2 merge)
  - REQ: R-1
  - Depends: T6a.2

#### Wave 6b — R-2 image_read cap

- **T6b.1 RED** — 5 tests covering `max_image_bytes` enforcement
  - Files: `crates/kay-tools/tests/image_read_r2.rs` (new)
  - Tests: `image_read_under_cap_succeeds`, `image_read_over_cap_returns_image_too_large`, `image_read_at_cap_boundary_succeeds`, `image_read_metadata_checked_before_read`, `image_read_default_cap_20mib`
  - Commit: `test(kay-tools): RED — image_read max_image_bytes cap (R-2; 5 cases)`
  - REQ: R-2
  - Depends: T1.0

- **T6b.2 GREEN** — Add `max_image_bytes: u64` (default 20 MiB) to `ForgeConfig::image_read`
  - Files: `crates/forge_config/src/...` (locate image_read config struct; add field via `#[serde(default = "default_max_image_bytes")]`)
  - Tests: existing forge_config tests remain green; serde-default test added
  - Commit: `feat(forge_config): add image_read.max_image_bytes (default 20 MiB) for R-2`
  - REQ: R-2
  - Depends: T6b.1

- **T6b.3 GREEN** — `ImageReadTool::invoke` reads cap; metadata-size check BEFORE file read; emit `ToolError::ImageTooLarge { path, actual_size, cap }` on over-cap
  - Files: `crates/kay-tools/src/builtins/image_read.rs`, `crates/kay-tools/src/errors.rs` (add ImageTooLarge variant to ToolError with `#[non_exhaustive]` preserved)
  - Tests: T6b.1 all green
  - Commit: `feat(kay-tools): GREEN — image_read enforces max_image_bytes via metadata-size-check (R-2)`
  - REQ: R-2
  - Depends: T6b.2

- **T6b.4 WIRE** — Add wire serialization for `ImageTooLarge` in AgentEventWire mirror + insta snapshot
  - Files: `crates/kay-tools/src/events_wire.rs`, `crates/kay-tools/tests/events_wire_snapshots.rs`
  - Tests: `snap_tool_error_image_too_large` (extends snapshot count 17 → 18)
  - Commit: `feat(kay-tools): wire serialization for ToolError::ImageTooLarge (R-2)`
  - REQ: R-2
  - Depends: T6b.3

#### Wave 6c — trybuild compile-fail tier

- **T6c.1 INFRA** — Create `crates/kay-tools/tests/compile_fail.rs` runner (`trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs")`)
  - Files: `crates/kay-tools/tests/compile_fail.rs` (new)
  - Tests: infrastructural; T6c.2 produces the fixtures it runs
  - Commit: `test(kay-tools): trybuild compile-fail test runner infrastructure`
  - REQ: infra
  - Depends: T1.0

- **T6c.2 FIXTURES** — Write 3 compile-fail fixtures locking object-safety invariants
  - Files: `crates/kay-tools/tests/compile_fail/tool_not_object_safe.rs`, `crates/kay-tools/tests/compile_fail/services_handle_not_object_safe.rs`, `crates/kay-tools/tests/compile_fail/default_tool_set_factory_closure_lifetime.rs`
  - Tests: runner picks them up; stderr snapshots auto-generated on first run
  - Commit: `test(kay-tools): 3 compile-fail fixtures lock object-safety invariants (Tool, ServicesHandle, default_tool_set)`
  - REQ: infra
  - Depends: T6c.1

- **T6c.3 STDERR-SNAPSHOT** — Run `TRYBUILD=overwrite cargo test --test compile_fail` to generate `.stderr` snapshots; commit them
  - Files: `crates/kay-tools/tests/compile_fail/*.stderr` (3 files, generated)
  - Tests: T6c.1 runner green
  - Commit: `test(kay-tools): lock trybuild .stderr snapshots (run once with TRYBUILD=overwrite)`
  - REQ: infra
  - Depends: T6c.2

**Wave 6 exit:** R-1 closed (6 tests green 3-OS); R-2 closed (5 tests green; wire snapshot added); trybuild tier green with 3 fixtures.

### Wave 7 — kay-cli binary + forge_main port (CLI-01/03/04/05/07)

**Depends on Waves 1-6 complete.**

#### Wave 7 pre-task commits

- **T7.0a TRACEABILITY** — Fix REQUIREMENTS.md traceability table (DL-6): add rows for CLI-04, CLI-05, CLI-07 linking each to Phase 5
  - Files: `.planning/REQUIREMENTS.md` (lines 242-249)
  - Tests: N/A (docs)
  - Commit: `docs(requirements): fix traceability table — add CLI-04, CLI-05, CLI-07 rows (DL-6)`
  - REQ: admin
  - Depends: none (pre-Wave-7)

- **T7.0b WORKSPACE-DEPS** — Add `assert_cmd` + `predicates` to `[workspace.dependencies]`
  - Files: `Cargo.toml` (root workspace)
  - Tests: `cargo check --workspace` green
  - Commit: `chore(workspace): add assert_cmd + predicates dev-deps for Phase 5 E2E tests`
  - REQ: infra
  - Depends: none

- **T7.0c CLI-CARGO** — Populate `crates/kay-cli/Cargo.toml` with Phase 5 runtime + dev deps (per 05-DEPENDENCIES.md §3.3)
  - Files: `crates/kay-cli/Cargo.toml`
  - Tests: `cargo check -p kay-cli` green (compiles against empty main.rs)
  - Commit: `chore(kay-cli): populate Cargo.toml with Phase 5 deps (kay-core, tokio, serde_yml, clap, assert_cmd, ...)`
  - REQ: CLI-04
  - Depends: T7.0b

#### Wave 7 TDD + port tasks

- **T7.1 RED** — 9 E2E tests in `crates/kay-cli/tests/cli_e2e.rs` (exit codes, JSONL stream, help strings, parity diff)
  - Files: `crates/kay-cli/tests/cli_e2e.rs` (new)
  - Tests: `headless_prompt_emits_events`, `exit_code_0_on_success`, `exit_code_1_on_max_turns`, `exit_code_2_on_sandbox_violation`, `exit_code_3_on_config_error`, `exit_code_130_on_sigint_nix` (`#[cfg(unix)]`), `kay_help_no_forge_mentions`, `kay_version_emits`, `interactive_parity_diff` (`#[cfg(not(windows))]` for SIGINT path)
  - Commit: `test(kay-cli): RED — 9 E2E test cases for kay-cli (exit codes + JSONL + help + parity)`
  - REQ: CLI-01, CLI-03, CLI-04, CLI-05, CLI-07
  - Depends: T7.0c

- **T7.2 GREEN** — Populate `crates/kay-cli/src/main.rs` — clap derives with `run` subcommand + interactive fallback
  - Files: `crates/kay-cli/src/main.rs` (new from stub)
  - Tests: T7.1 `headless_prompt_emits_events`, `kay_help_no_forge_mentions`, `kay_version_emits` green
  - Commit: `feat(kay-cli): GREEN — main.rs with clap derives for kay run + interactive fallback`
  - REQ: CLI-01, CLI-04
  - Depends: T7.1

- **T7.3 GREEN** — Wire `kay run --prompt <text>` to `kay-core::run_turn` with persona + wire JSONL output
  - Files: `crates/kay-cli/src/run.rs` (new), `crates/kay-cli/src/main.rs` (dispatch)
  - Tests: T7.1 `headless_prompt_emits_events` green
  - Commit: `feat(kay-cli): GREEN — kay run wires run_turn with persona loader + JSONL event stream`
  - REQ: CLI-01, CLI-05
  - Depends: T7.2

- **T7.4 PORT-BANNER** — Port `crates/forge_main/src/banner.rs` → `crates/kay-cli/src/banner.rs` with brand swap
  - Files: `crates/kay-cli/src/banner.rs` (new), `crates/kay-cli/src/main.rs` (invoke banner on interactive)
  - Tests: banner output contains "Kay" not "ForgeCode"/"forge"
  - Commit: `feat(kay-cli): port forge_main::banner with brand swap (forge→kay)`
  - REQ: CLI-04, CLI-07
  - Depends: T7.3

- **T7.5 PORT-CLI-HELP** — Port clap help strings from `forge_main::cli` to kay-cli; replace "forge" with "kay" (case-preserving)
  - Files: `crates/kay-cli/src/main.rs` (help strings on each subcommand)
  - Tests: T7.1 `kay_help_no_forge_mentions` green (zero "forge" in --help output)
  - Commit: `feat(kay-cli): port clap help strings from forge_main with brand swap`
  - REQ: CLI-04
  - Depends: T7.4

- **T7.6 PORT-PROMPT** — Port `forge_main::prompt` → `kay-cli::prompt`; replace `forge>` with `kay>`
  - Files: `crates/kay-cli/src/prompt.rs` (new)
  - Tests: prompt contains `kay>` not `forge>`
  - Commit: `feat(kay-cli): port prompt string forge> → kay>`
  - REQ: CLI-04, CLI-07
  - Depends: T7.5

- **T7.7 EXIT-CODES** — Map exit codes per CLI-03 (0/1/2/3/130)
  - Files: `crates/kay-cli/src/exit.rs` (new), `crates/kay-cli/src/main.rs` (final `process::exit(code)`)
  - Tests: T7.1 exit-code matrix tests green
  - Commit: `feat(kay-cli): exit code mapping 0/1/2/3/130 per CLI-03`
  - REQ: CLI-03
  - Depends: T7.6

- **T7.8 SIGNAL-HANDLER** — Install SIGINT handler at main → forwards ControlMsg::Abort to run_turn's control channel
  - Files: `crates/kay-cli/src/main.rs`
  - Tests: T7.1 `exit_code_130_on_sigint_nix` green on macOS+Linux (Windows gated)
  - Commit: `feat(kay-cli): SIGINT handler → ControlMsg::Abort with 2s grace → exit 130`
  - REQ: CLI-03, LOOP-06
  - Depends: T7.7

- **T7.9 INTERACTIVE** — Interactive mode: `kay` (no args) falls through to interactive fallback preserving ForgeCode UX parity
  - Files: `crates/kay-cli/src/main.rs` (subcommand-absent branch), `crates/kay-cli/src/interactive.rs` (new) — may delegate to existing forge_main modules via kay-cli dep on forge_main (CLEAN-ROOM NOTE: kay-cli does NOT import forge_main per DL-3; instead replicates minimal reedline integration here)
  - Tests: interactive startup emits Kay banner + kay> prompt
  - Commit: `feat(kay-cli): interactive fallback mode with ForgeCode-parity reedline integration`
  - REQ: CLI-07
  - Depends: T7.8

- **T7.10 FIXTURES** — Capture `tests/fixtures/forgecode-{banner,prompt}.txt` from `forgecode-parity-baseline` tag
  - Files: `tests/fixtures/forgecode-banner.txt`, `tests/fixtures/forgecode-prompt.txt`, `scripts/capture-parity-fixtures.sh` (new — documents the capture process)
  - Tests: N/A (fixture data)
  - Commit: `feat(test-fixtures): capture banner+prompt from forgecode-parity-baseline tag (DL-1)`
  - REQ: CLI-07
  - Depends: T7.9

- **T7.11 PARITY-DIFF** — T7.1 `interactive_parity_diff` uses fixtures to assert kay banner/prompt match forgecode-parity-baseline modulo brand-string swap
  - Files: `crates/kay-cli/tests/cli_e2e.rs` (parity test body)
  - Tests: T7.1 `interactive_parity_diff` green
  - Commit: `test(kay-cli): GREEN — interactive parity diff against forgecode-parity-baseline`
  - REQ: CLI-07
  - Depends: T7.10

- **T7.12 FORGE-MAIN-RETENTION** — Update `crates/forge_main/Cargo.toml` description to reflect Phase 10 deferral (DL-3); zero source edits
  - Files: `crates/forge_main/Cargo.toml` (description field only)
  - Tests: `cargo build -p forge_main` still green (binary `forge` still ships)
  - Commit: `chore(forge_main): update description — retained as-is through Phase 10 per DL-3 (no source changes)`
  - REQ: CLI-04 (retention)
  - Depends: T7.11

**Wave 7 exit:** all 9 T-5 tests green (Ctrl-C platform-gated); `kay --help` zero "forge" mentions; forge_main builds unchanged; parity diff green.

---

## 4. Verification steps (per-wave exit gates)

| Wave | Exit gate | Enforced by |
| ---- | --------- | ----------- |
| 1 | `cargo test -p kay-tools --test events_wire_snapshots` green on 3 OSes; 17 snapshots committed; CONTRACT-AgentEvent.md present | CI test job |
| 2 | `cargo llvm-cov -p kay-core --lib --branch` ≥ 100% line + 100% branch on event_filter module | CI coverage-event-filter job (SHIP BLOCK) |
| 3 | 3 bundled personas load; external loader green; 3 insta snapshots | CI test job |
| 4 | `cargo test -p kay-core --test loop` all 8 integration + 1 property tests green | CI test job |
| 5 | sage_query E2E green; regression sage-YAML excludes sage_query verified | CI test job |
| 6 | R-1 tokenizer 6 tests green 3-OS; R-2 5 tests green 3-OS; trybuild 3 fixtures green | CI test job |
| 7 | 9 T-5 E2E tests green (Ctrl-C platform-gated); `kay --help` grep for "forge" returns 0 lines; forge_main binary builds | CI test job + manual `kay --help | grep -c forge` == 0 |

**Final phase exit (required before ship):**
- [ ] All 8 ROADMAP Phase 5 success criteria met
- [ ] gsd-verify-work PASS 8/8 (or equivalent verifier report)
- [ ] silver:security PASS (Step 10)
- [ ] silver:quality-gates adversarial PASS (Step 13)
- [ ] ED25519-signed `v0.3.0` tag
- [ ] `event_filter` coverage ≥ 100% line + 100% branch in final CI run
- [ ] Zero BLOCK findings in any retroactive audit
- [ ] DCO on every commit (verified by CI dco job)

---

## 5. Risk mitigations (from BRAINSTORM + VALIDATION + QUALITY-GATES)

| Risk | Mitigation task |
| ---- | --------------- |
| QG-C4 breach (SandboxViolation re-injection) | T2.2 + T2.3 + T2.5 CI enforcement |
| Interactive regression vs ForgeCode | T7.10 + T7.11 parity diff |
| AgentEvent schema drift | T1.1-T1.5 insta snapshots + CONTRACT-AgentEvent.md |
| select! deadlocks | T4.7 + T4.8 property test |
| YAML persona injection | T3.1-T3.2 schema validation + deny_unknown_fields |
| Ctrl-C race conditions | T4.9 + T4.10 + T7.8 cooperative abort |
| Upstream ForgeCode divergence | forgecode-parity-baseline tag frozen (Phase 1 decision; honored here) |
| JSONL stdout DoS | Deferred per DL-5; placeholder only |
| sage_query recursion | T5.1-T5.3 nesting_depth guard |

All 9 risks have named owning tasks.

---

## 6. Deviation protocol

If a task's GREEN commit fails CI or a downstream task discovers an upstream defect:

1. **STOP** — do not cascade fixes into the current task's commit
2. **Root-cause** — identify the actual failing task (new RED test that exposes the defect)
3. **Deviate** — add a new task between existing tasks with clear prefix `TDEV-<wave>.<N>`
4. **Document** — append a row to 05-DEVIATIONS.md (create if missing) with: deviation-id, upstream-task, root-cause, corrective commit SHA
5. **Resume** — continue from the new task in sequence

No silent rewrites; no amended commits on hooks-failed commits (per project git safety protocol).

---

## 7. Parallelism execution hints (for gsd-autonomous)

After Wave 1 completes, the executor can dispatch the following sets in parallel:

- **Set A:** Wave 2 tasks T2.1-T2.7 (event_filter + control)
- **Set B:** Wave 3 tasks T3.1-T3.7 (persona loader)
- **Set C:** Wave 6 tasks T6a.1-T6c.3 (residuals + trybuild)

After Set {A, B} complete, Wave 4 tasks T4.1-T4.11 become available.
After Wave 4 T4.4 + Wave 3, Wave 5 tasks T5.1-T5.6 become available.
After ALL waves 1-6 complete, Wave 7 tasks T7.0a-T7.12 become available (serial within wave except T7.0a/b/c which are independent pre-commits).

Config `workflow.parallelization=true` already enabled (from config.json).

---

## 8. Plan-check verdict expected

This plan produces:
- **71 atomic tasks** organized into 7 waves
- **17 AgentEvent wire snapshots + 3 persona snapshots + 3 trybuild stderr snapshots = 23 snapshot artifacts**
- **~50-60 DCO-signed commits** (each task = 1 commit; some tasks combine RED+GREEN into 1)
- **~5000-7000 LOC** total across kay-core + kay-tools + kay-cli (estimate; actual depends on CLI interactive complexity)
- **3-OS CI matrix** with 1 new coverage-enforcement job

goal-backward audit: each of the 11 REQs + 2 residuals + QG-C4 carry-forward has at least one task owning it. No REQ orphaned. No task without a REQ (except pure infra T1.0/T7.0*/T2.5/T6c.*).

---

**Next step:** Step 7 `gsd-autonomous` execute → `05-EXECUTION-LOG.md` (implicit; commit history serves as primary log).

Executor mode: TDD strict per Step 7a superpowers:test-driven-development skill. Autonomous bypass-permissions per §10e. Parallel dispatch per §7.
