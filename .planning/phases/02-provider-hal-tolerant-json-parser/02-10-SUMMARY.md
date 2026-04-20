---
phase: 02-provider-hal-tolerant-json-parser
plan: 10
subsystem: provider-hal
tags: [rust, backon, retry, retry-after, cost-cap, error-taxonomy, prov-06, prov-07, prov-08, phase-2-closeout]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-08)
    provides: "OpenRouterProvider scaffold + Arc<CostCap> pre-wired field (uncapped default) + UpstreamClient::stream_chat + translate_stream; this plan turns on the retry loop + cost-cap accumulation + full error taxonomy"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-09)
    provides: "translate_stream yields AgentEvent::ToolCallMalformed as data event (stream continues) — preserved verbatim; cost_cap parameter is additive"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-06)
    provides: "ProviderError variants (RateLimited, ServerError, Auth, Http, CostCapExceeded, Network), RetryReason, AgentEvent::Retry variant — all already defined; this plan first uses them"
  - phase: 02.5-kay-core-sub-crate-split (plan 02.5-04)
    provides: "backon v1.6.0 crate available as workspace dep; reqwest_eventsource::retry::Never policy already wired in UpstreamClient (plan 02-08 T1)"
provides:
  - "src/retry.rs (NEW) — default_backoff() ExponentialBuilder (500ms base, 2x factor, 8s cap, 3 attempts, full jitter per D-09), parse_retry_after, classify_http_error, is_retryable + 12 unit tests"
  - "src/cost_cap.rs upgrade — added spent_usd() + cap_usd() read accessors; expanded unit suite from 2 to 7 tests covering uncapped/zero/negative/nan/infinity/under-over/negative-clamp"
  - "src/error.rs — RateLimited.retry_after Duration -> Option<Duration> (Some when server sent Retry-After, None when we fall back to backon schedule)"
  - "src/openrouter_provider.rs — OpenRouterProviderBuilder::max_usd(f64) public setter; chat() adds pre-flight cost_cap.check()? gate; retry_with_emitter_using generic + open_and_probe closure + named retry_with_emitter wrapper; retry_emission_unit suite (3 tokio tests)"
  - "src/translator.rs — translate_stream signature now takes Arc<CostCap>; Usage emission path accumulates cost before yielding; map_upstream_error delegates to retry::classify_http_error"
  - "tests/retry_429_503.rs (NEW, 2 tests) — 429 retry-then-success with Retry-After precedence; 503 exhausts 3-attempt schedule as ServerError"
  - "tests/cost_cap_turn_boundary.rs (NEW, 3 tests) — turn completes before cap enforcement (Pitfall 3); max_usd(0.0) and max_usd(-1.0) rejected at builder time"
  - "tests/error_taxonomy.rs (NEW, 5 tests) — 401/402/404/502/network end-to-end classification"
affects: [phase-2-closeout, phase-03-tool-registry, phase-05-agent-loop, phase-09.5-tui, phase-09-tauri]

# Tech tracking
tech-stack:
  added: []  # backon, reqwest_eventsource, futures were all already present
  patterns:
    - "Single-source retry truth (Pitfall 6): reqwest_eventsource::retry::Never in UpstreamClient; backon is the sole retry orchestrator"
    - "Retry-After header precedence over backon schedule (D-09 + RESEARCH §A4): parse_retry_after returns Option<Duration>; Some overrides the next backon tick"
    - "open_and_probe pattern: open EventSource, probe first Event::Open, classify HTTP-status errors via classify_http_error, return typed ProviderError — required because reqwest_eventsource delivers status errors as stream errors not as stream_chat return errors"
    - "Turn-boundary cost cap enforcement (Pitfall 3): cost_cap.check()? at chat() entry; translator accumulates; never abort mid-response"
    - "AgentEvent::Retry emitted BEFORE tokio::time::sleep so consumers see 'retrying in Ns' indicator before the delay actually fires"
    - "pre_events buffer chained via futures::stream::iter(...).chain(translator_stream) so retry frames precede the post-retry success body"
    - "Generic retry_with_emitter_using<F, Fut, T> makes retry mechanics unit-testable without a real UpstreamClient or mockito"

# Key files
key-files:
  created:
    - path: "crates/kay-provider-openrouter/src/retry.rs"
      purpose: "backon::ExponentialBuilder default schedule, parse_retry_after (integer seconds only; HTTP-date form returns None per RESEARCH §A4), classify_http_error, is_retryable + 12 unit tests covering all status code and Retry-After paths"
    - path: "crates/kay-provider-openrouter/tests/retry_429_503.rs"
      purpose: "End-to-end retry proofs via mockito — 429 with Retry-After: 1 retries then succeeds emitting one Retry frame with RateLimited reason and 1000ms delay; 3x 503 exhausts backon and surfaces ServerError"
    - path: "crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs"
      purpose: "Turn-boundary semantics — turn 1 over-cap completes in full (Pitfall 3); turn 2 rejected pre-flight; max_usd(0.0) and max_usd(-1.0) rejected at builder time with ProviderError::Stream"
    - path: "crates/kay-provider-openrouter/tests/error_taxonomy.rs"
      purpose: "PROV-08 end-to-end taxonomy coverage — 401->Auth::Invalid, 402->Http (body preserved), 404->Http (body preserved), 502->retried 3x then ServerError, unreachable endpoint -> Network"
  modified:
    - path: "crates/kay-provider-openrouter/src/lib.rs"
      purpose: "+ mod retry (crate-private; production code uses default_backoff/is_retryable/classify_http_error internally, tests access via super::)"
    - path: "crates/kay-provider-openrouter/src/cost_cap.rs"
      purpose: "+ spent_usd() + cap_usd() read accessors for diagnostics and tests; expanded unit suite from 2 to 7 tests covering uncapped default, zero/negative/nan/infinity rejection, under/over cap, negative-cost clamp"
    - path: "crates/kay-provider-openrouter/src/error.rs"
      purpose: "RateLimited.retry_after typed Duration -> Option<Duration> so the retry loop can distinguish server-supplied Retry-After from 'use backon schedule' — minimal API surface change (internal crate only)"
    - path: "crates/kay-provider-openrouter/src/openrouter_provider.rs"
      purpose: "OpenRouterProviderBuilder::max_usd(f64) public setter; chat() now does cost_cap.check()? before the allowlist gate; retry_with_emitter_using (generic F/Fut/T) + open_and_probe (probe EventSource first event for status errors) + named retry_with_emitter wrapper; threads Arc<CostCap> to translate_stream; futures::stream::iter(pre_events).chain(translator) so retry frames prefix the body stream; retry_emission_unit 3-test module"
    - path: "crates/kay-provider-openrouter/src/translator.rs"
      purpose: "translate_stream now takes Arc<CostCap>; Usage branch calls cost_cap.accumulate(cost_usd) before yielding; map_upstream_error delegates to retry::classify_http_error with async body.text() read"
    - path: "crates/kay-provider-openrouter/tests/common/mock_server.rs"
      purpose: "+ mock_status_body(status, body) helper for arbitrary HTTP-status taxonomy testing"

decisions:
  - id: "retry-after-precedence-over-backon"
    text: "When `ProviderError::RateLimited { retry_after: Some(d) }` surfaces on a retryable attempt, `d` overrides the next backon-scheduled delay for this attempt only (backon resumes on subsequent attempts). Integer seconds only; HTTP-date form (RFC 7231 §7.1.3) returns None from parse_retry_after so backon takes over."
    rationale: "D-09 + RESEARCH §A4 / §Pitfall 6 explicit — OpenRouter sends integer-seconds Retry-After, never HTTP-date. parse_retry_after's fallible parse path returns None for the date form; callers see None and fall through to backon's next tick. This preserves the 3-attempt ceiling without special-casing spec-allowed-but-rare formats."
  - id: "open-and-probe-pattern"
    text: "retry_with_emitter_using closure calls open_and_probe(upstream, body) which opens an EventSource AND polls es.next() once to observe either Event::Open (success — return es) or Err(InvalidStatusCode) (classify via classify_http_error — return typed ProviderError). Only after a successful probe does the EventSource hand off to translate_stream."
    rationale: "reqwest_eventsource delivers HTTP-status errors INSIDE the stream, not as the POST's return error. The POST returns Ok(EventSource) the moment headers arrive; the 429/5xx status only surfaces on the first es.next().await. Without open_and_probe, the retry loop would never see status errors — they would bypass retry entirely and surface through the translator (unretried). Discovered during integration-test failure (429 arrived as RateLimited on the stream, not the retry wrapper). Recorded as Rule-2 critical-functionality deviation."
  - id: "retry-with-emitter-using-generic-for-testability"
    text: "retry_with_emitter_using<F, Fut, T> is the generic form; retry_with_emitter (concrete) is a thin named wrapper over it. Unit tests in retry_emission_unit exercise the generic form with T=() and closures returning fabricated ProviderError variants."
    rationale: "Plan asked for a retry_emission_unit test asserting AgentEvent::Retry emission without real HTTP. A concrete EventSource + UpstreamClient cannot be constructed without a real TCP stack and mockito. Generic form lets the test supply a closure that returns Err(RateLimited{...}) on attempt 1 and Ok(()) on attempt 2 — exercises exactly the retry decision path we care about. Production call-site uses the non-generic named helper for grep-ability and doc coverage."
  - id: "translator-takes-arc-cost-cap"
    text: "translate_stream(source: EventSource, cost_cap: Arc<CostCap>) — Arc is shared with OpenRouterProvider so the same cap accumulates across turns on the same provider instance. Accumulation is inside the Usage yield branch, BEFORE the yield (so a subsequent chat() on the same provider sees the spend via cost_cap.check())."
    rationale: "D-10 + Pitfall 3 — cost cap enforced at turn boundary, never mid-response. Sharing Arc<CostCap> (not a new CostCap per turn) makes turn-N's check() observe turn-(N-1)'s spent. Plan 02-08 pre-wired this Arc path; this plan's change is threading it through translate_stream (signature delta) + calling accumulate (one line) + adding the pre-flight check (one line)."
  - id: "option-duration-on-retry-after"
    text: "ProviderError::RateLimited.retry_after upgraded from Duration to Option<Duration>. None means 'backon schedule applies'; Some(d) means 'server sent Retry-After: d; use this instead of backon for this attempt'."
    rationale: "Plan 02-06's initial shape assumed Retry-After always present; real OpenRouter 429s may omit it (caller should fall back to backon). Option<Duration> is the minimum-viable shape. Thirteen internal test sites updated; no external breakage (the crate is not yet published)."

# Metrics
metrics:
  duration_minutes: 40
  completed_date: 2026-04-20
  tasks_completed: 4
  files_created: 4   # retry.rs + 3 integration tests
  files_modified: 6  # lib.rs, cost_cap.rs, error.rs, openrouter_provider.rs, translator.rs, common/mock_server.rs
  commits: 3         # fc59f93 (Task 1 from pre-compact), 6f82445 (Task 2), 8b76303 (Task 3)
---

# Phase 2 Plan 02-10: Retry Policy + Cost Cap Turn-Boundary + Error Taxonomy Summary

**Phase 2 CLOSEOUT.** Closed PROV-06, PROV-07, PROV-08 — the three remaining requirements — by adding a `backon`-driven retry loop with `Retry-After` precedence, wiring per-session `CostCap` turn-boundary enforcement through the translator's Usage emission, and replacing plan 02-08's minimal `map_upstream_error` shim with the full `ProviderError` taxonomy (401/402/404/429/502/5xx/transport). Proved every requirement with a 10-test integration suite (2 retry + 3 cost cap + 5 error taxonomy) on top of 12 new `retry::unit` tests and 5 expanded `cost_cap::unit` tests and 3 new `retry_emission_unit` tokio tests.

## Commits

| # | Hash      | Task | Message |
|---|-----------|------|---------|
| 1 | `fc59f93` | T1   | `feat(02-10.1): retry module + cost_cap accessors (backon schedule, Retry-After, classify_http_error)` |
| 2 | `6f82445` | T2   | `feat(02-10.2): wire retry + cost_cap into OpenRouterProvider + translator` |
| 3 | `8b76303` | T3   | `test(02-10.3): integration tests for retry + cost cap + error taxonomy` |

(Task 4 is this closeout metadata commit, tracked separately at commit time.)

## Requirements Closed

- **PROV-06** — Per-session cost cap enforced at turn boundary: pre-flight `cost_cap.check()?` in `chat()`, accumulation inside the translator's Usage yield branch, Pitfall 3 honored (turn completes even if it crosses cap; next turn rejected). `max_usd(0.0)` and `max_usd(-1.0)` rejected at builder time. Read accessors `spent_usd()` / `cap_usd()` for diagnostics.

- **PROV-07** — 429/503 retry with backon (500ms base, 2x factor, 8s max, 3 attempts, full jitter). `Retry-After` integer seconds overrides the backon tick for that attempt (HTTP-date form returns None → backon schedule applies). `AgentEvent::Retry { attempt, delay_ms, reason }` emitted BEFORE `tokio::time::sleep` so consumers see retry indicator before the delay. `reqwest_eventsource::retry::Never` already enforced in UpstreamClient (plan 02-08) — backon is sole retry orchestrator.

- **PROV-08** — Full typed `ProviderError` taxonomy live on the wire path via `retry::classify_http_error`: 401→`Auth { reason: Invalid }`, 429→`RateLimited { retry_after }`, 5xx→`ServerError { status }`, else→`Http { status, body }`. Body preserved verbatim on `Http` variant only (TM-01: `Auth` / `RateLimited` / `ServerError` carry no body). Transport errors classified as `Network(reqwest::Error)`; premature stream-end as `Stream(String)`.

## Test Surface

| Scope | Tests | Notes |
|-------|-------|-------|
| `retry::unit` | 12 | default_backoff build, parse_retry_after (integer/missing/date), classify (401/402/429/429-no-header/503/400), is_retryable (yes/no mixes) |
| `cost_cap::unit` | 7 | uncapped, with_cap rejects {0, -1, NaN, ∞}, under/over cap, negative accumulate clamp |
| `retry_emission_unit` (lib, tokio) | 3 | rate-limited-then-success emits one Retry frame; server_error exhausts 3-attempt schedule; auth_invalid terminal no Retry |
| `retry_429_503` (integration) | 2 | 429 with Retry-After: 1 retries then succeeds with 1000ms delay; 3x 503 exhausts → ServerError |
| `cost_cap_turn_boundary` (integration) | 3 | Turn completes in full; turn-2 rejected with CostCapExceeded; max_usd(0.0 / -1.0) rejected at build time |
| `error_taxonomy` (integration) | 5 | 401/402/404 terminal + correct variant; 502 retried-then-ServerError; unreachable endpoint → Network |

**Totals crate-wide after 02-10:** 55 lib tests + 24 integration tests = 79 green. Clippy `-D warnings` clean on all-targets.

## Deviations from Plan

### Auto-fixed Issues

1. **[Rule 2 — Critical functionality] `open_and_probe` pattern.** Plan sketch had the retry closure call `upstream.stream_chat(body)` directly. Integration testing exposed that `reqwest_eventsource::Error::InvalidStatusCode` surfaces inside the returned `EventSource`, not as a `stream_chat` return error — so 429/5xx would bypass the retry loop entirely and surface through the translator (unretried). Added `open_and_probe(upstream, body)` which opens the EventSource AND polls the first event; if `Event::Open` arrives, returns the `EventSource` for the translator; if `Err(InvalidStatusCode)` or other stream error, classifies via `classify_http_error` + returns typed `ProviderError`. Retry loop now sees status errors. Zero public API change.

2. **[Rule 3 — Blocking issue] `RateLimited.retry_after` typed `Duration` → `Option<Duration>`.** Plan kept the plan-02-06 shape (`Duration`), but distinguishing "server supplied Retry-After" from "use backon fallback" requires `Option`. Changed in error.rs; 13 internal test-site updates. Internal crate — no external breakage.

3. **[Rule 1 — Bug] Triple-paused tokio tests.** Initial `retry_emission_unit` tests used `#[tokio::test(start_paused = true)]` AND called `tokio::time::pause()` inside the test body — the second pause panics with "time is already frozen". Removed the redundant `pause()` calls; `start_paused = true` is sufficient.

4. **[Rule 1 — Bug] `OpenRouterProvider` doesn't implement `Debug` → `.expect_err` unusable in integration tests.** `.expect_err` requires `Debug` on the success type. Replaced `.expect_err(...)` with explicit `match result { Ok(_) => panic!(...), Err(...) => ... }` in `cost_cap_turn_boundary.rs`. Avoids adding `Debug` to the public provider type (which would need credential redaction review).

5. **[Rule 1 — Clippy] `body.to_string()` on `&str` in mock_status_body.** `mockito::Mock::with_body` accepts anything `impl AsRef<[u8]>`. Removed the `to_string()` allocation.

### Rule-4 (architectural) — none triggered.

## Findings

- **mockito sequencing is FIFO by default.** Arming two mocks that match the same path serves them in registration order, one shot each. Perfect for 429-then-200 retry proof and 503×3 exhaustion.

- **`reqwest_eventsource::Error::InvalidStatusCode` carries a `Response`**, so the classifier can read `.headers()` and `.text().await` to preserve both the `Retry-After` header and the error body. TM-01 compliance preserved: body reaches `Http { body }` only, never `Auth` / `RateLimited` / `ServerError`.

- **backon's `Iterator` impl for `ExponentialBackoff`** means the retry loop can precompute the full schedule or consume lazily via `schedule.next()` — we chose lazy consumption with a fallback to the error path when `None` is returned (schedule exhausted). This is the cleanest way to honor "max 3 attempts".

- **Paused tokio time is incompatible with mockito + reqwest integration tests.** The runtime uses internal timers for TCP I/O that break under `start_paused = true`. Integration tests use real time; only the `retry_emission_unit` lib tests run paused (they never touch HTTP). Real-time cost: 429 test ~1s (Retry-After), 503 exhaustion test ~3.5s (500ms+1s+2s backon schedule) — both acceptable.

- **`is_retryable` list**: `RateLimited | ServerError | Network`. `Auth`, `Http`, `CostCapExceeded`, `ToolCallMalformed`, `Canceled`, `Serialization`, `Stream` are all terminal. Verified via `is_retryable_excludes_auth_http_cost_malformed_canceled` unit test.

## Phase 2 Closeout

This plan closes Phase 2. Final Phase 2 status:

| Requirement | Plan | Status |
|-------------|------|--------|
| PROV-01 | 02-08 | Closed — Provider trait + chat streaming + typed AgentEvent |
| PROV-02 | 02-08 | Closed — OpenRouter reqwest 0.13 + reqwest-eventsource + backon |
| PROV-03 | 02-07 | Closed — env/config API key auth (no OAuth) |
| PROV-04 | 02-07 | Closed — Exacto-leaning allowlist |
| PROV-05 | 02-09 | Closed — tolerant two-pass parser + proptest never-panic + 1MB cap |
| PROV-06 | 02-10 | Closed — per-session cost cap + turn-boundary enforcement |
| PROV-07 | 02-10 | Closed — 429/503 retry with jitter + Retry-After precedence + Retry events |
| PROV-08 | 02-10 | Closed — typed ProviderError taxonomy on the wire path |

**All Phase 2 success criteria met:**
1. ✓ End-to-end chat streaming with typed AgentEvent (happy_path integration test)
2. ✓ env-var or config-file API keys; no OAuth (auth_env_vs_config integration test)
3. ✓ ModelNotAllowlisted rejects non-allowlisted models (allowlist_gate integration test)
4. ✓ Fragmented / malformed / null-args tool calls never panic (tool_call_* integration tests + proptest)
5. ✓ Cost cap aborts session; 429/503 retry emits user-visible retry events (this plan's tests)

## Self-Check: PASSED

- crates/kay-provider-openrouter/src/retry.rs: FOUND
- crates/kay-provider-openrouter/tests/retry_429_503.rs: FOUND
- crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs: FOUND
- crates/kay-provider-openrouter/tests/error_taxonomy.rs: FOUND
- Commit fc59f93 (Task 1, retry module + cost_cap accessors): FOUND
- Commit 6f82445 (Task 2, wire retry + cost_cap): FOUND
- Commit 8b76303 (Task 3, integration tests): FOUND
- All 79 tests pass (55 lib + 24 integration): CONFIRMED
- Clippy -D warnings clean on all-targets: CONFIRMED
