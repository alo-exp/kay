# Phase 8 Brainstorm — Multi-Perspective Verification (KIRA Critics)

**Date:** 2026-04-22
**Phase:** 8
**Skill:** product-management:product-brainstorming + superpowers:brainstorming
**Mode:** autonomous (§10e)

---

## Product-Lens Brainstorm

### Problem Definition

Kay agents self-report task completion via `task_complete`. Self-reporting is the exam
student grading their own exam: there is no independent check that the task was
*actually* done well. Without critics, Kay ships broken code, fails TB 2.0 evaluation
tasks, and erodes developer trust.

**Root cause chain:**
```
Agent says "done" → task_complete calls NoOpVerifier → always Pending
→ loop never terminates via Pass → user manually checks
→ broken code reaches CI / TB 2.0 evaluator → failures
```

Phase 8 wires three KIRA critics as an independent examination layer **before**
`task_complete` emits a Pass verdict.

---

### User Segments + Jobs-to-be-Done

| Persona | Situation → Job → Outcome |
|---------|---------------------------|
| **Benchmark runner** (TB 2.0) | Running a benchmark suite → trust that task_complete fired only after the task truly passed → TB 2.0 score >81.8% |
| **Daily interactive dev** | Fixing a bug with Kay → confidence before pushing to CI → no surprise CI failures |
| **Agent chain orchestrator** | Kay is a sub-agent in a larger pipeline → know that each sub-task was verified before the parent chain continues → deterministic orchestration |
| **Open-source contributor** | Extending Kay's critic system → composable, testable critic API → easy to add new critic personas |

---

### Problem Decomposition

```
Task verification failures
├── Structural: tests not run / broken syntax / missing files
│   → Test-engineer critic: "Did the code compile and tests pass?"
├── Quality: edge cases missed, security gaps, off-spec behavior
│   → QA critic: "Does it handle X edge case? Is this safe?"
└── Intent: user asked for X, agent implemented Y
    → End-user critic: "Is this what the user actually wanted?"
```

---

### How Might We Questions

1. HMW ensure critics are *specific enough* to give actionable FAIL feedback (not just "code is wrong")?
2. HMW keep interactive mode fast + cheap (1 critic) while benchmark mode runs the full trio?
3. HMW prevent the re-work loop from becoming a spinning infinite regress?
4. HMW surface per-critic verdicts without overwhelming the event stream?
5. HMW make the cost ceiling *visible* before it trips (not a silent degradation)?
6. HMW make it impossible for a broken NoOpVerifier regression to sneak back in after Phase 8?

---

### Assumption Stress-Test

| Assumption | Confidence | Riskiest? | Test |
|------------|-----------|----------|------|
| Critics are accurate enough to be useful | Medium | **YES** | Compare critic FAIL rate on known-good vs known-bad task completions from TB 2.0 fixtures |
| Re-work loop converges | High | No | Bounded N retries (default 3) prevents infinite loops structurally |
| Cost stays manageable | Medium | No | 3 cheap-model calls ($0.001–0.005 each) = <$0.015/turn baseline |
| Ceiling breach is rare in practice | Medium | No | CLI flag `--verifier-ceiling` makes it tunable |
| Critics don't generate more hallucinations than they catch | Medium | **YES** | Need critic calibration; start with structured prompts, not free-form |

**Riskiest assumption mitigation:** Critic prompts must be *constrained* — the response
schema should be `{ "verdict": "PASS" | "FAIL", "reason": "..." }` not free text.
Structured output reduces hallucination surface dramatically.

---

### Inversion Brainstorm (how to make verification *worse*, then reverse)

| Make it worse | Reversal (what to build) |
|--------------|--------------------------|
| Critics give vague "code is wrong" FAIL | Critics emit structured `{ reason, context, suggested_fix }` |
| Single global cost ceiling for all critics | Per-critic ceiling + global session ceiling |
| Silent ceiling breach → verifier just stops | Explicit `AgentEvent::VerifierDisabled { reason, cost_so_far }` |
| Critics run after loop exit | Critics run inside `task_complete` before verdict emitted |
| All critics mandatory regardless of mode | Mode toggle: 1 critic interactive, 3 critics benchmark |
| Re-work loop retries forever | Max N retries (default 3), then forced `Pass` with trace event |

---

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| TB 2.0 score delta vs NoOpVerifier baseline | +≥2 pp | TB 2.0 eval run comparison |
| False-positive rate (FAIL on correctly-done tasks) | <10% | Held-out task subset from TB 2.0 |
| Re-work loop depth | median ≤1, P95 ≤3 retries/turn | Structured `AgentEvent::Verification` log |
| Cost per verification (benchmark mode, 3 critics) | <$0.15/turn | CostCap accumulator in event stream |
| Cost ceiling breach rate (interactive sessions) | <5% | Event log analysis |
| CI cost regression gate | <+30% vs baseline | CI cost gate (VERIFY-04 requirement) |

---

### Scope Boundaries

**IN (Phase 8):**
- Three critic personas: test-engineer, QA-engineer, end-user
- Sequential critic execution (simplest correct implementation)
- `MultiPerspectiveVerifier` replacing `NoOpVerifier` in `ServicesHandle`
- Bounded re-work loop (N retries per turn, injected as user message)
- `AgentEvent::Verification { critic_role, verdict, reason, cost_usd }` per critic verdict
- `AgentEvent::VerifierDisabled { reason, cost_usd }` trace event on ceiling breach
- Cost accumulation via `CostCap` in `kay-provider-openrouter`
- Mode toggle: 1 critic interactive, 3 critics benchmark (CLI flag `--benchmark-mode`)

**OUT (deferred):**
- Parallel async critic execution
- Critic fine-tuning / RL
- Multi-round critic debate / voting
- Critic confidence scoring / calibration curves
- OpenRouterEmbedder for semantic task context (DL-6 deferred)
- GUI critic dashboard (Phase 9+)
- Critic result caching across similar tasks

---

## Engineering-Lens Brainstorm

### Architecture Options

#### Option A: Extend `kay-tools` crate in-place

- Add `MultiPerspectiveVerifier` to `crates/kay-tools/src/seams/verifier.rs`
- Pros: no new crate, minimal workspace churn
- Cons: `kay-tools` pulls in `kay-provider-openrouter` (for OpenRouter API calls) → circular dep risk if not careful

#### Option B: New `kay-verifier` crate (PREFERRED)

- `crates/kay-verifier/` — new workspace member
- Deps: `kay-tools` (for `TaskVerifier` trait + `VerificationOutcome`), `kay-provider-openrouter` (for API calls + `CostCap`)
- `MultiPerspectiveVerifier` implements `TaskVerifier`
- Pros: clean dependency direction, no circular deps, isolated test surface
- Cons: +1 crate (minor)

**Decision: Option B** — `kay-verifier` crate follows the same pattern as `kay-context` (new crate per major subsystem).

---

### Critic Design

Each critic is a **structured OpenRouter call** with a fixed system prompt and `{ "verdict": "PASS" | "FAIL", "reason": "...", "context": "..." }` response schema.

| Critic Role | System Prompt Focus | Model |
|-------------|--------------------|----|
| `TestEngineer` | Did tests pass? Does code compile? Is the implementation structurally correct? | Haiku (cheap, fast) |
| `QAEngineer` | Edge cases, security, off-spec behavior, incomplete requirements coverage | Haiku |
| `EndUser` | Does this actually solve what the user asked? Natural language interpretation | Haiku |

**Structured output format (required for correctness):**
```json
{
  "verdict": "PASS",
  "reason": "All tests pass, implementation matches the requested behavior",
  "context": "Checked: test coverage, edge cases X/Y/Z"
}
```

---

### Re-Work Loop Design

```
task_complete(summary) called
  → MultiPerspectiveVerifier.verify(summary, transcript) called
    → Critic 1 (TestEngineer) → PASS/FAIL + reason
      → emit AgentEvent::Verification { role: TestEngineer, verdict, ... }
    → Critic 2 (QAEngineer) → PASS/FAIL + reason
      → emit AgentEvent::Verification { role: QAEngineer, verdict, ... }
    → Critic 3 (EndUser) → PASS/FAIL + reason
      → emit AgentEvent::Verification { role: EndUser, verdict, ... }
  
  → If any FAIL:
      → if retries < max_retries:
          → inject critic feedback as user message into transcript
          → return VerificationOutcome::Fail { reason: combined_feedback }
          → loop retries (agent sees Fail, continues working)
      → if retries >= max_retries:
          → return VerificationOutcome::Fail { reason: "max retries exhausted" }
  
  → If all PASS:
      → return VerificationOutcome::Pass { note: "all critics passed" }
  
  → If cost ceiling breached mid-critic:
      → emit AgentEvent::VerifierDisabled { reason, cost_usd }
      → return VerificationOutcome::Pass { note: "verifier disabled: cost ceiling" }
```

---

### AgentEvent Additions (VERIFY-04)

Two new additive variants to `AgentEvent`:

```rust
// Per-critic verdict (emitted during MultiPerspectiveVerifier.verify())
Verification {
    critic_role: String,         // "test_engineer" | "qa_engineer" | "end_user"
    verdict: String,             // "PASS" | "FAIL"
    reason: String,              // structured reason from critic
    cost_usd: f64,               // cost of this critic call
},

// Verifier ceiling breach
VerifierDisabled {
    reason: String,              // "cost_ceiling_exceeded" | "max_retries_exceeded"
    cost_usd: f64,               // total verifier cost this session
},
```

Both are `#[non_exhaustive]`-safe additive changes. They do NOT change `TaskComplete`.

---

### TaskVerifier Trait Evolution

Per Phase 3 comment: "Phase 8 signature gains `&Transcript` arg (03-RESEARCH §8 rec c)":

```rust
// Phase 8 expanded signature
#[async_trait]
pub trait TaskVerifier: Send + Sync {
    async fn verify(
        &self,
        task_summary: &str,
        // Phase 8: transcript provides context for critics
        transcript: Option<&[TranscriptEntry]>,
    ) -> VerificationOutcome;
}
```

`NoOpVerifier` must be updated to accept the new signature (ignores `transcript`).
`TaskCompleteTool` must pass transcript context to the verifier.

---

### Mode Toggle Design

```rust
pub enum VerifierMode {
    /// Single critic (EndUser only — lowest cost, most relevant)
    Interactive,
    /// All three critics — full KIRA trio
    Benchmark,
    /// No verification — for --no-verify bypass
    Disabled,
}
```

Default: `Interactive` in CLI without `--benchmark-mode`. `Benchmark` when `--benchmark-mode` flag passed.

---

### TDD Wave Plan (preliminary, for testing-strategy input)

| Wave | Scope | Tests |
|------|-------|-------|
| W-0 | Crate scaffold + test harness | 0 impl, fixture definitions |
| W-1 | `CriticPrompt` + structured response parsing | Unit: parse PASS/FAIL JSON, reject malformed |
| W-2 | `AgentEvent::Verification` + `VerifierDisabled` variants | Unit: event shape, serialization |
| W-3 | `MultiPerspectiveVerifier` core — single critic path | Unit: mock critic → PASS → VerificationOutcome::Pass |
| W-4 | Full trio + mode toggle | Unit: 3 critics, all PASS / any FAIL |
| W-5 | Re-work loop (retry injection) | Integration: Fail → feedback → retry → Pass |
| W-6 | Cost ceiling + VerifierDisabled | Integration: accumulate cost, ceiling breach → disabled |
| W-7 | kay-cli wiring (swap NoOpVerifier → MultiPerspectiveVerifier) | E2E: full agent turn with real verifier stub |

---

### Key Open Questions (for gsd-discuss-phase)

1. **Transcript type**: What is the concrete type of `TranscriptEntry` accessible in `task_complete`? Is it `kay-session`'s `Session` type, or a lighter `Vec<AgentEvent>` view?
2. **Retry injection mechanism**: When a critic returns FAIL, who injects the feedback as a user message — the `task_complete` tool or the agent loop? (Affects architectural boundary.)
3. **Max retries default**: 3 is the KIRA-cited value — should this be CLI-configurable or hard-coded for Phase 8?
4. **Critic model selection**: Should critics use the same `--model` as the main agent, or a fixed cheap model (Haiku)? Fixed Haiku optimizes for cost but may reduce accuracy.
5. **Ceiling scope**: Is the ceiling per-task-complete call, per-turn, or per-session? SESSION-level is simplest and consistent with CostCap design.
