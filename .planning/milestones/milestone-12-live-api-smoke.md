---
id: "milestone-12-live-api-smoke"
title: "Milestone 12 — Live API Smoke Testing + Test Pyramid Completeness"
phase: "12"
status: "in_progress"
depends_on: ["11-cross-platform-release"]
created: "2026-04-25"
---

# Milestone 12 — Live API Smoke Testing + Test Pyramid Completeness

## 1. Goal

Establish a production-grade live API smoke testing pipeline for Kay using MiniMax-M2.7, and audit + fill any gaps in the existing test pyramid (unit / integration / E2E / live E2E) across all Kay crates. After this milestone:

- MiniMax-M2.7 API key is configured and wired into `kay run`
- Every Kay crate has tests at every appropriate layer of the test pyramid
- Live smoke tests run against the MiniMax API and verify end-to-end agent behavior
- EVAL-01a is ready to execute (Harbor + TB 2.0, MiniMax-M2.7 key sufficient)

## 2. Requirements Covered

- REQ-TEST-01: MiniMax-M2.7 key configured as `MINIMAX_API_KEY`; no key committed to repo
- REQ-TEST-02: `kay run --live` wires real MiniMax provider into the agent loop
- REQ-TEST-03: Smoke test suite covers all major agent scenarios
- REQ-TEST-04: Integration tests cover all Kay crate pub APIs
- REQ-TEST-05: E2E subprocess tests cover CLI surface
- REQ-TEST-06: Live E2E tests cover real API round-trips
- REQ-TEST-07: EVAL-01a parity baseline ready (≥80% TB 2.0)

## 3. Test Pyramid Audit (Current State)

### 3.1 Existing Coverage

| Crate | Unit (#[cfg(test)]) | Integration (tests/) | E2E Subprocess | Live API |
|-------|---------------------|----------------------|----------------|---------|
| **kay-cli** | 4 (context_smoke, exit, banner, run args) | 4 files (cli_e2e, cli_parity_negative, session_e2e, context_smoke) + **live_smoke.rs (6 tests)** | YES (cli_e2e: 6 tests) | **YES** (`live_smoke.rs` — feature-gated `live` feature) |
| **kay-core** | 0 inline | 11 files (loop, control, persona, budget, etc.) | NO | **MISSING** |
| **kay-context** | 0 inline | 7 files (store, indexer, retriever_*, watcher, hardener, budget) | NO | **MISSING** |
| **kay-provider-openrouter** | 12 (auth, retry, cost_cap) | 8 files (auth, retry, allowlist, translator, cost_cap, nn7) | NO | **MISSING** |
| **kay-tools** | 0 inline | 0 files | NO | **MISSING** |
| **kay-verifier** | 0 inline | 0 files | NO | **MISSING** |
| **kay-session** | 0 inline | 0 files | NO | **MISSING** |
| **kay-sandbox-linux** | 1 | 0 files | NO | **MISSING** |
| **kay-sandbox-macos** | 1 | 0 files | NO | **MISSING** |
| **kay-sandbox-windows** | 1 | 0 files | NO | **MISSING** |
| **kay-sandbox-policy** | 1 | 0 files | NO | **MISSING** |

### 3.2 Gaps Identified

1. **kay-core**: 11 integration test files exist but ZERO inline `#[cfg(test)]` unit tests
2. **kay-context**: 7 integration test files exist but ZERO inline unit tests
3. **kay-provider-openrouter**: 8 integration tests + 12 unit tests — GOOD but no live API tests
4. **kay-tools**: NO tests at all
5. **kay-verifier**: NO tests at all
6. **kay-session**: NO tests at all
7. **kay-sandbox-***: Only inline unit tests; no `tests/` integration directories
8. **Live E2E**: ~~No test anywhere that makes a real API call to MiniMax~~ ✅ kay-cli `live_smoke.rs` + kay-provider-openrouter `minimax_live.rs`
9. **EVAL-01a**: TB 2.0 harness not wired into `kay eval tb2 --run` (Phase 12 deferred)

## 4. Task Breakdown

### M12-Task 1: Configure MiniMax API Key

**File changed:** `.env.example` (create)

**Action:**
```
Create .env.example in project root with:
  # MiniMax API key for live smoke testing and EVAL-01a
  # Get your key at: https://platform.minimax.io
  MINIMAX_API_KEY=your_key_here

Do NOT commit real keys. Use:
  export MINIMAX_API_KEY=$(cat ~/.minimax_api_key 2>/dev/null || echo "")
```

**Verify:** `.env.example` exists, `MINIMAX_API_KEY` is documented, no real key appears in repo.

**Commit:** `feat(kay): add .env.example with MINIMAX_API_KEY documentation`

---

### M12-Task 2: Audit Phase 09.1 Test Strategy Gaps Against Current State

**File changed:** `.planning/milestones/milestone-12-audit.md` (create)

**Action:**
Document all gaps between Phase 09.1 test strategy and current state. Specifically:
- Which gap-list crates from 09.1 PLAN still lack `tests/` directories
- Which crates from the 09.1 plan no longer exist or have been renamed
- Kay-specific gaps (kay-tools, kay-verifier, kay-session not in 09.1 plan)

**Verify:** Audit doc exists with all gaps listed.

---

### M12-Task 3: Add Inline Unit Tests for kay-core

**Files changed:** `crates/kay-core/src/lib.rs` + new `tests/` dir

**Action:**
Add `#[cfg(test)]` modules to kay-core source files for core logic:
- `src/r#loop.rs` — unit tests for loop state machine transitions
- `src/control.rs` — unit tests for control channel logic
- `src/persona.rs` — unit tests for persona loading/parsing

Create `crates/kay-core/tests/unit/` directory with unit tests:
- `loop_state.rs` — unit tests for RunTurnState enum transitions
- `control_channel.rs` — unit tests for channel capacity
- `persona_parsing.rs` — unit tests for persona YAML parsing

**Verify:** `cargo test -p kay-core` passes with new unit tests.

---

### M12-Task 4: Add Inline Unit Tests for kay-context

**Files changed:** `crates/kay-context/src/lib.rs`

**Action:**
Add `#[cfg(test)]` modules to kay-context source files:
- `src/engine.rs` — unit tests for NoOpContextEngine
- `src/budget.rs` — unit tests for ContextBudget limits
- `src/store.rs` — unit tests for store initialization

**Verify:** `cargo test -p kay-context` passes.

---

### M12-Task 5: Add tests/ Directories for kay-tools, kay-verifier, kay-session

**Files changed:** `crates/kay-tools/tests/`, `crates/kay-verifier/tests/`, `crates/kay-session/tests/`

**Action:**

```
crates/kay-tools/tests/
  tool_registry.rs    — test ToolRegistry::new(), default_tool_set()
  events_wire.rs      — snapshot tests for AgentEventWire serialization
  tool_call_context.rs — test ToolCallContext construction

crates/kay-verifier/tests/
  config.rs           — test VerifierConfig::default(), VerifierMode variants
  outcome.rs          — test VerificationOutcome serialization

crates/kay-session/tests/
  session_store.rs    — test SessionStore::open on temp dir
  session_index.rs    — test create_session, resume_session
```

Add `[[test]]` entries to each Cargo.toml.

**Verify:** All three crates compile with new tests; `cargo test -p kay-tools -p kay-verifier -p kay-session` passes.

---

### M12-Task 6: Wire Live MiniMax Provider into kay run

**Files changed:** `crates/kay-cli/src/run.rs`, `crates/kay-cli/src/live.rs` (new)

**Action:**

Create `crates/kay-cli/src/live.rs`:
```
- MiniMaxProvider: wraps forge_repo's minimax provider
- Uses MINIMAX_API_KEY env var
- Implements same event emission as offline_provider
- Emits: TextDelta (streaming), ToolCallComplete (if tools wired), TaskComplete
```

Update `crates/kay-cli/src/run.rs`:
- Add `--live` flag to `RunArgs` (default: false for offline)
- Add `--model` flag (default: "MiniMax-M2.7")
- When `--live`: spawn `live_provider` instead of `offline_provider`
- When `--live`: require `MINIMAX_API_KEY` env var; fail early with clear error if missing

Add test sentinel support to `live.rs`:
- `TEST:done` → `TaskComplete { verified: true, Pass }` (for smoke test)
- `TEST:error` → `Error` event (for error handling test)
- Non-sentinel → real MiniMax API call

**Verify:**
```
# Offline still works
cargo run -p kay-cli -- run --prompt "TEST:done" --offline | jq '.type'
# Live smoke with MiniMax
MINIMAX_API_KEY=<key> cargo run -p kay-cli -- run --prompt "Say hello" --live --model MiniMax-M2.7 | jq '.type'
```

---

### M12-Task 7: Live Smoke Test Suite

**Files changed:** `crates/kay-cli/tests/live_smoke.rs` (new)

**Action:**

Create `crates/kay-cli/tests/live_smoke.rs` with smoke tests that call MiniMax live API:

```
Test suite (run only with MINIMAX_API_KEY set):
  smoke_minimax_live()     — kay run --live --prompt "TEST:done" → TaskComplete event
  smoke_minimax_echo()     — kay run --live --prompt "What is 2+2?" → TextDelta contains answer
  smoke_minimax_tool_call() — kay run --live --prompt "Read the file Cargo.toml" (if tools wired)
  smoke_error_on_missing_key() — kay run --live without MINIMAX_API_KEY → clear error

Conditionally compiled: #[cfg(feature = "live-api-tests")]
Run with: cargo test --features live-api-tests
```

Add to `crates/kay-cli/Cargo.toml`:
```
[features]
default = []
live-api-tests = []

[dev-dependencies]
mockito = "1"  # For mocking other providers if needed
```

**Verify:** Tests compile; live tests pass with valid `MINIMAX_API_KEY`.

---

### M12-Task 8: kay-provider-openrouter Integration Test Suite

**Files changed:** `crates/kay-provider-openrouter/tests/live_provider.rs` (new)

**Action:**

Create `crates/kay-provider-openrouter/tests/live_provider.rs`:
```
live_minimax_chat()      — Real MiniMax API call; verify stream events emitted
live_cost_accumulation() — Verify CostCap accumulates usage from MiniMax response
live_allowlist_rejects() — Verify non-allowlisted model is rejected pre-HTTP
live_retry_on_429()      — Verify retry logic fires on rate limit (MiniMax may not rate-limit; use mockito fallback)
```

Conditionally compiled: `#[cfg(feature = "live-api-tests")]`

**Verify:** `cargo test -p kay-provider-openrouter --features live-api-tests` passes.

---

### M12-Task 9: Update EVAL-01a Documentation

**Files changed:** `.planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md`

**Action:**
Update PARITY-DEFERRED.md to reflect:
- MiniMax-M2.7 key configured as `MINIMAX_API_KEY`
- TB 2.0 setup exists in separate project under `Projects/`
- Docker container already configured for TB 2.0 workload
- EVAL-01a can run with:
  ```
  export MINIMAX_API_KEY=<key>
  cd <tb2-project-dir>
  harbor run -d terminal-bench/terminal-bench-2 -m minimax/MiniMax-M2.7 -n 89
  ```
- Archive results to `.planning/phases/01-fork-governance-infrastructure/parity-baseline/`

**Verify:** PARITY-DEFERRED.md updated with correct key env var and setup instructions.

---

### M12-Task 10: kay-sandbox-*/tests/ Integration Directories

**Files changed:** `crates/kay-sandbox-{linux,macos,windows}/tests/escape.rs`

**Action:**

Create `tests/` directories for each sandbox crate (Phase 09.1 W-4 planned this):
```
kay-sandbox-linux/tests/escape.rs
kay-sandbox-macos/tests/escape.rs
kay-sandbox-windows/tests/escape.rs
```

Each tests:
- Sandbox denies write outside project root
- Sandbox allows write inside allowed directory
- Real `std::process::Command` subprocess execution

**Verify:** Each test compiles on its respective OS; existing inline `#[cfg(test)]` continue to pass.

---

## 5. Verification Steps

1. `cargo test -p kay-core` — kay-core inline unit tests pass
2. `cargo test -p kay-context` — kay-context inline unit tests pass
3. `cargo test -p kay-tools` — kay-tools integration tests pass
4. `cargo test -p kay-verifier` — kay-verifier integration tests pass
5. `cargo test -p kay-session` — kay-session integration tests pass
6. `cargo test -p kay-sandbox-linux` — linux sandbox tests pass
7. `cargo test -p kay-sandbox-macos` — macos sandbox tests pass
8. `cargo test -p kay-sandbox-windows` — windows sandbox tests pass
9. `cargo test -p kay-provider-openrouter` — openrouter tests pass (unit + integration)
10. `cargo test -p kay-cli` — kay-cli E2E subprocess tests pass
11. `MINIMAX_API_KEY=<key> cargo test -p kay-cli --features live-api-tests` — live smoke tests pass
12. `MINIMAX_API_KEY=<key> cargo test -p kay-provider-openrouter --features live-api-tests` — live provider tests pass
13. `kay run --prompt "TEST:done" --offline` → exit 0
14. `MINIMAX_API_KEY=<key> kay run --prompt "TEST:done" --live` → exit 0 with TaskComplete event
15. `kay eval tb2 --dry-run` → prints correct instructions (EVAL-01a deferred to separate session)
16. `./scripts/coverage-gate.sh` → PASSED for all Kay crates

## 6. Threat Model

- **API key exposure**: `MINIMAX_API_KEY` must never be committed. `.env.example` documents the variable; real key set via shell environment. `grep -r "sk-cp-" .` is the pre-commit check.
- **Live API cost**: Live tests make real MiniMax API calls. Feature-gated (`live-api-tests`) so normal `cargo test` doesn't bill. Budget: ~$5/smoke-run.
- **TB 2.0 EVAL-01a**: Separate from smoke tests; managed in dedicated project directory with pre-configured Docker container.

## 7. Rollback

Each task is independently verifiable. Rollback = `git revert <commit>` for the specific task. No cross-task dependencies in this milestone.

## 8. Non-Negotiables

1. **DCO on every commit** — `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`
2. **TDD discipline** — RED commit (failing test) MUST precede GREEN commit per task
3. **No key in repo** — `MINIMAX_API_KEY` only via environment; verified by `grep -r "sk-cp-" .`
4. **Feature-gated live tests** — `live-api-tests` feature required to run live API tests
5. **All existing tests green** — no regression in existing test suite
