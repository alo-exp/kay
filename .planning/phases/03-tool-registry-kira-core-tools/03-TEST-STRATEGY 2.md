---
phase: 3
slug: tool-registry-kira-core-tools
created: 2026-04-21
status: draft
composed-by: /silver:feature FLOW 5a (testing-strategy)
reviewed-by: (pending FLOW 6 silver-quality-gates Testability dimension)
host-env: macOS 14+ arm64 (primary) + Linux CI + Windows best-effort
---

# Phase 3 — Test Strategy

> Rigorous test pyramid designed BEFORE plan revision (FLOW 9). Every test name below
> is a load-bearing obligation: FLOW 9 must map it to a specific PLAN task ID, and
> FLOW 10 TDD must drop the failing RED test before any GREEN impl ships.
>
> **Policy derived from user directive 2026-04-20:** 100% TDD, full pyramid (unit +
> integration + property + smoke + live E2E), macOS-native test-automation.

---

## 1. Pyramid Shape (quantitative targets)

```
                              ▲
                             ╱ ╲
                            ╱E2E╲                 2 tests   (macOS smoke + live harness check)
                           ╱─────╲
                          ╱ SMOKE ╲               2 tests   (CLI happy-path + PTY happy-path)
                         ╱─────────╲
                        ╱  PROPERTY ╲             3 suites  (10k+ iterations each)
                       ╱─────────────╲
                      ╱ INTEGRATION   ╲           18 tests  (seam-crossing, real fs/proc)
                     ╱─────────────────╲
                    ╱  TRYBUILD (UI)    ╲         2 fixtures (compile_fail for API fences)
                   ╱─────────────────────╲
                  ╱      UNIT TESTS        ╲      45 tests  (fast, focused, module-internal)
                 ╱───────────────────────────╲
```

**Totals:** 72 distinct test artifacts spanning 6 tiers. **Aggregate assertions (minimum):** 200+ individual assertions with `pretty_assertions` diffs.

**Coverage target on new code (crates/kay-tools):** ≥ 90% line coverage via `cargo-llvm-cov` (nightly-optional). Parity-delegation tools get lower line coverage but MUST hit 100% branch coverage on the delegation arm itself.

---

## 2. Tier-by-Tier Specification

### 2.1 Unit Tier *(45 tests, `cargo test -p kay-tools --lib`, budget < 15 s)*

Lives inline in `crates/kay-tools/src/**/*.rs` under `#[cfg(test)] mod tests {...}`. Zero external I/O; uses `tokio::test` for async and `pretty_assertions::assert_eq!` for diffs.

| # | Module path | Test name | Asserts | REQ | PLAN task |
|---|-------------|-----------|---------|-----|-----------|
| U-01 | `contract::tool` | `tool_trait_name_is_stable` | trait default `fn name()` matches registration | TOOL-01 | 3-01-01 |
| U-02 | `contract::tool` | `tool_trait_description_is_nonempty` | all 7 built-ins return non-empty desc | TOOL-01 | 3-02-01 |
| U-03 | `contract::context` | `context_frozen_shape` | serde_json roundtrip preserves all fields | TOOL-01 | 3-01-01 |
| U-04 | `contract::error` | `tool_error_variants_serializable` | all `ToolError` variants round-trip via Display | TOOL-01 | 3-02-04 |
| U-05 | `contract::output` | `tool_output_chunk_ordering` | `Chunk` variants ordered by monotonic seq | TOOL-06 | 3-03-02 |
| U-06 | `schema::hardening` | `hardening_sorts_required_keys` | `required` array is sorted lexicographically | TOOL-05 | 3-03-01 |
| U-07 | `schema::hardening` | `hardening_sets_additional_properties_false` | root `additionalProperties: false` | TOOL-05 | 3-03-01 |
| U-08 | `schema::hardening` | `hardening_flattens_all_of` | nested `allOf` flattened to parent | TOOL-05 | 3-03-01 |
| U-09 | `schema::hardening` | `hardening_strips_property_names` | `propertyNames` constraint removed | TOOL-05 | 3-03-01 |
| U-10 | `schema::hardening` | `hardening_nullable_to_any_of_null` | `nullable: true` → `anyOf: [..., {type:null}]` | TOOL-05 | 3-03-01 |
| U-11 | `schema::hardening` | `hardening_adds_truncation_reminder` | description ends with "(may be truncated...)" | TOOL-05 | 3-03-01 |
| U-12 | `schema::hardening` | `hardening_preserves_input_ordering_otherwise` | non-target keys preserved in order | TOOL-05 | 3-03-01 |
| U-13 | `schema::definitions` | `tool_definitions_emit_all_seven` | `tool_definitions().len() == 7` | TOOL-06 | 3-02-02 |
| U-14 | `schema::definitions` | `tool_definitions_json_schema_valid` | each definition validates against JSON Schema Draft 2020-12 | TOOL-06 | 3-02-02 |
| U-15 | `registry::registry` | `registry_get_by_name_returns_some` | round-trip registration + lookup | TOOL-01 | 3-02-01 |
| U-16 | `registry::registry` | `registry_get_missing_returns_none` | lookup of unregistered name → None | TOOL-01 | 3-02-01 |
| U-17 | `registry::registry` | `registry_is_immutable_after_build` | no `insert` method in public API (compile test) | TOOL-01 | 3-02-01 |
| U-18 | `registry::default_set` | `default_set_has_exactly_seven` | `default_tool_set(...).len() == 7` | TOOL-01 | 3-06-01 |
| U-19 | `registry::default_set` | `default_set_names_match_reference` | names match `{execute_commands, task_complete, image_read, fs_read, fs_write, fs_search, net_fetch}` | TOOL-01 | 3-06-01 |
| U-20 | `runtime::markers` | `marker_nonce_is_128_bit` | generated nonce is exactly 32 hex chars | SHELL-01 | 3-04-01 |
| U-21 | `runtime::markers` | `marker_sequence_increments` | seq counter is strictly monotonic | SHELL-01 | 3-04-01 |
| U-22 | `runtime::markers` | `scan_line_matches_valid_marker` | parses `__CMDEND_<hex32>_<seq>__EXITCODE=0` → `MarkerEnd{exit:0,seq:N}` | SHELL-01 | 3-04-01 |
| U-23 | `runtime::markers` | `scan_line_rejects_wrong_nonce` | same format with different nonce → `ScanResult::NotFound` | SHELL-05 | 3-04-02 |
| U-24 | `runtime::markers` | `scan_line_uses_constant_time_compare` | audits that `subtle::ConstantTimeEq::ct_eq` is invoked | SHELL-05 | 3-04-02 |
| U-25 | `runtime::markers` | `scan_line_rejects_missing_exitcode` | format missing `EXITCODE=` → NotFound | SHELL-01 | 3-04-01 |
| U-26 | `runtime::markers` | `scan_line_rejects_non_numeric_exitcode` | `EXITCODE=abc` → ParseError | SHELL-01 | 3-04-01 |
| U-27 | `runtime::pty` | `should_use_pty_denylist_hits` | `htop`, `vim`, `less`, `top`, `tmux`, `ssh` return true | SHELL-02 | 3-04-04 |
| U-28 | `runtime::pty` | `should_use_pty_respects_explicit_flag` | `tty: true` overrides denylist=false | SHELL-02 | 3-04-04 |
| U-29 | `runtime::pty` | `should_use_pty_default_false_for_plain_bash` | `echo hi` → false | SHELL-02 | 3-04-04 |
| U-30 | `runtime::timeout` | `timeout_cascade_sends_sigterm_first` | first signal is SIGTERM | SHELL-04 | 3-04-05 |
| U-31 | `runtime::timeout` | `timeout_grace_is_2_seconds` | grace interval matches const `GRACE` | SHELL-04 | 3-04-05 |
| U-32 | `runtime::stream` | `stream_emits_chunk_then_final` | Final chunk always last | SHELL-03 | 3-04-03 |
| U-33 | `seams::sandbox` | `noop_sandbox_allows_everything` | `NoOpSandbox::check(...)` returns Ok for all paths | (Phase 4 seam) | 3-01-01 |
| U-34 | `seams::verifier` | `noop_verifier_returns_pending` | `NoOpVerifier::verify(...).await == Pending` | TOOL-03 | 3-02-03 |
| U-35 | `seams::verifier` | `verification_outcome_variants_complete` | enum has `{Pending, Passed, Failed{reason}}` | TOOL-03 | 3-02-03 |
| U-36 | `builtins::execute_commands` | `execute_commands_input_schema_hardened` | schema has `additionalProperties: false` | TOOL-05 | 3-04-06 |
| U-37 | `builtins::execute_commands` | `execute_commands_name_constant` | `name()` returns `"execute_commands"` | TOOL-02 | 3-04-06 |
| U-38 | `builtins::task_complete` | `task_complete_input_schema_has_summary` | schema requires `summary` field | TOOL-03 | 3-02-03 |
| U-39 | `builtins::task_complete` | `task_complete_pending_by_default` | invoke returns `ToolOutput::VerifierPending` | TOOL-03 | 3-02-03 |
| U-40 | `builtins::image_read` | `image_read_quota_per_turn_enforced` | 3rd consecutive call same-turn → `ToolError::QuotaExceeded` | TOOL-04 | 3-05-01 |
| U-41 | `builtins::image_read` | `image_read_quota_per_session_enforced` | 21st call same-session → `QuotaExceeded` | TOOL-04 | 3-05-01 |
| U-42 | `builtins::image_read` | `image_read_missing_file_returns_io_error` | bad path → `ToolError::Io{..}` | TOOL-04 | 3-05-01 |
| U-43 | `builtins::image_read` | `image_read_mime_type_detected` | `.png` → `image/png`, `.jpg` → `image/jpeg` | TOOL-04 | 3-05-01 |
| U-44 | `builtins::fs_read` | `fs_read_delegates_to_forge_app` | mock factory invoked exactly once | (parity) | 3-05-02 |
| U-45 | `events` | `agent_event_additive_variants` | `ToolOutput` + `TaskComplete` present + preserves Phase 2 variants | (events) | 3-03-02 |

**Runtime gate:** `cargo test -p kay-tools --lib -- --test-threads=8` completes in < 15 s on M-series macOS. CI asserts this via a timeout.

---

### 2.2 Trybuild Compile-Fail Tier *(2 fixtures, `cargo test --test trybuild_ui`, budget < 20 s cold)*

Lives in `crates/kay-tools/tests/ui/`. Regression sentinels — if someone breaks object-safety or trait contract, the build-time assertion fires.

| # | Fixture | What it proves | REQ | PLAN |
|---|---------|---------------|-----|------|
| T-01 | `tests/ui/tool_not_object_safe.fail.rs` | Adding a generic method to `Tool` breaks dyn-compatibility | TOOL-01 | 3-01-01 |
| T-02 | `tests/ui/input_schema_wrong_return_type.fail.rs` | Returning `&schemars::Schema` instead of `Value` fails compile (locks E2 decision) | TOOL-01 | 3-01-01 |

---

### 2.3 Integration Tier *(18 tests, `cargo test -p kay-tools --tests`, budget < 3 min)*

Lives in `crates/kay-tools/tests/*.rs`. Spawns real processes, touches real filesystem via tempdir. Uses `forge_executor_fixture` (common test kit) for parity-delegation tests.

| # | File | Test name | Asserts | REQ | PLAN |
|---|------|-----------|---------|-----|------|
| I-01 | `registry_integration.rs` | `registry_full_roundtrip` | build → list → invoke each of 7 tools | TOOL-01 | 3-02-01 |
| I-02 | `registry_integration.rs` | `all_tools_emit_hardened_schema` | iterate registry, assert hardening invariants | TOOL-05, TOOL-06 | 3-03-01 |
| I-03 | `marker_streaming.rs` | `echo_hi_emits_chunk_before_final` | stdout "hi\n" frame arrives before `Final{exit:0}` | SHELL-03 | 3-04-03 |
| I-04 | `marker_streaming.rs` | `sleep_then_echo_streams_with_no_buffering` | chunk emits before 500ms elapsed | SHELL-03 | 3-04-03 |
| I-05 | `marker_race.rs` | `forged_marker_in_stdout_does_not_close_stream` | attacker prints fake marker → command still runs to real completion | SHELL-05 | 3-04-02 |
| I-06 | `marker_race.rs` | `real_marker_at_exact_byte_boundary_parses` | marker split across read() boundaries still parses | SHELL-01 | 3-04-01 |
| I-07 | `timeout_cascade.rs` | `sigterm_kills_cooperative_process` | `trap 'exit 0' TERM; sleep 10` dies fast | SHELL-04 | 3-04-05 |
| I-08 | `timeout_cascade.rs` | `sigkill_kills_stubborn_process` | `trap '' TERM; sleep 10` dies via SIGKILL after 2s grace | SHELL-04 | 3-04-05 |
| I-09 | `timeout_cascade.rs` | `zombie_children_reaped` | child-of-child reaped via wait loop | SHELL-04 | 3-04-05 |
| I-10 | `pty_integration.rs` | `pty_path_spawns_tty` | `sh -c 'tty'` returns `/dev/ttys*` under PTY path | SHELL-02 | 3-04-04 |
| I-11 | `pty_integration.rs` | `pty_path_preserves_color` | `echo -e '\x1b[31mX'` delivers raw bytes | SHELL-02 | 3-04-04 |
| I-12 | `image_quota.rs` | `quota_resets_on_new_turn` | after turn boundary, cap refreshes | TOOL-04 | 3-05-01 |
| I-13 | `image_quota.rs` | `image_512kb_png_base64_roundtrip` | write 512KB PNG, read, base64 decode, byte-equal | TOOL-04 | 3-05-01 |
| I-14 | `image_quota.rs` | `two_images_per_turn_both_succeed` | cap=2 allows exactly 2 reads | TOOL-04 | 3-05-01 |
| I-15 | `parity_delegation.rs` | `fs_read_parity_byte_identical` | same input to Kay vs `forge_app::ToolExecutor` → byte-identical output | (parity) | 3-05-02 |
| I-16 | `parity_delegation.rs` | `fs_write_fs_search_net_fetch_parity` | remaining 3 parity tools byte-identical | (parity) | 3-05-02 |
| I-17 | `execute_commands_e2e.rs` | `execute_commands_streams_and_exits_cleanly` | multi-line echo + exit 0 end-to-end | TOOL-02 | 3-04-06 |
| I-18 | `execute_commands_e2e.rs` | `execute_commands_nonzero_exit_reported` | `exit 42` surfaces as `ToolOutput::Final{exit:42}` | TOOL-02 | 3-04-06 |

**Runtime gate:** `cargo test -p kay-tools --tests -- --test-threads=2` completes in < 3 min on M-series macOS.

**Parallelism note:** `timeout_cascade.rs` MUST run with `--test-threads=1` (uses process groups); enforced via `#[serial_test::serial]` attribute.

---

### 2.4 Property Tier *(3 suites, `cargo test --test *_property`, budget < 60 s)*

Lives in `crates/kay-tools/tests/*_property.rs`. Uses `proptest = "1"`.

| # | Suite | Strategy | Cases | REQ | PLAN |
|---|-------|----------|-------|-----|------|
| P-01 | `schema_hardening_property.rs` | arbitrary JSON schema input | 1,024 | TOOL-05 | 3-03-01 |
| P-02 | `marker_forgery_property.rs` | adversarial stdout (random bytes interleaved with near-miss markers) | **10,000** | SHELL-05 | 3-04-02 |
| P-03 | `quota_arithmetic_property.rs` | random sequences of per-turn/per-session call counts | 4,096 | TOOL-04 | 3-05-01 |

**A1 risk closure:** P-02 runs 10k forgery attempts against `scan_line`. If any forgery closes the stream, test fails with the offending input minimized by proptest shrinking.

---

### 2.5 Smoke Tier *(2 scripts, `just smoke` or `make smoke`, budget < 30 s each)*

Lives in `scripts/smoke/`. Pure bash + `jq`. No Rust compilation beyond the CLI binary itself (which is already built by the time smoke runs).

| # | Script | What it proves | REQ | PLAN |
|---|--------|---------------|-----|------|
| S-01 | `scripts/smoke/phase3-cli.sh` | `kay-cli exec -- echo hi` streams JSONL `{event: "ToolOutput", chunk: {...}}` containing `"hi\n"` then `{event: "ToolOutput", chunk: {"Final": {"exit": 0}}}` in that order | TOOL-02 + SHELL-03 | 3-06-01 |
| S-02 | `scripts/smoke/phase3-pty.sh` | `kay-cli exec --tty -- sh -c 'tty'` emits output containing `/dev/ttys` | SHELL-02 | 3-06-01 |

**macOS specifics:**
- `expect(1)` pre-installed on macOS (BSD expect in `/usr/bin/expect`) — used only if smoke expands to interactive PTY drive; Phase 3 smoke is non-interactive.
- `jq` available via brew; CI uses actions/setup-jq. Local: `brew install jq`.
- `grep -E` portability: scripts use explicit `ERE` flag, avoid GNU-only extensions.

Smoke runs as the last step of `cargo build --release && ./scripts/smoke/phase3-cli.sh && ./scripts/smoke/phase3-pty.sh`, wall-clock budget 60 s combined.

---

### 2.6 Live-E2E Tier *(2 checks, pre-ship only, budget < 2 min each)*

Runs at FLOW 11 VERIFY gate, not per-commit. Covers the Phase 3 exit criteria end-to-end.

| # | Check | What it proves | Tooling | PLAN |
|---|-------|---------------|---------|------|
| E-01 | Phase 3 CLI real-run with Kay CLI built from source | User types `kay-cli exec -- cargo check`, sees streaming output, tool-call appears in JSONL event log | bash + watch | 3-06-01 |
| E-02 | Tool-calling smoke via OpenRouter stub (mock provider, no real network in CI; real in manual run) | Real OpenRouter-shaped request body has `required` sorted, `additionalProperties:false`, tool count = 7 | mockito + one-off manual run with real key | 3-06-01 |

**Why not TB 2.0 here?** TB 2.0 full eval is Phase 12. Phase 3 checks the *plumbing* (tools execute, stream, deliver hardened schemas). Phase 12 checks the *score*.

---

## 3. Per-REQ Closure Matrix

Every Phase 3 requirement has ≥ 2 tests across ≥ 2 tiers:

| REQ | Unit | Integration | Property | Smoke | Total |
|-----|-----:|-----------:|---------:|------:|-----:|
| TOOL-01 | 6 | 1 | — | — | 7 |
| TOOL-02 | 2 | 2 | — | 1 | 5 |
| TOOL-03 | 3 | — | — | — | 3 |
| TOOL-04 | 4 | 3 | 1 | — | 8 |
| TOOL-05 | 7 | 1 | 1 | — | 9 |
| TOOL-06 | 2 | 1 | — | — | 3 |
| SHELL-01 | 5 | 1 | — | — | 6 |
| SHELL-02 | 3 | 2 | — | 1 | 6 |
| SHELL-03 | 1 | 2 | — | 1 | 4 |
| SHELL-04 | 2 | 3 | — | — | 5 |
| SHELL-05 | 2 | 1 | 1 | — | 4 |
| **Total** | **37*** | **17*** | **3** | **3*** | **60+** |

*Some tests cover multiple REQs; total distinct tests ≥ 68.*

**Parity delegation** adds 3 additional tests (I-15, I-16, U-44) not tied to a specific new REQ but required by the parity invariant.

---

## 4. macOS-Native Tooling Matrix

| Tool | Version | Purpose | Install |
|------|---------|---------|---------|
| `cargo test` | Rust 1.85 stable | Unit + integration runner | rustup (host) |
| `cargo-nextest` | ≥ 0.9 (optional) | Faster runner, used by CI if installed | `cargo install cargo-nextest` |
| `cargo-llvm-cov` | ≥ 0.6 | Line/branch coverage (nightly-only feature) | `cargo install cargo-llvm-cov` + `rustup toolchain install nightly` |
| `cargo-fuzz` | latest (optional, deferred) | Adversarial marker input generation; Phase 3 optional | `cargo install cargo-fuzz` |
| `proptest` | 1.x | Property tests | workspace dep |
| `pretty_assertions` | 1.x | Rich diff output | workspace dep (Phase 2 already) |
| `tokio` with `test-util` feature | 1.x | `#[tokio::test]` | workspace dep |
| `serial_test` | 3.x | `#[serial]` for timeout tests | workspace dep (new) |
| `trybuild` | 1.x | Compile-fail UI tests | dev-dep (new) |
| `mockito` | 1.x | HTTP mock for E-02 | dev-dep (new) |
| `tempfile` | 3.x | Test fixtures | dev-dep (already) |
| `jq` | 1.7+ | Smoke-script JSONL parsing | `brew install jq` |
| `expect` | BSD (macOS stock) | PTY drive (deferred beyond Phase 3) | pre-installed |
| `just` (preferred) or `make` | latest | Test recipe runner | `brew install just` |

**New workspace dev-deps** (plan revision will add to root Cargo.toml):
```toml
[workspace.dependencies]
trybuild = "1"
serial_test = "3"
mockito = "1"
proptest = "1"
```

---

## 5. CI Invocation Matrix

| Job | Command | Runtime budget | Runs on |
|-----|---------|---------------:|---------|
| `test-unit` | `cargo test -p kay-tools --lib` | 15 s | every push |
| `test-integration` | `cargo test -p kay-tools --tests -- --test-threads=2` | 3 min | every push |
| `test-property` | `cargo test -p kay-tools --test *_property -- --nocapture` | 60 s | every push |
| `test-trybuild` | `cargo test -p kay-tools --test trybuild_ui` | 20 s | every push |
| `test-smoke` | `./scripts/smoke/phase3-cli.sh && ./scripts/smoke/phase3-pty.sh` | 60 s | every push (macOS only for PTY) |
| `test-coverage` | `cargo llvm-cov --workspace --fail-under-lines 90 -p kay-tools` | 2 min | nightly + pre-release |
| `test-parity-baseline` | `./scripts/parity/kay_vs_forge.sh` | 5 min | pre-merge |
| `clippy` | `cargo clippy -p kay-tools --all-targets -- -D warnings` | 30 s | every push |
| `cargo-deny` | `cargo deny check` | 10 s | every push |

**Matrix:** macOS-13 (x64), macOS-14 (arm64), ubuntu-22.04. Windows is best-effort — PTY integration (I-10, I-11) + timeout cascade (I-07..09) skipped under `#[cfg(windows)]` with a stub assertion that references the Phase 4 Job Objects TODO.

---

## 6. Runtime Budgets *(wall-clock, M2 MacBook Pro reference)*

| Tier | Budget | Fail threshold | Enforcement |
|------|-------:|----------------:|-------------|
| Unit | 15 s | 30 s | CI timeout |
| Trybuild | 20 s | 40 s | CI timeout |
| Integration | 3 min | 5 min | CI timeout |
| Property | 60 s | 2 min | proptest `config.cases` limit |
| Smoke | 60 s combined | 2 min | bash `timeout` |
| Live E2E | 2 min each | 4 min | manual / nightly workflow |
| Coverage | 2 min | 5 min | nightly |

**Per-commit feedback loop (developer inner loop):**
- `just check` runs unit + trybuild only → feedback in < 40 s.
- `just test` runs unit + integration + property + trybuild → feedback in < 4.5 min.
- `just smoke` runs cargo build + smoke → feedback in < 2 min (release build cached).

---

## 7. Gate Criteria for FLOW 6 (silver-quality-gates Design-Time)

For the **Testability** dimension of `silver:silver-quality-gates` to score ✅:

- [ ] Every new public API has ≥ 1 unit test
- [ ] Every new seam (Tool trait, Sandbox trait, TaskVerifier trait) has a compile-fail trybuild fixture proving the contract
- [ ] Every REQ-ID is closed by ≥ 2 tests across ≥ 2 tiers (see §3 matrix)
- [ ] Every MEDIUM-confidence assumption from §Product-Lens §6 has a named test (A1→P-02, A2→deferred criterion bench, A3→I-10+I-11, A4→I-07+I-08+I-09, A7→I-13)
- [ ] No production code uses `unimplemented!()`, `todo!()`, or `_at_planning_time_*` placeholders
- [ ] Smoke script executable + passes on a clean checkout on macOS
- [ ] Property tests have ≥ 1,024 cases (P-01, P-03) or ≥ 10,000 cases (P-02 adversarial)
- [ ] Parity tests (I-15, I-16) byte-equal fixture exists and passes
- [ ] Coverage target ≥ 90% line on `crates/kay-tools/src/**` new code
- [ ] Runtime budgets (§6) met on a reference macOS host

A single ❌ on any of the above blocks FLOW 6 from passing; plan revision (FLOW 9) must add the missing test before re-attempting.

---

## 8. FLOW 9 Plan-Revision Obligations

Each PLAN.md task MUST enumerate its test obligations in the `<acceptance_criteria>` block using the test IDs above. The plan revision pass will add rows like:

```
<acceptance_criteria>
  - U-20 (marker nonce 128-bit) passes
  - U-21 (seq monotonic) passes
  - U-22 (scan_line valid marker) passes
  - P-02 (10k forgery adversarial) passes
  - Coverage ≥ 95% on src/runtime/markers.rs
</acceptance_criteria>
```

**Traceability:** FLOW 9 plan-revision MUST include a traceability section at the bottom of each PLAN.md that reads the test IDs it is responsible for from this strategy doc. Missing tests are a plan-checker BLOCKER.

---

## 9. Risk-Weighted Hotspots (where to over-invest)

| Risk | Confidence | Over-invest? | How |
|------|-----------:|--------------|-----|
| A1 Marker entropy | High | Property test with 10k adversarial cases (P-02) | already scoped |
| A2 Timing side-channel | Medium | Criterion micro-bench (deferred — add as nice-to-have in §10, not blocker) | deferred |
| A3 Multi-OS PTY | High | Integration matrix macOS primary + Linux CI; Windows skipped | scoped |
| A4 Timeout cascade | Medium | 3 integration tests (I-07/08/09) covering cooperative, stubborn, zombie | scoped |
| A5 NoOpVerifier loop impact | Medium | Deferred to Phase 5 (agent loop owns that test) | deferred |
| A6 forge_app API stability | High | Byte-identical parity tests (I-15, I-16) | scoped |
| A7 Image base64 provider fit | Medium | I-13 with real 512KB PNG | scoped |
| A8 7-tool set sufficient | Medium | Deferred to Phase 12 TB 2.0 dry-run | deferred |

---

## 10. Deferred / Nice-to-have (capture list)

- Criterion timing-side-channel bench for A2 (`subtle::ConstantTimeEq`).
- `cargo-fuzz` adversarial marker input generator (requires nightly).
- Windows PTY test (requires ConPTY integration; Phase 4 prerequisite).
- Mutation testing via `cargo-mutants` (project-wide, not Phase-3-scoped).
- Full load test for image_quota under 10k concurrent turns (N/A until multi-session Phase 10).

---

## Cross-Reference

- §Product-Lens assumptions → `03-BRAINSTORM.md` §6 A1–A8
- §Engineering-Lens seam → `03-BRAINSTORM.md` §Engineering-Lens E7
- Validation map → `03-VALIDATION.md` (will be reconciled against this in plan-revision)
- Canonical flow contract → `.planning/CANONICAL-FLOW.md` §Test Pyramid Policy
- Workflow manifest → `.planning/WORKFLOW.md`

---

## Sign-off

- Author: /silver:feature FLOW 5a (testing-strategy)
- Reviewer (pending): `silver:silver-quality-gates` Testability dimension (FLOW 6)
- Approver (pending): `gsd-plan-checker` post plan-revision (FLOW 9 final loop)
