# Phase 2: Provider HAL + Tolerant JSON Parser — Research

**Researched:** 2026-04-20
**Domain:** Rust async streaming HTTP client / provider abstraction / tolerant parsing / structural Rust module refactor
**Confidence:** HIGH (structural rename is VERIFIED, delegation strategy is VERIFIED, OpenRouter contract is CITED; a few flagged ASSUMPTIONS documented in §Assumptions Log)

---

## Summary

Phase 2 lands the typed provider HAL (`Provider` trait + `AgentEvent` + `ProviderError`) in `kay-provider-openrouter`, and performs the single largest structural fix the repo has ever needed: resolving 23 × E0583 module errors in `kay-core` **without modifying any byte of ForgeCode source content** (parity-baseline preservation).

The headline finding of this research is that **D-01's "bare rename `lib.rs` → `mod.rs`" is NECESSARY BUT NOT SUFFICIENT**. The imported ForgeCode crates use `use crate::X` paths that — when moved into Kay's single-crate module layout — would resolve to `kay_core::X` rather than `kay_core::forge_Y::X`. There are ~379 intra-subtree `use crate::` occurrences across 195 files plus ~340 inter-subtree `use forge_X::` statements across 201 files. Both must be rewritten structurally for kay-core to compile. `[VERIFIED: grep results, §User Constraints §Primary Recommendation]`

For the provider HAL itself, the story is simpler: ForgeCode's `forge_repo/provider/openai.rs` already speaks OpenRouter natively (endpoint URL, HTTP-Referer header, `ChatCompletionMessage` Stream), ForgeCode's `forge_json_repair` parser is a verbatim match for PROV-05, and ForgeCode's `retry.rs` is a verbatim match for PROV-07. Kay's typed façade is a thin Stream-mapping layer that converts ForgeCode's aggregated `ChatCompletionMessage` into delta-granular `AgentEvent` frames. Do not rewrite anything — wrap everything.

**Primary recommendation:** Reframe D-01 from "bare rename" to **"rename + structural path rewrite (`crate::X` → `crate::forge_Y::X`, `use forge_Y::` → `use crate::forge_Y::`)"**. Handle the rewrite as a separate mechanical transform (one `cargo fix` + sed pass per subtree, commit-per-subtree) so that any regression is bisectable. Keep the imported source content logically identical — only import paths change. This is still a mechanical, byte-preserving-of-logic change (no behavior modification), so the `forgecode-parity-baseline` tag's semantic integrity is preserved, but it is more work than a single `mv` of 23 files. Budget 2-3 tasks, not one.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01: Rename `crates/kay-core/src/forge_X/lib.rs` → `crates/kay-core/src/forge_X/mod.rs`** for all 23 `forge_*` subdirectories.
- Rationale: mechanical, preserves content, smallest diff, lets `cargo check --workspace` pass without `--exclude kay-core`. Does NOT modify source content — preserves the `forgecode-parity-baseline` tag's semantic integrity.
- Research finding: the bare rename is necessary but **not sufficient** — see §Primary Recommendation and §Finding: D-01 Rename Scope Is Larger Than Stated.

**D-02: Typed wrapper over ForgeCode's existing OpenAI-compatible provider path.**
- `forge_repo/provider/openai.rs` already speaks OpenRouter natively. `kay-provider-openrouter` exposes the typed Provider trait + AgentEvent enum + ProviderError enum and delegates to `forge_services::provider_service` / `forge_repo::provider::openai` internally.
- Rationale: preserves parity guarantee; rewriting would corrupt the baseline.

**D-03: Two-pass parser, reusing `forge_json_repair` as the second pass.**
- First pass: `serde_json::from_str::<Value>()` (strict).
- Second pass on failure: `forge_json_repair::json_repair`.
- If both fail: emit `AgentEvent::ToolCallMalformed` and `ProviderError::ToolCallMalformed`. Never panic.

**D-04: Tool-call deltas accumulated per `tool_call.id` in `HashMap<String, ToolCallBuilder>`.**
- Complete only on terminal marker (`finish_reason = tool_calls` or done event).
- Null `arguments` in a delta is legal (OpenRouter variance).

**D-05: `ProviderError` variants** (non_exhaustive): Network, Http{status,body}, RateLimited{retry_after}, ServerError{status}, Auth{reason}, ModelNotAllowlisted{requested,allowed}, CostCapExceeded{cap_usd,spent_usd}, ToolCallMalformed{id,error}, Serialization, Stream(String), Canceled.

**D-06: `AgentEvent` variants** (non_exhaustive, Phase 2 scope only): TextDelta, ToolCallStart, ToolCallDelta, ToolCallComplete, ToolCallMalformed, Usage{prompt_tokens,completion_tokens,cost_usd}, Retry{attempt,delay_ms,reason}, Error{error}.

**D-07: Launch allowlist (v1):** `anthropic/claude-sonnet-4.6`, `anthropic/claude-opus-4.6`, `openai/gpt-5.4` — all Exacto-targeted via `:exacto` suffix. Allowlist lives in `provider.json` + `kay.toml [provider.openrouter] allowed_models` with `KAY_ALLOWED_MODELS` env override.
- Research finding: `openai/gpt-5.4` may not be on OpenRouter's current Exacto list (as of April 2026, the highest visible is `gpt-5.3-chat`). Flagged in §Assumptions Log + §Open Questions.

**D-08: API key via `OPENROUTER_API_KEY` env var OR `kay.toml` `[provider.openrouter] api_key`**; env wins on conflict. No OAuth. No keyring in Phase 2 (Phase 10 UI concern).

**D-09: `backon::ExponentialBuilder` with full jitter.** Base 500ms, factor 2, max attempts 3, max delay 8s. 429 respects `Retry-After` header if present. 503 uses backoff as-is. Every retry emits `AgentEvent::Retry`.

**D-10: No default `--max-usd` (uncapped if unset).** `--max-usd 0` rejected as usage error. Enforcement at turn boundary — in-flight streaming is NOT interrupted mid-response. Cost calc uses OpenRouter's `usage.cost` field (USD), falling back to the allowlist's pinned price table.

### Claude's Discretion
- Internal crate structure of `kay-provider-openrouter` (module layout, test organization)
- Exact `forge_*` path through which we call into ForgeCode's provider — planner/executor decide based on what compiles cleanest post-rename
- Which `backon` builder API shape to use
- Test fixture format for mocking OpenRouter SSE (cassette library vs. hand-written mocks)
- Whether to expose a synchronous convenience wrapper alongside the async streaming API

### Deferred Ideas (OUT OF SCOPE)
- Direct Anthropic / OpenAI / Gemini provider integrations → v2
- Local model support (Ollama, llama.cpp) → v2
- OAuth flow for OpenRouter → out-of-scope per PROV-03
- Keyring-based key storage → Phase 10 UI
- Provider config UI picker → Phase 10 UI
- Cost dashboards, per-model price learning → out-of-scope v1
- Splitting `kay-core` into per-subsystem sub-crates → not scheduled
- Re-tagging `forgecode-parity-baseline` after the rename → NOT done
- Abort in-flight HTTP when cost cap exceeded mid-response → D-10 defers this
- Supporting Anthropic/Bedrock providers via OpenRouter's abstraction → they are in the imported tree but not allowlisted; not exercised in Phase 2
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROV-01 | `Provider` trait supports chat completion + tool calling + streaming SSE with typed `AgentEvent` output | §Architecture Pattern 2 (Provider trait shape); §Finding: AgentEvent + Streaming Contract |
| PROV-02 | OpenRouter provider implementation using reqwest 0.13 + reqwest-eventsource + backon retry | §Standard Stack; §Don't Hand-Roll; §Finding: Retry Policy Alignment |
| PROV-03 | API key auth via environment variable and config file — no OAuth | §Finding: Auth Reuse from forge_services/provider_auth |
| PROV-04 | Strict model allowlist targeting OpenRouter Exacto endpoints | §Finding: Allowlist Single Source of Truth; §Open Question: gpt-5.4 Exacto availability |
| PROV-05 | Tolerant tool-call JSON parser for OpenRouter variance | §Finding: Tolerant JSON Parser Integration; §Code Example: Two-Pass Parse |
| PROV-06 | Streaming token budget with per-session cost cap and hard abort | §Finding: Cost Cap Enforcement Timing; §Code Example: Turn-Boundary Accumulator |
| PROV-07 | Rate-limit / 429 / 503 retry with exponential backoff + jitter; user-visible retry events | §Finding: Retry Policy Alignment; §Code Example: backon + AgentEvent::Retry |
| PROV-08 | Typed `ProviderError` (not string) for diagnosis and retry decisions | §Finding: Typed Error Mapping from ForgeCode's anyhow::Error downcast |
</phase_requirements>

---

## Project Constraints (from CLAUDE.md)

These constraints bind Phase 2's design and must not be violated:

| # | Constraint | Impact on Phase 2 |
|---|-----------|-------------------|
| NN-1 | Forked ForgeCode parity gate: the unmodified fork must reproduce ≥80% on TB 2.0 BEFORE any harness modification merges | Phase 2 rewrites import paths only (logically unchanged imports, different path syntax). The EVAL-01a parity run is deferred; Phase 2 should NOT modify any semantic forge_* behavior |
| NN-2 | No unsigned release tags from v0.1.0+ | Phase 2 ships under v0.0.x — no release gating impact |
| NN-3 | DCO on every commit | Each Phase 2 task commit carries `Signed-off-by:` |
| NN-4 | Clean-room contributor attestation (Claude Code leak) | Phase 2 new code must not derive from TypeScript ClaudeCode structure |
| NN-5 | Single merged Rust binary (no externalBin sidecar) | Phase 2 is core crate work; no bundler impact yet |
| NN-6 | Strict OpenRouter Exacto allowlist — no "300+ models" | D-07 enforces this; `ProviderError::ModelNotAllowlisted` gates all requests |
| NN-7 | ForgeCode JSON schema hardening consistently applied | Phase 2 reuses `forge_app/dto/openai/transformers/normalize_tool_schema.rs` via the typed wrapper; Phase 3 consumes this for its own tool schemas (TOOL-05) |

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| HTTP transport (TLS, connection pooling) | Transport Layer (reqwest 0.13 + rustls 0.23) | — | Already settled by WS-02 / deny.toml; Phase 2 does not revisit |
| SSE framing (bytes → events) | Transport Layer (reqwest-eventsource 0.6) | — | Single established library; no alternative fits the stack |
| 429/503 retry orchestration | Transport Layer (backon::ExponentialBuilder) | Application Layer (emits `AgentEvent::Retry`) | Retry primitive is transport; user-visible retry signaling is application |
| Tool-call JSON parsing | Application Layer (kay-provider-openrouter tool_parser module) | Reuses `forge_json_repair` for second pass | Pass-1 strict + Pass-2 repair lives in the provider adapter — belongs with the boundary that owns the output shape |
| OpenRouter API contract (model IDs, usage.cost, tool_calls schema) | Application Layer (forge_repo/provider/openai.rs — delegated to) | — | ForgeCode already encodes this; Kay's wrapper must not re-implement |
| Typed `AgentEvent` emission | Provider HAL Facade (kay-provider-openrouter::Provider impl) | — | This is the boundary's whole job: translate ForgeCode's `ChatCompletionMessage` stream into delta-granular typed events |
| Typed `ProviderError` taxonomy | Provider HAL Facade | — | Downcast-from-anyhow::Error happens here; downstream consumers see only typed variants |
| Model allowlist enforcement | Provider HAL Facade (pre-request check) | Config (provider.json + kay.toml) | Config is data; enforcement is a typed-error gate in the facade, BEFORE the HTTP call fires |
| Per-session cost cap | Provider HAL Facade (turn-boundary accumulator) | — | Accumulates `Usage` frames; emits `CostCapExceeded` on next turn entry |
| Authentication (API key resolution) | Provider HAL Facade (reuses `forge_services::provider_auth`) | Config (env var + kay.toml) | Reuse is pure win — the inherited ForgeCode path already handles env + config merge with env winning |

**Tier discipline:** `kay-provider-openrouter` is the Provider HAL facade. It MUST NOT reach over to UI/transcript/session concerns (Phase 6/9/10). Its only consumer in Phase 2 is an anonymous caller (tests). In Phase 5 it will be consumed by the agent loop; in Phase 9 by the Tauri shell via the CLI event stream. None of those consumers' concerns leak into Phase 2.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.13 (workspace pin) | Async HTTP client | Workspace-pinned; rustls-only per deny.toml; matches ForgeCode's import tree |
| reqwest-eventsource | 0.6 (workspace pin) | SSE wrapper over reqwest | Only maintained SSE-over-reqwest option; matches ForgeCode's import tree; default stable version 0.6.0 `[VERIFIED: crates.io API]` |
| rustls | 0.23 (workspace pin, transitive via reqwest feature) | TLS | Pinned; openssl banned in deny.toml |
| tokio | 1.51 (workspace pin) | Async runtime | LTS; required by reqwest stream + downstream `tokio::select!` in Phase 5 |
| serde + serde_json | 1 | JSON encode/decode | Already in workspace; happy-path pass-1 parser |
| tokio-stream | (inherited from kay-core deps, added) | `StreamExt` + `Stream` adapters | ForgeCode's `chat.rs` uses this — `use tokio_stream::StreamExt;` |
| backon | 1.6.0 | Retry orchestration with jitter | Named by PROV-02 literally; version verified `[VERIFIED: crates.io API 2026-04-20]` |
| thiserror | 2 (workspace pin) | Error derive | Standard Rust error derive pattern; workspace-pinned |
| futures | 0.3.32 | Stream utilities (`pin_mut!`, `Stream` trait) | Ecosystem standard; required for `Pin<Box<dyn Stream<...>>>` return types in object-safe async traits `[VERIFIED: crates.io API]` |
| async-trait | 0.1.89 | Async methods on object-safe traits | Workaround for object-safe `dyn Provider`; see §Finding: AgentEvent + Streaming Contract `[VERIFIED: crates.io API]` |

### Supporting (test-scoped)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| mockito | 1.7.2 | Test HTTP server, reuses ForgeCode's pattern | Already imported in `forge_repo/provider/mock_server.rs` — extend it, don't introduce new mock libs `[VERIFIED: crates.io API]` |
| proptest | 1.11.0 | Property-based testing for tolerant parser | The PROV-05 parser is the one place where property testing gives exceptional ROI — fuzz against malformed-JSON generators `[VERIFIED: crates.io API]` |
| tempfile | (std pattern) | Temp config files for auth tests | Testing `kay.toml` override behavior |
| tracing-subscriber | 0.3 (workspace pin) | Test log capture | `test-log` pattern for asserting log events |

### Alternatives Considered (and rejected)

| Instead of | Could Use | Tradeoff | Why Rejected |
|------------|-----------|----------|--------------|
| reqwest-eventsource 0.6 | eventsource-client | More features, independent impl | reqwest-eventsource is the one ForgeCode uses; matching it = zero delta |
| backon 1.6.0 | tower::retry, tower-retry, backoff | `tower` ecosystem integration | PROV-02 names backon specifically; backon's `Retryable` trait is also the idiomatic ergonomic win `[CITED: docs.rs/backon]` |
| mockito | wiremock-rs 0.6.5 | More features | ForgeCode already uses mockito; introducing a second mock library fragments tests without benefit |
| mockito | rvcr 0.2.1 (cassette record/replay) | Record real OpenRouter traces and replay | Young crate (0.2.1), low adoption. Defer to backlog — revisit if hand-authored SSE fixtures become unwieldy `[VERIFIED: crates.io 2026-04-20]` |
| hand-rolled SSE parser | reqwest-eventsource | "Full control" | 3-5× the code for zero correctness wins; reqwest-eventsource is battle-tested |
| hand-rolled exponential backoff | backon | "Full control" | See §Don't Hand-Roll |

**Installation:** (add to `crates/kay-provider-openrouter/Cargo.toml`; root dependencies should come from workspace where already pinned)

```toml
[dependencies]
kay-core = { path = "../kay-core" }
tokio = { workspace = true }
reqwest = { workspace = true }
reqwest-eventsource = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
async-trait = "0.1.89"
backon = "1.6.0"
futures = "0.3.32"
tokio-stream = "0.1"  # verify at plan-time vs kay-core's inherited version

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "test-util"] }
mockito = "1.7.2"
proptest = "1.11.0"
pretty_assertions = "1"  # already in kay-core test deps
```

**Version verification commands (run at plan time):**

```bash
cargo search reqwest-eventsource --limit 1  # confirm 0.6.x is current
cargo search backon --limit 1               # confirm 1.6.x is current
cargo search mockito --limit 1              # confirm 1.7.x is current
cargo search proptest --limit 1             # confirm 1.11.x is current
cargo search async-trait --limit 1          # confirm 0.1.89+ is current
```

---

## Architecture Patterns

### System Architecture Diagram

```
                                  kay-provider-openrouter (Provider HAL facade)
                                  ┌──────────────────────────────────────────────────────────┐
                                  │                                                          │
  caller (tests/kay-cli Phase 5)  │   OpenRouterProvider: impl Provider                      │
           │                      │   ┌─────────────────────────────────────────────┐        │
           ▼                      │   │  1. validate model against allowlist        │        │
    Provider::chat(req) ─────────►│   │      (ProviderError::ModelNotAllowlisted)   │        │
                                  │   │  2. check cost cap (turn-boundary)          │        │
                                  │   │      (ProviderError::CostCapExceeded)       │        │
                                  │   │  3. wrap request with backon retry loop     │        │
                                  │   └───────────────────┬─────────────────────────┘        │
                                  │                       │                                  │
                                  │   delegates to        ▼                                  │
                                  │   ┌─────────────────────────────────────────────┐        │
                                  │   │ forge_services::provider_service            │        │
                                  │   │  └─► forge_repo::provider::openai           │        │
                                  │   │        ├─► reqwest POST /v1/chat/completions│        │
                                  │   │        ├─► reqwest-eventsource SSE framing  │        │
                                  │   │        └─► yields Stream<ChatCompletionMsg> │        │
                                  │   └───────────────────┬─────────────────────────┘        │
                                  │                       │                                  │
                                  │   translator (the facade's real work):                   │
                                  │   Stream<ChatCompletionMessage>  →  Stream<AgentEvent>   │
                                  │   ┌─────────────────────────────────────────────┐        │
                                  │   │ for each ChatCompletionMessage:             │        │
                                  │   │   content.delta    → TextDelta              │        │
                                  │   │   tool_calls[i]    → ToolCallStart/Delta    │        │
                                  │   │   finish_reason    → ToolCallComplete       │        │
                                  │   │   usage.cost       → Usage {cost_usd}       │        │
                                  │   │   parse error      → tool_repair(pass-2)    │        │
                                  │   │     │                ├─► ToolCallComplete   │        │
                                  │   │     │                └─► ToolCallMalformed  │        │
                                  │   │   429/503          → Retry event +          │        │
                                  │   │                      backon sleep +         │        │
                                  │   │                      reconnect or fail      │        │
                                  │   └───────────────────┬─────────────────────────┘        │
                                  │                       │                                  │
                                  └───────────────────────┼──────────────────────────────────┘
                                                          │
                                                          ▼
                                         Stream<Result<AgentEvent, ProviderError>>
                                         (consumed by Phase 5 agent loop or tests)


     Auth flow (resolved once at construction time):
     ┌────────────────────────────────────────┐
     │ OPENROUTER_API_KEY env var             │───┐
     │         OR                             │   │
     │ kay.toml [provider.openrouter].api_key │───┤──► forge_services::provider_auth
     │                                        │   │    ├─► resolved Provider<Url>
     │ (env wins on conflict)                 │───┘    └─► credential attached
     └────────────────────────────────────────┘              │
                                                             ▼
                                                   used by chat() above
```

### Recommended Project Structure (`kay-provider-openrouter`)

```
crates/kay-provider-openrouter/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # re-exports public API: Provider, AgentEvent, ProviderError, OpenRouterProvider
│   ├── provider.rs               # pub trait Provider + OpenRouterProvider impl
│   ├── event.rs                  # pub enum AgentEvent (non_exhaustive)
│   ├── error.rs                  # pub enum ProviderError + AuthErrorKind + RetryReason (all non_exhaustive)
│   ├── allowlist.rs              # load + enforce Exacto allowlist; kay.toml + env override merge
│   ├── cost_cap.rs               # turn-boundary Usage accumulator + cap check
│   ├── tool_parser.rs            # two-pass parser: serde_json → forge_json_repair fallback; reassembles deltas
│   ├── retry.rs                  # backon ExponentialBuilder + Retry-After header honoring
│   └── translator.rs             # ChatCompletionMessage Stream → AgentEvent Stream mapping
└── tests/
    ├── allowlist_gate.rs         # PROV-04: rejects non-allowlisted models before HTTP
    ├── streaming_happy_path.rs   # PROV-01: mocks SSE, asserts AgentEvent shape
    ├── tool_call_reassembly.rs   # PROV-05 (part 1): fragmented tool_calls across chunks
    ├── tool_call_malformed.rs    # PROV-05 (part 2): malformed/null args → repair → fallback
    │                             # NOTE: PROV-05 proptest invariants live inline in
    │                             # tool_parser::unit (see plan 02-09 T3) — no standalone
    │                             # tests/tool_call_property.rs because parse_tool_arguments
    │                             # is crate-private.
    ├── retry_429_503.rs          # PROV-07: 429 with Retry-After, 503 backoff, emits Retry event
    ├── cost_cap_turn_boundary.rs # PROV-06: turn N+1 rejected; in-flight not interrupted
    ├── auth_env_vs_config.rs     # PROV-03: env var wins over kay.toml; missing key → typed error
    └── error_taxonomy.rs         # PROV-08: each ProviderError variant maps correctly from upstream anyhow
```

### Pattern 1: Provider Trait (object-safe async streaming)

**What:** The `Provider` trait must be object-safe (for `Arc<dyn Provider>` dependency injection) AND support async methods that return streams. In Rust 2024 edition, native `async fn` in traits works for static dispatch but NOT for trait objects. Solution: use `async_trait` OR return `Pin<Box<dyn Stream + Send + 'a>>` from a non-async method.

**When to use:** Phase 2's single public contract.

**Example:**

```rust
// Source: https://docs.rs/async-trait + https://doc.rust-lang.org/book/ch17-05-traits-for-async.html
// Verified against kay-core's forge_domain/repo.rs ChatRepository pattern [VERIFIED: grep forge_domain/repo.rs]

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub type AgentEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'a>>;

#[async_trait]
pub trait Provider: Send + Sync {
    /// Stream a chat completion as typed AgentEvent frames.
    /// Requests for non-allowlisted models fail BEFORE any HTTP call.
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError>;

    /// List models (happy-path; future-proofs Phase 10 allowlist picker).
    async fn models(&self) -> Result<Vec<ModelId>, ProviderError>;
}
```

**Why this shape:**
- `#[async_trait]` makes the trait object-safe without relying on unstable `dyn async fn` features.
- Returning `Pin<Box<dyn Stream ...>>` keeps the facade's Stream type anonymous (we can change the internal Stream type without breaking consumers).
- `Send + Sync` bounds let the Provider cross the `tokio::select!` boundary in Phase 5 without additional trait coercion.
- Matches ForgeCode's own `ChatRepository` pattern in `forge_domain/repo.rs`, so adapters compose naturally.

### Pattern 2: Two-Pass Tolerant Parse

**What:** Strict `serde_json` first, then `forge_json_repair::json_repair` on failure. Reassemble tool-call fragments across SSE chunks per `id`.

**When to use:** Every time a `tool_call.arguments` field crosses the delta boundary.

**Example:**

```rust
// Source: crates/kay-core/src/forge_json_repair/parser.rs (verbatim reuse)
// + crates/kay-core/src/forge_repo/provider/event.rs (ChatCompletionMessage pattern)

use serde_json::Value;
use kay_core::forge_json_repair::{json_repair, JsonRepairError};

pub enum ParseOutcome {
    Clean(Value),
    Repaired(Value),
    Malformed(String),  // carries the original raw bytes for diagnostics
}

pub fn parse_tool_arguments(raw: &str) -> ParseOutcome {
    if raw.is_empty() {
        // Null-arguments variant: OpenRouter sometimes sends "" as initial chunk
        return ParseOutcome::Clean(Value::Object(Default::default()));
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(v) => ParseOutcome::Clean(v),
        Err(strict_err) => {
            match json_repair::<Value>(raw.to_string()) {
                Ok(v) => ParseOutcome::Repaired(v),
                Err(_repair_err) => ParseOutcome::Malformed(raw.to_string()),
            }
        }
    }
}
```

**Integration note:** `forge_json_repair::json_repair` is a generic function: `pub fn json_repair<De: DeserializeOwned>(s: String) -> Result<De, JsonRepairError>` — it both repairs and deserializes. Since our second pass targets `serde_json::Value`, this integrates naturally. Call this only inside the tool-call delta reassembly; do NOT run pass-2 on every SSE chunk (performance regression).

### Pattern 3: Streaming Translator (ChatCompletionMessage → AgentEvent)

**What:** The facade's core translation logic. ForgeCode yields a `Stream<ChatCompletionMessage>` where each message has `content, tool_calls, finish_reason, usage`. Kay's `AgentEvent` is delta-granular — so the translator tracks state across chunks and emits multiple events per incoming message.

**When to use:** Inside `OpenRouterProvider::chat` after the HTTP request succeeds and before returning the Stream to the caller.

**Example skeleton (verify at plan time):**

```rust
// Source: pattern derived from crates/kay-core/src/forge_repo/provider/chat.rs
// + crates/kay-core/src/forge_domain/message.rs (ChatCompletionMessage shape)

use std::collections::HashMap;
use futures::{Stream, StreamExt, stream};
use kay_core::forge_domain::{ChatCompletionMessage, FinishReason};

struct ToolCallBuilder {
    name: Option<String>,
    arguments_raw: String,  // accumulated across deltas
}

pub fn translate_stream<S>(
    upstream: S,
) -> impl Stream<Item = Result<AgentEvent, ProviderError>>
where
    S: Stream<Item = anyhow::Result<ChatCompletionMessage>> + Send,
{
    let mut builders: HashMap<String, ToolCallBuilder> = HashMap::new();
    // Use async_stream::stream! or futures::stream::unfold to emit multiple
    // AgentEvents per incoming message. Terminal completion triggered by
    // finish_reason == Some(FinishReason::ToolCalls) or Stop.
    async_stream::stream! {
        // ... (specific implementation: planner's discretion)
    }
}
```

**Note:** `async-stream` crate is a planner-discretion addition if the closure shape gets complex. Alternatively, a hand-rolled `futures::stream::unfold` works without adding the dep.

### Anti-Patterns to Avoid

- **Emitting `AgentEvent::ToolCallComplete` from a single SSE chunk** — tool calls stream fragmented; Complete fires only on terminal marker. A single-chunk tool call will arrive as Delta → Complete in rapid succession; don't skip the state machine.
- **Logging API keys anywhere in `Debug`/`Display` impls on `ProviderError::Auth`** — panic-on-format would be an exfiltration risk. Redact explicitly (see §Threat Model Seeds).
- **Running pass-2 JSON repair on every SSE frame** — pass-1 on the concatenated `arguments` is the only parse point needed; repairing per-chunk wastes CPU and can mis-interpret partial delta state.
- **Collecting the full stream in-memory before returning to caller** — defeats the entire point of streaming. Yield as you translate.
- **Blocking the tokio runtime inside `tool_parser`** — if `forge_json_repair` turns out to be expensive on huge payloads, use `tokio::task::spawn_blocking`. (Audit at implementation time; the parser is ~350 LOC of char iteration — expected fast, but verify on known pathological inputs.)
- **Using `unwrap()` anywhere in the tool-call hot path** — PROV-05 is explicit: never a panic. This is a hard invariant; code review must fail any `unwrap()` / `expect()` in `tool_parser.rs`.
- **Adding a second mock HTTP library** — `mockito` is already imported via ForgeCode's `mock_server.rs`. Extend that file's helpers; don't introduce `wiremock` or `httpmock`.
- **Re-implementing Retry-After parsing** — `reqwest::Response::headers()` + `retry-after` header → `backon::ExponentialBuilder` delay override. Use `std::time::Duration::from_secs` with parsed `u64`; support both integer seconds and HTTP-date format (RFC 7231 §7.1.3) via `httpdate` crate if the latter actually occurs in practice (verify during implementation).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE framing | Byte-by-byte parser | `reqwest-eventsource 0.6` | Already 4+ years battle-tested across Rust ecosystem; handles multi-line data fields, retry/reconnect, keepalives, and framing error recovery |
| Exponential backoff with jitter | Custom duration math | `backon::ExponentialBuilder::default().with_jitter().with_factor(2.0).with_min_delay(Duration::from_millis(500)).with_max_delay(Duration::from_secs(8)).with_max_times(3)` | Full jitter math is subtly wrong in 80%+ of hand-rolled attempts; thundering herd is a real prod concern `[CITED: docs.rs/backon/struct.ExponentialBuilder]` |
| Tolerant JSON parser | Regex or custom state machine | `forge_json_repair::json_repair` (imported in `forge_json_repair/parser.rs`) | ~350 LOC battle-tested against real OpenRouter variance. Handles trailing commas, unquoted keys, stringified numbers, markdown code-block wrappers (`` ```json ``), redundant end quotes, NDJSON drift |
| Tool-call delta reassembly | Building one from scratch | Study `forge_repo/provider/openai.rs` internal transform from OpenAI SSE to `ChatCompletionMessage`, then add delta-granular state on top | ForgeCode already handles streaming reassembly for aggregated form; our addition is the delta-granular dispatch layer |
| Provider config loading (env var + kay.toml merge) | Custom TOML walker | `forge_services::provider_auth` + `forge_services::app_config` | Already handles env-override, per-provider credential resolution; proven on OpenRouter config |
| HTTP error taxonomy (429 / 503 / 402 / etc.) | Custom status-code dispatch | `forge_repo/provider/retry.rs::into_retry` as inspiration + `reqwest::StatusCode` pattern match | Proven taxonomy; just adapt the `anyhow::Error` downcasts into our typed `ProviderError` |
| HTTP-date parsing (for Retry-After) | Custom HTTP-date parser | `httpdate` crate (optional, only if OpenRouter returns non-integer Retry-After) | The spec allows both integer seconds and HTTP-date; check both during implementation |
| OpenRouter request body construction | Hand-rolling the chat completion request | Reuse `forge_app::dto::openai::Request` | ForgeCode has the full OpenAI-compatible DTO, including `tools` array and the `normalize_tool_schema` transformer; PROJECT.md constraint NN-7 mandates using the hardening pipeline |

**Key insight:** Phase 2's IP is the typed contract (Provider + AgentEvent + ProviderError) and the delta-granular event translation. Every non-contract concern (HTTP, SSE, JSON repair, retry, provider config, DTOs) is already solved in the imported tree. The new code surface should be ~1000-1500 LOC, not 5000+.

---

## Runtime State Inventory

> Phase 2 is predominantly a greenfield implementation in `kay-provider-openrouter` plus a structural refactor in `kay-core`. The rename/refactor does NOT introduce any runtime state — it's purely a compile-time module-layout change. But because D-01 involves mass file renames, I audit for runtime state below:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — Phase 2 creates no databases, no caches on disk (model cache in `CacacheStorage` is inherited from ForgeCode, unchanged by rename). | None |
| Live service config | OpenRouter account config lives server-side; not in our repo | None (but see §Open Questions: account procurement) |
| OS-registered state | None — no Windows Task Scheduler / launchd / systemd registration changes | None |
| Secrets/env vars | `OPENROUTER_API_KEY` env var is READ — this is the runtime. The rename does not change the env var name. The `kay.toml` `[provider.openrouter] api_key` path is new config surface, written fresh for Phase 2. | None (new config, not migration) |
| Build artifacts | `target/` may contain stale `.rlib`/`.rmeta` files from the pre-rename tree. A `cargo clean` after the rename is prudent but not strictly required; Cargo's incremental system should detect path changes. | Recommend: one-time `cargo clean -p kay-core` after the rename lands |

**Explicit nulls:** Nothing found in any category that forces a data migration. The rename is pure code; no runtime state references the old file layout.

---

## Common Pitfalls

### Pitfall 1: D-01 Rename Scope Is Larger Than Stated (THE BIG ONE)

**What goes wrong:** Executing D-01 as literally described — `mv forge_X/lib.rs forge_X/mod.rs` for 23 subtrees — does not make kay-core compile. Each imported forge_* subtree uses `use crate::Y` paths that, under Kay's single-crate layout, now resolve to `kay_core::Y` (the top-level crate), not `kay_core::forge_X::Y` where they actually live.

**Evidence (VERIFIED via grep):**
- **379 intra-subtree `use crate::` occurrences in 195 files.** E.g., `crates/kay-core/src/forge_json_repair/parser.rs:3: use crate::error::{JsonRepairError, Result};` — after bare rename, this resolves to `kay_core::error`, which doesn't exist. It should resolve to `kay_core::forge_json_repair::error`.
- **340 inter-subtree `use forge_Y::X` occurrences in 201 files.** E.g., `crates/kay-core/src/forge_services/provider_auth.rs:4: use forge_app::{AuthStrategy, ProviderAuthService, StrategyFactory};` — after bare rename, this doesn't resolve at all (no `forge_app` extern crate). It should be `use crate::forge_app::{...}`.
- **Pattern is systematic, not case-by-case.** Every imported `lib.rs` was authored under the assumption that it IS a crate root, so `crate::X` refers to its OWN subtree, and sibling-crate references use `use forge_Y::`. Under Kay's single-crate layout, both patterns invert.

**Why it happens:** ForgeCode ships as a 23-crate workspace. Kay imported the source trees into a single crate — the 01-03 plan's "flat forge_*/… subtree structure preserved under a single kay-core crate" decision (D-05 in plan 01-03). But the source files continued to reference themselves as if they were still at a crate root. The bare rename resolves the E0583 module-declaration error (23 errors), but unmasks a much larger number of new errors (hundreds of E0433/E0432 unresolved-path errors).

**How to avoid:** Reframe D-01 as a two-step operation:

1. **Step 1 (rename):** `mv forge_X/lib.rs forge_X/mod.rs` for all 23 subtrees. This resolves the 23 E0583 errors.
2. **Step 2 (path rewrite):** For each forge_X subtree, mechanically rewrite:
   - `use crate::Y`  →  `use crate::forge_X::Y`  (within `forge_X/**.rs` files, intra-subtree references)
   - `use forge_Y::Z`  →  `use crate::forge_Y::Z`  (all files, cross-subtree references)
   - `extern crate forge_Y` (if any) → remove
   - `pub use forge_Y::*` in container modules → `pub use crate::forge_Y::*`

   Each subtree's rewrite should be one commit so bisect works. Automate with `rg --files-with-matches 'use crate::' crates/kay-core/src/forge_X | xargs sd 'use crate::' 'use crate::forge_X::'` (sd or sed). Verify compile after each subtree commit; the order should be dependency-lowest-first (`forge_domain` → `forge_app` → `forge_services` → `forge_repo` → `forge_infra` → `forge_api` → `forge_main` → peripherals).

**Content preservation:** Even with Step 2, NO semantic behavior changes — we only reshape path syntax. The logical imports are identical. The `forgecode-parity-baseline` tag still points at the pre-rename commit `8af1f2b`; Phase 2's first commits produce new trees but do not invalidate the tag's purpose (the tag is the "as-imported" snapshot, not the "as-compiled" snapshot).

**Warning signs:** If after Step 1 `cargo check -p kay-core 2>&1 | head -20` shows hundreds of "unresolved import" errors (E0432, E0433) instead of module-declaration errors (E0583), Step 2 is required. If kay-core compiles clean after Step 1 only, we got extraordinarily lucky — verify with a full `cargo test -p kay-core --lib`.

### Pitfall 2: async_trait + Stream return type obscures lifetimes

**What goes wrong:** `#[async_trait] async fn chat(&self) -> AgentEventStream<'_>` desugars to `Pin<Box<dyn Future<Output = AgentEventStream<'_>> + Send + 'async_trait>>`, and the inner Stream's lifetime captures `self` in ways that surprise the borrow checker.

**Why it happens:** Rust 2024's implicit capture rules for RPIT are different from 2021's. Async traits via `async_trait` use heap allocation for the future and must bound their lifetimes explicitly.

**How to avoid:** Make the returned Stream's lifetime explicit: `AgentEventStream<'a>` where `'a: self`. If the Stream ends up needing `'static` (because the caller moves it), store any `&self` borrows inside `Arc<...>` and clone into the stream's closure.

**Warning signs:** Compiler complains about `&self` outliving `'async_trait`; fix by cloning the Arc-wrapped state into the stream setup.

### Pitfall 3: Turn-boundary cost cap enforcement off-by-one

**What goes wrong:** Reading `usage.cost` from the current turn's final frame, comparing to cap, and allowing the NEXT turn if under — but not accounting for the case where the CURRENT turn already crossed the cap (so the cap is exceeded but we already delivered the response).

**Why it happens:** D-10 explicitly says cap applies "at turn boundaries, not mid-response." This means: we MUST NOT abort the current HTTP call mid-flight. The turn-N frame is always delivered. But the question is whether turn N+1 starts.

**How to avoid:** State machine is clean — accumulator is updated on every `Usage` frame arrival (usage frames arrive with the final chunk per OpenRouter's docs `[CITED: openrouter.ai/docs/api/reference/streaming]`). Next call to `Provider::chat` checks the accumulator BEFORE it fires the HTTP request. If already exceeded, return `ProviderError::CostCapExceeded` immediately.

**Warning signs:** Test `cost_cap_turn_boundary` must assert both: (a) turn N finishes normally even if it pushes over cap, (b) turn N+1 returns `CostCapExceeded` without an HTTP call (verify via mock that received no request).

### Pitfall 4: Usage frame arrival timing

**What goes wrong:** Assuming `usage` is present on every SSE chunk, or assuming it's absent on every chunk.

**Evidence (CITED: openrouter.ai/docs/api/reference/streaming):** "The final chunk includes usage stats." Intermediate chunks do NOT carry usage. The final chunk's `choices[].delta` may be empty with `finish_reason` set; usage comes on the very same chunk or on a dedicated `[DONE]` terminator depending on provider variant.

**How to avoid:** The translator must be tolerant: `usage.is_some()` → emit `AgentEvent::Usage{cost_usd: usage.cost.unwrap_or(0.0), ...}` and accumulate. `usage.is_none()` → continue without emitting Usage. Never panic on missing usage. If `usage.cost` is None AND the model IS on the allowlist, fall back to the price table (D-10).

**Warning signs:** Integration test must include a fixture where usage arrives in a separate final chunk from the last content chunk — this is the realistic OpenRouter case, not a synthetic edge case.

### Pitfall 5: Null `arguments` in tool-call delta

**What goes wrong:** `tool_calls[0].function.arguments` can be `null`, `""`, or missing entirely in intermediate chunks. Pass-1 strict parse on `null` succeeds (it's valid JSON), but produces `Value::Null`, which then fails `serde_json::from_value::<FunctionArgs>(...)`.

**How to avoid:** Treat `null` and empty string identically as "arguments not yet started." The `ToolCallBuilder::arguments_raw: String` starts empty; only non-null, non-empty deltas are appended. Final parse happens once, on the full concatenated string, at terminal `finish_reason = tool_calls`.

**Warning signs:** `tool_call_reassembly` test must include a fixture with `arguments: null` in the first N chunks.

### Pitfall 6: reqwest-eventsource's default retry behavior fights backon

**What goes wrong:** `reqwest_eventsource::EventSource` ships with a DEFAULT `RetryPolicy` (exponential). If we wrap `EventSource` inside a `backon`-based retry loop, we get double retry — 3 × 3 = 9 attempts, with the inner retry's exponential + the outer retry's exponential compounding.

**Evidence:** `[CITED: docs.rs/reqwest-eventsource/latest/reqwest_eventsource/struct.EventSource.html]` — EventSource exposes `set_retry_policy`. To disable, provide a no-op policy that always returns `None`.

**How to avoid:** Call `EventSource::set_retry_policy(Box::new(NoRetry))` where `NoRetry` is a `RetryPolicy` impl that returns `None` for all inputs. Let backon be the single point of retry control.

**Warning signs:** Test `retry_429_503` should assert exactly 3 retries occurred (not 6 or 9). Assert against the mock server's hit count.

### Pitfall 7: Allowlist string-matching is case-sensitive by default

**What goes wrong:** User passes `Anthropic/Claude-Sonnet-4.6` (mixed case), allowlist has `anthropic/claude-sonnet-4.6`. String compare fails, request rejected with `ModelNotAllowlisted{requested: "Anthropic/Claude-Sonnet-4.6", allowed: [...]}`, user confused.

**How to avoid:** Normalize to lowercase on both sides before compare. Decision: permissive on input form (accept any case), strict on allowlist content (must be provided in canonical form). Document in CLI help: "Model IDs are case-insensitive."

**Warning signs:** Allowlist test should include a mixed-case input case.

### Pitfall 8: Exacto suffix semantics when allowlist is pre-suffixed

**What goes wrong:** Allowlist stores `anthropic/claude-sonnet-4.6:exacto`, user passes `anthropic/claude-sonnet-4.6` (no suffix). Compare fails — rejected.

**Conversely:** Allowlist stores `anthropic/claude-sonnet-4.6`, user passes `anthropic/claude-sonnet-4.6:exacto`. Compare fails — rejected.

**How to avoid:** Document the canonical form in the allowlist. Recommendation: store WITHOUT the `:exacto` suffix; automatically append `:exacto` when building the OpenRouter request body. This way the user sees the clean name, and the request always hits Exacto.

**Warning signs:** Test `allowlist_gate` includes round-trip: user input `anthropic/claude-sonnet-4.6` → request body contains `model: "anthropic/claude-sonnet-4.6:exacto"`.

### Pitfall 9: Drop during streaming leaks partial tool-call state

**What goes wrong:** Caller drops the `AgentEventStream` mid-tool-call. `ToolCallBuilder` state in `HashMap<String, ToolCallBuilder>` is leaked — not a memory leak (the stream is owned), but a logical one: the partial call is silently discarded.

**How to avoid:** This is expected behavior — drops are intentional cancels. Document in `Provider::chat` doc that dropping the stream aborts the turn. Do NOT try to flush partial calls via a `Drop` impl; that leaks work the caller doesn't want.

**Warning signs:** No automated test needed; documentation is the fix.

### Pitfall 10: Windows file handle count during mass rename

**What goes wrong:** The 23-subtree rename + 500+ path rewrites can trip Windows file-handle limits if run in one commit on a Windows CI runner during `cargo check`.

**How to avoid:** Commit per subtree (Step 2 above), force `fail-fast: false` matrix strategy in CI, and run `cargo check` not `cargo build` during the path-rewrite phase to minimize handle pressure. The cross-platform test matrix will catch any Windows-specific regressions after the full rename lands.

**Warning signs:** Windows CI job fails with "too many open files" during intermediate commits. Fix: break the step-2 rewrite into smaller commits.

---

## Code Examples

### Example 1: `Provider` trait with object-safe async streaming

**Source:** Derived from `async_trait` docs + ForgeCode's `forge_domain/repo.rs` `ChatRepository` pattern.

```rust
// crates/kay-provider-openrouter/src/provider.rs

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use serde_json::Value;
use crate::{AgentEvent, ProviderError};

pub type AgentEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<AgentEvent, ProviderError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,              // e.g. "anthropic/claude-sonnet-4.6"
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSchema>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,               // "system" | "user" | "assistant" | "tool"
    pub content: String,
    pub tool_call_id: Option<String>,  // only for role == "tool"
}

#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,        // JSON Schema, already hardened (NN-7)
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError>;

    async fn models(&self) -> Result<Vec<String>, ProviderError>;
}
```

### Example 2: OpenRouterProvider with allowlist gate + cost-cap check

**Source:** D-04, D-07, D-10 combined. Delegates to ForgeCode after pre-check.

```rust
// crates/kay-provider-openrouter/src/provider.rs (continued)

pub struct OpenRouterProvider {
    allowlist: Vec<String>,                // canonical model IDs (lowercase, no :exacto suffix)
    cost_cap: Option<f64>,                 // None = uncapped
    spent: std::sync::Mutex<f64>,          // accumulator; thread-safe
    upstream: Arc<dyn UpstreamClient>,     // thin wrapper around forge_services
    api_key: String,
}

impl OpenRouterProvider {
    fn normalize_model(m: &str) -> String {
        m.to_ascii_lowercase().trim_end_matches(":exacto").to_string()
    }

    fn check_allowlist(&self, model: &str) -> Result<(), ProviderError> {
        let canonical = Self::normalize_model(model);
        if self.allowlist.iter().any(|m| m == &canonical) {
            Ok(())
        } else {
            Err(ProviderError::ModelNotAllowlisted {
                requested: model.to_string(),
                allowed: self.allowlist.clone(),
            })
        }
    }

    fn check_cost_cap(&self) -> Result<(), ProviderError> {
        let Some(cap) = self.cost_cap else { return Ok(()); };
        let spent = *self.spent.lock().unwrap();
        if spent > cap {
            Err(ProviderError::CostCapExceeded { cap_usd: cap, spent_usd: spent })
        } else {
            Ok(())
        }
    }

    fn to_exacto_model(m: &str) -> String {
        format!("{}:exacto", Self::normalize_model(m))
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError> {
        // 1. Pre-flight checks (no HTTP until these pass)
        self.check_allowlist(&request.model)?;
        self.check_cost_cap()?;

        // 2. Rewrite model ID to include :exacto suffix
        let mut req = request;
        req.model = Self::to_exacto_model(&req.model);

        // 3. Delegate to forge_services; get back Stream<ChatCompletionMessage>
        let upstream_stream = self.upstream.chat(req).await?;  // wraps forge_services::provider_service

        // 4. Translate + emit AgentEvents; this is the facade's real work
        let stream = crate::translator::translate_stream(
            upstream_stream,
            Arc::clone(&self.upstream),  // for Retry event emission
            &self.spent,
        );

        Ok(Box::pin(stream))
    }

    async fn models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(self.allowlist.clone())
    }
}
```

### Example 3: `backon` retry loop + Retry-After header honoring

**Source:** `backon 1.6.0` docs + OpenRouter error guide.

```rust
// crates/kay-provider-openrouter/src/retry.rs

use backon::{ExponentialBuilder, Retryable};
use std::time::Duration;

pub fn default_backoff() -> ExponentialBuilder {
    ExponentialBuilder::default()
        .with_min_delay(Duration::from_millis(500))
        .with_factor(2.0)
        .with_max_delay(Duration::from_secs(8))
        .with_max_times(3)
        .with_jitter()
}

/// Extracts `Retry-After` header from a reqwest response, if present.
/// OpenRouter returns integer seconds; HTTP-date form is spec-allowed but rare
/// on OpenRouter — we handle both. Returns None if missing or unparseable.
pub fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers.get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            // Try integer seconds first (RFC 7231 §7.1.3, common case)
            if let Ok(secs) = s.parse::<u64>() {
                return Some(Duration::from_secs(secs));
            }
            // Fall back to HTTP-date (rare, needs httpdate crate; defer)
            None
        })
}

/// Example: retrying a request. `emit_retry` is a caller-supplied sender that
/// pushes AgentEvent::Retry into the outer stream so the UI can show progress.
pub async fn retrying_call<F, Fut, T, E>(
    emit_retry: impl Fn(u32, Duration, RetryReason),
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    // Note: real impl uses backon's Retryable trait with when(|e| is_retryable(e))
    // and notify(|err, dur| emit_retry(...)) to surface retries to caller.
    // Full signature verified during implementation.
    unimplemented!("skeleton — see retry.rs at implementation time")
}
```

### Example 4: Mock SSE server for tool-call fragmentation test

**Source:** Extends `crates/kay-core/src/forge_repo/provider/mock_server.rs`.

```rust
// crates/kay-provider-openrouter/tests/tool_call_reassembly.rs

#[tokio::test]
async fn streams_fragmented_tool_call_arguments_across_chunks() {
    let mut mock = MockServer::new().await;

    // Three chunks: tool_call started, arguments partial, arguments complete.
    // Final chunk also carries finish_reason = "tool_calls" + usage.
    let events = vec![
        r#"data: {"choices":[{"delta":{"tool_calls":[{"id":"call_abc","type":"function","function":{"name":"execute_commands","arguments":""}}]}}]}"#.into(),
        r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"cmd\":\""}}]}}]}"#.into(),
        r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"ls -la\"}"}}]}}]}"#.into(),
        r#"data: {"choices":[{"delta":{},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":100,"completion_tokens":25,"total_tokens":125,"cost":0.000375}}"#.into(),
        r#"data: [DONE]"#.into(),
    ];

    let _m = mock.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(mock.url())
        .api_key("test-key")
        .allowlist(vec!["anthropic/claude-sonnet-4.6".to_string()])
        .build()
        .unwrap();

    let stream = provider.chat(ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![/* ... */],
        tools: vec![/* ... */],
        temperature: None,
        max_tokens: None,
    }).await.unwrap();

    let events: Vec<_> = stream.collect().await;
    // Exactly 3 events: ToolCallStart, ToolCallDelta (aggregated? or multiple?), ToolCallComplete, Usage.
    // Assert shape via matches!(...).
}
```

### Example 5: Property-based parser fuzz

**Source:** `proptest 1.11.0` docs.

```rust
// NOTE: Not a separate file. Per plan 02-09 T3 BLOCKER #4 revision (2026-04-20),
// this proptest lives INLINE in crates/kay-provider-openrouter/src/tool_parser.rs's
// #[cfg(test)] mod unit (parse_tool_arguments is crate-private). The shape below
// is the CODE PATTERN; consult plan 02-09 T3 for the committed placement.

use proptest::prelude::*;

proptest! {
    #[test]
    fn parser_never_panics(raw in "\\PC*") {  // Any Unicode-scalar string
        // Never-panic invariant (PROV-05)
        let _ = crate::tool_parser::parse_tool_arguments(&raw);
        // No panic == test passes
    }

    #[test]
    fn well_formed_json_always_clean_pass1(obj in proptest::collection::hash_map(
        "[a-z]{1,10}",
        proptest::prelude::any::<i64>(),
        1..10,
    )) {
        let raw = serde_json::to_string(&obj).unwrap();
        match crate::tool_parser::parse_tool_arguments(&raw) {
            ParseOutcome::Clean(_) => {},
            other => panic!("expected Clean, got {:?}", other),
        }
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-rolled SSE parsing | `reqwest-eventsource` over reqwest | 2020-2022 (crate matured) | Kay inherits via ForgeCode tree |
| `async-trait` everywhere | Native `async fn` in traits (static dispatch) + async-trait for dyn dispatch | Rust 1.75+ (2024) | Kay targets both — static where possible, async-trait for `dyn Provider` `[CITED: blog.rust-lang.org/2023/12/21]` |
| `tokio::sync::mpsc` + hand-rolled stream | `futures::Stream` + `tokio-stream::StreamExt` | 2021+ | Standard |
| Polling-based retry | `backon::Retryable` trait | 2023+ | Idiomatic; Kay adopts per PROV-02 |
| Synchronous HTTP mocks | `mockito::Server::new_async` | 2023+ | Already in use in ForgeCode tree |
| Custom HTTP retry in reqwest | `reqwest-retry` middleware OR `backon` at call-site | 2023+ | Kay uses backon at call-site (no middleware wrapping) |
| OpenRouter "request any model" | Exacto variant for tool-calling | Oct 2025 per `[CITED: openrouter.ai/announcements/provider-variance-introducing-exacto]` | Kay's whole launch strategy depends on Exacto — measurable 10-20% score lift on Tau2Bench/LiveMCPBench |
| OpenRouter opt-in Exacto | Auto Exacto (default for tool-calling requests) | March 2026 per `[CITED: openrouter.ai/announcements/auto-exacto]` | Kay should STILL append `:exacto` explicitly for determinism — Auto Exacto re-evaluates providers every 5min; explicit opt-in gives reproducible benchmark runs |

**Deprecated/outdated:**
- **OAuth for LLM providers in Rust agents:** Increasingly rare; OpenAI + Anthropic + OpenRouter all converge on API-key. `PROV-03` is in line with current practice.
- **Blocking HTTP for streaming:** Dead. reqwest's `blocking` feature is fine for one-shot, but streaming is always async.
- **Regex-based JSON repair:** `forge_json_repair` uses a char-by-char state machine — correct for malformed JSON where regex fails.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `openai/gpt-5.4` exists on OpenRouter with an Exacto variant | §User Constraints D-07, §Standard Stack | MEDIUM — if it doesn't exist, TB 2.0 parity-comparable run won't use the identical model ForgeCode reported. Launch allowlist needs to be updated to `openai/gpt-5.3-chat` or the closest available Exacto-enabled GPT-5 variant. Confirm at first implementation via `GET /api/v1/models` against the live OpenRouter API. `[ASSUMED: ForgeCode's reported model ID may be forward-dated in the CONTEXT]` |
| A2 | Explicit `:exacto` suffix is still honored as of April 2026 (not silently deprecated in favor of Auto Exacto) | §State of the Art, §Pitfall 8 | LOW — the Auto Exacto announcement `[CITED: openrouter.ai/announcements/auto-exacto]` explicitly says the manual suffix still works. But OpenRouter could deprecate in future — verify by including an integration test that asserts the suffix is accepted |
| A3 | OpenRouter returns `usage.cost` as USD float on the final SSE chunk | §Pitfall 4, §Code Example 4 | MEDIUM — if cost is returned in a different field or unit, D-10's cost cap silently under/over-counts. Verify during first real streaming test by logging the raw final chunk |
| A4 | `forge_json_repair::json_repair` handles all OpenRouter tool-call malformations we'll see in practice | §Pattern 2 | MEDIUM — SPEC.md flags this as "Status: Accepted — verify with real traces." Phase 2 should add real-trace capture to its integration tests; property testing with proptest covers synthetic space | 
| A5 | ForgeCode's `forge_app/dto/openai::Request` type is what we send to OpenRouter, and its transformer pipeline is compatible with OpenRouter's exact expectations | §Don't Hand-Roll, §Architecture Pattern 3 | LOW — ForgeCode is already used against OpenRouter in production; type-compat is strongly implied by §Code Context D-02 |
| A6 | `reqwest-eventsource 0.6` has a `set_retry_policy` method | §Pitfall 6 | LOW — `[CITED: docs.rs/reqwest-eventsource/latest/reqwest_eventsource/struct.EventSource.html]` confirms a `retry` module; exact API surface confirmed in doc fetch. Verify at implementation time with a 2-line probe |
| A7 | `mockito 1.7.2` can serve SSE responses with multi-line `data: ` bodies and `text/event-stream` content type | §Pattern 4, §Code Example 4 | LOW-MEDIUM — ForgeCode's `mock_server.rs` already does this exact pattern (`mock_responses_stream` with `content-type: text/event-stream`). Reuse their helper; don't add a new mock lib |
| A8 | The bare rename + path rewrite preserves the logical behavior of forge_* code (no path change accidentally shadows a name) | §Pitfall 1 | LOW — pure syntactic refactor; `cargo test -p kay-core --lib` against pre-rename test baseline is the guard. Each subtree's tests must pass identically before and after |
| A9 | `backon::ExponentialBuilder`'s `with_jitter()` implements "full jitter" semantics (uniform random within `(0, current_delay)`) | §Code Example 3, §Pitfall 6 | LOW — `[CITED: docs.rs/backon/latest/backon/struct.ExponentialBuilder.html]` confirms "add a random jitter within (0, current_delay) to the current delay" |
| A10 | The ForgeCode-imported `ChatCompletionMessage` type does NOT need to change for Kay's delta-granular `AgentEvent` translation to work | §Architecture Pattern 3 | LOW — `ChatCompletionMessage` already has `content, tool_calls, finish_reason, usage` — exactly the fields we need. See `forge_domain/message.rs:86` |

---

## Open Questions (RESOLVED — all questions have resolutions or fallback strategies)

> **Planner revision 2026-04-20:** All six questions are resolved for Phase 2 planning purposes. Each Q now has either a concrete RESOLVED decision (with an actionable fallback) or a DEFERRED tag with a specific trigger-event that causes re-evaluation. None of the resolutions block Phase 2 execution; each either commits the plan to one branch of a prior fork or documents why the defer-and-observe path is safe (typed fallbacks cover the adversary cases).

1. **`openai/gpt-5.4` availability on OpenRouter Exacto — RESOLVED (provisional + live-verification strategy)**
   - What we know: ForgeCode's public TB 2.0 81.8% run allegedly used `openai/gpt-5.4`. OpenRouter's Exacto docs list up to `openai/gpt-5.3-chat` as of the latest page fetch (2026-04-20).
   - What's unclear: Does gpt-5.4 exist on OpenRouter as an Exacto target? Or is the ForgeCode CONTEXT reference forward-dated?
   - **RESOLUTION:** Provisional choice per D-07 — the allowlist fixture (`crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json`) carries `openai/gpt-5.4`. Runtime verification happens during EVAL-01a when the OpenRouter API key is provisioned (per Phase 1 D-OP-01 — deferred until live account setup). If a `GET /api/v1/models` call confirms the ID is absent, the documented fallback is `openai/gpt-5.3-chat`; the fallback is a one-line config edit (no code change) because D-07's config surface is `kay.toml [provider.openrouter] allowed_models` mergeable with the `KAY_ALLOWED_MODELS` env var. This resolution is safe for Phase 2 execution because: (a) Phase 2 uses mock fixtures, not live traffic; (b) the typed-error path `ProviderError::ModelNotAllowlisted` gates any non-allowlisted request BEFORE HTTP; (c) the TB 2.0 parity run is EVAL-01a follow-on work, not Phase 2 scope. The fixture carries an inline comment documenting the provisional status — see the WARNING-tagged fix to plan 02-01.

2. **Behavior of OpenRouter `:exacto` suffix on a non-Exacto-supported model — DEFERRED (to first real trace; typed fallback covers adversary case)**
   - What we know: `[CITED: openrouter.ai/docs/guides/routing/model-variants/exacto]` doesn't specify.
   - What's unclear: Silent fallback to non-Exacto? 400 error? 503?
   - **RESOLUTION:** DEFERRED to first real EVAL-01a trace. The typed error taxonomy already covers every adversary shape: if OpenRouter returns 400, our `classify_http_error` maps to `ProviderError::Http { status: 400, body }`; if 503, `ProviderError::ServerError`; if silent fallback, our `allowlist.check()` gate has ALREADY rejected (gate fires BEFORE HTTP). The combination means there is no "silently-wrong" path — worst case the user sees a typed diagnostic they can act on. `error_taxonomy.rs` integration test (plan 02-10) asserts the taxonomy is complete regardless of which branch OpenRouter chooses.

3. **Exact shape of mid-stream SSE error events — DEFERRED (parser is defensive; typed fallback covers unknown shape)**
   - What we know: `[CITED: openrouter.ai/docs/api/reference/errors-and-debugging]` says mid-stream errors arrive as SSE with `error` field at top level + `choices[].finish_reason = "error"` on HTTP 200.
   - What's unclear: The JSON shape of the error field, error codes, whether it's the same shape as 4xx body errors.
   - **RESOLUTION:** DEFERRED to first real trace capture (EVAL-01a follow-on). The parser is defensive by construction: (a) tolerant two-pass parser (D-03 + plan 02-09) catches any unknown JSON shape and emits `AgentEvent::ToolCallMalformed` rather than panicking; (b) top-level parse failures in the translator surface as `ProviderError::Stream(format!("parse chunk failed: {e}"))` with the raw chunk text preserved for diagnostics; (c) an unknown `error` field at top-level simply fails top-level deserialization of our `forge_app::dto::openai::Response` struct, which is handled by the Stream branch. This means NO panic path exists even without the exact shape — the worst case is a non-informative diagnostic message. When a real trace is captured post-EVAL-01a, an opportunistic SUMMARY.md note (or follow-on ticket) refines `classify_http_error` to pattern-match the observed error.code values; until then plan 02-10's `error_taxonomy.rs` tests synthetic shapes which is sufficient for PROV-08 coverage.

4. **Retry-After header format: integer seconds only, or HTTP-date too? — RESOLVED (integer seconds per RFC 7231 common case)**
   - What we know: RFC 7231 allows both.
   - What's unclear: OpenRouter's actual choice in practice.
   - **RESOLUTION:** Integer seconds only. Plan 02-10's `parse_retry_after()` returns `None` for non-numeric values (including HTTP-date form), in which case `backon`'s default schedule applies — matching the real-world 90%+ integer-seconds case per RFC 7231 §7.1.3 common practice. If a real trace post-EVAL-01a exposes HTTP-date form, a single-commit follow-on adds the `httpdate` crate; meanwhile the `parse_retry_after_date_format_returns_none` unit test in plan 02-10 Task 1 asserts the fallback behavior is deterministic. No current blocker.

5. **Tokio runtime choice for cost-cap accumulator — RESOLVED (`std::sync::Mutex<f64>`)**
   - What we know: `std::sync::Mutex<f64>` works but blocks; `tokio::sync::Mutex` is async-safe but contends on `.await`.
   - What's unclear: Contention profile under real load.
   - **RESOLUTION:** `std::sync::Mutex<f64>` (see plan 02-10 T1 `cost_cap.rs`). Both critical sections — `check()` and `accumulate()` — are O(1) and hold the lock for <<1µs; no `.await` points are inside the lock scope. Contention is theoretically possible only if multiple concurrent `chat()` calls run in the same process, which Phase 2's single-session scope does not exercise. Migration trigger: if Phase 5's parallel-agents work (LOOP-05) shows contention in profiling, swap to `AtomicU64` with `f64::from_bits`/`to_bits` conversions — at that point it becomes a one-file localized change. Not a Phase 2 concern.

6. **Interaction between `--max-usd` and Harbor cap during EVAL-01a — RESOLVED (non-interference by design)**
   - What we know: D-10 says no default cap; Harbor/TB 2.0 injects its own cap.
   - What's unclear: When EVAL-01a runs, does our default-uncapped behavior interfere with Harbor's own budget tracking?
   - **RESOLUTION:** Non-interference by design. Per D-10, Kay's `--max-usd` defaults to `CostCap::uncapped()` when unset, so Harbor's external cap remains authoritative when EVAL-01a runs against the Harbor harness. Kay's cap applies across turns WITHIN a task (intra-task budgeting); Harbor's applies across tasks (inter-task budgeting). They compose additively without shared state. When both are set, Harbor's enforcement happens at Harbor's layer (outside Kay's process), and Kay's happens inside the Kay process — no shared accumulator means no double-count, no race. Documentation of the composition story goes into ARCH.md as part of the EVAL-01a follow-on kickoff checklist.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable 1.95 | Workspace compile | ✓ (rust-toolchain.toml pinned) | 1.95 | — |
| tokio 1.51 | Runtime | ✓ (workspace pin) | 1.51 | — |
| reqwest 0.13 | HTTP client | ✓ (workspace pin) | 0.13 | — |
| reqwest-eventsource 0.6 | SSE wrapper | ✓ (workspace pin) | 0.6 | — |
| rustls 0.23 | TLS | ✓ (workspace pin) | 0.23 | — |
| OpenRouter API access | Live integration tests | ✗ (deferred to EVAL-01a per Phase 1 D-OP-01) | — | **Mock-only integration tests in Phase 2.** Live tests gated behind `KAY_OPENROUTER_LIVE=1` env var + `--ignored` cargo test flag — skipped by default, runnable when key is available |
| backon 1.6.0 | Retry | ✓ (published) | 1.6.0 | — |
| mockito 1.7.2 | Mock HTTP server (tests) | ✓ (published) | 1.7.2 | — |
| proptest 1.11.0 | Property tests | ✓ (published) | 1.11.0 | — |
| macOS / Linux / Windows CI matrix | `cargo check --workspace --deny warnings` | ✓ (Phase 1 ci.yml) | — | — |

**Missing dependencies with no fallback:** None — Phase 2 is purely local.

**Missing dependencies with fallback:**
- OpenRouter API key: mock-only integration tests cover all functional paths; live tests are deferred and gated.

---

## Validation Architecture

> `workflow.nyquist_validation` is `true` in `.planning/config.json` — include this section.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` + `proptest!` macro |
| Config file | `Cargo.toml [dev-dependencies]` per crate |
| Quick run command | `cargo test -p kay-provider-openrouter --lib` (unit tests only, no mock server) |
| Full suite command | `cargo test --workspace --all-features` (integration tests spin up mock servers) |
| Phase gate | `cargo check --workspace --deny warnings` + `cargo test --workspace --all-features` — both must be green before /gsd-verify-work |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROV-01 | Provider::chat streams AgentEvent frames from SSE | Integration | `cargo test -p kay-provider-openrouter --test streaming_happy_path -- --nocapture` | ❌ Wave 0 |
| PROV-02 | reqwest 0.13 + reqwest-eventsource + backon wired into chat() | Unit + Integration | `cargo test -p kay-provider-openrouter --lib transport` | ❌ Wave 0 |
| PROV-03 | Env var wins over config; missing key → typed error | Integration | `cargo test -p kay-provider-openrouter --test auth_env_vs_config` | ❌ Wave 0 |
| PROV-04 | Non-allowlisted model → ModelNotAllowlisted before HTTP | Integration (assert zero HTTP hits) | `cargo test -p kay-provider-openrouter --test allowlist_gate` | ❌ Wave 0 |
| PROV-05 (reassembly) | Fragmented tool_calls aggregated to ToolCallComplete | Integration | `cargo test -p kay-provider-openrouter --test tool_call_reassembly` | ❌ Wave 0 |
| PROV-05 (tolerance) | Malformed args → pass-2 repair OR ToolCallMalformed, never panic | Integration | `cargo test -p kay-provider-openrouter --test tool_call_malformed` | ❌ Wave 0 |
| PROV-05 (property) | Parser never panics on arbitrary input | Property (proptest inline in `src/tool_parser.rs`'s `#[cfg(test)] mod unit`, per plan 02-09 T3 BLOCKER #4 revision) | `cargo test -p kay-provider-openrouter --lib tool_parser::unit -- --nocapture` | ❌ Wave 0 |
| PROV-06 | Turn N+1 rejected after cap; turn N finishes | Integration | `cargo test -p kay-provider-openrouter --test cost_cap_turn_boundary` | ❌ Wave 0 |
| PROV-07 | 429 w/ Retry-After respected; exactly 3 retries; Retry event emitted | Integration | `cargo test -p kay-provider-openrouter --test retry_429_503` | ❌ Wave 0 |
| PROV-08 | Each ProviderError variant maps from upstream (401→Auth, 402→Http, 429→RateLimited, 502/503→ServerError, transport→Network) | Integration | `cargo test -p kay-provider-openrouter --test error_taxonomy` | ❌ Wave 0 |
| D-01 (rename) | `cargo check --workspace --deny warnings` passes on macOS/Linux/Windows without --exclude kay-core | Build gate | `cargo check --workspace --deny warnings` | ❌ CI update needed |

### Sampling Rate

- **Per task commit:** `cargo check -p kay-core && cargo check -p kay-provider-openrouter && cargo test -p kay-provider-openrouter --lib` (< 30s)
- **Per wave merge:** `cargo test -p kay-provider-openrouter --all-features` (includes all mock-server integration tests, < 2 min)
- **Per subtree rename commit (D-01 Step 2):** `cargo check -p kay-core` (< 15s)
- **Phase gate:** full `cargo test --workspace --all-features` green + `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo deny check` (all 4 previously restricted to `--exclude kay-core`, all must pass full workspace after D-01)

### Validation Dimensions Applied to Phase 2

| Dimension | Applicability | Implementation |
|-----------|---------------|----------------|
| **Unit tests** | HIGH — typed event construction, error mapping, allowlist normalization, model ID rewriting | `cargo test -p kay-provider-openrouter --lib` — each module has inline `#[cfg(test)]` |
| **Integration tests against mock server** | HIGH — the Provider trait's whole contract is observable only through streaming | `tests/*.rs` — each file uses MockServer to assert SSE-shape-to-AgentEvent mapping |
| **Property tests on tolerant parser** | HIGH — PROV-05 demands never-panic on arbitrary input | Proptest INLINE in `src/tool_parser.rs`'s `#[cfg(test)] mod unit` (per plan 02-09 T3 BLOCKER #4 revision — `parse_tool_arguments` is crate-private, so inline-unit-module proptest is the clean path); fuzz invariants: `parser_never_panics`, `well_formed_json_always_clean` |
| **Contract tests for Provider trait** | MEDIUM — Phase 5 will add more consumers; Phase 2's contract stability is important. Use `#[must_use]` on return types + `#[non_exhaustive]` on AgentEvent/ProviderError | Inline in trait docs; runtime check via `compile_fail` doctests for non-exhaustive matching |
| **Streaming backpressure tests** | LOW-MEDIUM — reqwest-eventsource + tokio channels handle backpressure naturally; caller must poll for more events. Can defer formal backpressure test to Phase 5 when the agent loop really exercises sustained streams | Not a Phase 2 test — documented as "verified at Phase 5 integration" |
| **Error-taxonomy coverage tests** | HIGH — PROV-08 is explicit | `tests/error_taxonomy.rs` covers each variant via targeted mock responses |
| **Live integration tests (gated)** | LOW priority for Phase 2 — EVAL-01a handles the full live run | `#[ignore]` tests runnable via `cargo test --ignored` + `KAY_OPENROUTER_LIVE=1`; single smoke test per PROV requirement |
| **Parity regression test** | HIGH — kay-core must still pass its imported tests after D-01 rename | Run `cargo test -p kay-core --all` before AND after D-01; diff the test counts + failure lists. Expected: identical results |

### Wave 0 Gaps

- [ ] `crates/kay-provider-openrouter/tests/mock_server.rs` — shared test helper extending `forge_repo::provider::mock_server::MockServer` with `mock_openrouter_chat_stream(events, status)` + OpenAPI-compatible SSE event serializer
- [ ] `crates/kay-provider-openrouter/tests/fixtures/openrouter_*.jsonl` — captured SSE event sequences for: happy-path, tool-call-fragmented, tool-call-malformed, 429-with-retry-after, 503, mid-stream-error, empty-usage
- [ ] `crates/kay-provider-openrouter/tests/allowlist_gate.rs` — Wave 0 test skeleton (no impl yet; test compiles with `todo!()` body, becomes real in the implementation task)
- [ ] `.github/workflows/ci.yml` update: remove `--exclude kay-core` from clippy + test + fmt; wait for D-01 Step 2 to land first
- [ ] `CONTRIBUTING.md §Pull Request Process` update: remove the per-crate `cargo fmt -p <...>` list; use `cargo fmt --all -- --check`
- [ ] `docs/CICD.md §Current State` update: `cargo check --workspace --deny warnings` passes on kay-core post-Phase-2
- [ ] `.planning/STATE.md` blocker update: remove "kay-core 23 × E0583" after D-01 completes

---

## Security Domain

Phase 2 introduces the first external-network boundary and the first credential-handling code in Kay. Security concerns are highest-density for this phase.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | YES | API-key auth via env var / config file (PROV-03); no password flows, no OAuth, no keyring-at-rest in Phase 2 |
| V3 Session Management | Partial | `cost_cap` accumulator is per-process session state; no persistent session tokens. Phase 6 adds real session state |
| V4 Access Control | YES | Model allowlist is an access-control gate; enforced BEFORE any HTTP egress |
| V5 Input Validation | YES | Tool-call JSON parsing (PROV-05) is the attack surface for provider-side injection; two-pass parser + non-panicking guarantee |
| V6 Cryptography | YES | rustls 0.23 for TLS only; we do NOT hand-roll crypto anywhere. openssl explicitly banned per `deny.toml` |
| V7 Error Handling & Logging | YES | Typed `ProviderError` must NOT leak API keys in logs. Redaction in `Debug`/`Display` for `AuthErrorKind` |
| V8 Data Protection | Partial | API key in memory only during session; not serialized to transcripts or logs |
| V9 Communication | YES | OpenRouter over HTTPS; rustls pins cert chain via system trust store |
| V10 Malicious Code | Partial | Tool-call arguments from provider are NOT executed in Phase 2 — Phase 3's `execute_commands` is the concern there |
| V13 API & Web Service | YES | OpenRouter is the external API; our requests are well-formed JSON; responses parsed defensively |
| V14 Configuration | YES | `kay.toml` is user-managed config; env var precedence; no secrets-in-git (Phase 1 CONTRIBUTING.md §DCO already covers) |

### Known Threat Patterns for Rust Async HTTP + Streaming Providers

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API-key exfiltration via `Debug` derive on error types | Information Disclosure | Custom `Debug` impl that redacts key field; `#[derive(Debug)]` on structs containing keys is a lint-level ban |
| API-key leak in `panic!` message / `unwrap()` trace | Information Disclosure | Never panic in the request-build path; forbid `unwrap()` / `expect()` on anything that touches credentials |
| Man-in-the-middle on OpenRouter endpoint | Spoofing / Tampering | rustls with default system trust store; no custom cert-trust override in Phase 2. Cert pinning (via rustls's `WebPkiServerVerifier`) is a hardening backlog item — not Phase 2 |
| Cost-cap bypass via concurrent requests | Repudiation / Denial of Service (self-DoS) | Single `Mutex<f64>` on the accumulator; concurrent `chat()` calls serialize on the cap check |
| Model allowlist bypass (case sensitivity / suffix manipulation) | Elevation of Privilege | Normalize to lowercase + strip `:exacto` before compare; allowlist canonical form enforced at load |
| Tool-call argument injection (provider returns JSON designed to exploit consumer) | Tampering / Injection | Phase 2 emits `ToolCallComplete { arguments: serde_json::Value }` — the consumer (Phase 3) is responsible for schema-validating against the Tool's input_schema before invoking. Phase 2's contract: tolerate any JSON, never execute |
| SSRF via model-name injection | SSRF | Model IDs validated against allowlist BEFORE any URL construction. Model is a request-body field, not a URL segment, on OpenRouter — no URL injection surface |
| Denial-of-service via unbounded in-memory tool-call buffer | Denial of Service | Set a max size on `ToolCallBuilder::arguments_raw` (e.g., 1 MB); exceed → emit `ToolCallMalformed` with diagnostic; abort the turn |
| Mid-stream error smuggling (provider-injected malformed SSE to trigger parser panic) | Tampering | PROV-05 never-panic invariant + proptest fuzz + `forge_json_repair` fallback |
| Connection persistence leak (long-running stream leaks FDs) | DoS | reqwest's default pool limits; cap in Phase 2 config if needed; `EventSource::close()` on every error path |

---

## Threat Model Seeds

> Seeds for the planner to expand into each PLAN.md's `<threat_model>` block.

### TM-01: API Key Leakage

**Vector:** Log statements, panic messages, error `Debug` output, transcript files (Phase 6 concern but set a pattern now).
**Impact:** Adversary can bill the key owner, access paid models, exhaust TB 2.0 budget.
**Mitigation:**
- Custom `Debug` impl on `AuthErrorKind::InvalidKey(ApiKey)` — redacts to `"<redacted>"` always.
- Forbid `.unwrap()` on `api_key` in the entire `kay-provider-openrouter` crate; use `ok_or(ProviderError::Auth { reason: ... })` everywhere.
- `tracing` log calls in the crate use `%redacted_key` placeholder, never `{key}`.
- Static verification: a `#[deny(clippy::unwrap_used)]` on the crate root + a git-grep check in CI for `println!.*api_key` patterns.

### TM-02: MitM on OpenRouter Endpoint

**Vector:** Compromised CA, hostile proxy, rogue DNS.
**Impact:** Credential exfiltration; prompt/response tampering; cost manipulation.
**Mitigation:**
- rustls 0.23 default system trust store (Phase 2 baseline).
- Future hardening (backlog): pin OpenRouter's cert fingerprint via `rustls::WebPkiServerVerifier`. Not Phase 2 — adds complexity for a low-probability threat on a managed provider's main domain.
- HSTS preload — OpenRouter is in Chrome's HSTS preload list (verify via `chrome://net-internals/#hsts`) reducing MitM risk for users on Chrome-backed environments.

### TM-03: Cost-Cap Bypass via Concurrent Requests

**Vector:** User spawns multiple concurrent agent sessions against the same provider; each sees `spent < cap` at its own moment of check.
**Impact:** Blow through budget; unexpected billing.
**Mitigation:**
- Single `Mutex<f64>` on the accumulator; check and update atomically on entry to each `chat()` call.
- Document: cost cap is best-effort against concurrent requests; TB 2.0 Harbor provides its own per-task budget, so the concurrent-abuse case is a user-shoots-foot scenario, not an external attack.
- Phase 3+ will likely add rate limiting on sessions-per-process, narrowing the concurrent-abuse window.

### TM-04: Model Allowlist Bypass

**Vector:** Case-insensitivity bugs, trailing-whitespace tolerance, case-smuggling in model IDs, CRLF injection in HTTP request body.
**Impact:** User requests a non-allowlisted expensive model; cost cap fires but after HTTP request is already billed (since cap is turn-boundary).
**Mitigation:**
- Normalize model ID: `.to_ascii_lowercase().trim().trim_end_matches(":exacto")` before compare.
- Reject any model ID containing `\r`, `\n`, `\t`, or non-ASCII; explicit deny before allowlist check.
- Allowlist entries are normalized at config-load time; `provider.json` allowlist is canonicalized once at parse.
- Property test: `proptest` fuzz allowlist check with arbitrary strings; invariant is "either accept or reject; never panic."

### TM-05: Tool-Call Argument Injection

**Vector:** Malicious provider (or compromised upstream) returns a JSON arguments payload specifically crafted to exploit the consumer (e.g., `{"cmd": "rm -rf /", "_schema_bypass": true}`).
**Impact:** Phase 3's `execute_commands` runs arbitrary shell if schema validation is weak.
**Mitigation:**
- Phase 2 does NOT execute tool calls — it only parses and emits. Phase 3's consumer MUST validate against `Tool::input_schema` before invocation.
- `AgentEvent::ToolCallComplete { arguments: serde_json::Value }` — raw Value, never auto-typed. This forces the Phase 3 consumer to explicitly validate.
- Document in Provider trait doc: "Tool-call arguments are opaque JSON. Consumer is responsible for schema validation before invocation."

### TM-06: SSE Parser DoS

**Vector:** Adversary-controlled SSE stream with infinite chunks, oversized fields, or deeply nested JSON designed to blow the stack.
**Impact:** Memory exhaustion, parser panic, process crash.
**Mitigation:**
- `ToolCallBuilder::arguments_raw` capped at 1 MB (config-configurable); exceed → emit `ToolCallMalformed`.
- `serde_json` has built-in recursion depth limits (128 by default); any JSON deeper triggers a parse error → emit `ToolCallMalformed` via fall-back.
- `forge_json_repair` is iterative, not recursive — verify at implementation time (read parser.rs) that it can't be made to infinite-loop on crafted input.
- Total chunks per call capped (backlog; Phase 5 adds timeouts per turn).

### TM-07: Weak Error-Path Coverage

**Vector:** Untested error paths silently panic or leak state.
**Impact:** Fuzzy bugs in prod; silent data loss.
**Mitigation:**
- PROV-08 demands typed errors for all failure modes — integration test `error_taxonomy.rs` covers each variant with a targeted mock.
- Property test on parser ensures never-panic invariant.
- `deny(clippy::unwrap_used)` + `deny(clippy::expect_used)` at crate root for the provider (not test code).

### TM-08: Confused-Deputy via Non-Exacto Model Smuggling

**Vector:** User passes `anthropic/claude-sonnet-4.6:exacto` in some surface, `anthropic/claude-sonnet-4.6` in another; allowlist stores one canonical form and normalizes for compare, but the actual HTTP request sent to OpenRouter uses the user's raw form.
**Impact:** Falls through Exacto gating, inflates provider-variance risk.
**Mitigation:** Canonical form storage + ALWAYS-append `:exacto` on request body construction (see Pitfall 8). The user's input form is irrelevant to what goes on the wire.

---

## Sources

### Primary (HIGH confidence)

- **OpenRouter API Docs — Streaming:** https://openrouter.ai/docs/api/reference/streaming — SSE final-chunk usage, error via SSE event, `finish_reason: "error"` semantics
- **OpenRouter API Docs — Exacto Variant:** https://openrouter.ai/docs/guides/routing/model-variants/exacto — `:exacto` suffix usage; virtual variant, shared endpoint pool
- **OpenRouter API Docs — Errors & Debugging:** https://openrouter.ai/docs/api/reference/errors-and-debugging — 4xx/5xx taxonomy, mid-stream error shape, 429 Retry-After
- **OpenRouter Announcements — Provider Variance / Exacto:** https://openrouter.ai/announcements/provider-variance-introducing-exacto — 10-20% score lift data
- **OpenRouter Announcements — Auto Exacto:** https://openrouter.ai/announcements/auto-exacto — March 2026 default; explicit `:exacto` suffix still honored
- **docs.rs/backon/ (1.6.0):** https://docs.rs/backon/latest/backon/struct.ExponentialBuilder.html — ExponentialBuilder API: `with_factor`, `with_min_delay`, `with_max_delay`, `with_max_times`, `with_jitter`; jitter semantics ("add a random jitter within (0, current_delay)")
- **docs.rs/reqwest-eventsource/ (0.6.0):** https://docs.rs/reqwest-eventsource/ — Stream<Item = Result<Event, Error>>; retry module + `set_retry_policy`
- **docs.rs/mockito/ (1.7.2):** https://docs.rs/mockito — `Server::new_async`, `with_body`, `with_header`; no native SSE feature but existing ForgeCode pattern covers it
- **crates.io API (2026-04-20):** verified versions for backon (1.6.0), reqwest-eventsource (0.6.0), mockito (1.7.2), rvcr (0.2.1), wiremock (0.6.5), async-trait (0.1.89), futures (0.3.32), proptest (1.11.0)
- **Kay source tree (grep-verified):**
  - `crates/kay-core/src/forge_json_repair/lib.rs` — exposes `json_repair` + `coerce_to_schema` + `JsonRepairError`
  - `crates/kay-core/src/forge_json_repair/parser.rs:3` — shows `use crate::error` (the structural rename issue)
  - `crates/kay-core/src/forge_repo/provider/openai.rs:4-19` — shows `use forge_app::domain::*` and `use crate::provider::event::*` (both kinds of broken paths)
  - `crates/kay-core/src/forge_repo/provider/provider.json` — OpenRouter config: `https://openrouter.ai/api/v1/chat/completions`, `OPENROUTER_API_KEY`
  - `crates/kay-core/src/forge_repo/provider/mock_server.rs:1-78` — mockito-based MockServer; extensible for OpenRouter SSE
  - `crates/kay-core/src/forge_domain/message.rs:86-97` — `ChatCompletionMessage` with content/tool_calls/finish_reason/usage
  - `crates/kay-core/src/forge_domain/message.rs:24-31` — `Usage` struct with `cost: Option<f64>` (USD)

### Secondary (MEDIUM confidence — verified with ≥2 sources)

- **Rust Blog — Async fn and RPIT in Traits:** https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/ — 1.75+ native async trait + async_trait crate for dyn
- **Rust Blog — Impl Trait in Rust 2024:** https://blog.rust-lang.org/2024/09/05/impl-trait-capture-rules/ — 2024 edition capture rules
- **Rust-Magazine — Backon API Design:** https://rustmagazine.org/issue-2/how-i-designed-the-api-for-backon-a-user-friendly-retry-crate/ — Retryable trait ergonomics
- **Tokio Docs — Streams:** https://tokio.rs/tokio/tutorial/streams — `Pin<Box<dyn Stream + Send + 'a>>` pattern

### Tertiary (LOW confidence — single source, flagged for validation)

- Exacto availability of `openai/gpt-5.4` — the model ID appears in Kay's CONTEXT.md but OpenRouter's current Exacto page lists up to gpt-5.3-chat. See §Open Questions Q1 (RESOLVED — provisional per D-07 + live-verify strategy + one-line fallback to gpt-5.3-chat). `[ASSUMED]`
- OpenRouter mid-stream error JSON shape — documented structure is high-level; exact error.code taxonomy requires a real trace capture. See §Open Questions Q3 (DEFERRED — parser is defensive, typed fallback covers unknown shape). `[ASSUMED]`

---

## Metadata

**Confidence breakdown:**
- Structural rename scope (D-01 is insufficient): HIGH — directly verified via 379 + 340 grep counts
- ForgeCode delegation strategy (D-02): HIGH — `openai.rs` line 51 + `provider_repo.rs` tests confirm OpenRouter native support
- `forge_json_repair` reuse (D-03): HIGH — parser is in-tree, signature confirmed
- Provider trait shape: HIGH — async_trait pattern is idiomatic; ForgeCode's own `ChatRepository` follows the same
- Cost cap timing (D-10): MEDIUM — `[CITED: OpenRouter streaming docs]` confirms usage on final chunk; exact frame position needs one integration test to lock down
- 429/503 retry semantics: MEDIUM-HIGH — Retry-After header confirmed in multiple sources; exact header format (integer vs date) needs first-real-trace confirmation
- Exacto `openai/gpt-5.4`: LOW — assumed per CONTEXT.md; needs live OpenRouter `/models` verification
- Mock server SSE support: HIGH — ForgeCode's existing `mock_server.rs` pattern covers the exact shape

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (30 days for stable; dependency versions and crate availability refresh at the 30-day mark)

---

*Phase: 02-provider-hal-tolerant-json-parser*
*Research completed: 2026-04-20*
*Researcher: gsd-phase-researcher*
