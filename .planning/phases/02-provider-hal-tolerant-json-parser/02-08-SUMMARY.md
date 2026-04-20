---
phase: 02-provider-hal-tolerant-json-parser
plan: 08
subsystem: provider-hal
tags: [rust, openrouter-provider, sse-translator, tool-call-reassembly, cost-cap-stub, nn7-ordering, path-a-indexmap, dto-divergence]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-07)
    provides: "Allowlist gate + ApiKey newtype + resolve_api_key — composed at OpenRouterProvider::builder"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-06)
    provides: "Provider trait + AgentEvent enum + ProviderError taxonomy — contract surface this plan implements"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-01)
    provides: "tests/fixtures/sse/{happy_path,tool_call_fragmented}.jsonl + tests/common/mock_server.rs — integration-test harness"
provides:
  - "UpstreamClient (src/client.rs) — reqwest::Client wrapper with OpenRouter headers (Authorization, HTTP-Referer, X-Title); stream_chat() -> EventSource with retry::Never (backon owns retry)"
  - "translate_stream (src/translator.rs) — Stream<Result<AgentEvent, ProviderError>> with per-tool_call.id reassembly via HashMap<String, ToolCallBuilder> + index->id fallback map"
  - "OpenRouterProvider (src/openrouter_provider.rs) — builder() + impl Provider::chat composing allowlist.check -> to_wire_model -> build_request_body -> stream_chat -> translate_stream"
  - "CostCap stub (src/cost_cap.rs) — uncapped() default + with_cap() validation + check()/accumulate() hooks; pre-wired Arc<CostCap> on OpenRouterProvider per BLOCKER #5 (plan 02-10 extends)"
  - "NN-7 enforcement via OrderedObject + custom Serialize (Path-A variant) — required before properties in serialized tool_parameters without enabling serde_json/preserve_order workspace-wide"
  - "Two integration tests: streaming_happy_path.rs (TextDelta + Usage) and tool_call_reassembly.rs (fragmented id/index tool_call)"
affects: [plan-02-09, plan-02-10, phase-03-tool-registry, phase-05-agent-loop]

# Tech tracking
tech-stack:
  added:
    - "async-stream 0.3 (stream! macro for ergonomic Stream construction in translator)"
    - "bytes 1 (Bytes body type for EventSource::RequestBuilder)"
    - "indexmap (workspace) — IndexMap<String, OVal> backs OrderedObject; insertion-order preserved without serde_json/preserve_order (which would flip clippy large_enum_variant in forge_app)"
  patterns:
    - "reqwest_eventsource::retry::Never — sole-orchestrator discipline so backon (plan 02-10) owns all retry logic (Pitfall 6)"
    - "index->id fallback mapping for Anthropic-via-OpenRouter first-chunk shape (id present, index absent)"
    - "Local minimal SseChunk DTO instead of forge_app untagged enum Response (Rule-2 sub-deviation documented below)"
    - "OVal { Value | Object(OrderedObject) | Array(Vec<OVal>) } — nested insertion-order preservation without round-tripping through serde_json::Value::Object (BTreeMap-backed when preserve_order is off)"
    - "Option B body build (hand-rolled serde_json) per plan <interfaces> allowance — avoids forge_app Request ModelId + ToolCatalog plumbing this wave"

# Key files
key-files:
  created:
    - path: "crates/kay-provider-openrouter/src/client.rs"
      purpose: "UpstreamClient — POST /chat/completions + SSE handshake (PROV-02)"
    - path: "crates/kay-provider-openrouter/src/translator.rs"
      purpose: "translate_stream — SSE chunk -> AgentEvent delta-granular mapping with per-id reassembly (PROV-01)"
    - path: "crates/kay-provider-openrouter/src/openrouter_provider.rs"
      purpose: "OpenRouterProvider + builder + impl Provider::chat (PROV-01 main)"
    - path: "crates/kay-provider-openrouter/src/cost_cap.rs"
      purpose: "CostCap stub — uncapped() default + with_cap validation (BLOCKER #5 pre-wire for plan 02-10)"
    - path: "crates/kay-provider-openrouter/tests/streaming_happy_path.rs"
      purpose: "End-to-end proof: mock OpenRouter SSE -> Provider::chat -> asserts TextDelta sequence + Usage"
    - path: "crates/kay-provider-openrouter/tests/tool_call_reassembly.rs"
      purpose: "End-to-end proof: fragmented tool_call cassette -> assert exactly one ToolCallStart/Complete with correct args JSON"
  modified:
    - path: "crates/kay-provider-openrouter/Cargo.toml"
      purpose: "+ async-stream, + bytes, + indexmap (workspace)"
    - path: "crates/kay-provider-openrouter/src/lib.rs"
      purpose: "mod client/cost_cap/openrouter_provider/translator; re-export OpenRouterProvider + OpenRouterProviderBuilder + CostCap"
    - path: "crates/kay-provider-openrouter/src/auth.rs"
      purpose: "+ impl From<String> for ApiKey (builder-side ctor)"

decisions:
  - id: "dto-response-local"
    text: "Response-side SSE chunks decode through a local minimal SseChunk struct, not forge_app::dto::openai::Response (which is an untagged enum that routes id-less chunks to CostOnly and loses usage)."
    rationale: "Rule-2 sub-deviation. The request-side build still goes through hand-rolled Option B (NN-7 ordering) and does not touch forge_app either — parity is preserved at both boundaries. The untagged-enum incompatibility is a real pitfall, not a convenience skip."
  - id: "nn7-path-a-ordered-object"
    text: "NN-7 (required-before-properties) enforced via local OrderedObject wrapper backed by IndexMap + custom Serialize, iterating insertion order and holding nested OrderedObjects as-is (no Value round-trip)."
    rationale: "Path-A variant of plan 02-08 T2 Step 6. The originally-suggested fix (enable serde_json/preserve_order) unified across the workspace and flipped clippy large_enum_variant in forge_app::dto::openai::error::Error — parity forbids patching forge_app. Path-A sidesteps the global feature flip."
  - id: "cost-cap-pre-wire"
    text: "OpenRouterProvider struct carries cost_cap: Arc<CostCap> today with CostCap::uncapped() default; builder supports max_usd but no public .max_usd() setter yet."
    rationale: "Checker BLOCKER #5 resolution — stable struct shape from 02-08 so plan 02-10 T2 is a one-file setter + a pre-flight check?, not a struct-shape change."
  - id: "option-b-body-build"
    text: "Request body is built by hand (stream=true + messages + optional tools) with OrderedObject for NN-7-critical subtrees; we do not depend on forge_app::dto::openai::Request."
    rationale: "forge_app Request requires ModelId newtype + ToolCatalog plumbing — more wiring than this wave warrants. Plan <interfaces> explicitly allows Option B. Plan 02-11+ may revisit once the agent loop needs the transformer pipeline."
  - id: "first-chunk-index-backfill"
    text: "When a tool_call delta carries `id` but omits `index`, we backfill index_to_id[index_to_id.len()] = id so later index-only chunks resolve."
    rationale: "Anthropic-via-OpenRouter SSE shape: first tool_call chunk has id without index; subsequent chunks have index without id. Without the backfill, the reassembly integration test silently dropped the entire tool_call."

# Metrics
metrics:
  duration_minutes: 55   # approximate; plan 02-08 T1 + T2 + T3 across multiple deviation iterations
  completed_date: 2026-04-20
  tasks_completed: 3
  files_created: 6
  files_modified: 3
  commits: 3   # 786bd7a, e754631, 84e1893
---

# Phase 2 Plan 02-08: OpenRouterProvider Impl Summary

End-to-end `Provider::chat` wired: allowlist gate + API-key resolution + hand-rolled Option B request body (with NN-7 ordering) + POST /chat/completions + SSE translator with per-tool_call.id reassembly, surfacing delta-granular `AgentEvent` frames. 31 lib unit tests + 13 integration tests green; clippy `-D warnings` clean on all targets; `forge_app` parity untouched.

## Commits

| # | Hash     | Task | Message |
|---|----------|------|---------|
| 1 | `786bd7a` | T1 | `feat(02-08.1): UpstreamClient — reqwest + SSE with typed error surface` |
| 2 | `e754631` | T2 | `feat(02-08.2): OpenRouterProvider + SSE->AgentEvent translator` |
| 3 | `84e1893` | T3 | `test(02-08.3): integration tests for streaming + tool_call reassembly` |

## What Landed

### T1 — UpstreamClient (PROV-02)

- `pub(crate) struct UpstreamClient { client, endpoint: Url, api_key: ApiKey, referer, title }`
- `try_new()` — fallible (maps `reqwest::Client::build` to `ProviderError::Network`); no `.unwrap()/.expect()`
- `with_headers(referer, title)` — builder-style identity header injection
- `async stream_chat(body: Bytes) -> Result<EventSource, ProviderError>` — `set_retry_policy(Box::new(retry::Never))` to honor Pitfall 6 (backon is sole retry orchestrator)
- `build_headers()` — Authorization mandatory; HTTP-Referer + X-Title optional
- Plus `From<String> for ApiKey` builder-side ctor (auth.rs)

### T2 — Translator + OpenRouterProvider + CostCap stub (PROV-01, D-02, D-04)

**`src/translator.rs`:**
- `translate_stream(EventSource) -> Stream<Result<AgentEvent, ProviderError>>` using `async_stream::stream!`
- State: `builders: HashMap<String, ToolCallBuilder>` + `index_to_id: HashMap<u32, String>` + `terminal_seen: bool`
- Per-chunk processing: text content → `TextDelta`; `tool_calls[].function.{name, arguments}` → `ToolCallStart` (first sighting) + `ToolCallDelta` (non-empty args); `finish_reason in {"tool_calls", "stop"}` → drain builders as `ToolCallComplete` (or `ProviderError::ToolCallMalformed` on strict-parse failure; tolerant fallback is plan 02-09); `[DONE]` sentinel → break
- `usage` (when present on any chunk) → `Usage { prompt_tokens, completion_tokens, cost_usd }`
- Strict `serde_json::from_str` for `arguments_raw`; empty buffer → empty object (zero-arg tools)
- `map_upstream_error` handles `StreamEnded`, `InvalidStatusCode`, `InvalidContentType`, `Transport` (full taxonomy in plan 02-10)

**`src/openrouter_provider.rs`:**
- `OpenRouterProvider { allowlist, upstream, cost_cap: Arc<CostCap> }`
- `OpenRouterProviderBuilder` fields: endpoint, api_key, config_auth, allowlist, referer, title, max_usd
- `build()` composes: `Url::parse(endpoint)` → `Allowlist::from_models(vec![])` default → api_key via direct or `resolve_api_key(config_auth)?` → `UpstreamClient::try_new(...)?.with_headers(...)` → `CostCap::uncapped()` | `CostCap::with_cap(n)?`
- `impl Provider::chat`: (1) `allowlist.check(&request.model)?` (2) `to_wire_model` (3) `build_request_body` (4) `upstream.stream_chat(Bytes::from(body))` (5) `translate_stream(es)` → `Box::pin(stream)`
- `build_request_body` hand-rolled with OrderedObject emitting `required` before `properties` for every tool's parameters schema

**`src/cost_cap.rs`:**
- `CostCap::uncapped()` / `with_cap(f64)` / `check()` / `accumulate(f64)`
- `with_cap` rejects non-finite and non-positive (plan 02-10 extends validation suite + accessors)
- Poisoned-mutex recovery via `.lock().unwrap_or_else(|e| e.into_inner())` — crate-root `deny(unwrap_used)` honored

**`src/lib.rs`:** mod + re-export OpenRouterProvider + OpenRouterProviderBuilder + CostCap.

### T3 — Integration tests

- `tests/streaming_happy_path.rs`: mocks `/api/v1/chat/completions` with `happy_path.jsonl` → asserts `[TextDelta("Hello"), TextDelta(" world"), Usage{10, 2, 0.000015}]` + `non_allowlisted_model_rejected_before_http_call` short-circuit test
- `tests/tool_call_reassembly.rs`: mocks with `tool_call_fragmented.jsonl` → asserts single `ToolCallStart{call_abc, execute_commands}`, two non-empty `ToolCallDelta` fragments keyed to `call_abc` (empty opening elided per Pitfall 5), one `ToolCallComplete{call_abc, execute_commands, {"cmd":"ls -la"}}`, one `Usage{100, 25, 0.000375}`

## Tests

- **31 lib unit tests** green — includes 3 client, 2 cost_cap, 6 translator, 3 openrouter_provider::unit, 1 openrouter_provider::nn7 (serialized_tool_schema_puts_required_before_properties), plus 16 from prior plans
- **13 integration tests** green — 6 allowlist_gate + 4 auth_env_vs_config + 2 streaming_happy_path + 1 tool_call_reassembly
- **Clippy** `-D warnings` clean on `--all-targets` for kay-provider-openrouter
- **`forge_app` clippy** clean (parity held — preserve_order NOT enabled; NN-7 enforced via OrderedObject)

## Deviations from Plan

### Rule-1 / Rule-3 — auto-fixed

1. **[Rule 3 - Blocker] forge_app untagged-enum incompatibility**
   - **Found during:** T2 (translator)
   - **Issue:** Plan pointed at `forge_app::dto::openai::Response` for SSE chunk decode, but Response is `#[serde(untagged)]` with the `Success` variant requiring `id`. OpenRouter SSE chunks routinely omit `id`, which routes them to the `CostOnly` variant (no `usage` field) and silently drops usage data.
   - **Fix:** Introduced a local minimal `SseChunk { choices, usage }` + `SseChoice`/`SseDelta`/`SseToolCallDelta`/`SseFunctionDelta`/`SseUsage` DTO in `src/translator.rs`. Request-side body build still uses hand-rolled Option B (unchanged parity boundary).
   - **Files:** `src/translator.rs`
   - **Commit:** `e754631`

2. **[Rule 3 - Blocker] serde_json/preserve_order flips clippy large_enum_variant in forge_app**
   - **Found during:** T2 (build_request_body NN-7 path)
   - **Issue:** The originally-suggested NN-7 fix was to add `features = ["preserve_order"]` to `serde_json` in our Cargo.toml. This unifies across the workspace (one Cargo resolution) and changes `serde_json::Map` backing from BTreeMap → IndexMap, which is larger and tripped `clippy::large_enum_variant` on `forge_app::dto::openai::error::Error`. Parity forbids patching forge_app.
   - **Fix:** Path-A variant of plan T2 Step 6. Introduced `OrderedObject(IndexMap<String, OVal>)` with custom `Serialize` impl; `OVal` enum holds `Value | Object(OrderedObject) | Array(Vec<OVal>)` so nested objects don't round-trip through `serde_json::Value` (which would collapse order). Added `indexmap = { workspace = true }` as direct dep.
   - **Files:** `crates/kay-provider-openrouter/Cargo.toml`, `crates/kay-provider-openrouter/src/openrouter_provider.rs`
   - **Commit:** `e754631`

3. **[Rule 1 - Bug] First-chunk index-backfill for tool_call reassembly**
   - **Found during:** T3 (tool_call_reassembly.rs revealed zero emitted deltas)
   - **Issue:** The fragmented cassette (which mirrors Anthropic-via-OpenRouter SSE shape) carries `id` without `index` on chunk 1, and `index` without `id` on chunks 2–3. Original `resolve_call_id` only registered `index → id` when both were present, so chunks 2–3 found no mapping and the entire tool_call was silently discarded.
   - **Fix:** When `id` is present without `index`, backfill `index_to_id[index_to_id.len() as u32] = id`. Preserves explicit-index behavior when both are present. Unit tests unchanged (first-chunk fixture supplies both).
   - **Files:** `src/translator.rs`
   - **Commit:** `84e1893`

4. **[Rule 1 - Bug] Non-exhaustive match + Debug-on-Ok in tests**
   - **Found during:** T3 (first test-build attempt)
   - **Issue:** `AgentEvent` is `#[non_exhaustive]` — exhaustive matches in tests failed to build without a wildcard arm. Separately, `Result::unwrap_err` requires `Ok` variant to implement `Debug`, but `AgentEventStream` does not.
   - **Fix:** Added `other => ...` arms; replaced `unwrap_err()` with `match ... { Ok(_) => panic!(...), Err(e) => ... }`.
   - **Files:** `tests/streaming_happy_path.rs`, `tests/tool_call_reassembly.rs`
   - **Commit:** `84e1893`

5. **[Rule 1 - Bug] clippy neg_cmp_op_on_partial_ord + doc_overindented_list_items**
   - **Found during:** T2 (clippy pass)
   - **Issue:** `!(cap > 0.0)` triggered `clippy::neg_cmp_op_on_partial_ord`. Module doc-comment for `ApiKey` indented 34 columns and tripped `clippy::doc_overindented_list_items`.
   - **Fix:** Rewrote cap check as `!cap.is_finite() || cap <= 0.0` (same semantics for well-behaved floats; NaN is caught by `is_finite`). Re-indented doc bullet to 4 spaces.
   - **Files:** `src/cost_cap.rs`, `src/openrouter_provider.rs`
   - **Commit:** `e754631` (followed by amend-free re-lint pass in T3 workflow)

### Rule 2 — auto-added missing critical functionality

- None this plan. PROV-05 posture is enforced at crate scope (`#![deny(clippy::unwrap_used, clippy::expect_used)]`); TM-01 redaction inherited from plan 02-07 `ApiKey`; TM-04/08 inherited via `Allowlist`; no new threat surface introduced.

## Authentication Gates

None. No user auth was required during plan execution.

## Known Stubs

- `CostCap::accumulate()` is implemented but NOT yet wired from the translator's Usage-emission site. Plan 02-10 T2 wires it one-liner. This is a known pre-wire (decision `cost-cap-pre-wire`), not a stub that blocks plan completion: cost-cap enforcement is a Phase-10 requirement and plan 02-10 is the designated wiring point. Documented in `src/cost_cap.rs` module-level comment.

## Self-Check

- Created files:
  - FOUND: crates/kay-provider-openrouter/src/client.rs
  - FOUND: crates/kay-provider-openrouter/src/translator.rs
  - FOUND: crates/kay-provider-openrouter/src/openrouter_provider.rs
  - FOUND: crates/kay-provider-openrouter/src/cost_cap.rs
  - FOUND: crates/kay-provider-openrouter/tests/streaming_happy_path.rs
  - FOUND: crates/kay-provider-openrouter/tests/tool_call_reassembly.rs
- Commit hashes reachable from HEAD:
  - FOUND: 786bd7a — feat(02-08.1): UpstreamClient
  - FOUND: e754631 — feat(02-08.2): OpenRouterProvider + translator
  - FOUND: 84e1893 — test(02-08.3): integration tests

## Self-Check: PASSED
