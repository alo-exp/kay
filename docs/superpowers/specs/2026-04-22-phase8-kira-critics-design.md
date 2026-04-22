# Design: Phase 8 — Multi-Perspective Verification (KIRA Critics)

**Date:** 2026-04-22
**Phase:** 8
**Requirements:** VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04
**Author:** /silver:feature (autonomous mode)
**Status:** Approved (autonomous §10e) — v2 after spec review

---

## Problem

Kay agents self-report task completion. `task_complete` calls `NoOpVerifier.verify()`
which always returns `VerificationOutcome::Pending`. The agent loop only terminates a
turn on `TaskComplete { verified: true, outcome: Pass }` — which never fires. Phase 8
wires three real KIRA critics into the `TaskVerifier` seam so the agent cannot claim
completion unless independent critics agree.

---

## Architecture

### New crate: `kay-verifier`

Dependency direction: `kay-verifier` → `kay-tools` + `kay-provider-openrouter`. This
avoids the reverse (kay-tools must not pull in OpenRouter network code).

```
crates/
  kay-verifier/
    Cargo.toml
    src/
      lib.rs        — pub re-exports: MultiPerspectiveVerifier, VerifierConfig, VerifierMode
      critic.rs     — CriticPrompt, CriticResponse (crate-private)
      mode.rs       — VerifierMode, VerifierConfig
      verifier.rs   — MultiPerspectiveVerifier, implements TaskVerifier
```

### Changes to existing crates

**`crates/kay-tools/src/events.rs`** — ADD two new `AgentEvent` variants.

`AgentEvent` does NOT derive `Clone` (the `Error` variant holds non-Clone types).
**Consequence for emit:** the verifier must move each `AgentEvent::Verification` value
into the stream sink; it cannot retain a copy after emission. The verifier builds the
event fields locally, emits via `(self.stream_sink)(ev)` (consuming move), then
discards — no accumulation after emit.

```rust
/// Emitted once per critic verdict during MultiPerspectiveVerifier.verify().
Verification {
    critic_role: String,   // "test_engineer" | "qa_engineer" | "end_user"
    verdict: String,       // "pass" | "fail"
    reason: String,        // structured critic reason from CriticResponse
    cost_usd: f64,         // cost of this individual critic call
},

/// Emitted when the verifier disables itself due to cost or retry ceiling.
VerifierDisabled {
    reason: String,        // "cost_ceiling_exceeded" | "max_retries_exhausted"
    cost_usd: f64,         // total verifier cost accumulated this session
},
```

**`crates/kay-tools/src/seams/verifier.rs`** — EXPAND `TaskVerifier` trait signature:

```rust
#[async_trait]
pub trait TaskVerifier: Send + Sync {
    /// Phase 8: task_context is a pre-built summary string assembled by the
    /// agent loop (recent tool calls + outputs). Passed via ToolCallContext.
    /// NoOpVerifier ignores it; MultiPerspectiveVerifier feeds it to critics.
    async fn verify(
        &self,
        task_summary: &str,
        task_context: &str,   // NEW: loop-assembled context string, "" if unavailable
    ) -> VerificationOutcome;
}
```

Using `&str` (not `&[AgentEvent]`) avoids the `AgentEvent` non-Clone problem and
simplifies the data flow. The loop builds `task_context` incrementally as a `String`.

**`crates/kay-tools/src/runtime/context.rs`** — ADD `task_context` to `ToolCallContext`:

```rust
pub struct ToolCallContext {
    // ... existing fields ...
    /// Incrementally-built summary of tool calls + outputs this turn.
    /// The agent loop appends to this String as events are processed.
    /// task_complete reads it to pass to the verifier.
    pub task_context: Arc<Mutex<String>>,
}
```

The loop starts with an empty `String`, appends `ToolOutput` summaries and
`ToolCallComplete` names as each event fires. `task_complete.invoke()` takes a
snapshot: `ctx.task_context.lock()?.clone()`.

`NoOpVerifier` updated to accept `task_context: &str` (ignores it).
`task_complete.rs` updated to call `ctx.verifier.verify(&summary, &task_ctx_snapshot)`.
All test mock verifiers updated to the new signature.

**`crates/kay-core/src/loop.rs`** — TWO changes:

1. `RunTurnArgs` gets a new field:
```rust
pub struct RunTurnArgs {
    // ... existing fields ...
    pub verifier_config: VerifierConfig,   // NEW: drives re-work loop
}
```
Every caller of `run_turn` (currently `kay-cli/src/run.rs`) must supply this field.
Default: `VerifierConfig::default()` (Interactive mode, 3 retries, generous ceiling).

2. ADD re-work loop in `run_turn` — outer retry wrapper:
```rust
let mut rework_count: u32 = 0;
let max_rework = args.verifier_config.max_retries; // default 3

// Outer rework loop — wraps the existing model-stream inner loop
loop {
    // ... existing model stream + handle_model_event logic ...

    // handle_model_event now returns ModelOutcome::VerificationFail { reason }
    // in addition to Continue / Exit.
    //
    // On VerificationFail:
    if rework_count < max_rework {
        // Inject critic feedback as a new user message
        append_user_message(&mut messages, &format!("Verification failed: {reason}. Please address these issues."));
        rework_count += 1;
        continue; // restart model stream with updated messages
    } else {
        // Emit VerifierDisabled trace event (already emitted by verifier on cost breach;
        // loop emits it here for the max_retries case)
        emit_agent_event(&event_tx, AgentEvent::VerifierDisabled {
            reason: "max_retries_exhausted".into(),
            cost_usd: 0.0, // loop doesn't track verifier cost; verifier already emitted per-verdict
        }).await;
        return Ok(TurnResult::VerificationFailed);
    }
    // On Exit (Pass): break out of outer loop
    break;
}
```

**`crates/kay-cli/src/run.rs`** — SWAP verifier + supply verifier_config to RunTurnArgs:

```rust
// Before (Phase 3):
Arc::new(NoOpVerifier)

// After (Phase 8):
Arc::new(MultiPerspectiveVerifier::new(
    provider.clone(),
    cost_cap.clone(),
    verifier_config.clone(),
    stream_sink.clone(),
))

// RunTurnArgs construction gains:
RunTurnArgs {
    // ... existing ...
    verifier_config: verifier_config.clone(),
}
```

---

## Data Structures

### CriticPrompt (crate-private in kay-verifier)

```rust
pub(crate) struct CriticPrompt {
    pub role: CriticRole,
    pub system_prompt: &'static str,
}

pub(crate) enum CriticRole {
    TestEngineer,
    QAEngineer,
    EndUser,
}
```

### CriticResponse (crate-private in kay-verifier)

Parsed from structured OpenRouter response. Schema (ForgeCode hardened — `required`
before `properties`, `additionalProperties: false`):

```json
{
  "required": ["verdict", "reason"],
  "properties": {
    "verdict": { "type": "string", "enum": ["pass", "fail"] },
    "reason": { "type": "string" }
  },
  "additionalProperties": false
}
```

The `context` field from earlier brainstorming is DROPPED — it was not surfaced in
`AgentEvent::Verification` and added ambiguity. Only `verdict` + `reason` are required.

### VerifierConfig (pub in kay-verifier)

```rust
pub struct VerifierConfig {
    pub mode: VerifierMode,
    pub max_retries: u32,            // default 3
    pub cost_ceiling_usd: f64,       // per-session verifier budget; default 1.0
    pub model: String,               // default: "openai/gpt-4o-mini" (cheap)
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            mode: VerifierMode::Interactive,
            max_retries: 3,
            cost_ceiling_usd: 1.0,
            model: "openai/gpt-4o-mini".into(),
        }
    }
}

pub enum VerifierMode {
    Interactive,   // 1 critic (EndUser only) — default
    Benchmark,     // 3 critics (TestEngineer + QAEngineer + EndUser)
    Disabled,      // bypass — for --no-verify
}
```

### MultiPerspectiveVerifier

```rust
pub struct MultiPerspectiveVerifier {
    provider: Arc<OpenRouterProvider>,
    cost_cap: Arc<CostCap>,             // shared with session cost tracker
    config: VerifierConfig,
    verifier_cost: Arc<Mutex<f64>>,     // verifier-specific cost accumulator
    stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
}
```

`verifier_cost` is separate from the main `CostCap` so the CI cost regression gate
can measure verifier-specific spend without interference from main-session costs.
Both trackers accumulate independently; `cost_cap` is also updated so total session
spend is correct.

---

## Execution Flow

```
Agent loop builds task_context: String incrementally as tool events fire:
  → ToolCallComplete(name) → append "called tool: {name}\n"
  → ToolOutput(text chunk) → append truncated output

task_complete(summary) invoked by model
  → snapshot: task_ctx = ctx.task_context.lock().clone()
  → ctx.verifier.verify(&summary, &task_ctx)
    → MultiPerspectiveVerifier::verify(summary, task_context)
      → if mode == Disabled: return Pass immediately
      → select critics per VerifierMode
      → FOR each critic:
           → build prompt (system_prompt + summary + task_context)
           → POST to OpenRouter (provider.stream_chat) → parse CriticResponse
           → emit AgentEvent::Verification { role, verdict, reason, cost_usd }
              [Note: emit is a MOVE — event consumed by sink; no clone needed]
           → accumulate: verifier_cost += critic_cost; cost_cap.accumulate(critic_cost)
           → if verifier_cost > config.cost_ceiling_usd:
               emit AgentEvent::VerifierDisabled { reason: "cost_ceiling_exceeded", cost_usd }
               return VerificationOutcome::Pass { note: "verifier disabled: cost ceiling" }
      → IF any FAIL:
           → return VerificationOutcome::Fail { reason: combined_fail_reasons }
      → IF all PASS:
           → return VerificationOutcome::Pass { note: "all critics passed" }

  → MultiPerspectiveVerifier NEVER returns Pending.
    (Pending is only a valid outcome from NoOpVerifier stub — illegal from real impl)

  → task_complete receives VerificationOutcome
  → emits AgentEvent::TaskComplete { verified, outcome }

Agent loop (run_turn outer rework loop) sees TaskComplete:
  → IF Pass + verified=true: break outer loop → return TurnResult::Verified
  → IF Fail + rework_count < max_retries:
      → append user message: "Verification failed: {reason}. Please fix."
      → rework_count += 1
      → CONTINUE outer loop (restart model stream)
  → IF Fail + rework_count >= max_retries:
      → emit AgentEvent::VerifierDisabled { reason: "max_retries_exhausted", cost_usd: 0.0 }
      → return TurnResult::VerificationFailed
```

---

## Error Handling

| Error condition | Behavior |
|----------------|----------|
| OpenRouter call fails (network error) | `tracing::warn!`, treat critic as PASS |
| JSON parse fails on critic response | `tracing::warn!`, treat critic as PASS |
| Cost ceiling exceeded mid-critic | Emit `VerifierDisabled`, return Pass (graceful bypass) |
| Max retries exhausted | Emit `VerifierDisabled { max_retries_exhausted }`, return VerificationFailed |
| Verifier mode = Disabled | Return Pass immediately, no critic calls |
| `VerificationOutcome::Pending` from real verifier | Must never occur — panic in debug, return Fail in release |

Network and parse failures degrade gracefully to PASS so the verifier never blocks
agent progress when OpenRouter is unavailable.

---

## Testing Strategy (Preview)

| Tier | Scope | Key tests |
|------|-------|-----------|
| Unit (T1) | CriticResponse parsing | Parse PASS/FAIL JSON; reject `{ "verdict": "other" }` |
| Unit (T1) | AgentEvent::Verification + VerifierDisabled | Shape, serialization |
| Unit (T1) | MultiPerspectiveVerifier with mock sink | All Pass → Pass; any Fail → Fail; mode = Disabled → Pass |
| Integration (T2) | Re-work loop in kay-core | Fail → message injection → retry → Pass |
| Integration (T2) | Cost ceiling | Accumulate, breach, VerifierDisabled emitted |
| E2E (T3) | Full agent turn with MultiPerspectiveVerifier | Turn terminates on Pass; retries on Fail |
| Property (T4) | CriticResponse parser | proptest: any JSON with verdict field → no panic |

---

## CI Gate: Cost Regression (VERIFY-04)

- Baseline stored in `.planning/phases/08-multi-perspective-verification/cost-baseline.json`
- Fixed input: canonical 5-sentence task summary + 3-tool transcript (pinned fixture)
- CI gate: verifier cost regression >30% vs baseline = FAIL (cargo test parses event stream)
- Gate measures `verifier_cost` accumulator, not main `CostCap`

---

## Backlog Items Closed in This Phase

- **999.6**: Rename `crates/kay-cli/tests/context_e2e.rs` → `context_smoke.rs` + add
  Phase 8 behavioral E2E tests
- **999.7**: `SymbolKind::from_kind_str` unknown arm → emit `tracing::warn!`

---

## Non-Negotiables

1. `TaskVerifier` trait MUST remain dyn-compatible (object-safe) — no `where Self: Sized`
2. `AgentEvent::Verification` and `VerifierDisabled` are additive — no existing variant modified
3. Critics MUST run inside `task_complete.invoke()` via the verifier seam
4. Re-work feedback injection is in the agent loop (`run_turn` outer loop), NOT in `task_complete`
5. All verifier costs accumulate against BOTH the shared `Arc<CostCap>` (correct total session spend) AND the separate `verifier_cost: Arc<Mutex<f64>>` field (CI cost regression isolation). Both counters are mandatory.
6. `MultiPerspectiveVerifier` MUST NEVER return `VerificationOutcome::Pending`
7. `ToolCallContext::task_context` is the sole channel for transcript context — no `Arc<dyn SessionStore>` dep
8. Every commit carries `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`
