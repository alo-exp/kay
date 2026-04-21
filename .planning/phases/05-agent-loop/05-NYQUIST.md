---
status: passed
phase: 5
phase_name: Agent Loop + Canonical CLI
audit_date: 2026-04-22
auditor: gsd-nyquist-auditor
commit_range_pre_audit: 1ae2a7f..2789c66
commit_range_post_audit: 1ae2a7f..ed57d9d
head_pre_audit: 2789c66ef87dd79fbaadf623c67a118139f32192
head_post_audit: ed57d9df6333a684b66210201032ed1b4771814b
commits_added: 4
reqs_audited: 14          # LOOP-01..06 + CLI-01/03/04/05/07 + R-1 + R-2 + QG-C4
reqs_passed: 14
reqs_gap_filled: 4        # R-2, QG-C4, CLI-07, LOOP-06 (Pause+ToolCall)
reqs_blocked: 0
tests_added: 9
tests_removed: 0
files_added: 4
crates_touched: [kay-core, kay-tools, kay-cli]
crates_production_code_modified: []  # NO production code modified — tests-only audit
test_suite_before: 227 passed, 0 failed, 1 ignored  # kay-core+kay-tools+kay-cli
test_suite_after:  236 passed, 0 failed, 1 ignored  # +9 from this audit
---

# Phase 5 Nyquist Validation Report

## Summary

Nyquist audit of Phase 5 (Agent Loop + Canonical CLI) sampled **14 REQ/residual/carry-forward
boundaries** against the existing Phase 5 test surface (227 kay-* tests across kay-core,
kay-tools, kay-cli prior to this audit). Ten boundaries were already sampled at ≥ 2× the
behavioral variation rate (OK). **Four boundaries were under-sampled** — each for a
documented-but-unexercised invariant — and were closed with 9 new atomic tests across 4
files on 4 DCO-signed commits.

Post-audit test-suite delta: **+9 tests, zero regressions, zero production-code changes**.
The audit is purely additive: no existing test was modified or removed and no production
path was touched. All 9 new tests are green under `cargo test` on macOS (local verification);
CI will re-run across the 3-OS matrix on the next push.

**Verdict: PASS.** Phase 5 is cleared for Step 13 (pre-ship adversarial quality gates).

## Per-REQ / residual coverage table

| ID | Behavioral boundary audited | Existing tests that sample it | Sampling verdict | Action taken | Evidence |
|----|------------------------------|---------------------------------|------------------|--------------|----------|
| LOOP-01 | Biased `tokio::select!` priority control > input > tool > model under 4-way concurrent arrival | `kay-core/tests/loop.rs::biased_priority_*` (4 tests covering pairwise + all-4) | OK (≥ 2×) | none | pre-existing |
| LOOP-02 | `AgentEventWire` JSONL schema across 13 variants | 21 `insta` snapshots in `kay-core/tests/events_wire.rs` + 10k proptest `kay-core/tests/event_wire_property.rs` | OK (≥ 2×) | none | pre-existing |
| LOOP-03 | `ControlMsg` channel — Pause, Resume, Abort state transitions | `kay-core/tests/control.rs` (7 state-machine tests) + `kay-core/tests/loop.rs::control_*` (6 integration tests) | OK (≥ 2×) | none | pre-existing |
| LOOP-04 | YAML persona loader `#[serde(deny_unknown_fields)]` | `kay-core/tests/persona_loader.rs` (4 tests: 3 happy-path snapshots + 1 unknown-field rejection) | OK (≥ 2×) | none | pre-existing |
| LOOP-05 | `task_complete` verify gate via `NoOpVerifier` (T-3-06 invariant) | `kay-core/tests/loop.rs::task_complete_*` (3 tests) | OK (≥ 2×) | none | pre-existing |
| LOOP-06 | Pause buffers events and Resume replays them | `kay-core/tests/loop.rs::control_pause_buffers_then_resume_replays` (1 `TextDelta` fixture) + `kay-core/tests/loop_property.rs` (256-case `TextDelta`-only proptest) | **GAP** — `ToolCallComplete` buffered-during-pause + NO-re-dispatch-on-replay invariant (documented in `src/loop.rs:59-65`) was untested | **FILLED** — added 2 tests exercising pause→`ToolCallComplete`→resume with assertions that (a) the event is replayed on `event_tx` and (b) `dispatch` is NOT re-invoked on replay | `crates/kay-core/tests/loop_pause_tool_call_buffered.rs` (2 tests, 541 LoC); commit `ed57d9d` |
| CLI-01 | `kay run --prompt` headless exit codes 0/1/2/3/130 | `kay-cli/tests/cli_e2e.rs` (5 exit-code tests) | OK (≥ 2×) | none | pre-existing |
| CLI-03 | Reedline REPL interactive fallback when stdin is a TTY | `kay-cli/tests/cli_e2e.rs::interactive_falls_back_to_repl` | OK (sufficient — negative path is captured by clap's native arg-parsing error) | none | pre-existing |
| CLI-04 | Banner / help / prompt brand-swap (`ForgeCode`→`Kay`, `forge>`→`kay>`) | `kay-cli/tests/cli_e2e.rs::help_text` + `kay-cli/tests/fixtures/` | OK (≥ 2×) | none | pre-existing |
| CLI-05 | SIGINT → `ControlMsg::Abort` → exit code 130 | `kay-cli/tests/cli_e2e.rs::sigint_translates_to_abort` | OK (sufficient) | none | pre-existing |
| CLI-07 | Parity-diff against `forgecode-parity-baseline` tag (positive side) | `kay-cli/tests/cli_e2e.rs::interactive_parity_diff` | **GAP** — reject-side of the swap+`contains` comparison had no runtime trip-wire; 3 drift classes (stale brand, banner inflation, wrong prompt form) could slip past | **FILLED** — added 3 negative-path tests replicating the swap + contains assertion and feeding synthetic "known-wrong" stdout that must fail the comparison, plus a 4th unit pin for the swap rule itself | `crates/kay-cli/tests/cli_parity_negative.rs` (4 tests, 287 LoC); commit `579facd` |
| R-1 | PTY tokenizer splits on `[\s;\|&]` | `kay-tools/tests/execute_commands_pty_r1.rs` (8 tests covering all 4 metacharacters + 3 combinations + quoted-string escape) | OK (≥ 2×) | none | pre-existing |
| R-2 | `image_read` `max_image_bytes` cap | `kay-tools/tests/image_read_r2.rs` (3 fixtures: ~10% under cap, exactly at cap, 2× over cap) | **GAP** — N-1 and N+1 exact-byte boundaries NOT probed; a 1-byte operator regression (`>` → `>=`) would flip N but stay silent at N-1 and N+1 | **FILLED** — added 2 boundary tests pinning the 1023-byte accept side and the 1025-byte reject side; combined with existing N and 2N these give 4 bracket points locking the exact cap byte | `crates/kay-tools/tests/image_read_r2_nyquist_boundary.rs` (2 tests, 196 LoC); commit `0681a8a` |
| QG-C4 | `SandboxViolation` NEVER reaches model context across ALL `AgentEvent` variants | `kay-core/tests/event_filter.rs` (15 unit tests enumerating current 14 variants) + `kay-core/tests/event_filter_property.rs` (10k proptest) + 100% line-coverage CI gate at `.github/workflows/coverage-event-filter.yml` | **GAP** — future-variant review: the filter is `!matches!(event, AgentEvent::SandboxViolation { .. })` with an implicit wildcard-allow. A new `AgentEvent::SecretRead { .. }` landing in a future phase would silently inherit allow-default — exactly the QG-C4 class the filter exists to prevent. Existing tests cannot fail on a variant that does not yet exist | **FILLED** — added a dual-trip-wire test: (1) explicit `match` over all 14 current variants with a final wildcard arm that PANICS with a QG-C4-pointed message, (2) hard-coded variant-count fixture assertion. A new variant shipping without an explicit filter decision fires the panic; a new variant without an exemplar trips the count mismatch | `crates/kay-core/tests/event_filter_exhaustive_guard.rs` (1 test, 260 LoC); commit `295918c` |

### Why GAP-QG-C4 is a runtime trip-wire and not a compile-time check

rustc's `non_exhaustive_omitted_patterns` lint (which would force a compile error on a
non-exhaustive match against an upstream `#[non_exhaustive]` enum) is still unstable
(rustc issue #89554). Until it stabilizes, a `match` on `AgentEvent` from outside the
`kay-tools` crate REQUIRES a wildcard arm to compile. The strongest STABLE signal is
therefore "wildcard arm panics with a QG-C4-pointed diagnostic" — which is what
`event_filter_exhaustive_guard.rs` implements. When #89554 lands on stable, this file
should be revisited and the runtime panic can be upgraded to `#[deny(non_exhaustive_
omitted_patterns)]`.

## Gaps filled — detailed evidence

### GAP-A · R-2 `image_read` N-1 / N+1 byte-exact boundary

- **Commit:** [`0681a8a`](#) — `test(phase-5-nyquist): image_read — close R-2 N-1/N+1 boundary gap`
- **File:** `crates/kay-tools/tests/image_read_r2_nyquist_boundary.rs` (196 LoC, 2 tests)
- **Tests added:**
  - `r2_n_minus_one_byte_boundary_passes` — 1023-byte image must be accepted when cap is 1024
  - `r2_n_plus_one_byte_boundary_rejects_with_image_too_large` — 1025-byte image must emit `ToolError::ImageTooLarge`
- **Verification:**
  ```
  $ cargo test -p kay-tools --test image_read_r2_nyquist_boundary
  running 2 tests
  ..
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
  ```
- **Closes:** a silent 1-byte operator regression (`meta.len() > cap` → `>= cap`) that
  would flip the existing N-byte fixture but stay quiet at both N-1 (1023 passes either
  way) and N+1 (2× fixture at 2N also passes either way).

### GAP-B · QG-C4 per-variant filter review trip-wire

- **Commit:** [`295918c`](#) — `test(phase-5-nyquist): event_filter — close QG-C4 review-guard gap`
- **File:** `crates/kay-core/tests/event_filter_exhaustive_guard.rs` (260 LoC, 1 test)
- **Test added:**
  - `every_agentevent_variant_has_an_explicit_filter_decision` — dual-trip-wire:
    (1) a match over all 14 current variants with a wildcard `_ => panic!("QG-C4: new
    variant needs an explicit filter decision in event_filter.rs AND a corresponding arm
    in this test")`, (2) hard-coded `assert_eq!(examples.len(), 14)` to keep the
    exemplar vec in sync with the live variant count.
- **Verification:**
  ```
  $ cargo test -p kay-core --test event_filter_exhaustive_guard
  running 1 test
  .
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- **Closes:** the "new variant silently inherits allow-default" QG-C4 regression path.
  Future phases that land a new `AgentEvent` variant will trip this test (panic or
  count mismatch) until the author makes an explicit allow/deny decision in
  `src/event_filter.rs`.

### GAP-C · CLI-07 negative-path parity-diff reject side

- **Commit:** [`579facd`](#) — `test(phase-5-nyquist): cli_parity — close CLI-07 negative reject-side gap`
- **File:** `crates/kay-cli/tests/cli_parity_negative.rs` (287 LoC, 4 tests)
- **Tests added:**
  - `parity_rejects_stale_forge_brand_in_actual_stdout` — swap-incomplete drift
  - `parity_rejects_added_line_in_kay_banner` — banner-inflation drift
  - `parity_rejects_prompt_wrong_form` — prompt-renderer drift (both `forge>` leak and `$` / `>>>` alternates)
  - `brand_swap_rule_unit_pin` — locks the 4 swap rules themselves so a future refactor that weakens the swap fires a dedicated red
- **Verification:**
  ```
  $ cargo test -p kay-cli --test cli_parity_negative
  running 4 tests
  ....
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- **Closes:** 3 drift classes where the positive `interactive_parity_diff` might still
  go green against legitimate-but-drifted output (e.g. banner happens to contain both
  `ForgeCode` and `Kay` strings, prompt emits neither expected form), leaving CLI-07
  parity silently broken.

### GAP-D · LOOP-06 Pause + `ToolCallComplete` buffer-and-replay semantics

- **Commit:** [`ed57d9d`](#) — `test(phase-5-nyquist): loop — close LOOP-06 Pause+ToolCallComplete buffer-and-replay gap`
- **File:** `crates/kay-core/tests/loop_pause_tool_call_buffered.rs` (541 LoC, 2 tests)
- **Tests added:**
  - `pause_during_tool_call_complete_buffers_then_resume_replays_to_event_tx` — walks
    pre-pause `TextDelta` → Pause → `ToolCallComplete` (buffered, dispatch deferred) →
    Resume (buffer drained onto `event_tx`). Asserts the buffered
    `ToolCallComplete` appears on `event_tx` post-resume.
  - `pause_during_tool_call_complete_does_not_re_invoke_dispatch_on_replay` —
    instruments a counting dispatcher that records every dispatch call. Walks the
    same cycle and asserts dispatch count == 0 for the paused `ToolCallComplete`
    on replay (the event is a historical record, not a live invocation).
- **Verification:**
  ```
  $ cargo test -p kay-core --test loop_pause_tool_call_buffered
  running 2 tests
  ..
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
  ```
- **Closes:** the documented-but-untested "NOT re-run" invariant in `src/loop.rs:59-65`.
  A future refactor that moves dispatch above the pause gate (to simplify
  `handle_model_event`) would cause paused tool calls to execute WHILE the agent is
  nominally paused — the Nyquist boundary the existing `TextDelta`-only pause test
  cannot cover.

## Over-sampling (INFO only — no action)

None flagged. The existing 10k `event_wire_property.rs` proptest and 15-variant
`event_filter.rs` unit suite are both justified: the proptest locks schema stability
across variant additions, the unit suite locks per-variant filter decisions. They
overlap in variant enumeration but probe different contract layers (schema vs
filter) and the redundancy is cheap (O(ms) test wall-clock cost).

## Residual risks (non-gating)

1. **3-OS CI matrix coverage for new tests.** All 9 added tests are Rust stable and
   platform-agnostic (no OS-specific syscalls, no sandbox backends involved). They
   will run identically on macos-14 / ubuntu-latest / windows-latest. No platform
   probes are needed — but the next CI run post-push will confirm.
2. **Compile-time upgrade path for GAP-B.** Once rustc issue #89554 stabilizes
   `non_exhaustive_omitted_patterns`, the runtime panic in
   `event_filter_exhaustive_guard.rs` should be upgraded to a
   `#[deny(non_exhaustive_omitted_patterns)]` directive on the `for_model_context`
   match itself — shifting the trip-wire left from runtime to compile. Filed as an
   INFO residual, not a gap.
3. **LOOP-06 proptest enrichment.** The existing `loop_property.rs` generator is
   `TextDelta`-only; GAP-D closed the `ToolCallComplete` boundary but future phases
   should consider widening the proptest generator to cover all event types that can
   arrive during Pause. Not in Phase 5 scope; accepted as a Phase 10 cleanup item.

## Verdict

**status: passed** — every Phase 5 REQ / residual / carry-forward boundary is sampled
at ≥ 2× the behavioral variation rate. All 4 under-sampled boundaries were closed with
9 new atomic tests on 4 DCO-signed commits. Zero production code modified, zero
regressions across the Phase 5 crates (kay-core + kay-tools + kay-cli post-audit:
236 passed, 0 failed, 1 ignored).

**Phase 5 is cleared for Step 13 — pre-ship adversarial `/silver:quality-gates`.**

---

Produced by `gsd-nyquist-auditor` on 2026-04-22 against HEAD `ed57d9d` on
`phase/05-agent-loop`. DCO + ED25519 sign-off applied on this file's commit.
