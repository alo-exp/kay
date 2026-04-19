# Phase 2: Provider HAL + Tolerant JSON Parser - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 02-provider-hal-tolerant-json-parser
**Mode:** auto-resolved (no interactive AskUserQuestion) per user directive "Proceed autonomously"
**Areas covered:** structural integration, ForgeCode reuse strategy, tolerant JSON parser, tool-call reassembly, typed errors, AgentEvent shape, model allowlist, authentication, retry policy, cost cap

Claude's "proceed autonomously" interpretation for this session: apply the GSD discuss-phase `--auto` flag semantics — auto-select all gray areas, auto-pick the recommended option for each, log inline for user review at CONTEXT.md read-back. Do **not** auto-advance to `/gsd-plan-phase 2` (that needs explicit user review for this load-bearing phase).

---

## Structural integration (kay-core E0583 fix)

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Rename `forge_X/lib.rs` → `forge_X/mod.rs` | Mechanical rename; preserves content byte-identical; 23 renames, one commit; `cargo check --workspace` passes without `--exclude kay-core` | ✓ |
| (b) Thin `forge_X.rs` shim with `include!("forge_X/lib.rs")` | Preserves lib.rs name; adds indirection; same compile result as (a) | |
| (c) Split kay-core into 23 sub-crates | Proper architectural factoring; 5-10× effort; drains Phase 2's real deliverable budget | |

**Selected:** (a)
**Rationale:** Smallest diff, preserves parity-baseline semantic integrity (`forgecode-parity-baseline` tag at commit `8af1f2b` is byte-unmodified; rename is source-equivalent). Unblocks every CI/test escape clause currently in place.

---

## ForgeCode reuse strategy

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Typed wrapper, delegate to ForgeCode's `forge_repo/provider/openai.rs` | OpenRouter already speaks OpenAI-compatible; ForgeCode's impl is proven; minimum rewrite | ✓ |
| (b) Fresh-in-crate rewrite from reqwest + backon | Full control over AgentEvent; parallel codepath corrupts parity guarantee | |
| (c) Literal extraction of `forge_repo/provider/*` into `kay-provider-openrouter` | Medium refactor; fragments import tree; hurts future upstream sync | |

**Selected:** (a)
**Rationale:** EVAL-01a (parity run) has not yet executed. Parity-equivalent behavior is load-bearing for the TB 2.0 score target. A typed wrapper satisfies PROV-01..08 without touching provider semantics. Evolvable post-EVAL-01a.

---

## Tolerant JSON parser

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Two-pass: strict `serde_json` → fallback `forge_json_repair` | Reuses ForgeCode's battle-tested repair; no rewrite | ✓ |
| (b) Single-pass custom permissive parser | Would duplicate what `forge_json_repair` already does correctly | |
| (c) Use `json5` crate for the relaxed syntax | Weaker than `forge_json_repair` on OpenRouter-specific malformations | |

**Selected:** (a)
**Rationale:** `forge_json_repair` literally exists to handle OpenRouter's output variance. Reuse is free parity-preserving value.

---

## Tool-call reassembly

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Per-id `HashMap<String, ToolCallBuilder>` accumulator; emit `Complete` on finish_reason=tool_calls | Matches OpenAI/OpenRouter spec; delta events for UI liveness | ✓ |
| (b) Buffer entire response, parse once at `data: [DONE]` | Loses streaming UI; contradicts PROV-01 | |

**Selected:** (a)
**Rationale:** PROV-01 requires "streaming SSE"; (a) is the only shape that hides fragmentation from consumers while preserving streaming.

---

## Typed errors (`ProviderError`)

Decision captured as initial variant list with `#[non_exhaustive]`. See CONTEXT.md D-05. No alternative considered — PROV-08 is explicit ("typed, not string"); extensibility via `#[non_exhaustive]` is Rust idiom.

---

## Initial `AgentEvent` shape

Decision captured in CONTEXT.md D-06. Phase 5's LOOP-02 formalizes the freeze with `#[non_exhaustive]`; Phase 2 seeds the provider-tier variants (text, tool-call lifecycle, usage, retry, error) while downstream phases add their own (ToolOutput/Phase 3, SandboxViolation/Phase 4, TurnEnd/Phase 5, Verification/Phase 8).

---

## Exacto model allowlist (launch composition)

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Match ForgeCode's reference set | `anthropic/claude-sonnet-4.6`, `anthropic/claude-opus-4.6`, `openai/gpt-5.4` (all Exacto) | ✓ |
| (b) Broader experimental set | Add DeepSeek R2, Qwen 3, Mistral Large 3; more parser variance, unproven on TB 2.0 | |
| (c) Claude-only | Opus 4.6 + Sonnet 4.6 only; safest but single-vendor | |

**Selected:** (a)
**Rationale:** TB 2.0 acceptance gate requires beating ForgeCode's 81.8%. Using the same models eliminates model-variance so the score delta reflects harness quality (our actual wedge), not model choice.

---

## Authentication

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Env var (`OPENROUTER_API_KEY`) OR `kay.toml [provider.openrouter] api_key`; env wins | Matches ForgeCode `provider_auth.rs`; no OAuth (PROV-03) | ✓ |
| (b) OAuth flow | Explicitly forbidden by PROV-03 | |
| (c) Keyring integration | Phase 10 UI concern (UI-04), not Phase 2 | |

**Selected:** (a)
**Rationale:** PROV-03 forbids OAuth; keyring is a Phase 10 UI concern.

---

## Retry policy (429 / 503)

| Option | Description | Selected |
|--------|-------------|----------|
| (a) `backon::ExponentialBackoff`, base 500ms, factor 2, max 3 attempts, 8s ceiling, full jitter; respect `Retry-After` on 429 | Matches PROV-02 + PROV-07; aligned with ForgeCode's `retry.rs` defaults | ✓ |
| (b) Infinite retries with circuit breaker | Unbounded cost risk under sustained 503 | |
| (c) Fixed 3 attempts, fixed 1s spacing | No jitter → thundering herd on rate-limit clear | |

**Selected:** (a)
**Rationale:** Matches ForgeCode parity; explicit PROV-02 + PROV-07 satisfaction; `Retry-After` compliance on 429 is API-polite.

---

## Per-session cost cap

| Option | Description | Selected |
|--------|-------------|----------|
| (a) No default — must be explicit `--max-usd`; unset → uncapped | TB 2.0 harness controls its own cap; our default can't interfere | ✓ |
| (b) Default $5 for interactive, uncapped for `--headless` | Requires mode detection; surprising behavior difference | |
| (c) Default $50 | Arbitrary; either too low for TB 2.0 or too high for casual use | |

**Selected:** (a)
**Rationale:** PROV-06 satisfied with user-explicit opt-in. TB 2.0 Harbor config injects its own cap from the harness side; our default must not pre-empt that.

---

## Claude's Discretion

Items intentionally left flexible for the planner/executor:

- Internal module layout of `kay-provider-openrouter` (single `src/lib.rs` vs. subdirs)
- Test organization (unit vs. integration split, which provider calls get `#[ignore]`)
- `backon` builder API shape (builder pattern variant chosen by executor)
- Mock OpenRouter SSE fixture format (recorded cassette vs. hand-written)
- Whether to expose a synchronous convenience wrapper alongside async streaming

---

## Deferred Ideas

See CONTEXT.md `<deferred>` section — nothing surfaced during auto-resolution that wasn't already in PROJECT.md §Out of Scope or REQUIREMENTS.md §v2.

---

## Meta

**Why auto-resolved, not interactive:** User explicitly said "Proceed autonomously" on session restart and the standing project rule ("Pls proceed autonomously — ask me only if there is any super critical design decision") is well-established. Every decision above is traceable to (a) prior phase artifacts, (b) explicit REQ-IDs, or (c) PROJECT.md Key Decisions. If any decision reads wrong on review, the user can revise before `/gsd-plan-phase 2` — planner will re-read CONTEXT.md at its start.

**User review checkpoint:** CONTEXT.md is the source of truth. Review at `.planning/phases/02-provider-hal-tolerant-json-parser/02-CONTEXT.md` before running `/gsd-plan-phase 2`.
