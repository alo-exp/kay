# Phase 2: Provider HAL + Tolerant JSON Parser — Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 4 tier-1 structural, 8 tier-2 public API, 1 tier-3 adapter, 4 tier-4 parser/retry/cost, 9 tier-5 test, 3 tier-6 CI/docs = 29 logical units (≈895 file modifications inclusive of the mechanical path-rewrite class)
**Analogs found:** 11 strong in-tree analogs / 29 — the three always-new tiers (allowlist, cost_cap, and the typed trait itself) have no direct analog

---

## Orientation

Phase 2's work splits into four tiers + two meta tiers:

| Tier | Scope | File count | Analog strategy |
|------|-------|-----------|------------------|
| 1. Structural rename + path rewrite | `crates/kay-core/src/forge_*/{lib.rs → mod.rs}` for 23 subtrees; `use crate::X → use crate::forge_Y::X` (~379 intra-subtree); `use forge_Y::Z → use crate::forge_Y::Z` (~340 inter-subtree) | 23 renames + ~720 edits across ~396 files | Pattern-per-class (there IS no in-tree analog; analog is CODE itself — current state is the analog) |
| 2. Public API surface | `kay-provider-openrouter`: `provider.rs`, `event.rs`, `error.rs` | 3 | Analog: `forge_domain/repo.rs` (trait shape), `forge_domain/error.rs` (enum + thiserror), `forge_repo/provider/event.rs` (stream translator shape) |
| 3. Adapter implementation | `kay-provider-openrouter`: `OpenRouterProvider` inside `provider.rs`; `translator.rs`, `retry.rs` | 3 | Analog: `forge_services/provider_service.rs` (delegation pattern), `forge_repo/provider/openai.rs` (the thing being delegated TO), `forge_repo/provider/retry.rs` (retry semantics) |
| 4. Parser + cost + allowlist | `kay-provider-openrouter`: `tool_parser.rs`, `cost_cap.rs`, `allowlist.rs` | 3 | Analog: `forge_json_repair::json_repair` (pass-2 parser), `forge_repo/provider/provider.json` (config-as-data), `forge_repo/provider/retry.rs::RetryableApiErrorCode` (enum-of-conditions shape). **`cost_cap.rs` and `allowlist.rs` have NO analog** — new greenfield logic |
| 5. Test fixtures | `kay-provider-openrouter/tests/*.rs` + `tests/fixtures/*.jsonl` | 9 | Analog: `forge_repo/provider/mock_server.rs` (mockito-based SSE helper), `forge_repo/provider/openai.rs` tests (`#[tokio::test]` + `MockServer::new_async`) |
| 6. CI/governance updates | `.github/workflows/ci.yml`, `CONTRIBUTING.md`, `docs/CICD.md`, `.planning/STATE.md` | 4 | Pattern: "remove `--exclude kay-core` escape clause"; no code analog needed |

---

## File Classification

### Tier 1: Structural — `kay-core` rename + path rewrite

| File class | Role | Data Flow | Closest Analog | Match Quality |
|-----------|------|-----------|----------------|---------------|
| `crates/kay-core/src/forge_X/lib.rs → mod.rs` (×23) | structural-rename | none (compile-time only) | **No in-tree analog** — 23 subtrees ARE the pattern | n/a (mechanical) |
| `forge_X/**/*.rs` intra-subtree `use crate::Y` → `use crate::forge_X::Y` (~379 edits in 195 files) | path-rewrite | none (compile-time only) | **Current broken state IS the analog** — grep the pattern, sed the replacement | n/a (mechanical) |
| `forge_X/**/*.rs` inter-subtree `use forge_Y::Z` → `use crate::forge_Y::Z` (~340 edits in 201 files) | path-rewrite | none (compile-time only) | **Current broken state IS the analog** — `ast-grep` or `sd` per subtree | n/a (mechanical) |

### Tier 2: Public API surface (`kay-provider-openrouter`)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/kay-provider-openrouter/src/lib.rs` | facade-index | re-export | `crates/kay-core/src/forge_services/lib.rs` | role-match (module declarations + `pub use`) |
| `crates/kay-provider-openrouter/src/provider.rs` (trait) | trait-definition | request-response + streaming | `crates/kay-core/src/forge_domain/repo.rs:92-101` (`ChatRepository`) | exact (async_trait + ResultStream) |
| `crates/kay-provider-openrouter/src/event.rs` | domain-type (enum) | stream-item | `crates/kay-core/src/forge_domain/message.rs:86-97` (`ChatCompletionMessage`) — compare-and-contrast (our `AgentEvent` is delta-granular; theirs is aggregated) | partial (same domain, different granularity) |
| `crates/kay-provider-openrouter/src/error.rs` | domain-type (enum) | error-return | `crates/kay-core/src/forge_domain/error.rs:1-107` (`Error` enum + thiserror) | role-match (enum + thiserror), but non_exhaustive semantics new |

### Tier 3: Adapter implementation

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/kay-provider-openrouter/src/provider.rs` (`OpenRouterProvider` impl) | adapter-wrapper | request-response + streaming | `crates/kay-core/src/forge_services/provider_service.rs:16-135` (`ForgeProviderService`) | exact (same wraps-a-repository delegation pattern) |
| `crates/kay-provider-openrouter/src/translator.rs` | stream-transform | transform (map `ChatCompletionMessage` → `AgentEvent`) | `crates/kay-core/src/forge_repo/provider/event.rs:1-83` (`into_chat_completion_message`) | exact (same Stream-map pattern, different types) |
| `crates/kay-provider-openrouter/src/retry.rs` | retry-logic | decorator | `crates/kay-core/src/forge_repo/provider/retry.rs:9-29` (`into_retry`) | role-match (ours uses backon + Retry-After, theirs uses status-code dispatch — same goal, different mechanism) |

### Tier 4: Parser + cost + allowlist

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/kay-provider-openrouter/src/tool_parser.rs` | parser (two-pass wrapper) | transform (raw → Value or Malformed) | `crates/kay-core/src/forge_json_repair/parser.rs:1070-1073` (`json_repair`) — entry point we wrap | exact (we LITERALLY wrap this function) |
| `crates/kay-provider-openrouter/src/cost_cap.rs` | accumulator + gate | event-driven (consumes Usage events) | **No in-tree analog** — pure greenfield | none (new logic) |
| `crates/kay-provider-openrouter/src/allowlist.rs` | validator (config + gate) | request-response (pre-HTTP check) | `crates/kay-core/src/forge_repo/provider/provider.json:46-54` (config shape); `forge_repo/provider/retry.rs:71-87` (enum-of-conditions `RetryableApiErrorCode`) | partial (config shape from provider.json; enum shape from retry.rs; but allowlist *gating* is new) |

### Tier 5: Test fixtures

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/kay-provider-openrouter/tests/mock_server.rs` (shared helper) | test-fixture | mock HTTP | `crates/kay-core/src/forge_repo/provider/mock_server.rs:1-91` (`MockServer`) | exact (extend this pattern; add `mock_openrouter_chat_stream`) |
| `crates/kay-provider-openrouter/tests/streaming_happy_path.rs` | test-integration | request-response + streaming | `crates/kay-core/src/forge_repo/provider/openai.rs` test module (`#[tokio::test]` + `MockServer::new()`) | exact (same test shape) |
| `crates/kay-provider-openrouter/tests/tool_call_reassembly.rs` | test-integration | streaming + reassembly | extends `mock_server.rs` helper + RESEARCH §Example 4 | partial (no direct analog of fragmented tool_calls) |
| `crates/kay-provider-openrouter/tests/tool_call_malformed.rs` | test-integration | error-path | extends `mock_server.rs`; tests `forge_json_repair::json_repair` fallback | role-match |
| `crates/kay-provider-openrouter/tests/tool_call_property.rs` | test-property (proptest) | fuzz | RESEARCH §Example 5 (no in-tree proptest analog — new dep) | none (proptest new) |
| `crates/kay-provider-openrouter/tests/retry_429_503.rs` | test-integration | error + retry | `crates/kay-core/src/forge_repo/provider/retry.rs` test module (lines 148-392) | partial (they test `into_retry`, we test end-to-end retry loop) |
| `crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs` | test-integration | state machine | **No in-tree analog** | none |
| `crates/kay-provider-openrouter/tests/auth_env_vs_config.rs` | test-integration | config resolution | `crates/kay-core/src/forge_repo/provider/provider_repo.rs:586-609` (`test_load_provider_configs`) | role-match (config test shape) |
| `crates/kay-provider-openrouter/tests/allowlist_gate.rs` | test-integration | pre-request gate | **No in-tree analog** | none |
| `crates/kay-provider-openrouter/tests/error_taxonomy.rs` | test-integration | error mapping | `crates/kay-core/src/forge_repo/provider/retry.rs` test module (lines 191-392) | role-match (error-variant coverage pattern) |

### Tier 6: CI / docs / governance

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `.github/workflows/ci.yml` (lines 83, 116) | CI-config | build gate | **No code analog; text-replacement** — remove `--exclude kay-core` | n/a (text edit) |
| `CONTRIBUTING.md` (line 50) | docs | — | **Text-replacement** — same as above | n/a |
| `docs/CICD.md` §Current State | docs | — | **Text-replacement** | n/a |
| `.planning/STATE.md` (Phase 2 blocker) | state | — | **Text-replacement** | n/a |

---

## Pattern Assignments

### Tier 1 Meta-Pattern: Structural rename + path rewrite (mechanical, no analog)

**Scope:** 23 subtrees + ~720 import statements across ~396 files.

**Why no in-tree analog:** The repo has never done this before — Phase 1 imported the ForgeCode tree verbatim and deliberately deferred the rename to Phase 2.

**Canonical transformation (per subtree):**

```bash
# Step 1: rename the module root (one git mv per subtree)
git mv crates/kay-core/src/forge_X/lib.rs crates/kay-core/src/forge_X/mod.rs

# Step 2a: intra-subtree path rewrite
#   Before: use crate::Y;            (resolves to kay_core::Y — wrong)
#   After:  use crate::forge_X::Y;   (resolves to kay_core::forge_X::Y — correct)
rg --files-with-matches 'use crate::' crates/kay-core/src/forge_X \
  | xargs sd 'use crate::' 'use crate::forge_X::'

# Step 2b: inter-subtree path rewrite (all subtrees reference other subtrees)
#   Before: use forge_Y::Z;          (no extern crate — unresolved)
#   After:  use crate::forge_Y::Z;   (correct)
rg --files-with-matches 'use forge_' crates/kay-core/src \
  | xargs sd 'use forge_([a-z_]+)::' 'use crate::forge_$1::'

# Step 3: commit-per-subtree in dependency-lowest-first order
#   forge_domain → forge_app → forge_services → forge_repo → forge_infra →
#   forge_api → forge_main → peripherals (forge_display, forge_embed, etc.)
```

**Concrete "before" evidence** (canonical current-state that the rewrite targets):

```rust
// crates/kay-core/src/forge_json_repair/parser.rs:3
use crate::error::{JsonRepairError, Result};
// After rewrite:
// use crate::forge_json_repair::error::{JsonRepairError, Result};
```

```rust
// crates/kay-core/src/forge_services/provider_auth.rs:4-7
use forge_app::{AuthStrategy, ProviderAuthService, StrategyFactory};
use forge_domain::{
    AuthContextRequest, AuthContextResponse, AuthMethod, Provider, ProviderId, ProviderRepository,
};
// After rewrite:
// use crate::forge_app::{AuthStrategy, ProviderAuthService, StrategyFactory};
// use crate::forge_domain::{...};
```

```rust
// crates/kay-core/src/forge_repo/provider/openai.rs:4-10
use forge_app::domain::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, ProviderId, ResultStream,
    Transformer,
};
use forge_app::dto::openai::{ListModelResponse, ProviderPipeline, Request, Response};
use forge_app::{EnvironmentInfra, HttpInfra};
use forge_domain::{ChatRepository, Provider};
use forge_infra::sanitize_headers;
// After rewrite:
// use crate::forge_app::domain::{...};
// use crate::forge_app::dto::openai::{...};
// use crate::forge_app::{EnvironmentInfra, HttpInfra};
// use crate::forge_domain::{ChatRepository, Provider};
// use crate::forge_infra::sanitize_headers;
```

**Verification (per-commit):**

```bash
cargo check -p kay-core 2>&1 | head -30  # errors should monotonically decrease
# Phase-gate target: cargo check --workspace --deny warnings  (no --exclude)
```

**lib.rs convention to mimic post-rename** (from `crates/kay-core/src/forge_domain/lib.rs:1-30` and `crates/kay-core/src/forge_services/lib.rs:1-25`):

```rust
// Sample forge_domain/lib.rs content (to become mod.rs):
mod agent;
mod attachment;
mod auth;
// ... 40+ more mod declarations
```

The existing `lib.rs` files are already shaped correctly — they just need to be renamed to `mod.rs`. No content change inside them. The rewrite only affects path syntax inside the sub-files.

---

### `crates/kay-provider-openrouter/src/lib.rs` (facade-index, re-export)

**Analog:** `crates/kay-core/src/forge_services/lib.rs:1-36`

**Module declaration + re-export pattern** (lines 1-36):
```rust
mod agent_registry;
mod app_config;
// ...
mod provider_auth;
mod provider_service;
// ...

pub use app_config::*;
pub use provider_auth::*;
// ...
```

**Apply to `kay-provider-openrouter/src/lib.rs`:**
```rust
mod allowlist;
mod cost_cap;
mod error;
mod event;
mod provider;
mod retry;
mod tool_parser;
mod translator;

pub use error::{ProviderError, AuthErrorKind, RetryReason};
pub use event::AgentEvent;
pub use provider::{Provider, OpenRouterProvider, ChatRequest, Message, ToolSchema, AgentEventStream};
```

---

### `crates/kay-provider-openrouter/src/provider.rs` — `Provider` trait (trait-definition, async streaming)

**Analog:** `crates/kay-core/src/forge_domain/repo.rs:92-101` (`ChatRepository`)

**Trait shape** (lines 92-101):
```rust
#[async_trait::async_trait]
pub trait ChatRepository: Send + Sync {
    async fn chat(
        &self,
        model_id: &ModelId,
        context: Context,
        provider: Provider<Url>,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error>;
    async fn models(&self, provider: Provider<Url>) -> anyhow::Result<Vec<Model>>;
}
```

**Key patterns to copy:**
- `#[async_trait::async_trait]` macro on the trait
- `Send + Sync` supertraits (required for `Arc<dyn Provider>`)
- Async method with explicit return type
- `ResultStream<T, E>` alias from `forge_domain::error` (lines 110-113):
  ```rust
  pub type BoxStream<A, E> = Pin<Box<dyn tokio_stream::Stream<Item = Result<A, E>> + Send>>;
  pub type ResultStream<A, E> = Result<BoxStream<A, E>, E>;
  ```

**Kay-specific adjustment (vs. analog):**
- Stream item type: `Result<AgentEvent, ProviderError>` (typed error, not `anyhow::Error`)
- Add lifetime parameter on the return stream: `AgentEventStream<'a>` (see RESEARCH §Pattern 1) because our trait wants to be explicitly object-safe for `Arc<dyn Provider>` across phases

---

### `crates/kay-provider-openrouter/src/event.rs` — `AgentEvent` enum (domain-type)

**Analog (compare-and-contrast):** `crates/kay-core/src/forge_domain/message.rs:86-97` (`ChatCompletionMessage`)

**Aggregated shape (what we translate FROM)**, lines 84-97:
```rust
#[derive(Default, Clone, Debug, Setters, PartialEq)]
#[setters(into, strip_option)]
pub struct ChatCompletionMessage {
    pub content: Option<Content>,
    pub thought_signature: Option<String>,
    pub reasoning: Option<Content>,
    pub reasoning_details: Option<Vec<Reasoning>>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<Usage>,
    pub phase: Option<MessagePhase>,
}
```

**Usage struct to reuse or mirror** (`forge_domain/message.rs:24-31`):
```rust
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub prompt_tokens: TokenCount,
    pub completion_tokens: TokenCount,
    pub total_tokens: TokenCount,
    pub cached_tokens: TokenCount,
    pub cost: Option<f64>,  // USD — the field D-10 reads
}
```

**Kay-specific (from CONTEXT.md D-06)** — delta-granular, `#[non_exhaustive]`:
```rust
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta { content: String },
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, arguments_delta: String },
    ToolCallComplete { id: String, name: String, arguments: serde_json::Value },
    ToolCallMalformed { id: String, raw: String, error: String },
    Usage { prompt_tokens: u64, completion_tokens: u64, cost_usd: f64 },
    Retry { attempt: u32, delay_ms: u64, reason: RetryReason },
    Error { error: ProviderError },
    // Phase 3 adds: ToolOutput
    // Phase 4 adds: SandboxViolation
    // Phase 5 adds: TurnEnd { reason: TurnEndReason }
    // Phase 8 adds: Verification { critic, verdict }
}
```

**`#[non_exhaustive]` precedent in tree** (exactly one use so far):
- `crates/kay-core/src/forge_domain/node.rs:288` — only existing usage. Kay standardizes on it for all cross-phase public enums.

---

### `crates/kay-provider-openrouter/src/error.rs` — `ProviderError` enum (domain-type)

**Analog:** `crates/kay-core/src/forge_domain/error.rs:1-107` (ForgeCode's `Error` enum)

**thiserror derive + variants pattern** (lines 13-107):
```rust
#[derive(Debug, Error, From)]
pub enum Error {
    #[error("Missing tool name")]
    ToolCallMissingName,

    #[error("Unsupported role: {0}")]
    #[from(skip)]
    UnsupportedRole(String),

    #[error("JSON deserialization error: {error}")]
    #[from(skip)]
    ToolCallArgument {
        error: JsonRepairError,
        args: String,
    },

    #[error("Agent not found in the arena: {0}")]
    AgentUndefined(AgentId),

    #[error("Environment variable {env_var} not found for provider {provider}")]
    EnvironmentVariableNotFound {
        provider: ProviderId,
        env_var: String,
    },

    #[error(transparent)]
    Retryable(anyhow::Error),
    // ... more variants
}

pub type Result<A> = std::result::Result<A, Error>;
```

**Author's note in-tree** (lines 9-12, to heed):
> "Deriving From for error is a really bad idea. This is because you end up converting errors incorrectly without much context. For eg: You don't want all serde error to be treated as the same."

**Action:** Apply the author's rule — use `#[from(skip)]` on every variant that could accidentally catch too much. Do NOT use blanket `#[from]` on `ProviderError::Network(reqwest::Error)` or `ProviderError::Serialization(serde_json::Error)`; instead explicitly `.map_err(ProviderError::Network)` at the call sites so we know where each came from.

**Kay-specific (from CONTEXT.md D-05):**
```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(reqwest::Error),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("rate limited; retry after {retry_after:?}")]
    RateLimited { retry_after: std::time::Duration },

    #[error("server error HTTP {status}")]
    ServerError { status: u16 },

    #[error("authentication failed: {reason:?}")]
    Auth { reason: AuthErrorKind },

    #[error("model {requested} not allowlisted (allowed: {allowed:?})")]
    ModelNotAllowlisted { requested: String, allowed: Vec<String> },

    #[error("cost cap ${cap_usd} exceeded (spent ${spent_usd})")]
    CostCapExceeded { cap_usd: f64, spent_usd: f64 },

    #[error("tool call {id} malformed: {error}")]
    ToolCallMalformed { id: String, error: String },

    #[error("serialization: {0}")]
    Serialization(serde_json::Error),

    #[error("stream: {0}")]
    Stream(String),

    #[error("canceled")]
    Canceled,
}
```

**Auth-key redaction (threat model TM-01 — API key exfiltration):** Custom `Debug` impl for `AuthErrorKind::InvalidKey(ApiKey)` that redacts the key. Never `#[derive(Debug)]` on structs containing `ApiKey`.

---

### `crates/kay-provider-openrouter/src/provider.rs` — `OpenRouterProvider` impl (adapter-wrapper)

**Analog:** `crates/kay-core/src/forge_services/provider_service.rs:16-135` (`ForgeProviderService<R>`)

**Wraps-a-repository pattern** (lines 16-89):
```rust
pub struct ForgeProviderService<R> {
    repository: Arc<R>,  // the wrapped thing
}

impl<R> ForgeProviderService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
    // ... private helpers like render_url_template
}

#[async_trait::async_trait]
impl<R: ChatRepository + ProviderRepository> ProviderService for ForgeProviderService<R> {
    async fn chat(
        &self,
        model_id: &ModelId,
        context: Context,
        provider: Provider<Url>,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.repository.chat(model_id, context, provider).await  // delegation
    }
    // ... more delegated methods
}
```

**Key patterns to copy:**
- `Arc<R>` wrapping (cheap clone; shared ownership)
- `#[async_trait::async_trait]` on the impl
- Private helper methods for data transform BEFORE delegation
- Delegated methods are thin pass-throughs to `self.repository.*`

**Kay-specific adjustment:** Delegation target is `Arc<dyn UpstreamClient>` wrapping `forge_services::ForgeProviderService<…>` or `forge_repo::provider::OpenAIResponseRepository<…>`. The wrapper adds three pre-flight gates BEFORE delegation:
1. `check_allowlist(model)` → typed error before HTTP fires
2. `check_cost_cap()` → typed error before HTTP fires
3. Rewrite `request.model` to append `:exacto` suffix (see Shared Patterns §Exacto suffix)

See RESEARCH §Code Example 2 for the full `OpenRouterProvider::chat` skeleton (lines 622-699).

---

### `crates/kay-provider-openrouter/src/translator.rs` (stream-transform)

**Analog:** `crates/kay-core/src/forge_repo/provider/event.rs:1-83` (`into_chat_completion_message`)

**Stream-map-with-async-filter pattern** (lines 12-83):
```rust
pub fn into_chat_completion_message<Response>(
    url: Url,
    source: EventSource,
) -> impl Stream<Item = anyhow::Result<ChatCompletionMessage>>
where
    Response: DeserializeOwned,
    ChatCompletionMessage: TryFrom<Response, Error = anyhow::Error>,
{
    source
        .take_while(|message| !matches!(message, Err(reqwest_eventsource::Error::StreamEnded)))
        .then(|event| async {
            match event {
                Ok(event) => match event {
                    Event::Open => None,
                    Event::Message(event) if ["[DONE]", ""].contains(&event.data.as_str()) => {
                        debug!("Received completion from Upstream");
                        None
                    }
                    Event::Message(message) => Some(
                        serde_json::from_str::<Response>(&message.data)
                            .with_context(|| format!("Failed to parse provider response: {}", message.data))
                            .and_then(|response| {
                                ChatCompletionMessage::try_from(response).with_context(...)
                            })
                    ),
                },
                Err(error) => match error {
                    reqwest_eventsource::Error::StreamEnded => None,
                    reqwest_eventsource::Error::InvalidStatusCode(_, response) => {
                        let status = response.status();
                        let body = response.text().await.ok();
                        Some(Err(Error::InvalidStatusCode(status.as_u16())).with_context(...))
                    }
                    // ...
                }
            }
        })
        .filter_map(move |response| {
            response.map(|result| result.with_context(|| format_http_context(None, "POST", url.clone())))
        })
}
```

**Key patterns to copy:**
- `.take_while()` on stream-ended markers
- `.then(|event| async { … })` to allow async inside the mapper
- Event matcher arms with fall-through `None` for events we suppress
- `.filter_map()` at the tail to drop the `None`s
- `[DONE]` sentinel treatment (same convention OpenRouter uses)

**Kay-specific extension:**
- Translator adds STATEFUL reassembly (`HashMap<String, ToolCallBuilder>`) — emits multiple `AgentEvent`s per incoming `ChatCompletionMessage`
- Use `async_stream::stream!` or `futures::stream::unfold` to yield multiple events per input (see RESEARCH §Architecture Pattern 3, lines 377-405)
- State-machine terminal: `finish_reason == Some(FinishReason::ToolCalls) | Some(FinishReason::Stop)`

---

### `crates/kay-provider-openrouter/src/retry.rs` (retry-logic)

**Analog:** `crates/kay-core/src/forge_repo/provider/retry.rs:9-146` (`into_retry` + status-code classification)

**Classification-enum pattern** (lines 70-87):
```rust
#[derive(Clone, Copy)]
enum RetryableApiErrorCode {
    Transport,
    OpenAIOverloaded,
}

impl RetryableApiErrorCode {
    fn matches(self, code: &ErrorCode) -> bool {
        let Some(code) = code.as_str() else { return false; };
        match self {
            RetryableApiErrorCode::Transport => TRANSPORT_ERROR_CODES.contains(&code),
            RetryableApiErrorCode::OpenAIOverloaded => code == OPENAI_OVERLOADED_ERROR_CODE,
        }
    }
}
```

**Classification entry point pattern** (lines 9-29):
```rust
pub fn into_retry(error: anyhow::Error, retry_config: &RetryConfig) -> anyhow::Error {
    if let Some(code) = get_req_status_code(&error)
        .or(get_event_req_status_code(&error))
        .or(get_api_status_code(&error))
        && retry_config.status_codes.contains(&code)
    {
        return DomainError::Retryable(error).into();
    }

    if is_api_transport_error(&error)
        || is_req_transport_error(&error)
        || is_event_transport_error(&error)
        // ... more retryable conditions
    {
        return DomainError::Retryable(error).into();
    }

    error
}
```

**Status-code probe pattern** (lines 49-68):
```rust
fn get_req_status_code(error: &anyhow::Error) -> Option<u16> {
    error.downcast_ref::<reqwest::Error>()
        .and_then(|error| error.status())
        .map(|status| status.as_u16())
}

fn get_event_req_status_code(error: &anyhow::Error) -> Option<u16> {
    error.downcast_ref::<reqwest_eventsource::Error>()
        .and_then(|error| match error {
            reqwest_eventsource::Error::InvalidStatusCode(_, response) => Some(response.status().as_u16()),
            // ...
        })
}
```

**Kay-specific adjustment:**
- Wrap with `backon::ExponentialBuilder` at the outer layer (ForgeCode tags errors as retryable but delegates the retry loop to the caller; we centralize here)
- Honor `Retry-After` header on 429s (see RESEARCH §Code Example 3, lines 720-734 — `parse_retry_after` helper)
- Emit `AgentEvent::Retry { attempt, delay_ms, reason }` via caller-supplied `emit_retry` closure — this is Kay's novel addition

**Test analog reference:** `forge_repo/provider/retry.rs:148-392` is a strong test module — copy the `fixture_response_error`/`fixture_transport_error` helper shape for `retry_429_503.rs` tests.

---

### `crates/kay-provider-openrouter/src/tool_parser.rs` (parser, two-pass)

**Analog (direct reuse):** `crates/kay-core/src/forge_json_repair/parser.rs:1070-1073` (entry point)

**Wrap this exact function:**
```rust
// crates/kay-core/src/forge_json_repair/parser.rs:1070-1073
pub fn json_repair<De: for<'de> Deserialize<'de>>(text: &str) -> Result<De> {
    let parser = JsonRepairParser::new(text.to_string());
    parser.parse()
}
```

**Public API of `forge_json_repair`** (`lib.rs:1-7`):
```rust
mod error;
mod parser;
mod schema_coercion;

pub use error::{JsonRepairError, Result};
pub use parser::json_repair;
pub use schema_coercion::coerce_to_schema;
```

**Error type exposed** (`forge_json_repair/error.rs:1-28`):
```rust
#[derive(Error, Debug)]
pub enum JsonRepairError {
    InvalidCharacter { character: char, position: usize },
    UnexpectedCharacter { character: char, position: usize },
    UnexpectedEnd { position: usize },
    // ...
    JsonError(#[from] serde_json::Error),
}
```

**Kay two-pass wrapper (RESEARCH §Pattern 2, lines 339-367):**
```rust
// crates/kay-provider-openrouter/src/tool_parser.rs
use serde_json::Value;
use kay_core::forge_json_repair::json_repair;  // post-rename path

pub enum ParseOutcome {
    Clean(Value),
    Repaired(Value),
    Malformed(String),
}

pub fn parse_tool_arguments(raw: &str) -> ParseOutcome {
    if raw.is_empty() {
        return ParseOutcome::Clean(Value::Object(Default::default()));
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(v) => ParseOutcome::Clean(v),
        Err(_strict_err) => {
            match json_repair::<Value>(raw) {
                Ok(v) => ParseOutcome::Repaired(v),
                Err(_repair_err) => ParseOutcome::Malformed(raw.to_string()),
            }
        }
    }
}
```

**Hard invariant to encode in crate-root lint** (PROV-05 + TM-01/TM-07):
```rust
// crates/kay-provider-openrouter/src/lib.rs (top of file)
#![deny(clippy::unwrap_used, clippy::expect_used)]  // excluding tests
```

---

### `crates/kay-provider-openrouter/src/cost_cap.rs` (accumulator + gate — NO in-tree analog)

**No analog.** Novel logic.

**Design contract (from CONTEXT.md D-10, RESEARCH §Pitfall 3):**
```rust
use std::sync::Mutex;

pub struct CostCap {
    cap_usd: Option<f64>,   // None = uncapped
    spent_usd: Mutex<f64>,  // accumulator
}

impl CostCap {
    pub fn new(cap_usd: Option<f64>) -> Self {
        Self { cap_usd, spent_usd: Mutex::new(0.0) }
    }

    /// Called BEFORE a chat() request fires. Returns Err if already over cap.
    pub fn check(&self) -> Result<(), ProviderError> {
        let Some(cap) = self.cap_usd else { return Ok(()); };
        let spent = *self.spent_usd.lock().unwrap();
        if spent > cap {
            Err(ProviderError::CostCapExceeded { cap_usd: cap, spent_usd: spent })
        } else {
            Ok(())
        }
    }

    /// Called on every AgentEvent::Usage arrival.
    pub fn accumulate(&self, cost_usd: f64) {
        *self.spent_usd.lock().unwrap() += cost_usd;
    }
}
```

**Key off-by-one contract (RESEARCH §Pitfall 3):**
- Turn N may push us OVER the cap and still complete normally (the HTTP call is already in flight when `Usage` arrives)
- Turn N+1's `check()` catches it BEFORE any HTTP call

**Test shape (`cost_cap_turn_boundary.rs`) invariants:**
- (a) Turn N finishes normally even if it pushes over cap
- (b) Turn N+1 returns `CostCapExceeded` without an HTTP call (assert via `mockito::Mock::assert_not_hit()`)

---

### `crates/kay-provider-openrouter/src/allowlist.rs` (validator + config-merge — partial analogs)

**Config-shape analog:** `crates/kay-core/src/forge_repo/provider/provider.json:46-54` (the OpenRouter provider block)
```json
{
    "id": "open_router",
    "api_key_vars": "OPENROUTER_API_KEY",
    "url_param_vars": [],
    "response_type": "OpenAI",
    "url": "https://openrouter.ai/api/v1/chat/completions",
    "models": "https://openrouter.ai/api/v1/models",
    "auth_methods": ["api_key"]
}
```

**Kay extension (CONTEXT.md D-07):** Add an `allowed_models` array to the same JSON entry (or to `kay.toml [provider.openrouter]`):
```json
{
    "id": "open_router",
    "api_key_vars": "OPENROUTER_API_KEY",
    "url": "https://openrouter.ai/api/v1/chat/completions",
    "allowed_models": [
        "anthropic/claude-sonnet-4.6",
        "anthropic/claude-opus-4.6",
        "openai/gpt-5.4"
    ]
}
```

**Config-test analog:** `forge_repo/provider/provider_repo.rs:586-609` (`test_load_provider_configs`)
```rust
#[test]
fn test_load_provider_configs() {
    let configs = get_provider_configs();
    assert!(!configs.is_empty());

    let openrouter_config = configs.iter()
        .find(|c| c.id == ProviderId::OPEN_ROUTER)
        .unwrap();
    assert_eq!(openrouter_config.api_key_vars, Some("OPENROUTER_API_KEY".to_string()));
    assert_eq!(openrouter_config.url.as_str(), "https://openrouter.ai/api/v1/chat/completions");
}
```

**Copy this test shape for `auth_env_vs_config.rs`** and extend for `allowlist_gate.rs` (assert `allowed_models` parses and normalizes).

**Gating logic (NO analog — novel):**
```rust
pub struct Allowlist {
    models: Vec<String>,  // canonical form: lowercase, no :exacto suffix
}

impl Allowlist {
    pub fn from_config(cfg: &Config) -> Self {
        let mut models = cfg.allowed_models.clone();
        // env override: KAY_ALLOWED_MODELS="model1,model2,..."
        if let Ok(env) = std::env::var("KAY_ALLOWED_MODELS") {
            models = env.split(',').map(|s| Self::canonicalize(s.trim())).collect();
        }
        Self { models }
    }

    fn canonicalize(m: &str) -> String {
        m.to_ascii_lowercase().trim_end_matches(":exacto").to_string()
    }

    pub fn check(&self, requested: &str) -> Result<(), ProviderError> {
        let canonical = Self::canonicalize(requested);
        if self.models.iter().any(|m| m == &canonical) {
            Ok(())
        } else {
            Err(ProviderError::ModelNotAllowlisted {
                requested: requested.to_string(),
                allowed: self.models.clone(),
            })
        }
    }
}
```

**Threat model mitigation (TM-04):** Reject `\r`, `\n`, `\t`, non-ASCII in model IDs BEFORE the allowlist compare.

---

### `crates/kay-provider-openrouter/tests/mock_server.rs` (test-fixture — shared helper)

**Analog:** `crates/kay-core/src/forge_repo/provider/mock_server.rs:1-91` (extend this)

**Full mockito wrapper** (lines 1-78):
```rust
use mockito::{Mock, Server, ServerGuard};

pub struct MockServer {
    server: ServerGuard,
}

impl MockServer {
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        Self { server }
    }

    pub async fn mock_models(&mut self, body: serde_json::Value, status: usize) -> Mock {
        self.server
            .mock("GET", "/models")
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(body.to_string())
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    /// SSE mock — the pattern Kay extends
    pub async fn mock_responses_stream(&mut self, events: Vec<String>, status: usize) -> Mock {
        let sse_body = events.join("\n\n");
        self.server
            .mock("POST", "/v1/responses")
            .with_status(status)
            .with_header("content-type", "text/event-stream")
            .with_header("cache-control", "no-cache")
            .with_body(sse_body)
            .create_async()
            .await
    }

    pub async fn mock_google_chat_stream(&mut self, model: &str, events: Vec<String>, status: usize) -> Mock {
        let mut sse_body = events.join("\n\n");
        sse_body.push_str("\n\n");
        let path = format!("/models/{}:streamGenerateContent", model);
        self.server.mock("POST", path.as_str())
            .match_query(mockito::Matcher::UrlEncoded("alt".into(), "sse".into()))
            .with_status(status)
            .with_header("content-type", "text/event-stream")
            .with_body(sse_body)
            .create_async()
            .await
    }
}

pub fn normalize_ports(input: String) -> String {
    use regex::Regex;
    let re_ip_port = Regex::new(r"127\.0\.0\.1:\d+").unwrap();
    let re_http = Regex::new(r"http://127\.0\.0\.1:\d+").unwrap();
    let normalized = re_http.replace_all(&input, "http://127.0.0.1:<port>");
    let normalized = re_ip_port.replace_all(&normalized, "127.0.0.1:<port>");
    normalized.to_string()
}
```

**Kay extension — new helper:**
```rust
impl MockServer {
    /// OpenRouter-flavored SSE: /v1/chat/completions, text/event-stream,
    /// final chunk carries usage.cost. Follows OpenRouter SSE conventions.
    pub async fn mock_openrouter_chat_stream(
        &mut self,
        events: Vec<String>,
        status: usize,
    ) -> Mock {
        let sse_body = events.join("\n\n");
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(status)
            .with_header("content-type", "text/event-stream")
            .with_header("cache-control", "no-cache")
            .with_body(sse_body)
            .create_async()
            .await
    }

    /// 429 with Retry-After header (integer seconds)
    pub async fn mock_rate_limit(&mut self, retry_after_secs: u64) -> Mock {
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(429)
            .with_header("retry-after", &retry_after_secs.to_string())
            .with_body(r#"{"error":{"code":"rate_limit","message":"..."}}"#)
            .create_async()
            .await
    }
}
```

---

### `crates/kay-provider-openrouter/tests/streaming_happy_path.rs` (test-integration)

**Analog:** `crates/kay-core/src/forge_repo/provider/openai.rs` test module (lines 365-end, not fully read — shape confirmed by imports at 370-380)

**Test-harness imports** (from openai.rs:365-380):
```rust
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use anyhow::Context;
    use bytes::Bytes;
    use forge_app::HttpInfra;
    use forge_app::domain::{Provider, ProviderId, ProviderResponse};
    use forge_app::dto::openai::{ContentPart, ImageUrl, Message, MessageContent, Role};
    use reqwest::header::HeaderMap;
    use reqwest_eventsource::EventSource;
    use url::Url;

    use super::*;
    use crate::provider::mock_server::{MockServer, normalize_ports};
    // ...
}
```

**Key patterns to copy:**
- `#[tokio::test]` attribute (via `tokio` workspace dep `macros` feature, already in Cargo.toml)
- `MockServer::new()` then `mock_xxx_stream()` then `provider.chat()` then `collect()` assertion
- Use of `normalize_ports()` on any snapshot-like test output (insta is used elsewhere but for this phase, direct `assert_eq!` is simpler)

---

### `crates/kay-provider-openrouter/tests/retry_429_503.rs` (test-integration)

**Analog:** `crates/kay-core/src/forge_repo/provider/retry.rs` test module (lines 148-392)

**Fixture factory pattern** (lines 169-190):
```rust
fn fixture_retry_config(codes: Vec<u16>) -> RetryConfig {
    RetryConfig::default().status_codes(codes)
}

fn fixture_response_error(code: Option<u16>) -> anyhow::Error {
    let error = if let Some(code) = code {
        ErrorResponse::default().code(ErrorCode::Number(code))
    } else {
        ErrorResponse::default()
    };
    anyhow::Error::from(Error::Response(error))
}

fn fixture_transport_error(code: &str) -> anyhow::Error {
    let error = ErrorResponse::default().code(ErrorCode::String(code.to_string()));
    anyhow::Error::from(Error::Response(error))
}
```

**Test shape pattern** (lines 191-221):
```rust
#[test]
fn test_into_retry_with_status_codes() {
    let retry_config = fixture_retry_config(vec![429, 500, 502, 503, 504]);

    // Retryable status codes
    for code in [429, 500, 502, 503, 504] {
        let error = fixture_response_error(Some(code));
        assert!(is_retryable(into_retry(error, &retry_config)));
    }

    // Non-retryable status codes
    for code in [400, 401, 403, 404] {
        let error = fixture_response_error(Some(code));
        assert!(!is_retryable(into_retry(error, &retry_config)));
    }
}
```

**Integration-level analog (real network):** lines 374-392 — `test_incomplete_message_is_retryable` spins up a `tokio::net::TcpListener` and asserts a real transport error:
```rust
#[tokio::test]
async fn test_incomplete_message_is_retryable() {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
    });
    let req_err = reqwest::Client::new().get(format!("http://{addr}")).send().await.unwrap_err();
    let retry_config = fixture_retry_config(vec![]);
    assert!(is_retryable(into_retry(req_err.into(), &retry_config)));
}
```

Copy this pattern for Kay's end-to-end retry tests (assert mock was hit exactly N times).

---

### `crates/kay-provider-openrouter/tests/auth_env_vs_config.rs` (test-integration)

**Analog:** `forge_repo/provider/provider_repo.rs:586-609` (`test_load_provider_configs`)

Already shown above. Copy the `assert_eq!(openrouter_config.api_key_vars, Some("OPENROUTER_API_KEY".to_string()))` assertion shape for env-var resolution tests.

---

### Tier 6: CI/docs text-replacements

**Files:**
- `.github/workflows/ci.yml:83` — `cargo clippy --workspace --exclude kay-core --all-targets --all-features -- -D warnings` → remove `--exclude kay-core`
- `.github/workflows/ci.yml:116` — `cargo test --workspace --exclude kay-core --all-features` → remove `--exclude kay-core`
- `CONTRIBUTING.md:50` — same command, same removal
- `docs/CICD.md` §Current State — update to state "Phase 2 landed; kay-core now compiles clean under full workspace."
- `.planning/STATE.md` Phase 2 blocker — remove the "23 × E0583" blocker note

**Verification:** After all three tiers land, `cargo check --workspace --deny warnings` passes without escape clauses on macOS + Linux + Windows matrix (the gate criterion).

---

## Shared Patterns (cross-cutting concerns)

### Authentication reuse (D-08)

**Source:** `crates/kay-core/src/forge_services/provider_auth.rs:1-20` (`ForgeProviderAuthService<I>::new`)

**Apply to:** `OpenRouterProvider` construction in `provider.rs`

```rust
// Reuse ForgeCode's existing env + config resolution; don't reimplement.
// forge_services::provider_auth.rs:11-20
pub struct ForgeProviderAuthService<I> {
    infra: Arc<I>,  // I: StrategyFactory + ProviderRepository + Send + Sync + 'static
}
impl<I> ForgeProviderAuthService<I> {
    pub fn new(infra: Arc<I>) -> Self { Self { infra } }
}
```

**Kay usage:** Compose via `Arc<ForgeProviderAuthService<…>>` at `OpenRouterProvider::builder()` time. Env var `OPENROUTER_API_KEY` is already in `provider.json` (`api_key_vars` field at line 48 of `provider.json`). Env-over-config precedence is already wired into `ForgeProviderAuthService`.

**Threat model (TM-01):** Never `.unwrap()` the API key; route missing-key through `ProviderError::Auth { reason: AuthErrorKind::Missing }` at first `chat()` call (not at crate load — per D-08).

### async_trait + Send + Sync bound

**Source:** Applied consistently across `forge_domain/repo.rs` (all 9+ traits) and `forge_services/provider_service.rs`

**Apply to:** `Provider` trait in `kay-provider-openrouter/src/provider.rs`

Every async trait in the inherited tree uses:
```rust
#[async_trait::async_trait]
pub trait FooRepository: Send + Sync {
    async fn method(&self, ...) -> Result<...>;
}
```

**Hard rule:** All Kay public traits match this shape. Downstream `Arc<dyn Provider>` dependency injection (Phase 5 agent loop) requires it.

### Exacto suffix management (Pitfall 8 / TM-08)

**Apply to:** `OpenRouterProvider::to_exacto_model()` and allowlist canonicalization

**Rule:** Allowlist stores WITHOUT `:exacto` suffix. Always APPEND `:exacto` when constructing the HTTP request body. User-facing model names are clean; wire format is always Exacto.

```rust
// Canonical allowlist form
allowlist = ["anthropic/claude-sonnet-4.6", "openai/gpt-5.4"]  // no :exacto

// Pre-request rewrite
fn to_wire_model(&self, m: &str) -> String {
    format!("{}:exacto", Self::canonicalize(m))
}
```

**Test:** `allowlist_gate.rs` must include the round-trip assertion: user input `anthropic/claude-sonnet-4.6` → request body `model: "anthropic/claude-sonnet-4.6:exacto"`.

### `#[non_exhaustive]` on public enums

**Apply to:** Every public enum in `kay-provider-openrouter`: `AgentEvent`, `ProviderError`, `AuthErrorKind`, `RetryReason`, plus future `TurnEndReason` (Phase 5).

**Precedent:** `forge_domain/node.rs:288` is the only existing usage; Kay standardizes on it for cross-phase evolution safety.

```rust
#[non_exhaustive]
pub enum AgentEvent { … }
```

### `thiserror` + explicit `#[from(skip)]`

**Source:** `forge_domain/error.rs:9-12` carries an in-source author note:
> "Deriving From for error is a really bad idea. This is because you end up converting errors incorrectly without much context."

**Apply to:** `ProviderError` — use `#[from(skip)]` on every variant that wraps a common type (`reqwest::Error`, `serde_json::Error`). Map explicitly at call sites, not via blanket `From`.

### HTTP request plumbing — delegated, not reimplemented

**Source:** `crates/kay-core/src/forge_repo/provider/openai.rs:187-221` (`inner_chat`)

**Rule:** Kay's provider facade NEVER constructs a `reqwest::Request` directly. All HTTP happens inside `forge_repo::provider::openai::OpenAIProvider::inner_chat` (or its delegates). The facade translates at the Stream boundary only.

**Key excerpt** (lines 208-221):
```rust
let json_bytes = serde_json::to_vec(&request).with_context(|| "Failed to serialize request")?;

let es = self.http
    .http_eventsource(&url, Some(headers), json_bytes.into())
    .await
    .with_context(|| format_http_context(None, "POST", &url))
    .map_err(|e| enhance_error(e, &self.provider.id))?;

let stream = into_chat_completion_message::<Response>(url, es);

Ok(Box::pin(stream))
```

**OpenRouter URL evidence** (that this code already speaks OpenRouter):
- `openai.rs:51` — "OpenRouter optional headers ref: https://openrouter.ai/docs/api-reference/overview#headers"
- `openai.rs:318` — "OpenRouter" in the supported-providers comment block
- `provider.json:47-53` — `"url": "https://openrouter.ai/api/v1/chat/completions"`, `"api_key_vars": "OPENROUTER_API_KEY"`
- `provider_repo.rs:591-608` — in-tree test asserts this config parses correctly

---

## No Analog Found

Files where Kay introduces genuinely new logic. The planner should cite RESEARCH.md code examples and design decisions rather than trying to fit these to an in-tree analog:

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/kay-provider-openrouter/src/cost_cap.rs` | accumulator + gate | event-driven | No per-session cost tracking exists in ForgeCode. Design from CONTEXT.md D-10 + RESEARCH §Pitfall 3 + §Code Example 2 |
| `crates/kay-provider-openrouter/src/allowlist.rs` (gating logic) | validator | request-response pre-check | Config shape analog exists (`provider.json`), but pre-HTTP typed-error gate is new. Design from CONTEXT.md D-07, NN-6 |
| `crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs` | test-integration | state machine | Tests greenfield logic; shape analog is `retry.rs` test module for fixture-driven coverage |
| `crates/kay-provider-openrouter/tests/allowlist_gate.rs` | test-integration | pre-HTTP gate | Must assert ZERO HTTP hits on rejected model (use `mockito::Mock::assert_not_hit()`) |
| `crates/kay-provider-openrouter/tests/tool_call_property.rs` | test-property | fuzz | `proptest` is a new workspace dep; RESEARCH §Code Example 5 is the only reference |
| `crates/kay-provider-openrouter/src/translator.rs` state machine | stream-transform | streaming reassembly | Stream-map shape is analogous (`forge_repo/provider/event.rs`) but the per-`tool_call.id` reassembly HashMap is new. Design from CONTEXT.md D-04, RESEARCH §Architecture Pattern 3 |
| Tier 1 path-rewrite class (~720 import statements) | path-rewrite | compile-time | No analog; it IS the class of change. Mechanical sd/sed; commit-per-subtree |

---

## Key Decisions the Planner Should Bake into PLAN.md

1. **Commit-per-subtree for Tier 1:** Each of the 23 subtrees gets its own commit (rename + path rewrites for that subtree). Order: `forge_domain → forge_app → forge_services → forge_repo → forge_infra → forge_api → forge_main → peripherals (forge_display, forge_embed, forge_fs, forge_spinner, forge_markdown_stream, forge_select, forge_snaps, forge_stream, forge_template, forge_test_kit, forge_tool_macros, forge_tracker, forge_walker, forge_ci, forge_config, forge_json_repair)`. Dependency-lowest-first means bisect works cleanly.

2. **Delegate, never reimplement:** Tier 3's `OpenRouterProvider::chat` delegates to `forge_services::provider_service::ForgeProviderService` (or directly to `forge_repo::provider::openai::OpenAIResponseRepository` if that compiles cleaner post-rename — planner's discretion per CONTEXT.md "Claude's Discretion" bullet 2). Do NOT copy `openai.rs` code into the new crate.

3. **Two-pass parser wraps, not replaces:** `tool_parser.rs` calls `kay_core::forge_json_repair::json_repair` as its fallback. Zero new parser logic.

4. **Exacto suffix is wire-only, never user-facing:** Allowlist uses canonical form (no suffix); facade appends `:exacto` on every request.

5. **CI gate removal requires Tier 1 complete:** Don't touch `.github/workflows/ci.yml` until `cargo check --workspace --deny warnings` passes locally on all three OSes. Order: Tier 1 (all 23 subtree commits) → verify locally → Tier 6 (CI/docs text edits).

6. **Crate lint at `lib.rs`:** `#![deny(clippy::unwrap_used, clippy::expect_used)]` scoped to non-test code. Enforces TM-01, TM-07, and PROV-05 invariants at compile time.

7. **Allowlist canonical form is lowercase, suffix-stripped:** `canonicalize(m) = m.to_ascii_lowercase().trim_end_matches(":exacto").trim()`. Reject model IDs with `\r`, `\n`, `\t`, or non-ASCII before allowlist compare (TM-04).

---

## Metadata

**Analog search scope:**
- `crates/kay-core/src/forge_json_repair/` (parser, error, lib.rs)
- `crates/kay-core/src/forge_repo/provider/` (openai.rs, event.rs, retry.rs, mock_server.rs, provider_repo.rs, provider.json)
- `crates/kay-core/src/forge_services/` (provider_service.rs, provider_auth.rs, lib.rs)
- `crates/kay-core/src/forge_domain/` (repo.rs, error.rs, message.rs, provider.rs, node.rs)
- `crates/kay-provider-openrouter/` (current stub: lib.rs, Cargo.toml)
- Workspace root: `Cargo.toml`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`

**Files read (Read tool):**
- `.planning/phases/02-provider-hal-tolerant-json-parser/02-CONTEXT.md`
- `.planning/phases/02-provider-hal-tolerant-json-parser/02-RESEARCH.md` (two segments: lines 1-400, 400-800, 800-1157)
- `crates/kay-core/src/forge_repo/provider/openai.rs` (lines 1-120, 120-280, 280-410)
- `crates/kay-core/src/forge_repo/provider/retry.rs` (full)
- `crates/kay-core/src/forge_repo/provider/event.rs` (lines 1-150)
- `crates/kay-core/src/forge_repo/provider/mock_server.rs` (full)
- `crates/kay-core/src/forge_repo/provider/provider_repo.rs` (lines 560-630)
- `crates/kay-core/src/forge_repo/provider/provider.json` (lines 1-80)
- `crates/kay-core/src/forge_services/provider_service.rs` (full)
- `crates/kay-core/src/forge_services/provider_auth.rs` (full)
- `crates/kay-core/src/forge_services/lib.rs` (full)
- `crates/kay-core/src/forge_domain/repo.rs` (lines 1-235)
- `crates/kay-core/src/forge_domain/error.rs` (full)
- `crates/kay-core/src/forge_domain/message.rs` (lines 1-130)
- `crates/kay-core/src/forge_json_repair/lib.rs` (full, 7 lines)
- `crates/kay-core/src/forge_json_repair/parser.rs` (lines 1-10, 1065-1073)
- `crates/kay-core/src/forge_json_repair/error.rs` (full)
- `crates/kay-provider-openrouter/src/lib.rs` (full, 5 lines)
- `Cargo.toml` (workspace)

**Pattern extraction date:** 2026-04-20
**Phase:** 02-provider-hal-tolerant-json-parser
