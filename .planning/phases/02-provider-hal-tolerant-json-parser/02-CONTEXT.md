# Phase 2: Provider HAL + Tolerant JSON Parser - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning
**Mode:** Auto-resolved from prior context per user directive ("proceed autonomously")

<domain>
## Phase Boundary

Any agent turn can stream chat completions and tool calls through **OpenRouter** (OpenAI-compatible endpoint) with:

- typed `AgentEvent` stream (text deltas, reassembled tool calls, usage, retry, error frames),
- typed `ProviderError` taxonomy (not string errors),
- tolerant two-pass JSON parser that never panics on malformed tool-call deltas,
- strict Exacto-leaning model allowlist (no silent fallback),
- per-session `--max-usd` cost cap with hard abort,
- 429 / 503 retry with jittered exponential backoff and user-visible retry events.

**In scope:**
- `kay-provider-openrouter` crate implementation (currently a stub `lib.rs`)
- Structural integration fix for `kay-core`'s 23 × `E0583` errors (blocker to any Phase 2 compile)
- Initial `AgentEvent` enum shape (frozen/`#[non_exhaustive]` formally in Phase 5 per LOOP-02)
- Initial `ProviderError` enum (Phase 5 may add variants)
- Typed `Provider` trait signature
- Wiring ForgeCode's existing OpenAI-compatible provider path (`forge_repo/provider/openai.rs` + `forge_services/provider_service.rs`) as the implementation under the new typed façade

**Out of scope:**
- Direct Anthropic / OpenAI / Gemini / Groq APIs (v2, per PROJECT.md Out of Scope)
- Local model support (v2, per PROJECT.md Out of Scope)
- `AgentEvent` variants that other phases own:
  - `ToolOutput` frames — Phase 3 (SHELL-03)
  - `SandboxViolation` — Phase 4 (SBX-04)
  - `Verification` — Phase 8 (VERIFY-04)
  - `TurnEnd` semantics — Phase 5 (LOOP-05)
- Session persistence / transcript storage (Phase 6)
- Tree-sitter context retrieval (Phase 7)
- Actual TB 2.0 parity run (EVAL-01a — blocked on OpenRouter key + ~$100 budget per Phase 1 D-OP-01)

</domain>

<decisions>
## Implementation Decisions

Decisions below are organized by load-bearing → derivative. Claude auto-resolved each per user's "proceed autonomously" directive, citing prior context. Any decision can be revisited during `/gsd-plan-phase 2` research/planning.

### Structural integration (kay-core E0583 fix)

- **D-01:** **Rename `crates/kay-core/src/forge_X/lib.rs` → `crates/kay-core/src/forge_X/mod.rs`** for all 23 `forge_*` subdirectories.
  - **Rationale:** Plan 01-03's three-option menu (rename vs. shim vs. split crates) listed this as the "mechanical, preserves content" path. Smallest diff, lets `cargo check --workspace` pass without `--exclude kay-core`, and crucially **does not modify source content** — preserves the `forgecode-parity-baseline` tag's semantic integrity (the imported source files are byte-identical under their new module-system path).
  - **Parity-baseline implication:** None. The tag points at commit `8af1f2b` which captured the pre-rename tree. Phase 2's first commit for this rename produces a new tree; the tag remains valid as the canonical "unmodified import." We do NOT re-tag.
  - **Alternatives rejected:**
    - (b) `include!` shim — adds indirection for no benefit; doesn't make the underlying lib.rs compile cleanly either (they still reference their own crate's lib.rs declarations).
    - (c) Split `forge_*` into sub-crates — correct architecture but 5-10× the effort and drains Phase 2's real deliverable budget (typed HAL). Defer to a later restructuring phase if ever needed; the mono-crate form works for v1.
  - **Verification:** `cargo check --workspace --deny warnings` must pass on macOS + Linux + Windows (removes the `--exclude kay-core` escape clause in `CONTRIBUTING.md §PR Process` and `.github/workflows/ci.yml` lint/test jobs).

### ForgeCode reuse strategy

- **D-02:** **Typed wrapper over ForgeCode's existing OpenAI-compatible provider path.**
  - ForgeCode's `forge_repo/provider/openai.rs` already speaks **OpenRouter natively** — OpenRouter ships an OpenAI-compatible endpoint and `openai.rs` references `openrouter.ai/api/v1/chat/completions` directly (see `openai.rs:51`, `provider_repo.rs:597-607`, `provider.json`). The infrastructure (reqwest client, streaming, retry, provider-config JSON) is already in the imported tree.
  - `kay-provider-openrouter` exposes the **typed Provider trait + AgentEvent enum + ProviderError enum** and delegates to `forge_services::provider_service` / `forge_repo::provider::openai` internally.
  - **Rationale:** EVAL-01a (parity run) has not yet run. Rewriting the provider from scratch in `kay-provider-openrouter` would create a parallel code path that would **not** be parity-equivalent to the baseline tag, corrupting the benchmark guarantee. A typed wrapper keeps ForgeCode's exact behavior while adding Kay's type contract. Post-EVAL-01a, we can evolve.
  - **Alternatives rejected:**
    - Fresh-in-crate rewrite — loses parity guarantee; violates EVAL-01 principle.
    - Extract `forge_infra`/`forge_repo` provider code literally into `kay-provider-openrouter` — medium refactor that fragments the import tree and makes future upstream sync harder. Defer.

### Tolerant JSON parser

- **D-03:** **Two-pass parser, reusing `forge_json_repair` as the second pass.**
  - First pass: `serde_json::from_str::<Value>()` — strict happy path.
  - Second pass on failure: feed the raw bytes to `forge_json_repair::parser` (already in the import tree, ForgeCode-vetted, handles trailing commas, unquoted keys, stringified numbers, etc.).
  - If both passes fail, emit `AgentEvent::ToolCallMalformed { id, raw, error }` and a typed `ProviderError::ToolCallMalformed`. **Never panic.**
  - **Rationale:** `forge_json_repair` is battle-tested against OpenRouter's actual output variance (that's its literal reason for existing in ForgeCode). Reusing it is free parity-preserving value; writing our own second pass would regress.
  - PROV-05 satisfied: "malformed strings, stringified args, partial tool deltas" → the two-pass covers all three categories.

### Tool-call reassembly

- **D-04:** **Accumulate tool-call deltas per `tool_call.id` across SSE chunks into a `HashMap<String, ToolCallBuilder>`; emit `AgentEvent::ToolCallComplete` only after receiving a terminal marker** (finish_reason = `tool_calls` in the OpenAI/OpenRouter spec, or a `done` event).
  - Partial deltas flow to consumers as `AgentEvent::ToolCallDelta { id, arguments_delta }` for UI liveness (Tauri + TUI).
  - Null `arguments` in a delta is legal (OpenRouter variance) — treated as "arguments not yet started." Never panics.
  - **Rationale:** OpenRouter's SSE can fragment a single tool_call across 10+ chunks. Provider HAL must hide this; consumers see exactly one `Complete` per call.

### Typed errors

- **D-05:** Initial `ProviderError` variants (extend in later phases, all non-exhaustive):
  ```rust
  #[non_exhaustive]
  pub enum ProviderError {
      Network(reqwest::Error),             // transport failure
      Http { status: u16, body: String },  // non-2xx non-retry
      RateLimited { retry_after: Duration }, // 429
      ServerError { status: u16 },         // 5xx
      Auth { reason: AuthErrorKind },      // missing/invalid key, expired
      ModelNotAllowlisted { requested: String, allowed: Vec<String> },
      CostCapExceeded { cap_usd: f64, spent_usd: f64 },
      ToolCallMalformed { id: String, error: String },
      Serialization(serde_json::Error),    // request-build failure
      Stream(String),                      // SSE framing error
      Canceled,                            // user cancel / control channel
  }
  ```
  - **Rationale:** PROV-08 explicit — "typed `ProviderError` (not string) for diagnosis and retry decisions." Phase 5 may add loop-control variants.

### Initial `AgentEvent` shape (Phase 5 formalizes the freeze)

- **D-06:** Provider-tier frames only in this phase. Variants:
  ```rust
  #[non_exhaustive]
  pub enum AgentEvent {
      TextDelta { content: String },
      ToolCallStart { id: String, name: String },
      ToolCallDelta { id: String, arguments_delta: String },
      ToolCallComplete { id: String, name: String, arguments: serde_json::Value },
      ToolCallMalformed { id: String, raw: String, error: String },
      Usage {
          prompt_tokens: u64,
          completion_tokens: u64,
          cost_usd: f64,
      },
      Retry { attempt: u32, delay_ms: u64, reason: RetryReason },
      Error { error: ProviderError },
      // Phase 3 will add: ToolOutput
      // Phase 4 will add: SandboxViolation
      // Phase 5 will add: TurnEnd { reason: TurnEndReason }
      // Phase 8 will add: Verification { critic, verdict }
  }
  ```
  - **Rationale:** LOOP-02 (Phase 5) freezes `AgentEvent` as `#[non_exhaustive]`. Phase 2 seeds the provider-tier frames; downstream phases extend without breakage.

### Exacto model allowlist (launch composition)

- **D-07:** **Launch allowlist (v1):** Match ForgeCode's reference TB 2.0 run set — the proven performers.
  - `anthropic/claude-sonnet-4.6` (Exacto endpoint)
  - `anthropic/claude-opus-4.6` (Exacto endpoint)
  - `openai/gpt-5.4` (Exacto endpoint; the model ForgeCode used to hit 81.8%)
  - **Rationale:** Phase 12 acceptance gate is "beat ForgeCode on TB 2.0" — using the same models ForgeCode reported eliminates model-variance as a factor, so the delta between Kay's score and ForgeCode's 81.8% is purely *harness quality* (our actual wedge). Adding more models later is trivial (edit one config); removing them after launch is not.
  - **Config surface:** allowlist lives in `kay.toml` under `[provider.openrouter] allowed_models = [...]` and is mergeable with env override `KAY_ALLOWED_MODELS="..."` (comma-separated). CLI rejects requests for non-allowlisted models with `ProviderError::ModelNotAllowlisted` **before** the HTTP request fires.
  - **Exacto requirement:** PROJECT.md Constraint §Provider says "OpenRouter Exacto endpoints" — we request the `:exacto` suffix explicitly (OpenRouter routes these to higher-reliability backends).

### Authentication

- **D-08:** **API key via `OPENROUTER_API_KEY` env var OR `kay.toml` `[provider.openrouter] api_key`**; env wins on conflict.
  - **No OAuth** — PROV-03 explicit.
  - **No keyring on Phase 2** — keyring binding is a Phase 10 UI-layer concern (UI-04). Phase 2 reads from env/config only.
  - **Missing key behavior:** `ProviderError::Auth { reason: AuthErrorKind::Missing }` surfaced at first provider call; not at crate-load.

### Retry policy (429 / 503)

- **D-09:** **`backon::ExponentialBackoff` with full jitter.** Base 500ms, factor 2, max attempts 3, max delay 8s.
  - 429 respects `Retry-After` header if present (takes precedence over backoff calculation).
  - 503 uses backoff as-is.
  - Every retry emits `AgentEvent::Retry { attempt, delay_ms, reason: RetryReason::RateLimited | ::ServerError }` so UI can surface a "retrying in 2s…" indicator.
  - After 3 attempts exhaust, surface the final `ProviderError::RateLimited` or `ProviderError::ServerError` and stop.
  - **Rationale:** PROV-02 names backon; PROV-07 specifies "jittered exponential backoff + user-visible retry events." Three attempts + 8s max delay is the ForgeCode default observed in `forge_repo/provider/retry.rs` — matches parity.

### Per-session cost cap

- **D-10:** **No default `--max-usd` — must be explicit.** If unset, the session is **uncapped** (i.e., the cap is `f64::INFINITY`). Passing `--max-usd 0` is rejected with a usage error.
  - **Rationale:** PROV-06 says "per-session cost cap and hard abort." Making this opt-in rather than opt-out avoids silently throttling benchmark runs (TB 2.0 Harbor config injects its own cap; our default should not interfere).
  - **Enforcement:** after each `AgentEvent::Usage` frame, provider service accumulates `spent_usd`. If `spent_usd > cap_usd`, the next turn fails with `ProviderError::CostCapExceeded { cap_usd, spent_usd }` and a user-visible `AgentEvent::Error`. In-flight streaming is **not** interrupted mid-response (we don't abort the current HTTP call); the cap applies at turn boundaries.
  - **Cost calculation:** OpenRouter returns per-token cost in the `usage.cost` field of completions (USD). We trust that figure. When absent, fall back to the allowlist's pinned price table (to be committed alongside this phase).

### Claude's Discretion

- Internal crate structure of `kay-provider-openrouter` (module layout, test organization)
- Exact `forge_*` path through which we call into ForgeCode's provider — planner/executor decide based on what compiles cleanest post-rename
- Which `backon` builder API shape to use
- Test fixture format for mocking OpenRouter SSE (planner decides between a recorded-cassette library like `rvcr` or hand-written mocks)
- Whether to expose a synchronous convenience wrapper alongside the async streaming API

### Folded Todos

None — no pre-existing todos matched Phase 2 at workflow entry.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executor) MUST read these before planning or implementing.**

### Phase & project specs
- `.planning/PROJECT.md` — vision, Key Decisions, OpenRouter-only constraint, Exacto allowlist
- `.planning/REQUIREMENTS.md` §PROV (PROV-01 … PROV-08) — the locked requirements for this phase
- `.planning/ROADMAP.md` §Phase 2 — phase goal + success criteria
- `.planning/STATE.md` — Phase 2 research flag: OpenRouter SSE retry semantics need real-trace validation

### Phase 1 artifacts that Phase 2 directly inherits from
- `.planning/phases/01-fork-governance-infrastructure/VERIFICATION.md` §SC-4 — the `--exclude kay-core` escape clause that Phase 2 must remove
- `.planning/phases/01-fork-governance-infrastructure/01-03-SUMMARY.md` §cargo check — the 23 × E0583 failure analysis + Resolution menu (D-01 draws from this)
- `CONTRIBUTING.md` §Pull Request Process — the explicit `cargo fmt -p <...>` list that Phase 2 must unify once kay-core compiles
- `.github/workflows/ci.yml` — the `cargo test --workspace --exclude kay-core` line that Phase 2 must update
- `docs/CICD.md` §Current State — the documented Phase 2 transition points
- `SECURITY.md` §Release Signing — v0.0.x carve-out still applies; no change in Phase 2

### ForgeCode-inherited code (imported into `crates/kay-core/src/forge_*/`)
- `crates/kay-core/src/forge_json_repair/parser.rs` + `schema_coercion.rs` — D-03's second-pass parser (reuse)
- `crates/kay-core/src/forge_repo/provider/openai.rs` — the OpenAI-compatible provider that already speaks OpenRouter (D-02's delegation target); see lines 51 (HTTP-Referer), 318 (OpenRouter comment)
- `crates/kay-core/src/forge_repo/provider/provider_repo.rs` — provider config loader; test at lines 591-608 verifies OpenRouter config loads correctly
- `crates/kay-core/src/forge_repo/provider/provider.json` — the provider endpoint config (contains OpenRouter URL)
- `crates/kay-core/src/forge_repo/provider/retry.rs` — existing backoff implementation to align D-09 with
- `crates/kay-core/src/forge_repo/provider/chat.rs` — streaming chat plumbing
- `crates/kay-core/src/forge_repo/provider/event.rs` — provider event types (compare/reconcile with D-06's `AgentEvent`)
- `crates/kay-core/src/forge_domain/provider.rs` — domain types (may become Provider trait's input types)
- `crates/kay-core/src/forge_services/provider_service.rs` — service layer above provider_repo
- `crates/kay-core/src/forge_services/provider_auth.rs` — auth (env var + config key resolution); D-08 reuses
- `crates/kay-core/src/forge_app/agent_provider_resolver.rs` — app-layer provider resolution

### Rust toolchain + dependency pins
- `Cargo.toml` (workspace) — pinned tokio 1.51, reqwest 0.13, rustls 0.23, serde_json, schemars
- `rust-toolchain.toml` — Rust stable 1.95, 2024 edition
- `deny.toml` — rustls-only, openssl banned, license allow-list

### License / governance
- `NOTICE` — ForgeCode attribution must be preserved when we rename files
- `crates/kay-core/NOTICE` — crate-level notice already in place
- `ATTRIBUTIONS.md` — upstream SHA (unchanged by rename)

### External reference (fetch during research)
- OpenRouter API docs: https://openrouter.ai/docs (SSE format, Exacto suffix, cost field in usage, 429 Retry-After)
- `backon` crate docs: https://docs.rs/backon (ExponentialBackoff builder)
- `reqwest-eventsource`: https://docs.rs/reqwest-eventsource (SSE integration with reqwest 0.13)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets (in the ForgeCode-imported tree — ready to delegate to post-rename)
- `forge_json_repair` → tolerant JSON parser (D-03 reuses directly — no rewrite)
- `forge_repo/provider/openai.rs` → OpenAI-compatible client that speaks OpenRouter natively (D-02's delegation target)
- `forge_repo/provider/retry.rs` → existing jittered backoff (D-09 aligns to match)
- `forge_repo/provider/chat.rs` → streaming chat, SSE handling
- `forge_services/provider_service.rs` → service layer that orchestrates provider_repo
- `forge_services/provider_auth.rs` → env var + config key auth (D-08 reuses)
- `forge_repo/provider/mock_server.rs` → mock OpenAI-compatible server for tests (reuse for kay-provider-openrouter integration tests)

### Established patterns
- **Provider config is JSON, not Rust** — `provider.json` + `provider_repo.rs` deserialize it. Kay should align: launch allowlist lives in `provider.json` (edit the imported file) rather than hardcoded Rust consts.
- **Error surfacing via `Result<T, ProviderError>`** — ForgeCode already does typed errors in `forge_domain`. Kay's `ProviderError` should be compatible (likely wraps or re-exports).
- **Stream-first** — ForgeCode uses `reqwest-eventsource` + `futures::Stream`. Kay's `Provider` trait keeps this.

### Integration points (where `kay-provider-openrouter` connects)
- Exposes `pub trait Provider` + `pub enum AgentEvent` + `pub enum ProviderError` — will be consumed by `kay-core`'s future agent loop (Phase 5) and by `kay-cli` (Phase 5).
- Depends on `kay-core` (which owns the ForgeCode-imported provider internals post-rename).
- NOT depended on by `kay-tauri` / `kay-tui` in Phase 2 (those consume `kay-cli`'s JSONL stream in Phase 9/9.5).

### Blocked by (must be resolved before real Phase 2 work)
- **D-01 (structural rename)** is the literal first task. Without it, nothing in `kay-core` compiles and `kay-provider-openrouter` cannot reference internals. This is a pure-mechanical change that can be verified with `cargo check --workspace` passing *without* `--exclude kay-core`.

</code_context>

<specifics>
## Specific Ideas

- **ForgeCode's actual parity run** is the north star — every decision here is biased toward "whatever keeps our behavior observationally equivalent to ForgeCode on TB 2.0." When in doubt, delegate to imported code, don't reimplement.
- **`#[non_exhaustive]` everywhere public** — `AgentEvent`, `ProviderError`, `RetryReason`, `AuthErrorKind`, `TurnEndReason` (future). Makes additive evolution across phases breakage-free.
- **No `unwrap()` in the tool-call parsing hot path** — PROV-05 says "never a panic." This is a hard invariant for code review.
- **Allowlist → one source of truth** — `provider.json` (inherited from ForgeCode) is the config. CLI flag and env var override it at load-time; we do not have two allowlist maps.

</specifics>

<deferred>
## Deferred Ideas

Items that surfaced during analysis but belong to other phases or post-v1:

- **Direct Anthropic / OpenAI / Gemini provider integrations** → v2 (PROV-v2-01..04 already in REQUIREMENTS.md §v2)
- **Local model support (Ollama, llama.cpp)** → v2 (PROV-v2-05)
- **OAuth flow for OpenRouter** → explicit out-of-scope per PROV-03
- **Keyring-based key storage** → Phase 10 UI (UI-04)
- **Provider config UI picker** → Phase 10 UI (UI-07 Settings panel)
- **Cost dashboards, per-model price learning** → out-of-scope v1
- **Splitting `kay-core` into per-subsystem sub-crates** → not scheduled; mono-crate form is adequate for v1
- **Re-tagging `forgecode-parity-baseline` after the rename** → NOT done; the tag points at commit `8af1f2b` which is the *unmodified* import. Rename happens in a post-tag commit. Phase 11 re-tag (signed) is still on the table per D-OP-04.
- **Abort in-flight HTTP when cost cap is exceeded mid-response** → D-10 deliberately defers this; cost cap enforcement happens at turn boundary.
- **Supporting the Anthropic or Bedrock providers via OpenRouter's abstraction** → they are in the imported tree but not on the Exacto allowlist; not exercised in Phase 2. Phase 3+ may revisit if specific tool-use patterns require it.

### Reviewed Todos (not folded)
None.

</deferred>

---

## Appendix A — Post-Phase-2.5 Realignment (added 2026-04-20)

Phase 2.5 (the kay-core sub-crate split, executed 2026-04-20) changed the physical
layout that plans 02-06..02-10 were authored against. The plans themselves remain
semantically valid — their decisions (D-01..D-13), target REQ-IDs, task
decompositions, and type contracts are unchanged. What **has** changed is the
import-path and dependency surface. During execution of plans 02-06..02-10,
the executor applies these three mechanical substitution rules wherever a plan
references the old mono-crate layout:

### Substitution Rule 1 — Cargo.toml `[dependencies]`

Wherever a plan says:

```toml
kay-core = { path = "../kay-core" }
```

substitute the specific forge_* sub-crates that plan actually consumes. The
sub-crate set for `kay-provider-openrouter` (already wired by Phase 2.5-04
commit `9d6a32a`) is:

```toml
forge_domain       = { path = "../forge_domain" }
forge_config       = { path = "../forge_config" }
forge_services     = { path = "../forge_services" }
forge_repo         = { path = "../forge_repo" }
forge_json_repair  = { path = "../forge_json_repair" }
```

Add `forge_app = { path = "../forge_app" }` only if a plan references
`forge_app::dto::openai::*` (plan 02-08 does; plan 02-06 does not).

If a plan needs a type that isn't exposed by any of the 6 top-of-DAG crates
(`forge_api`, `forge_config`, `forge_domain`, `forge_json_repair`, `forge_repo`,
`forge_services`), consult the 23-crate DAG in `.planning/phases/02.5-kay-core-sub-crate-split/02.5-CONTEXT.md`
to pick the right leaf sub-crate and add it to Cargo.toml.

### Substitution Rule 2 — Rust `use` paths

Wherever a plan writes:

```rust
use kay_core::forge_X::Y;
```

substitute:

```rust
use forge_X::Y;
```

(The sub-crate IS `forge_X`; no `kay_core::` prefix.) This applies to every
`kay_core::forge_*::*` path reference in plans 02-06..02-10 — approximately
620 references across the phase directory, but every one follows this single
mechanical rule.

### Substitution Rule 3 — Doc references & file paths

Wherever a plan cites `crates/kay-core/src/forge_X/Y.rs` in `read_first` or
documentation lines, substitute `crates/forge_X/src/Y.rs`. The file content
is byte-identical to upstream (governance invariant still holds); only the
physical path within the workspace has changed.

### What does NOT need substitution

- **`kay-core` the crate itself** is still a valid workspace member — now an
  aggregator re-exporter with 6 `pub extern crate forge_*` lines. Plans may
  still say "kay-core aggregates the forge_* crates" descriptively.
- **Decision IDs (D-01..D-13)**, type contracts (ProviderError, AgentEvent,
  Provider trait), REQ-ID mapping, task acceptance criteria — unchanged.
- **Phase 2.5 added dep version bumps** (reedline 0.47, humantime 2.1,
  convert_case 0.11, strum 0.28, update-informer pinned). These are already in
  the workspace Cargo.toml and need no re-application.
- **`crates/kay-provider-openrouter/Cargo.toml`** — already has the correct
  direct forge_* deps (committed in Phase 2.5-04 task 2, `9d6a32a`). Plan
  02-06 Task 1 is reduced to "add backon, async-trait, futures, tokio-stream"
  since the forge_* deps are already present.

### Deviation recording during execution

When the executor encounters a mismatch between a plan's literal text and
the post-2.5 reality (e.g. plan says `kay-core = { path = "../kay-core" }`
but Cargo.toml already shows the direct deps), it applies the relevant rule
above and records it as a **Rule-2 deviation** (plan text superseded by
post-phase realignment) in the plan's SUMMARY.md `<deviations>` section.
Do NOT modify the PLAN.md itself — leave the plan as authored and record
all substitutions in SUMMARY.md for traceability.

### Plan 02-05 supersession

Plan 02-05 (mechanical mono-crate path rewrite for forge_services / _infra /
_repo / _api / _main + CI cleanup) is **superseded** by Phase 2.5 and
archived at `archive/02-05-PLAN.md.superseded`. Its CI cleanup scope (remove
`--exclude kay-core`) was executed in plan 02.5-04 task 3. No replacement
plan; plans 02-06..02-10 target the sub-crate layout directly.

---

*Phase: 02-provider-hal-tolerant-json-parser*
*Context gathered: 2026-04-20*
*Mode: auto-resolved per user "proceed autonomously" directive; every decision traceable to prior phase artifacts or explicit REQ-IDs*
*Appendix A added: 2026-04-20 — post-Phase-2.5 realignment (3 substitution rules)*
