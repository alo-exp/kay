# Phase 8 Testing Strategy — Multi-Perspective Verification (KIRA Critics)

**Date:** 2026-04-22
**Phase:** 8
**Requirements covered:** VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04
**TDD discipline:** RED commit (all tests fail) BEFORE GREEN commit (implementation) per wave — no skipping.

---

## Testing Pyramid

```
        ┌──────────────────────────────────┐
        │  T3: E2E (2 tests)               │  Full agent turn, real SQLite + mock OpenRouter
        │  slow, high confidence           │
        ├──────────────────────────────────┤
        │  T2: Integration (6 tests)       │  Cross-crate: re-work loop, cost ceiling
        │  medium speed, cross-crate       │
        ├──────────────────────────────────┤
        │  T1: Unit (~20 tests)            │  CriticResponse, events, verifier modes
        │  fast, isolated                  │
        └──────────────────────────────────┘
        + T4: Property (2 proptest suites) — parser never panics + retry invariants
```

---

## Tier 1 — Unit Tests (inline `#[cfg(test)]`)

### T1-01: CriticResponse JSON parsing (`crates/kay-verifier/src/critic.rs`)

| Test ID | Input | Expected | REQ |
|---------|-------|----------|-----|
| T1-01a | `{"verdict":"pass","reason":"tests pass"}` | `CriticResponse { verdict: Pass, reason: "tests pass" }` | VERIFY-01 |
| T1-01b | `{"verdict":"fail","reason":"missing edge case"}` | `CriticResponse { verdict: Fail, reason: "missing edge case" }` | VERIFY-01 |
| T1-01c | `{"verdict":"unknown","reason":"x"}` | `Err(ParseError)` — only "pass"/"fail" accepted | VERIFY-01 |
| T1-01d | `{"verdict":"pass"}` (missing `reason`) | `Err(ParseError)` — `reason` is required | VERIFY-01 |
| T1-01e | `{}` (empty object) | `Err(ParseError)` — both fields required | VERIFY-01 |
| T1-01f | `{"verdict":"pass","reason":"ok","extra":"field"}` | `Err(ParseError)` — `additionalProperties: false` | VERIFY-01 |
| T1-01g | `"just a string"` (not an object) | `Err(ParseError)` — must be object | VERIFY-01 |

**Note:** `CriticResponse` serde uses `#[serde(deny_unknown_fields)]` to enforce `additionalProperties: false`.

---

### T1-02: AgentEvent new variants (`crates/kay-tools/src/events.rs`)

| Test ID | What | Assertion | REQ |
|---------|------|-----------|-----|
| T1-02a | `AgentEvent::Verification` shape | Fields: `critic_role: String`, `verdict: String`, `reason: String`, `cost_usd: f64` | VERIFY-04 |
| T1-02b | `AgentEvent::VerifierDisabled` shape | Fields: `reason: String`, `cost_usd: f64` | VERIFY-03 |
| T1-02c | `Verification` serializes to JSON | `serde_json::to_value(&ev)` succeeds; all fields present | VERIFY-04 |
| T1-02d | `VerifierDisabled` serializes to JSON | `serde_json::to_value(&ev)` succeeds | VERIFY-03 |
| T1-02e | Existing variants unaffected | All pre-Phase-8 `AgentEvent` tests still pass | — |
| T1-02f | `AgentEvent` is NOT Clone | `std::marker::PhantomData<fn() -> AgentEvent>` compile check | — |

---

### T1-03: VerifierMode + VerifierConfig (`crates/kay-verifier/src/mode.rs`)

| Test ID | What | Assertion | REQ |
|---------|------|-----------|-----|
| T1-03a | `VerifierConfig::default()` values | `mode=Interactive, max_retries=3, cost_ceiling=1.0, model="openai/gpt-4o-mini"` | VERIFY-03 |
| T1-03b | `Interactive` mode selects 1 critic | `critics_for_mode(Interactive)` returns `[EndUser]` only | VERIFY-02 |
| T1-03c | `Benchmark` mode selects 3 critics | Returns `[TestEngineer, QAEngineer, EndUser]` | VERIFY-01 |
| T1-03d | `Disabled` mode returns empty | Returns `[]` | VERIFY-03 |

---

### T1-04: MultiPerspectiveVerifier core logic (`crates/kay-verifier/src/verifier.rs`)

Uses a `MockProvider` (in-process, returns canned responses) and a `Vec<AgentEvent>` collector as the stream sink.

**Important:** Sink is `Arc<Mutex<Vec<...>>>` — stores role+verdict+cost fields BEFORE moving the event. Since `AgentEvent` is non-Clone, extract needed fields before the move.

| Test ID | Setup | Expected outcome | REQ |
|---------|-------|-----------------|-----|
| T1-04a | All critics return PASS | `VerificationOutcome::Pass { note: "all critics passed" }` | VERIFY-01 |
| T1-04b | First critic FAIL, rest PASS | `VerificationOutcome::Fail { reason: combined }` | VERIFY-01 |
| T1-04c | All critics FAIL | `Fail` with all reasons combined | VERIFY-01 |
| T1-04d | `VerifierMode::Disabled` | `Pass` immediately; zero critic calls; no `Verification` events emitted | VERIFY-03 |
| T1-04e | All critics PASS → emits 1 `Verification` event per critic | Sink receives exactly N events (N = critic count per mode) | VERIFY-04 |
| T1-04f | `MultiPerspectiveVerifier::verify` NEVER returns `Pending` | Enumerate all code paths — `Pending` is unreachable | VERIFY-01 |
| T1-04g | Network error from MockProvider | Treats critic as PASS, logs warn | VERIFY-03 |
| T1-04h | Malformed JSON from MockProvider | Treats critic as PASS, logs warn | VERIFY-03 |

---

### T1-05: TaskVerifier dyn-safety compile test (`crates/kay-verifier/tests/compile_fail/`)

Uses `trybuild` (already a workspace dev-dep from Phase 5):

| Test ID | What | Assertion |
|---------|------|-----------|
| T1-05a | `Box<dyn TaskVerifier>` compiles | Object-safety preserved after signature expansion |
| T1-05b | `Arc<dyn TaskVerifier>` compiles | Same |
| T1-05c | `TaskVerifier` with `where Self: Sized` method | Compile FAIL if someone adds such a method |

---

### T1-06: `ToolCallContext::task_context` accumulation (`crates/kay-tools/src/runtime/context.rs`)

| Test ID | What | Assertion | REQ |
|---------|------|-----------|-----|
| T1-06a | Fresh context starts empty | `ctx.task_context.lock().is_empty()` | VERIFY-01 |
| T1-06b | Appending tool name | After append, contains "called tool: fs_read\n" | VERIFY-01 |
| T1-06c | Snapshot is independent | Mutating original after snapshot doesn't affect snapshot | VERIFY-01 |

---

### T1-07: `NoOpVerifier` updated signature

| Test ID | What | Assertion |
|---------|------|-----------|
| T1-07a | `NoOpVerifier.verify(summary, context)` | Still returns `Pending` (Phase 3 invariant preserved) |
| T1-07b | `context` param ignored | Same result with `context = ""` and `context = "big context"` |

---

### T1-08: `task_complete` tool with new verifier signature (`crates/kay-tools/src/builtins/task_complete.rs`)

| Test ID | What | Assertion | REQ |
|---------|------|-----------|-----|
| T1-08a | `task_complete` with `PassVerifier` stub | Emits `TaskComplete { verified: true, outcome: Pass }` | VERIFY-01 |
| T1-08b | `task_complete` with `FailVerifier` stub | Emits `TaskComplete { verified: false, outcome: Fail }` | VERIFY-01 |
| T1-08c | Snapshot of `task_context` passed to verifier | Verifier receives the current `task_context` string | VERIFY-01 |

---

## Tier 2 — Integration Tests (cross-crate, `crates/*/tests/`)

### T2-01: Re-work loop in `run_turn` (`crates/kay-core/tests/rework_loop.rs`)

Tests the outer retry wrapper in `run_turn` using a mock model that always calls `task_complete`.

| Test ID | Setup | Expected | REQ |
|---------|-------|----------|-----|
| T2-01a | Verifier: PASS on first call | Turn exits with `TurnResult::Verified`; no retry | VERIFY-01 |
| T2-01b | Verifier: FAIL once, then PASS | Turn exits `Verified` after 1 retry; feedback message visible in event stream | VERIFY-02 |
| T2-01c | Verifier: FAIL × max_retries | `VerifierDisabled { max_retries_exhausted }` emitted; turn returns `VerificationFailed` | VERIFY-02/03 |
| T2-01d | Verifier: FAIL × (max_retries - 1), then PASS | Exits `Verified`; exactly (max_retries - 1) retry messages in stream | VERIFY-02 |
| T2-01e | Feedback message format | User message contains `"Verification failed:"` + critic's reason string | VERIFY-02 |
| T2-01f | `max_retries = 0` | No retries allowed; single FAIL → immediate `VerificationFailed` | VERIFY-02 |

---

### T2-02: Cost ceiling breach (`crates/kay-verifier/tests/cost_ceiling.rs`)

| Test ID | Setup | Expected | REQ |
|---------|-------|----------|-----|
| T2-02a | Cost ceiling = $0.001; first critic costs $0.002 | After first critic: `VerifierDisabled { cost_ceiling_exceeded }` emitted; returns `Pass` | VERIFY-03 |
| T2-02b | Cost ceiling = $0.10; 3 critics each cost $0.03 | After critic 3 (cumulative $0.09 < ceiling): all 3 `Verification` events; `Pass` | VERIFY-03 |
| T2-02c | Cost ceiling = $0.10; critic 2 breaches ceiling ($0.06 cumulative) | `VerifierDisabled` after critic 2; critic 3 NOT called; returns `Pass` | VERIFY-03 |
| T2-02d | `verifier_cost` accumulator AND `cost_cap` both updated | Both counters show same cumulative value after critics | VERIFY-03 |
| T2-02e | After ceiling breach: zero additional critics | No more `Verification` events after `VerifierDisabled` | VERIFY-03 |

---

### T2-03: `Verification` event stream ordering (`crates/kay-verifier/tests/event_order.rs`)

| Test ID | What | Assertion | REQ |
|---------|------|-----------|-----|
| T2-03a | Benchmark mode: event ordering | `Verification(TestEngineer)` before `Verification(QAEngineer)` before `Verification(EndUser)` | VERIFY-04 |
| T2-03b | `TaskComplete` follows all `Verification` events | `Verification` events precede `TaskComplete` in stream | VERIFY-04 |

---

## Tier 3 — E2E Tests (full agent loop, `crates/kay-cli/tests/`)

Uses a mock `OpenRouterProvider` (in-process SSE server simulating OpenRouter responses) and real SQLite session store.

### T3-01: `verifier_e2e.rs` — full turn terminates on Pass

**Setup:** `VerifierMode::Benchmark`, mock provider returns 3 × PASS critic responses, then model calls `task_complete`.

**Assertions:**
- Turn exits with `TurnResult::Verified`
- Event stream contains exactly 3 `AgentEvent::Verification` events (one per critic)
- All verdicts are "pass"
- Final event is `TaskComplete { verified: true, outcome: Pass }`
- No `VerifierDisabled` event

**Requirements:** VERIFY-01, VERIFY-04

---

### T3-02: `verifier_e2e.rs` — rework loop with real session append

**Setup:** `VerifierMode::Interactive`, `max_retries: 1`. Mock provider: first `task_complete` → FAIL response, second → PASS response.

**Assertions:**
- Event stream contains 1 `Verification(fail)` event, then 1 `Verification(pass)` event
- User message injection visible in session transcript (via SQLite read-back)
- Final event is `TaskComplete { verified: true }`
- `rework_count` incremented exactly once (verify via session transcript message count)

**Requirements:** VERIFY-01, VERIFY-02, VERIFY-04

---

### T3-03 (backlog 999.6): `context_smoke.rs` — rename + Phase 8 behavioral tests

- Rename `crates/kay-cli/tests/context_e2e.rs` → `context_smoke.rs`
- Add module-level doc comment: "Compilation smoke checks for Phase 7 context DI seams. Behavioral E2E tests live in verifier_e2e.rs (Phase 8)."
- Move existing 2 compilation tests unchanged
- Add: `engine_wired_into_run_turn_with_real_verifier()` — verifies `KayContextEngine` is used with `MultiPerspectiveVerifier` in the same `RunTurnArgs`

---

## Tier 4 — Property Tests (`crates/kay-verifier/src/critic.rs` inline)

### T4-01: `CriticResponse` parser never panics

```rust
proptest! {
    #[test]
    fn critic_response_parser_never_panics(s in "\\PC*") {
        // Any arbitrary string → parse attempt → no panic
        let _ = CriticResponse::parse_from_str(&s);
    }
}
```

- 10,000 cases, default proptest config
- Verifies the parser is total (never panics on any input)
- `ParseError` is an acceptable return; panic is not

**Requirements:** VERIFY-01 robustness

---

### T4-02: Retry counter invariant

```rust
proptest! {
    #[test]
    fn rework_count_never_exceeds_max_retries(max in 0u32..=10u32) {
        // Build a mock verifier that always returns Fail
        // Run rework loop with max_retries = max
        // Assert: VerificationFailed returned after exactly max retries
        // Assert: rework_count == max
    }
}
```

**Requirements:** VERIFY-02 bounded-retries invariant

---

## Coverage Targets

| Crate | Target | Critical paths |
|-------|--------|---------------|
| `kay-verifier` | 90% line coverage | `verify()` all code paths, cost accumulation |
| `kay-tools/events.rs` (new variants) | 100% serialization | `Verification` + `VerifierDisabled` |
| `kay-tools/seams/verifier.rs` | 100% trait coverage | dyn-safe, all `VerificationOutcome` variants |
| `kay-core/loop.rs` (rework loop) | 100% branch coverage | Pass/Fail/MaxRetries branches |
| `kay-cli/run.rs` (verifier swap) | Smoke only | Constructor called with correct args |

---

## CI Cost Regression Gate (VERIFY-04 / VERIFY-03)

```
File: .planning/phases/08-multi-perspective-verification/cost-baseline.json
{
  "fixture_summary": "Fixed 5-sentence task summary + 3-tool transcript",
  "baseline_cost_usd": <measured after first green CI run>,
  "tolerance_pct": 30
}
```

**Cargo test `cost_regression_gate`:**
1. Run `MultiPerspectiveVerifier` in `Benchmark` mode with pinned fixture input
2. Read `verifier_cost` accumulator value
3. Assert: `actual_cost <= baseline * 1.30`
4. If baseline file missing: create it, don't fail

---

## Test File Layout

```
crates/
  kay-verifier/
    src/
      critic.rs         ← T1-01, T4-01 (proptest parser)
      mode.rs           ← T1-03
      verifier.rs       ← T1-04
    tests/
      cost_ceiling.rs   ← T2-02
      event_order.rs    ← T2-03
      compile_fail/
        dyn_safe.rs     ← T1-05a/b
        no_sized_methods.rs ← T1-05c
  kay-tools/
    src/
      events.rs         ← T1-02
      seams/verifier.rs ← T1-07
      builtins/task_complete.rs ← T1-08
      runtime/context.rs ← T1-06
  kay-core/
    tests/
      rework_loop.rs    ← T2-01, T4-02
  kay-cli/
    tests/
      verifier_e2e.rs   ← T3-01, T3-02
      context_smoke.rs  ← T3-03 (renamed from context_e2e.rs)
```

---

## TDD Wave → Test Mapping

| Wave | RED tests | GREEN impl |
|------|-----------|-----------|
| W-0 | (scaffold: fixtures, mock provider, test helpers) | crate skeleton |
| W-1 | T1-01a-g, T1-02a-f | `CriticResponse` parsing + `AgentEvent` variants |
| W-2 | T1-03a-d, T1-05a-c | `VerifierMode`, `VerifierConfig`, dyn-safety |
| W-3 | T1-04a-h, T1-07a-b | `MultiPerspectiveVerifier` core + `NoOpVerifier` update |
| W-4 | T1-06a-c, T1-08a-c | `ToolCallContext::task_context` + `task_complete` update |
| W-5 | T2-01a-f, T4-02 | Re-work loop in `run_turn` |
| W-6 | T2-02a-e, T2-03a-b | Cost ceiling + event ordering |
| W-7 | T3-01, T3-02, T3-03 | Kay-cli wiring + backlog 999.6/999.7 |

---

## Gaps in Existing Coverage (pre-Phase 8)

| Gap | Where | Priority |
|-----|-------|----------|
| `context_e2e.rs` is compilation-only, not behavioral | `kay-cli/tests/` | High — backlog 999.6 closes this |
| `NoOpVerifier.verify()` signature will need updating | `kay-tools` | Medium — existing tests still pass after sig expansion |
| No test for `run_turn` with `TaskComplete(Fail)` outcome | `kay-core` | High — T2-01 adds these |
| `SymbolKind::from_kind_str` unknown arm silently maps to `FileBoundary` | `kay-context` | Low — backlog 999.7 closes |
