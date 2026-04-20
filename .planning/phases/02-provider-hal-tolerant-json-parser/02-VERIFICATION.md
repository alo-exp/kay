---
phase: 02-provider-hal-tolerant-json-parser
verified: 2026-04-20T12:00:00Z
status: passed
score: 5/5 success criteria verified
prov_requirements_verified: 8/8
threat_model_mitigations_verified: 5/5
design_decisions_honored: 3/3 (D-03, D-09, D-10)
overrides_applied: 0
re_verification: false
test_counts:
  lib_unit_tests: 55
  integration_tests: 24
  total: 79
  proptest_invariants: 2
  clippy_all_targets_D_warnings: clean
  workspace_cargo_check: clean
non_negotiables:
  forge_star_src_byte_identical: true   # No Phase 2 commit (02-06..02-10) touches crates/forge_*/src/
  dco_signoff_every_commit: 13/13
  crate_root_deny_unwrap_expect: true    # src/lib.rs:17
  non_exhaustive_on_public_enums: true  # AgentEvent, ProviderError, AuthErrorKind, RetryReason
deferred_to_later_phases:
  - item: "AgentEvent freeze (LOOP-02) formally"
    addressed_in: "Phase 5"
    evidence: "ROADMAP Phase 5 SC-4 — AgentEvent is marked #[non_exhaustive] and documented as a frozen API surface"
  - item: "ToolOutput / SandboxViolation / Verification / TurnEnd variants"
    addressed_in: "Phases 3, 4, 5, 8"
    evidence: "CONTEXT.md D-06 explicit variant carve-outs"
  - item: "EVAL-01a TB 2.0 parity run"
    addressed_in: "Post-Phase-1 follow-on"
    evidence: "Phase 1 D-OP-01 — blocked on OpenRouter key + ~$100 budget"
  - item: "HTTP-date Retry-After form"
    addressed_in: "v1 hardening (RESEARCH §A4 edge case)"
    evidence: "retry::parse_retry_after returns None for HTTP-date, falls through to backon — test parse_retry_after_date_format_returns_none"
  - item: "Cross-session circuit breaker"
    addressed_in: "Phase 999.2 backlog"
    evidence: "ROADMAP Backlog — captured 2026-04-20 from quality-gates advisory"
gaps: []
human_verification: []
---

# Phase 2: Provider HAL + Tolerant JSON Parser — Verification Report

**Phase Goal (ROADMAP):** Any agent turn can stream chat completions and tool calls through OpenRouter with typed events, tolerate provider JSON variance without panicking, recover from transient rate limits, and enforce a per-session cost cap.

**Verified:** 2026-04-20
**Status:** **PASSED**
**Re-verification:** No — initial verification after Plan 02-10 closeout.

---

## Goal Achievement

### Success Criteria (from ROADMAP.md §Phase 2)

| # | Success Criterion | Status | Evidence |
| - | ----------------- | ------ | -------- |
| 1 | A caller can stream a chat completion from OpenRouter via a mock and a real client and receive a typed `AgentEvent` stream carrying text, reassembled tool calls, and usage frames. | VERIFIED | `src/openrouter_provider.rs:157-206` implements `Provider::chat`; integration tests `streaming_happy_path::happy_path_emits_text_deltas_and_usage` (2 TextDelta + Usage) and `tool_call_reassembly::fragmented_tool_call_reassembles_under_single_id` (ToolCallStart/Delta×2/Complete + Usage) prove wire path through mockito. |
| 2 | User authenticates with an API key supplied via environment variable or config file, and OAuth is deliberately absent. | VERIFIED | `src/auth.rs:62-85` implements D-08 precedence (env wins). Integration tests `auth_env_vs_config::{env_wins_on_conflict, config_fallback_when_env_unset, missing_everywhere_yields_typed_error}`. No OAuth code path exists; `AuthErrorKind` lacks OAuth variants. |
| 3 | Requests against models outside the Exacto-leaning allowlist are rejected with a typed `ProviderError::ModelNotAllowlisted` — no silent fallback to ICL parsing. | VERIFIED | `src/allowlist.rs:82-93` + pre-flight gate at `openrouter_provider.rs:169`. Tests `allowlist_gate::launch_allowlist_rejects_random_model` and `streaming_happy_path::non_allowlisted_model_rejected_before_http_call` (no HTTP mock registered; confirmed pre-flight short-circuit). |
| 4 | Feeding fragmented, malformed, or null-`arguments` tool-call deltas into the parser yields a valid reassembled tool call (or a typed `ToolCallMalformed` error), never a panic. | VERIFIED | `src/tool_parser.rs:49-66` two-pass parser. Proptest `parser_never_panics` (unicode universe `\PC*`) proves never-panic. `tool_call_malformed` integration test proves stream continues after malformed tool_call. `translator.rs:237-271` handles empty/null `arguments_delta` defensively. |
| 5 | A session that crosses its `--max-usd` budget aborts with a user-visible event; 429/503 responses retry with jittered exponential backoff and surface user-visible retry events. | VERIFIED | **Cost cap:** `cost_cap_turn_boundary::turn_n_completes_even_if_it_crosses_cap_then_n_plus_1_rejected` — turn 1 completes in full (Pitfall 3), turn 2 rejected pre-flight with `CostCapExceeded { cap_usd: 0.000010, spent_usd: 0.000015 }`. **Retry:** `retry_429_503::{rate_limit_429_retries_then_succeeds, server_error_503_retries_then_exhausts}` — AgentEvent::Retry emitted BEFORE backoff sleep; Retry-After overrides backon (1000ms from header); 3-attempt cap on 503 exhausts to `ServerError{502}`. |

**Score:** **5/5 success criteria verified.**

---

## Requirements Coverage (PROV-01 … PROV-08)

| Req | Requirement | Plan | Artifacts | Tests | Status |
| --- | ----------- | ---- | --------- | ----- | ------ |
| PROV-01 | Provider trait supports chat completion + tool calling + streaming SSE with typed `AgentEvent` output | 02-06, 02-08 | `src/provider.rs` (object-safe via `#[async_trait]`, returns `Pin<Box<dyn Stream<Item=Result<AgentEvent, ProviderError>> + Send + 'a>>`); `src/openrouter_provider.rs` (concrete impl); `src/event.rs` (8 variants, `#[non_exhaustive]`) | `streaming_happy_path`, `tool_call_reassembly`, `retry_429_503` — all pass | SATISFIED |
| PROV-02 | OpenRouter implementation using reqwest 0.13 + reqwest-eventsource + backon | 02-08, 02-10 | `src/client.rs` (reqwest 0.13 + EventSource with `retry::Never`), `src/retry.rs` (backon `ExponentialBuilder` defaults), `src/openrouter_provider.rs:184-189` (open_and_probe wrapper) | `streaming_happy_path::happy_path_emits_text_deltas_and_usage` (full wire round-trip) | SATISFIED |
| PROV-03 | API key auth via env var and config file; no OAuth | 02-07 | `src/auth.rs::resolve_api_key` (env-wins precedence, trim, typed Missing/Invalid) | `auth_env_vs_config.rs` (4 tests) | SATISFIED |
| PROV-04 | Strict model allowlist targeting Exacto endpoints | 02-07 | `src/allowlist.rs` (canonicalize, validate_charset, check, to_wire_model) | `allowlist_gate.rs` (6 tests) | SATISFIED |
| PROV-05 | Tolerant tool-call JSON parser handles OpenRouter variance | 02-08, 02-09 | `src/tool_parser.rs` (Clean/Repaired/Malformed two-pass); wired through `translator.rs:287-309`; `#![deny(clippy::unwrap_used, clippy::expect_used)]` at lib.rs:17 | 8 unit tests + 2 proptests (never_panics, well_formed_always_clean) + `tool_call_malformed` integration | SATISFIED |
| PROV-06 | Per-session cost cap and hard abort | 02-10 | `src/cost_cap.rs` (Mutex<f64> accumulator; uncapped/with_cap); wired in `openrouter_provider.rs:165` (pre-flight check) + `translator.rs:318-326` (accumulate in Usage branch) | `cost_cap.rs unit` (7 tests) + `cost_cap_turn_boundary.rs` (3 integration) | SATISFIED |
| PROV-07 | Rate-limit / 429 / 503 retry with exponential backoff + jitter + user-visible retry events | 02-10 | `src/retry.rs::default_backoff` (500ms base, 2x, 8s cap, 3 attempts, full jitter); `openrouter_provider.rs::retry_with_emitter_using` (generic retry loop with `AgentEvent::Retry` emission BEFORE sleep); `open_and_probe` (Rule-2 critical deviation that probes first SSE event so HTTP status errors reach the retry loop) | `retry.rs unit` (10 tests) + `retry_emission_unit` (3 tests on generic retry path) + `retry_429_503.rs` (2 integration) | SATISFIED |
| PROV-08 | Typed `ProviderError` for diagnosis and retry decisions | 02-06, 02-10 | `src/error.rs` (11 `ProviderError` variants; `AuthErrorKind`, `RetryReason` sub-enums; all `#[non_exhaustive]`); `retry.rs::classify_http_error` (401→Auth::Invalid, 429→RateLimited{Retry-After}, 5xx→ServerError, else→Http{body}) | `error.rs unit` (2 tests) + `retry::unit::classify_*` (5 classifier tests) + `error_taxonomy.rs` (5 integration) | SATISFIED |

**Coverage:** **8/8 PROV-* requirements satisfied.**

---

## Threat-Model Mitigations

| TM | Mitigation | Location | Evidence | Status |
| -- | ---------- | -------- | -------- | ------ |
| TM-01 | API key never logged (Debug redaction) | `src/auth.rs:41-45` (`ApiKey` custom `Debug` returns `ApiKey(<redacted>)`; no `Display`; `pub(crate) as_str()`; NOT re-exported from lib.rs) | `auth.rs::debug_redacts_key_material` ("sk-super-secret-value" → "ApiKey(<redacted>)"); `auth_env_vs_config::debug_never_leaks_credential_in_error_display`; `error.rs::debug_impl_never_prints_credential_material` | VERIFIED |
| TM-04 | Control-character rejection in model IDs (prevents CRLF injection into HTTP headers) | `src/allowlist.rs:116-128` (`validate_charset` rejects `\r \n \t` + non-ASCII with EMPTY allowed list to avoid leaking allowlist to smuggler) | `allowlist_gate::crlf_smuggling_rejected_before_allowlist_compare`; `allowlist unit::{check_rejects_crlf_and_tabs, check_rejects_non_ascii}` | VERIFIED |
| TM-06 | 1MB tool-args cap (DoS mitigation) | `src/tool_parser.rs:32` (`MAX_TOOL_ARGS_BYTES = 1_048_576`); `translator.rs:251-264` (eviction + empty-raw Malformed emission before buffer grows past cap) | `tool_parser::max_bytes_constant_is_one_mebibyte`; proptest `parser_never_panics` on arbitrary Unicode | VERIFIED |
| TM-08 | `:exacto` wire-model rewrite discipline | `src/allowlist.rs:98-100` (`to_wire_model` always appends `:exacto` after canonicalize); wired at `openrouter_provider.rs:172` before body build | `allowlist_gate::wire_model_always_has_exacto_suffix`; `allowlist unit::to_wire_model_always_appends_exacto` (idempotent for already-suffixed input) | VERIFIED |
| TM-09 | EventSource `NeverRetry` policy (prevents 3x3=9 retry amplification when both reqwest-eventsource and backon retry) | `src/client.rs:83` (`es.set_retry_policy(Box::new(retry::Never))`) | Code inspection + PROV-07 test count (exactly 3 attempts observed on 503×3 mock, not 9) | VERIFIED |

**Coverage:** **5/5 threat-model mitigations verified.**

---

## Design Decisions Honored

| D | Decision | Implementation | Verified |
| - | -------- | -------------- | -------- |
| D-03 | Two-pass parser (strict serde_json → forge_json_repair fallback) | `src/tool_parser.rs:49-66` — Pass 1 `serde_json::from_str::<Value>()`, on failure Pass 2 `forge_json_repair::json_repair::<Value>(raw)`, on failure `ParseOutcome::Malformed { error }` | `tool_parser::unit` 8 tests + 2 proptests; `translator.rs:287-309` wires the outcome into `ToolCallComplete` / `ToolCallMalformed` events |
| D-09 | backon ExponentialBuilder: base 500ms, factor 2x, max 3 attempts, 8s cap, full jitter; Retry-After honored on 429 | `src/retry.rs:21-28` exactly as specified; `openrouter_provider.rs:249-260` gives Retry-After precedence over backon schedule | `retry_429_503::rate_limit_429_retries_then_succeeds` asserts `delay_ms == 1000` (Retry-After: 1 overrides backon); `retry.rs::parse_retry_after_*` tests |
| D-10 | Per-session cost cap; Mutex-protected; turn-boundary enforcement (not mid-stream) | `src/cost_cap.rs` (Mutex<f64>); accumulate inside translator Usage branch (`translator.rs:320`); check() pre-flight at `openrouter_provider.rs:165` — NOT during stream | `cost_cap_turn_boundary::turn_n_completes_even_if_it_crosses_cap_then_n_plus_1_rejected` — turn 1 runs to completion even when it crosses the cap, turn 2 rejected before HTTP. Mutex poison recovery via `unwrap_or_else(|e| e.into_inner())` (clippy-clean without `unwrap_used` lint trigger) |

**Coverage:** **3/3 design decisions honored.**

---

## Test Counts

Final `cargo test -p kay-provider-openrouter --no-fail-fast`:

| Binary | Tests | Status |
| ------ | ----- | ------ |
| lib (unit, inside src/) | 55 | all pass |
| `tests/allowlist_gate.rs` | 6 | all pass |
| `tests/auth_env_vs_config.rs` | 4 | all pass |
| `tests/cost_cap_turn_boundary.rs` | 3 | all pass |
| `tests/error_taxonomy.rs` | 5 | all pass |
| `tests/retry_429_503.rs` | 2 | all pass |
| `tests/streaming_happy_path.rs` | 2 | all pass |
| `tests/tool_call_malformed.rs` | 1 | all pass |
| `tests/tool_call_reassembly.rs` | 1 | all pass |
| **Total** | **79** (55 lib + 24 integration) | **all pass** |

- `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` → clean
- `cargo check --workspace` → clean (no `--exclude` needed; Phase 2.5 unblocked)

Proptest invariants: 2 (`parser_never_panics`, `well_formed_json_always_clean`) — 256 cases each by default, never panicked in this run.

---

## Non-Negotiables Compliance

| # | Non-Negotiable | Check | Status |
| - | --------------- | ----- | ------ |
| 1 | Forked ForgeCode parity gate (EVAL-01) | `.forgecode-upstream-sha` unchanged; `forgecode-parity-baseline` tag still points at upstream SHA `022ecd994eaec30b519b13348c64ef314f825e21`; EVAL-01a run deferred per Phase 1 D-OP-01 (explicit policy) | HONORED |
| 2 | No unsigned release tags | No release tags cut during Phase 2 | N/A (honored by absence) |
| 3 | DCO not CLA | All 13 Phase 2 code commits (f36083f, b0bcc8d, 0b4a8c1, f3586e8, 786bd7a, e754631, 84e1893, 73adc6e, e7f91c7, 7d3031b, fc59f93, 6f82445, 8b76303) carry `Signed-off-by:` trailer | HONORED |
| 4 | Clean-room contributor attestation | Sole committer throughout Phase 2 is the project maintainer; no third-party contribution merged | HONORED |
| 5 | Single merged Rust binary (no `externalBin` sidecar) | `kay-provider-openrouter` is a workspace library crate — no Tauri binary involved in Phase 2 | N/A (honored; applies to Phase 9) |
| 6 | Strict OpenRouter model allowlist | Launch allowlist fixed at 3 models (anthropic/claude-sonnet-4.6, anthropic/claude-opus-4.6, openai/gpt-5.4) in `tests/fixtures/config/allowlist.json`; `Allowlist::check` enforces strict membership; every wire request gets `:exacto` suffix | HONORED |
| 7 | ForgeCode JSON schema hardening (required-before-properties, flattening, truncation reminders) | `openrouter_provider::build_request_body` + `reorder_tool_parameters` emit `"required"` before `"properties"` via `IndexMap` + custom `Serialize`; the `nn7::serialized_tool_schema_puts_required_before_properties` test asserts `r_pos < p_pos` on every CI run. Path-A avoids enabling `serde_json/preserve_order` which would trip `clippy::large_enum_variant` in upstream `forge_app` (parity preservation) | HONORED |

Additional Phase-2 self-imposed invariants:

| Invariant | Check | Status |
| --------- | ----- | ------ |
| `#![deny(clippy::unwrap_used, clippy::expect_used)]` at crate root | `src/lib.rs:17` | HONORED |
| `#[non_exhaustive]` on all public enums | `AgentEvent` (event.rs:15), `ProviderError` (error.rs:13), `AuthErrorKind` (error.rs:57), `RetryReason` (error.rs:70) | HONORED |
| `ApiKey` NOT re-exported via `pub use` | `src/lib.rs:35-36` comment + absence of `pub use auth::ApiKey` | HONORED |
| `crates/forge_*/src/` byte-identical to upstream for Phase 2 scope | `git log --diff-filter=M -- 'crates/forge_*' 64515b1..HEAD` → the only match is Phase 2.5's `cargo fmt` whitespace fix on `forge_main/src/info.rs` (commit 3268b55, explicitly scoped and documented); zero Phase 2 (02-06..02-10) commits touch `crates/forge_*/src/` | HONORED |

---

## Anti-Patterns Scan

| Pattern | Source files | Test modules | Impact |
| ------- | ------------ | ------------ | ------ |
| `TODO`/`FIXME`/`XXX`/`HACK` | 0 matches | 0 matches | None |
| `unimplemented!()`/`todo!()` | 0 matches | 0 matches | None |
| `.unwrap()` / `.expect()` in non-test code | 0 (crate-wide lint `#![deny(...)]` enforces) | test code only, under `#[allow(...)]` attributes on test modules | None |
| `return null`/`return []` stubs | N/A (Rust) | — | None |
| Hardcoded empty default that masks missing wiring | None found; `CostCap::uncapped()` is a legitimate default that `cost_cap_turn_boundary::max_usd_zero_rejected_at_builder_time` validates is not accidentally routed through for capped sessions | None |
| Unused pub re-exports | `pub use openrouter_provider::{OpenRouterProvider, OpenRouterProviderBuilder}` — both consumed by integration tests | None |

---

## Gaps / Residual Risks

**None that block phase closeout.** Items of note, all deliberately deferred or already tracked:

1. **HTTP-date Retry-After form** (RFC 7231 §7.1.3) — spec-allowed but rare. `retry::parse_retry_after` returns `None` for this form, so backon's default schedule applies. Test `parse_retry_after_date_format_returns_none` documents this intentional behavior. Upgrade path: parse via `httpdate` crate when OpenRouter real traces show it in use. No known OpenRouter trace uses this form.

2. **Cross-session circuit breaker** — each session pays its full retry budget during sustained OpenRouter outages. Filed as backlog Phase 999.2 (ROADMAP). Advised during /silver-quality-gates Phase 2 review.

3. **Error message "what-to-do" audit** — Typed `ProviderError` Display language has not been user-tested for remediation clarity. Backlog Phase 999.3.

4. **AgentEvent not yet formally `#[non_exhaustive]`-frozen per LOOP-02** — annotation is already applied; Phase 5 is where the freeze is officially documented as a cross-phase contract. No change expected.

5. **DTO divergence (Rule-2 sub-deviation recorded in 02-08-SUMMARY)** — `translator.rs` uses a local minimal `SseChunk` instead of `forge_app::dto::openai::Response` because that upstream type is `#[serde(untagged)]` keyed on top-level `id` and OpenRouter omits `id` on usage-carrying chunks (routes them to a `CostOnly` variant that discards usage). The divergence is response-side only; request-side body still honors parity via `build_request_body`. Documented trade-off; no functional gap.

6. **OpenRouter SSE retry semantics real-trace validation** — STATE.md Phase 2 research flag. Addressed at mock level (mockito FIFO sequencing + real `Retry-After: 1` observed wait). Real-upstream validation happens at EVAL-01a (blocked on OpenRouter key + ~$100 budget, Phase 1 D-OP-01).

7. **`Provider::models()` returns static allowlist, not live OpenRouter catalog** — by design (D-07 launch allowlist composition). Live catalog integration is a Phase 10 UI concern (UI-04 model picker).

8. **No `unsafe` block audit required** — the only `unsafe` in provider code is test-scoped env-var mutation (Rust 2024 marks `std::env::set_var` unsafe) under `ENV_LOCK: Mutex<()>` for serialization. Not reachable from library code.

### Latent bugs noticed during review

None. Code is internally consistent:

- `resolve_call_id` index-backfill for Anthropic-via-OpenRouter first-chunk-id-no-index edge case is explicitly handled (`translator.rs:152-156`).
- `open_and_probe` defensively handles all four `reqwest_eventsource::Error` variants (InvalidStatusCode, InvalidContentType, Transport, StreamEnded) plus `None` early stream close (`openrouter_provider.rs:301-334`).
- Cost-cap negative-value clamping (`cost_cap.rs:78`) prevents wire noise from de-accumulating session spend.
- TM-06 1 MB cap yields empty-raw Malformed (not near-1MB raw) to avoid yielding large strings through AgentEvent (`translator.rs:256-263`).

---

## Final Verdict

**PASSED.** Phase 2 delivers its ROADMAP goal end-to-end:

1. Goal achieved — a caller can instantiate `OpenRouterProvider::builder()`, hand it an allowlist + API key + endpoint, call `.chat(ChatRequest { .. })`, and receive a typed `AgentEvent` stream carrying TextDelta / ToolCallStart / ToolCallDelta / ToolCallComplete / ToolCallMalformed / Usage / Retry / Error frames. The happy-path, fragmented-tool-call, malformed-tool-call, 429-retry-then-succeed, 503-exhaust, cost-cap-turn-boundary, and error-taxonomy (401/402/404/502/transport) paths all ship green.
2. All 8 PROV-* requirements verified by real code + tests (not just REQUIREMENTS.md checkboxes).
3. All 5 threat-model mitigations (TM-01, TM-04, TM-06, TM-08, TM-09) structurally enforced and tested.
4. All 3 Phase 2 design decisions (D-03, D-09, D-10) implemented as specified.
5. 79 tests green; clippy -D warnings clean; workspace cargo check clean.
6. Non-negotiables preserved: DCO signoff on every commit; `forge_*/src/` byte-identical to upstream (no Phase 2 commit modifies it); `#![deny(clippy::unwrap_used, clippy::expect_used)]` at crate root; `#[non_exhaustive]` on every public enum; `ApiKey` redacted Debug and not re-exported.

**Recommended next step:** Begin Phase 3 planning via `/gsd-discuss-phase 3` (Tool Registry + KIRA Core Tools). ROADMAP §Phase 3 has "Plans: TBD"; no `03-xx-PLAN.md` exists yet.

---

*Verified: 2026-04-20*
*Verifier: Claude (gsd-verifier, Opus 4.7)*
*Report file: `.planning/phases/02-provider-hal-tolerant-json-parser/02-VERIFICATION.md`*
