---
phase: "08-multi-perspective-verification"
plan: "08-01"
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/kay-verifier/Cargo.toml
  - crates/kay-verifier/src/lib.rs
  - crates/kay-verifier/src/mode.rs
  - crates/kay-verifier/src/critic.rs
  - crates/kay-verifier/src/verifier.rs
  - crates/kay-verifier/tests/integration_verifier.rs
  - crates/kay-verifier/tests/cost_ceiling.rs
  - crates/kay-verifier/tests/event_order.rs
  - crates/kay-verifier/tests/compile_fail/dyn_safe.rs
  - crates/kay-verifier/tests/dyn_safety.rs
  - crates/kay-tools/src/events.rs
  - crates/kay-tools/src/seams/verifier.rs
  - crates/kay-tools/src/runtime/context.rs
  - crates/kay-tools/src/builtins/task_complete.rs
  - crates/kay-tools/src/builtins/sage_query.rs
  - crates/kay-core/src/loop.rs
  - crates/kay-cli/src/run.rs
  - crates/kay-cli/tests/verifier_e2e.rs
  - crates/kay-cli/tests/context_smoke.rs
  - crates/kay-context/src/symbol_store.rs
  - .github/workflows/ci.yml
  - .planning/phases/08-multi-perspective-verification/cost-baseline.json
autonomous: true
requirements:
  - VERIFY-01
  - VERIFY-02
  - VERIFY-03
  - VERIFY-04

must_haves:
  truths:
    - "task_complete calls MultiPerspectiveVerifier.verify() and only returns success when all critics pass"
    - "A failing critic verdict injects feedback as a user message and restarts the model stream"
    - "Re-work loop terminates after max_retries (default 3) and returns TurnResult::VerificationFailed"
    - "Verifier disables itself gracefully (returns Pass) when cost_ceiling_usd is breached"
    - "Every critic verdict emits AgentEvent::Verification { critic_role, verdict, reason, cost_usd }"
    - "AgentEvent::VerifierDisabled is emitted on cost ceiling breach and on max-retries exhaustion"
    - "NoOpVerifier is no longer wired in kay-cli/src/run.rs; MultiPerspectiveVerifier is"
    - "CI cost regression gate fails a run if verifier token cost increases >30% vs baseline"
    - "context_e2e.rs is renamed to context_smoke.rs"
    - "SymbolKind::from_kind_str emits tracing::warn! on unknown arm"
  artifacts:
    - path: "crates/kay-verifier/Cargo.toml"
      provides: "kay-verifier crate manifest"
    - path: "crates/kay-verifier/src/lib.rs"
      provides: "pub re-exports: MultiPerspectiveVerifier, VerifierConfig, VerifierMode"
    - path: "crates/kay-verifier/src/mode.rs"
      provides: "VerifierMode enum + VerifierConfig struct with Default impl"
    - path: "crates/kay-verifier/src/critic.rs"
      provides: "CriticRole, CriticResponse, CriticResponseWire (deny_unknown_fields)"
    - path: "crates/kay-verifier/src/verifier.rs"
      provides: "MultiPerspectiveVerifier implementing TaskVerifier"
    - path: "crates/kay-tools/src/events.rs"
      provides: "AgentEvent::Verification and AgentEvent::VerifierDisabled variants"
    - path: "crates/kay-tools/src/seams/verifier.rs"
      provides: "TaskVerifier::verify with task_context: &str second arg"
    - path: "crates/kay-tools/src/runtime/context.rs"
      provides: "ToolCallContext with task_context: Arc<Mutex<String>> (8th field)"
    - path: "crates/kay-core/src/loop.rs"
      provides: "RunTurnArgs.verifier_config + outer re-work loop + TurnResult enum"
    - path: "crates/kay-cli/src/run.rs"
      provides: "MultiPerspectiveVerifier wired; NoOpVerifier removed"
    - path: "crates/kay-cli/tests/verifier_e2e.rs"
      provides: "E2E tests: full turn terminates on pass + re-work loop with session append"
    - path: "crates/kay-cli/tests/context_smoke.rs"
      provides: "Renamed from context_e2e.rs; doc comment clarifies scope"
  key_links:
    - from: "crates/kay-tools/src/builtins/task_complete.rs"
      to: "crates/kay-verifier/src/verifier.rs"
      via: "ctx.verifier.verify(&summary, &task_ctx_snapshot)"
      pattern: "verifier\\.verify"
    - from: "crates/kay-core/src/loop.rs"
      to: "crates/kay-tools/src/builtins/task_complete.rs"
      via: "TaskComplete event triggers outer rework loop decision"
      pattern: "TurnResult::"
    - from: "crates/kay-cli/src/run.rs"
      to: "crates/kay-verifier/src/verifier.rs"
      via: "MultiPerspectiveVerifier::new(...)"
      pattern: "MultiPerspectiveVerifier::new"
    - from: "crates/kay-verifier/src/verifier.rs"
      to: "crates/kay-tools/src/events.rs"
      via: "(self.stream_sink)(AgentEvent::Verification { .. })"
      pattern: "stream_sink.*Verification"
---

<objective>
Replace the `NoOpVerifier` stub with a fully-wired `MultiPerspectiveVerifier` carrying three KIRA critics (test-engineer, QA-engineer, end-user). Critics run inside `task_complete.invoke()` via the `TaskVerifier` seam before any turn is accepted as finished. A bounded re-work loop (max 3 retries) in `kay-core/src/loop.rs` injects critic feedback as user messages and restarts the model stream. A cost ceiling prevents runaway spend; CI gate locks verifier token cost against a baseline.

Purpose: Close VERIFY-01 through VERIFY-04 — the last missing harness component before Tauri/TUI frontends and Terminal-Bench submission. Without real critics, Kay agents self-certify completion, which is the root failure mode KIRA was designed to prevent.

Output:
- New crate `crates/kay-verifier/` with MultiPerspectiveVerifier
- Expanded `TaskVerifier` seam (task_context: &str arg)
- ToolCallContext gains `task_context: Arc<Mutex<String>>` (8th field)
- RunTurnArgs gains `verifier_config: VerifierConfig`; loop gains TurnResult enum + outer re-work loop
- AgentEvent gains Verification + VerifierDisabled variants
- kay-cli wired to MultiPerspectiveVerifier (NoOpVerifier removed)
- Backlog 999.6 closed (context_e2e.rs → context_smoke.rs)
- Backlog 999.7 closed (SymbolKind::from_kind_str tracing::warn!)
- CI cost regression gate (.github/workflows/ci.yml addition)
- All 4 test tiers green (unit, integration, E2E, property)
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/08-multi-perspective-verification/08-TEST-STRATEGY.md
@docs/superpowers/specs/2026-04-22-phase8-kira-critics-design.md
@docs/superpowers/plans/2026-04-22-phase8-kira-critics.md

<interfaces>
<!-- Key contracts the executor must honor. Extracted from codebase at plan-time. -->

From crates/kay-tools/src/seams/verifier.rs (CURRENT — Phase 3 shape, will be expanded in W-3):
```rust
#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    async fn verify(&self, task_summary: &str) -> VerificationOutcome;
}

pub struct NoOpVerifier;
// Returns VerificationOutcome::Pending always

#[non_exhaustive]
pub enum VerificationOutcome {
    Pending { reason: String },
    Pass { note: String },
    Fail { reason: String },
}
```

From crates/kay-tools/src/runtime/context.rs (CURRENT — 7-field shape, W-4 adds 8th):
```rust
#[non_exhaustive]
pub struct ToolCallContext {
    pub services: Arc<dyn ServicesHandle>,
    pub stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
    pub image_budget: Arc<ImageBudget>,
    pub cancel_token: CancellationToken,
    pub sandbox: Arc<dyn Sandbox>,
    pub verifier: Arc<dyn TaskVerifier>,
    pub nesting_depth: u8,  // field 7 — W-4 adds task_context as field 8
}
// ToolCallContext::new() currently takes 7 positional params.
// W-4 adds task_context: Arc<Mutex<String>> as the 8th param.
// Two call sites must be updated: kay-cli/src/run.rs:~324 and
//   kay-tools/src/builtins/sage_query.rs:~225
```

From crates/kay-core/src/loop.rs (CURRENT — RunTurnArgs, no verifier_config yet):
```rust
pub struct RunTurnArgs {
    pub persona: Persona,
    pub control_rx: mpsc::Receiver<ControlMsg>,
    pub model_rx: mpsc::Receiver<Result<AgentEvent, ProviderError>>,
    pub event_tx: mpsc::Sender<AgentEvent>,
    pub registry: Arc<ToolRegistry>,
    pub tool_ctx: ToolCallContext,
    pub context_engine: Arc<dyn kay_context::engine::ContextEngine>,
    pub context_budget: kay_context::budget::ContextBudget,
    // W-5 adds: pub verifier_config: VerifierConfig,
}
// TurnResult enum does NOT exist yet. W-5 creates it:
// pub enum TurnResult { Verified, VerificationFailed, Aborted, Completed }
// run_turn currently returns Result<(), LoopError> — W-5 changes to Result<TurnResult, LoopError>
```

From crates/kay-tools/src/events.rs (CURRENT — AgentEvent without Verification variants):
```rust
#[non_exhaustive]
#[derive(Debug)]  // NOT Clone, NOT Serialize — intentional
pub enum AgentEvent {
    TextDelta { content: String },   // field is `content`, NOT `text`
    ToolCallStart { .. },
    ToolCallComplete { .. },
    ToolOutput { .. },
    TaskComplete { verified: bool, outcome: VerificationOutcome, .. },
    ImageRead { path: String, bytes: Vec<u8> },
    SandboxViolation { .. },
    // W-1 adds:
    // Verification { critic_role: String, verdict: String, reason: String, cost_usd: f64 }
    // VerifierDisabled { reason: String, cost_usd: f64 }
}
// CRITICAL: AgentEvent is NOT Clone. Emit via (self.stream_sink)(ev) — consuming MOVE.
// Extract fields BEFORE the move if you need them after.
```

Import disambiguation (use in kay-verifier):
```rust
use kay_tools::events::AgentEvent as KayAgentEvent;
use kay_provider_openrouter::event::AgentEvent as ProviderAgentEvent;
```

Module path for r#loop (raw identifier — module named `loop`):
```rust
// In imports from outside kay-core:
use kay_core::r#loop::{RunTurnArgs, TurnResult};
// Inside kay-core, files in src/loop.rs are the module directly.
```
</interfaces>
</context>

<!-- ═══════════════════════════════════════════════════════════════════
     WAVE STRUCTURE
     W-0  Scaffold            (plan task 1)
     W-1  CriticResponse + Events   (plan task 2 — RED then GREEN)
     W-2  VerifierConfig + dyn-safety (plan task 3 — RED then GREEN)
     W-3  MultiPerspectiveVerifier core (plan task 4 — RED then GREEN)
     W-4  ToolCallContext + task_complete (plan task 5 — RED then GREEN)
     W-5  Re-work loop + TurnResult (plan task 6 — RED then GREEN)
     W-6  Cost ceiling + event ordering (plan task 7 — RED then GREEN)
     W-7  CLI wiring + E2E + backlogs + CI gate (plan task 8)
     ═══════════════════════════════════════════════════════════════════ -->

<tasks>

<!-- ──────────────────────────────────────────────────────────────────
     WAVE 0: Crate Scaffold
     Goal-backward: Without a compilable kay-verifier skeleton, every
     subsequent wave cannot import the crate. W-0 has no behavior — it
     only establishes the workspace member and empty module stubs that
     compile clean.
     REQ coverage: structural prerequisite for VERIFY-01..04
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="false">
  <name>Task W-0: Register kay-verifier workspace member + crate skeleton</name>
  <files>
    Cargo.toml,
    crates/kay-verifier/Cargo.toml,
    crates/kay-verifier/src/lib.rs,
    crates/kay-verifier/src/mode.rs,
    crates/kay-verifier/src/critic.rs,
    crates/kay-verifier/src/verifier.rs
  </files>
  <action>
Create the `kay-verifier` crate from scratch. Every file must compile as an empty stub — no behavior yet, just enough structure to satisfy `cargo check -p kay-verifier`.

**Step 1 — Workspace registration**

In the root `Cargo.toml`, add `"crates/kay-verifier"` to the `members` array alongside the other `crates/*` entries.

**Step 2 — Create directory layout**

```
crates/kay-verifier/
  Cargo.toml
  src/
    lib.rs
    mode.rs
    critic.rs
    verifier.rs
  tests/
    (empty for now — integration tests added in W-6)
  tests/compile_fail/
    (trybuild fixtures added in W-2)
```

**Step 3 — Cargo.toml for kay-verifier**

```toml
[package]
name = "kay-verifier"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[dependencies]
kay-tools             = { path = "../kay-tools" }
kay-provider-openrouter = { path = "../kay-provider-openrouter" }
kay-provider-errors   = { path = "../kay-provider-errors" }
async-trait           = "0.1"
serde                 = { version = "1", features = ["derive"] }
serde_json            = "1"
tokio                 = { version = "1", features = ["sync"] }
tracing               = "0.1"
futures               = "0.3"

[dev-dependencies]
tokio     = { version = "1", features = ["rt", "macros", "rt-multi-thread"] }
proptest  = "1"
trybuild  = { version = "1", features = ["diff"] }
static_assertions = "1"
wiremock  = "0.6"
```

**Step 4 — src/lib.rs stub**

```rust
mod critic;
mod mode;
mod verifier;

pub use mode::{VerifierConfig, VerifierMode};
pub use verifier::MultiPerspectiveVerifier;
```

**Step 5 — src/mode.rs stub**

```rust
// Phase 8: VerifierMode + VerifierConfig — stubs filled in W-2
pub enum VerifierMode {}
pub struct VerifierConfig {}
```

**Step 6 — src/critic.rs stub**

```rust
// Phase 8: CriticRole + CriticPrompt + CriticResponse — stubs filled in W-1 / W-2
```

**Step 7 — src/verifier.rs stub**

```rust
// Phase 8: MultiPerspectiveVerifier — filled in W-3
pub struct MultiPerspectiveVerifier;
```

**Step 8 — Commit**

```
git add crates/kay-verifier/ Cargo.toml
git commit -m "feat(verifier): W-0 scaffold — register kay-verifier workspace member

VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo check -p kay-verifier 2>&1 | grep -c "^error" || echo 0 errors</automated>
  </verify>
  <done>
    - `cargo check -p kay-verifier` exits 0 with zero errors
    - `crates/kay-verifier/` directory exists with Cargo.toml + src/{lib,mode,critic,verifier}.rs
    - Root Cargo.toml members array contains `"crates/kay-verifier"`
    - Commit message follows DCO format with Signed-off-by
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 1: CriticResponse parsing + AgentEvent variants
     Goal-backward → VERIFY-04 (emit Verification frames per verdict)
     and VERIFY-01 (critic verdicts are parsed from structured JSON).
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-1: CriticResponse JSON parsing + AgentEvent::Verification + VerifierDisabled variants (RED then GREEN)</name>
  <files>
    crates/kay-verifier/src/critic.rs,
    crates/kay-tools/src/events.rs,
    crates/kay-tools/Cargo.toml
  </files>
  <behavior>
    - T1-01a: parse `{"verdict":"pass","reason":"all tests pass"}` → CriticResponse { verdict: Pass, reason }
    - T1-01b: parse `{"verdict":"fail","reason":"test X failed"}` → CriticResponse { verdict: Fail, reason }
    - T1-01c: `{"verdict":"unknown","reason":"x"}` → Err (only "pass"/"fail" accepted)
    - T1-01d: `{"verdict":"pass"}` missing reason → Err
    - T1-01e: `{}` → Err (both fields required)
    - T1-01f: `{"verdict":"pass","reason":"ok","extra":"field"}` → Err (deny_unknown_fields)
    - T1-01g: `"just a string"` (not object) → Err
    - T1-02a: AgentEvent::Verification has fields critic_role: String, verdict: String, reason: String, cost_usd: f64
    - T1-02b: AgentEvent::VerifierDisabled has fields reason: String, cost_usd: f64
    - T1-02c: Verification variant serializes to JSON with all fields
    - T1-02d: VerifierDisabled variant serializes to JSON
    - T1-02f: static_assertions::assert_not_impl_any!(AgentEvent: Clone) — compile-time enforcement
  </behavior>
  <action>
**TDD protocol: RED commit first, then GREEN commit.**

### RED phase

**Step R-1 — Write failing CriticResponse tests in `crates/kay-verifier/src/critic.rs`**

Replace the stub with the full struct + test-only impl that returns `Err("not implemented")` for `from_json`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CriticVerdict { Pass, Fail }

#[derive(Debug, Clone)]
pub(crate) struct CriticResponse {
    pub verdict: CriticVerdict,
    pub reason: String,
}

impl CriticResponse {
    pub(crate) fn from_json(s: &str) -> Result<Self, String> {
        // TODO: implement in GREEN
        Err("not implemented".into())
    }
    pub(crate) fn is_pass(&self) -> bool { self.verdict == CriticVerdict::Pass }
}

#[cfg(test)]
mod tests {
    use super::*;
    // (paste all T1-01a through T1-01g test bodies from 08-TEST-STRATEGY.md §T1-01)
}
```

**Step R-2 — Add phase8_event_tests module to `crates/kay-tools/src/events.rs`**

At the bottom of the file, OUTSIDE any existing mod block, add:

```rust
#[cfg(test)]
mod phase8_event_tests {
    use super::*;

    #[test]
    fn verification_event_has_expected_fields() {
        // Will fail to compile until Verification variant is added
        let ev = AgentEvent::Verification {
            critic_role: "test_engineer".into(),
            verdict: "pass".into(),
            reason: "all tests pass".into(),
            cost_usd: 0.001,
        };
        match ev {
            AgentEvent::Verification { critic_role, verdict, reason, cost_usd } => {
                assert_eq!(critic_role, "test_engineer");
                assert_eq!(verdict, "pass");
                assert!(!reason.is_empty());
                assert!(cost_usd >= 0.0);
            }
            _ => panic!("expected Verification variant"),
        }
    }

    #[test]
    fn verifier_disabled_event_has_expected_fields() {
        let ev = AgentEvent::VerifierDisabled {
            reason: "cost_ceiling_exceeded".into(),
            cost_usd: 1.05,
        };
        match ev {
            AgentEvent::VerifierDisabled { reason, cost_usd } => {
                assert!(!reason.is_empty());
                assert!(cost_usd > 0.0);
            }
            _ => panic!("expected VerifierDisabled variant"),
        }
    }

    #[test]
    fn agent_event_not_clone_static_assertion() {
        static_assertions::assert_not_impl_any!(super::AgentEvent: Clone);
    }
}
```

**Step R-3 — Add `static_assertions = "1"` to `crates/kay-tools/Cargo.toml` [dev-dependencies]**

**Step R-4 — Confirm RED**

```bash
cargo test -p kay-verifier 2>&1 | tail -5
cargo test -p kay-tools -- phase8_event_tests 2>&1 | tail -5
```

Expected: compilation errors (from_json stub returns Err; AgentEvent variants don't exist yet).

**Step R-5 — RED commit**

```
git add crates/kay-verifier/src/critic.rs crates/kay-tools/src/events.rs crates/kay-tools/Cargo.toml
git commit -m "test(verifier): W-1 RED — failing tests for CriticResponse parsing + AgentEvent variants

VERIFY-01, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Implement `CriticResponse::from_json` in `crates/kay-verifier/src/critic.rs`**

Add a wire DTO with `deny_unknown_fields` (ForgeCode JSON schema hardening: `required` before `properties`, `additionalProperties: false`):

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CriticResponseWire {
    verdict: CriticVerdict,
    reason: String,
}

impl CriticResponse {
    pub(crate) fn from_json(s: &str) -> Result<Self, String> {
        let wire: CriticResponseWire =
            serde_json::from_str(s).map_err(|e| format!("parse error: {e}"))?;
        Ok(Self { verdict: wire.verdict, reason: wire.reason })
    }
}
```

**Step G-2 — Add Verification + VerifierDisabled variants to `AgentEvent` in `crates/kay-tools/src/events.rs`**

Locate the `AgentEvent` enum. Add as the LAST variants BEFORE the closing `}`:

```rust
    /// Emitted once per critic verdict during MultiPerspectiveVerifier.verify(). VERIFY-04
    Verification {
        critic_role: String,
        verdict: String,   // "pass" | "fail"
        reason: String,
        cost_usd: f64,
    },

    /// Emitted when the verifier disables itself due to cost or retry ceiling. VERIFY-03
    VerifierDisabled {
        reason: String,    // "cost_ceiling_exceeded" | "max_retries_exhausted"
        cost_usd: f64,
    },
```

**IMPORTANT:** AgentEvent is `#[non_exhaustive]` + `#[derive(Debug)]` ONLY — do NOT add Clone or Serialize to the enum itself. Adding these two variants is purely additive and must not break any existing match arms.

**Step G-3 — Confirm GREEN**

```bash
cargo test -p kay-verifier -- tests 2>&1 | tail -10
cargo test -p kay-tools -- phase8_event_tests 2>&1 | tail -10
cargo clippy -p kay-verifier -p kay-tools -- -D warnings 2>&1 | tail -10
```

All tests must pass. Zero clippy warnings.

**Step G-4 — GREEN commit**

```
git add crates/kay-verifier/src/critic.rs crates/kay-tools/src/events.rs
git commit -m "feat(verifier): W-1 GREEN — CriticResponse parsing + AgentEvent::Verification/VerifierDisabled

CriticResponse::from_json uses deny_unknown_fields (ForgeCode hardening).
AgentEvent gains two additive variants: Verification (VERIFY-04) and
VerifierDisabled (VERIFY-03). Non-Clone constraint preserved and asserted
via static_assertions::assert_not_impl_any!.

VERIFY-01, VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test -p kay-verifier -p kay-tools 2>&1 | tail -15</automated>
  </verify>
  <done>
    - All T1-01a through T1-01g parse tests pass
    - AgentEvent::Verification and AgentEvent::VerifierDisabled variants exist
    - static_assertions::assert_not_impl_any!(AgentEvent: Clone) compiles (meaning AgentEvent is NOT Clone)
    - Two commits exist: W-1 RED (compile-fail) then W-1 GREEN (passing)
    - cargo clippy -p kay-verifier -p kay-tools -- -D warnings exits 0
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 2: VerifierMode, VerifierConfig, CriticRole, Dyn-safety
     Goal-backward → VERIFY-02 (mode controls which critics run),
     VERIFY-03 (cost ceiling in VerifierConfig).
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-2: VerifierMode + VerifierConfig + CriticRole prompts + trybuild dyn-safety (RED then GREEN)</name>
  <files>
    crates/kay-verifier/src/mode.rs,
    crates/kay-verifier/src/critic.rs,
    crates/kay-verifier/tests/compile_fail/dyn_safe.rs,
    crates/kay-verifier/tests/dyn_safety.rs,
    crates/kay-verifier/Cargo.toml
  </files>
  <behavior>
    - T1-03a: VerifierConfig::default() → mode=Interactive, max_retries=3, cost_ceiling_usd=1.0, model="openai/gpt-4o-mini"
    - T1-03b: Interactive mode → critics_for_mode returns [EndUser] only (1 critic)
    - T1-03c: Benchmark mode → [TestEngineer, QAEngineer, EndUser] (3 critics)
    - T1-03d: Disabled mode → [] (no critics)
    - T1-05a: Box&lt;dyn TaskVerifier&gt; compiles (trybuild pass test)
    - T1-05b: Arc&lt;dyn TaskVerifier&gt; compiles (trybuild pass test)
    - System prompts for all three CriticRole variants are non-empty strings
  </behavior>
  <action>
**TDD protocol: RED commit first, then GREEN commit.**

### RED phase

**Step R-1 — Replace `crates/kay-verifier/src/mode.rs` with tests + todo!() impl**

```rust
/// Controls which critics run per turn.
pub enum VerifierMode {
    /// Single critic (EndUser only) — default for interactive sessions.
    Interactive,
    /// All three critics — for benchmark mode.
    Benchmark,
    /// Bypass all verification — for --no-verify.
    Disabled,
}

pub struct VerifierConfig {
    pub mode: VerifierMode,
    pub max_retries: u32,
    pub cost_ceiling_usd: f64,
    pub model: String,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        todo!("fill in W-2 GREEN")  // RED: panics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn default_mode_is_interactive() {
        let cfg = VerifierConfig::default();
        assert!(matches!(cfg.mode, VerifierMode::Interactive));
    }
    #[test] fn default_max_retries_is_3() {
        assert_eq!(VerifierConfig::default().max_retries, 3);
    }
    #[test] fn default_cost_ceiling_is_1_usd() {
        assert!((VerifierConfig::default().cost_ceiling_usd - 1.0).abs() < 1e-9);
    }
    #[test] fn default_model_is_gpt4o_mini() {
        assert_eq!(VerifierConfig::default().model, "openai/gpt-4o-mini");
    }
}
```

**Step R-2 — Append CriticRole + CriticPrompt to `crates/kay-verifier/src/critic.rs`**

After existing code, add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CriticRole { TestEngineer, QAEngineer, EndUser }

impl CriticRole {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            CriticRole::TestEngineer => "test_engineer",
            CriticRole::QAEngineer   => "qa_engineer",
            CriticRole::EndUser      => "end_user",
        }
    }

    pub(crate) fn system_prompt(&self) -> &'static str {
        ""  // RED: empty string — tests will fail
    }
}

pub(crate) struct CriticPrompt { pub role: CriticRole }

#[cfg(test)]
mod critic_role_tests {
    use super::*;
    #[test] fn role_as_str_test_engineer() { assert_eq!(CriticRole::TestEngineer.as_str(), "test_engineer"); }
    #[test] fn role_as_str_qa_engineer()   { assert_eq!(CriticRole::QAEngineer.as_str(), "qa_engineer"); }
    #[test] fn role_as_str_end_user()      { assert_eq!(CriticRole::EndUser.as_str(), "end_user"); }
    #[test] fn system_prompt_nonempty() {
        // Will fail until W-2 GREEN fills in the prompts
        assert!(!CriticRole::TestEngineer.system_prompt().is_empty());
        assert!(!CriticRole::QAEngineer.system_prompt().is_empty());
        assert!(!CriticRole::EndUser.system_prompt().is_empty());
    }
}
```

**Step R-3 — Create trybuild compile-pass test for dyn-safety**

`crates/kay-verifier/tests/compile_fail/dyn_safe.rs`:
```rust
// trybuild compile-pass: TaskVerifier must be usable as a dyn object.
// If this file fails to compile, the trait lost object-safety.
use kay_tools::seams::verifier::TaskVerifier;
fn _uses_as_dyn_object(_v: &dyn TaskVerifier) {}
fn main() {}
```

`crates/kay-verifier/tests/dyn_safety.rs`:
```rust
//! T1-05a/b: TaskVerifier must be dyn-compatible (object-safe).
#[test]
fn task_verifier_is_dyn_compatible() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_fail/dyn_safe.rs");
}
```

**Step R-4 — Confirm RED**

```bash
cargo test -p kay-verifier 2>&1 | tail -10
```

Expected: `default_mode_is_interactive` panics (todo!()); `system_prompt_nonempty` fails (empty string).

**Step R-5 — RED commit**

```
git add crates/kay-verifier/src/mode.rs crates/kay-verifier/src/critic.rs \
        crates/kay-verifier/tests/compile_fail/dyn_safe.rs \
        crates/kay-verifier/tests/dyn_safety.rs \
        crates/kay-verifier/Cargo.toml
git commit -m "test(verifier): W-2 RED — failing tests for VerifierConfig defaults + CriticRole prompts + dyn-safety

T1-03a-d: VerifierConfig::default panics (todo!).
T1-03(prompts): system_prompt returns empty string.
T1-05a/b: trybuild dyn-safety compile-pass fixture added.

VERIFY-01, VERIFY-02

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Implement `VerifierConfig::default()` in `crates/kay-verifier/src/mode.rs`**

```rust
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
```

Also add a `critics_for_mode` function (needed for T1-03b-d and consumed by MultiPerspectiveVerifier in W-3):

```rust
impl VerifierConfig {
    pub fn critics_for_mode(&self) -> &'static [crate::critic::CriticRole] {
        use crate::critic::CriticRole::*;
        match self.mode {
            VerifierMode::Interactive => &[EndUser],
            VerifierMode::Benchmark   => &[TestEngineer, QAEngineer, EndUser],
            VerifierMode::Disabled    => &[],
        }
    }
}
```

**Step G-2 — Fill in CriticRole system prompts in `crates/kay-verifier/src/critic.rs`**

Replace the empty `system_prompt` stub:

```rust
pub(crate) fn system_prompt(&self) -> &'static str {
    match self {
        CriticRole::TestEngineer => {
            "You are a test engineer reviewing a coding task completion. \
             Evaluate whether: (1) the implementation compiles correctly, \
             (2) tests were run and pass, (3) structural requirements are met. \
             Respond with JSON ONLY: {\"verdict\":\"pass\"|\"fail\",\"reason\":\"...\"}. \
             verdict must be exactly 'pass' or 'fail' (lowercase). \
             reason must be a single sentence explaining your verdict."
        }
        CriticRole::QAEngineer => {
            "You are a QA engineer reviewing a coding task completion. \
             Evaluate whether: (1) edge cases are handled, (2) there are no \
             obvious security gaps, (3) behavior matches requirements fully. \
             Respond with JSON ONLY: {\"verdict\":\"pass\"|\"fail\",\"reason\":\"...\"}. \
             verdict must be exactly 'pass' or 'fail' (lowercase). \
             reason must be a single sentence explaining your verdict."
        }
        CriticRole::EndUser => {
            "You are an end user reviewing whether a coding task was completed \
             as requested. Evaluate whether the implementation does what the user \
             asked — not just technically correct, but intent-matching. \
             Respond with JSON ONLY: {\"verdict\":\"pass\"|\"fail\",\"reason\":\"...\"}. \
             verdict must be exactly 'pass' or 'fail' (lowercase). \
             reason must be a single sentence explaining your verdict."
        }
    }
}
```

**Step G-3 — Confirm GREEN**

```bash
cargo test -p kay-verifier 2>&1 | tail -15
cargo clippy -p kay-verifier -- -D warnings 2>&1 | tail -5
```

All W-0, W-1, W-2 tests must pass. Zero clippy warnings.

**Step G-4 — GREEN commit**

```
git add crates/kay-verifier/src/mode.rs crates/kay-verifier/src/critic.rs
git commit -m "feat(verifier): W-2 GREEN — VerifierConfig defaults, CriticRole system prompts, dyn-safety

Default: Interactive mode, 3 retries, \$1.00 ceiling, gpt-4o-mini.
System prompts enforce JSON-only structured output (ForgeCode hardening).
critics_for_mode() selects 1 (Interactive) or 3 (Benchmark) critics.
trybuild dyn-safety test: TaskVerifier remains object-safe.

VERIFY-01, VERIFY-02, VERIFY-03

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test -p kay-verifier 2>&1 | tail -15</automated>
  </verify>
  <done>
    - VerifierConfig::default() returns Interactive / 3 / 1.0 / "openai/gpt-4o-mini"
    - critics_for_mode returns correct slice per mode
    - CriticRole::system_prompt() returns non-empty strings for all three roles
    - trybuild dyn-safety test passes (TaskVerifier is object-safe)
    - Two commits: W-2 RED then W-2 GREEN
    - cargo clippy -p kay-verifier -- -D warnings passes
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 3: TaskVerifier signature expansion + MultiPerspectiveVerifier core
     Goal-backward → VERIFY-01 (critics run inside task_complete),
     VERIFY-04 (Verification events emitted per critic).
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-3: Expand TaskVerifier signature + implement MultiPerspectiveVerifier core (RED then GREEN)</name>
  <files>
    crates/kay-tools/src/seams/verifier.rs,
    crates/kay-verifier/src/verifier.rs
  </files>
  <behavior>
    - T1-07a: NoOpVerifier.verify("summary", "") still returns VerificationOutcome::Pending
    - T1-07b: NoOpVerifier.verify("summary", "big context") returns Pending (context param ignored)
    - T1-04a: All critics PASS → VerificationOutcome::Pass { note: "all critics passed" }
    - T1-04b: First critic FAIL, rest PASS → VerificationOutcome::Fail { reason: combined }
    - T1-04c: All critics FAIL → Fail with combined reasons
    - T1-04d: VerifierMode::Disabled → Pass immediately; zero critic calls; no Verification events
    - T1-04e: All critics PASS in Benchmark mode → exactly 3 Verification events emitted to sink
    - T1-04f: MultiPerspectiveVerifier::verify NEVER returns Pending (all code paths return Pass or Fail)
    - T1-04g: Network error from provider → treat critic as PASS + tracing::warn!
    - T1-04h: Malformed JSON from provider → treat critic as PASS + tracing::warn!
  </behavior>
  <action>
**Pre-step: Verify the kay-provider-openrouter API before writing tests.**

```bash
grep -n "fn uncapped\|pub fn uncapped" crates/kay-provider-openrouter/src/*.rs 2>/dev/null | head -5
grep -n "fn builder\|pub fn builder\|fn new\|pub fn new" crates/kay-provider-openrouter/src/openrouter_provider.rs 2>/dev/null | head -10
```

Note the correct constructor path for `OpenRouterProvider` and `CostCap` — use whatever actually exists. Do not assume `builder().endpoint(...).build()` without confirming.

**TDD protocol: RED commit first, then GREEN commit.**

### RED phase

**Step R-1 — Expand `TaskVerifier` trait signature in `crates/kay-tools/src/seams/verifier.rs`**

Change the trait definition to add `task_context: &str` as the second parameter:

```rust
#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    /// Verify a task-completion summary.
    /// `task_context` is a pre-built string assembled by the agent loop
    /// (recent tool names + truncated outputs). NoOpVerifier ignores it.
    /// MultiPerspectiveVerifier feeds it to critics as context.
    async fn verify(
        &self,
        task_summary: &str,
        task_context: &str,   // NEW: "" when unavailable
    ) -> VerificationOutcome;
}
```

Update `NoOpVerifier::verify` to accept but ignore the new param:

```rust
#[async_trait::async_trait]
impl TaskVerifier for NoOpVerifier {
    async fn verify(&self, _task_summary: &str, _task_context: &str) -> VerificationOutcome {
        VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".into(),
        }
    }
}
```

Update the existing test in verifier.rs to pass the new arg. Find `noop_verifier_returns_pending` and `noop_verifier_never_returns_pass` tests; add `""` as the second argument to every `v.verify(...)` call.

Add the W-3 RED tests:

```rust
    #[tokio::test]
    async fn noop_verifier_accepts_task_context_arg() {
        let v = NoOpVerifier;
        let outcome = v.verify("summary", "tool context string").await;
        match outcome {
            VerificationOutcome::Pending { .. } => {}
            other => panic!("expected Pending, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn noop_verifier_ignores_task_context() {
        let v = NoOpVerifier;
        let a = v.verify("s", "").await;
        let b = v.verify("s", "big context text here").await;
        assert!(matches!(a, VerificationOutcome::Pending { .. }));
        assert!(matches!(b, VerificationOutcome::Pending { .. }));
    }
```

**Step R-2 — Write failing MultiPerspectiveVerifier tests in `crates/kay-verifier/src/verifier.rs`**

Write the full struct + `todo!()` impl + test module. The tests use an in-process `MockProvider` (a newtype over a `Vec<String>` of canned JSON responses). Since `AgentEvent` is non-Clone, the sink captures `(critic_role, verdict, cost_usd)` tuples BEFORE moving the event:

```rust
use std::sync::{Arc, Mutex};
// ... imports ...

pub struct MultiPerspectiveVerifier { /* fields as per spec */ }

impl MultiPerspectiveVerifier {
    pub fn new(
        provider: Arc<OpenRouterProvider>,
        cost_cap: Arc<CostCap>,
        config: VerifierConfig,
        stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
    ) -> Self { /* ... */ }
}

#[async_trait::async_trait]
impl TaskVerifier for MultiPerspectiveVerifier {
    async fn verify(&self, _task_summary: &str, _task_context: &str) -> VerificationOutcome {
        todo!("MultiPerspectiveVerifier not yet implemented")
    }
}
```

Write the test module with T1-04a through T1-04h using a `MockProvider` helper that returns canned `CriticResponse` JSON. The mock provider must be an in-process type (not a network server — save wiremock for W-6).

**Step R-3 — Confirm RED**

```bash
cargo test -p kay-tools 2>&1 | tail -10
cargo test -p kay-verifier -- verifier::tests 2>&1 | tail -10
```

Expected: kay-tools tests pass (NoOpVerifier updated); kay-verifier verifier tests fail (todo!()).

**Step R-4 — RED commit**

```
git add crates/kay-tools/src/seams/verifier.rs crates/kay-verifier/src/verifier.rs
git commit -m "test(verifier): W-3 RED — expand TaskVerifier::verify signature + failing MPV unit tests

TaskVerifier gains task_context: &str second arg.
NoOpVerifier updated to match; existing tests updated.
MultiPerspectiveVerifier struct + todo!() impl + T1-04a-h tests written.

VERIFY-01, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Implement `MultiPerspectiveVerifier::verify` in `crates/kay-verifier/src/verifier.rs`**

Implement the full verify logic following the execution flow in the design spec:

```
1. If mode == Disabled: return Pass immediately (no events, no critic calls)
2. Select critics via config.critics_for_mode()
3. For each critic:
   a. Build prompt: system_prompt + "\n\nTask summary:\n" + task_summary + "\n\nContext:\n" + task_context
   b. POST to OpenRouter via provider (single-turn chat, not streaming, or streaming with accumulator)
   c. Parse response as CriticResponse::from_json
   d. Extract: role_str = critic.as_str(), verdict_str, reason_str, critic_cost
   e. Accumulate: *self.verifier_cost.lock().unwrap() += critic_cost
   f. Call self.cost_cap.accumulate(critic_cost)
   g. EMIT: (self.stream_sink)(AgentEvent::Verification {
          critic_role: role_str.to_string(),
          verdict: verdict_str.to_string(),
          reason: reason_str.to_string(),
          cost_usd: critic_cost,
      })  — this is a MOVE; do not use any field after this line
   h. If verifier_cost > config.cost_ceiling_usd:
        emit VerifierDisabled { reason: "cost_ceiling_exceeded".into(), cost_usd: total }
        return Pass { note: "verifier disabled: cost ceiling exceeded".into() }
   i. On network error or parse error: tracing::warn!; treat as Pass (do not propagate errors)
4. Collect any Fail verdicts (extracted before the move in step d above)
5. If any Fail: return Fail { reason: combined fail reasons }
6. If all Pass: return Pass { note: "all critics passed".into() }
```

**CRITICAL — Non-Clone emit pattern:** Extract all fields you need BEFORE calling stream_sink:

```rust
// CORRECT: extract before move
let role_str = critic.as_str().to_string();
let verdict_str = if response.is_pass() { "pass" } else { "fail" }.to_string();
let reason_str = response.reason.clone();
let is_pass = response.is_pass();
(self.stream_sink)(AgentEvent::Verification {
    critic_role: role_str,
    verdict: verdict_str,
    reason: reason_str,
    cost_usd: critic_cost,
});
// Now use is_pass (not response — it was moved)
if !is_pass { fail_reasons.push(response.reason); }  // WRONG: response already moved
// Keep a copy before the move:
let reason_for_fail = response.reason.clone(); // clone BEFORE the emit
```

**Step G-2 — Confirm GREEN**

```bash
cargo test -p kay-tools -p kay-verifier 2>&1 | tail -20
cargo clippy -p kay-tools -p kay-verifier -- -D warnings 2>&1 | tail -5
```

All T1-04a through T1-04h must pass. Zero clippy warnings.

**Step G-3 — GREEN commit**

```
git add crates/kay-verifier/src/verifier.rs crates/kay-tools/src/seams/verifier.rs
git commit -m "feat(verifier): W-3 GREEN — MultiPerspectiveVerifier core + TaskVerifier sig expansion

MPV runs critics per VerifierMode, emits Verification events (MOVE pattern),
accumulates dual cost counters, disables on ceiling breach, treats errors as Pass.
TaskVerifier::verify(summary, task_context) expanded in kay-tools seam.

VERIFY-01, VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test -p kay-tools -p kay-verifier 2>&1 | tail -20</automated>
  </verify>
  <done>
    - TaskVerifier::verify has two params: task_summary: &str, task_context: &str
    - NoOpVerifier still returns Pending (existing invariant preserved)
    - All T1-04a through T1-04h tests pass
    - MultiPerspectiveVerifier never returns Pending (all code paths return Pass or Fail)
    - Two commits: W-3 RED then W-3 GREEN
    - cargo test -p kay-tools exits 0 (no regressions in existing tests)
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 4: ToolCallContext + task_complete update
     Goal-backward → VERIFY-01 (task_complete passes task_context to verifier).
     The 8th field addition to ToolCallContext has TWO call sites that must
     both be updated atomically to prevent compile failures: run.rs:~324
     and sage_query.rs:~225.
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-4: ToolCallContext::task_context field + task_complete verifier call update (RED then GREEN)</name>
  <files>
    crates/kay-tools/src/runtime/context.rs,
    crates/kay-tools/src/builtins/task_complete.rs,
    crates/kay-tools/src/builtins/sage_query.rs,
    crates/kay-cli/src/run.rs
  </files>
  <behavior>
    - T1-06a: Fresh ToolCallContext → task_context.lock().unwrap().is_empty() == true
    - T1-06b: After append "called tool: fs_read\n" → task_context contains that string
    - T1-06c: Snapshot is independent — mutating original after snapshot doesn't affect snapshot
    - T1-08a: task_complete with PassVerifier stub → emits TaskComplete { verified: true, outcome: Pass }
    - T1-08b: task_complete with FailVerifier stub → emits TaskComplete { verified: false, outcome: Fail }
    - T1-08c: Snapshot of task_context is passed to verifier (not empty string)
  </behavior>
  <action>
**TDD protocol: RED commit first, then GREEN commit.**

### RED phase

**Step R-1 — Add failing tests for task_context to `crates/kay-tools/src/runtime/context.rs`**

First, read the full context.rs file to understand the current `ToolCallContext::new()` signature (7 params). Do not modify the struct yet — only add tests to the existing `#[cfg(test)]` block:

```rust
#[cfg(test)]
mod phase8_ctx_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Helper: build a minimal ToolCallContext with a known task_context.
    // This will fail to compile until the 8th param is added.
    fn make_ctx_with_task_context(initial: String) -> ToolCallContext {
        // TODO: call ToolCallContext::new(..., Arc::new(Mutex::new(initial)))
        // For now leave as compile-error placeholder
        todo!("update after W-4 GREEN adds 8th param")
    }

    #[test]
    fn fresh_task_context_is_empty() {
        // Compile-fail until task_context field exists
        // let ctx = make_ctx_with_task_context(String::new());
        // assert!(ctx.task_context.lock().unwrap().is_empty());
        todo!("RED: task_context field not yet added")
    }
}
```

Note: this RED test panics (todo!) rather than compile-fails, which still satisfies the TDD protocol — the test is RED.

**Step R-2 — Add failing tests for task_complete to `crates/kay-tools/src/builtins/task_complete.rs`**

Read task_complete.rs to find the existing `#[cfg(test)]` block and add:

```rust
    #[tokio::test]
    async fn task_complete_passes_task_context_to_verifier() {
        // RED: this test will panic until the verifier.verify call is updated
        // to pass ctx.task_context snapshot as second argument
        todo!("RED: task_complete does not yet pass task_context to verifier")
    }
```

**Step R-3 — Confirm RED**

```bash
cargo test -p kay-tools -- phase8_ctx_tests 2>&1 | tail -10
cargo test -p kay-tools -- task_complete 2>&1 | tail -10
```

Expected: todo! panics.

**Step R-4 — RED commit**

```
git add crates/kay-tools/src/runtime/context.rs crates/kay-tools/src/builtins/task_complete.rs
git commit -m "test(verifier): W-4 RED — failing tests for ToolCallContext::task_context + task_complete snapshot

T1-06a-c: task_context accumulation tests (todo! panic).
T1-08c: task_complete passes snapshot to verifier (todo! panic).

VERIFY-01

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Add `task_context` field to `ToolCallContext` in `crates/kay-tools/src/runtime/context.rs`**

Read the full file first. Locate the struct definition and `new()` constructor. Add the 8th field:

```rust
// In the struct (add after nesting_depth or as the last field):
/// Incrementally-built summary of tool calls + outputs this turn.
/// Loop appends ToolCallComplete names and ToolOutput summaries.
/// task_complete.invoke() snapshots this before calling verifier.verify().
pub task_context: Arc<Mutex<String>>,
```

In `ToolCallContext::new()`, add the 8th positional param:

```rust
pub fn new(
    services: Arc<dyn ServicesHandle>,
    stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
    image_budget: Arc<ImageBudget>,
    cancel_token: CancellationToken,
    sandbox: Arc<dyn Sandbox>,
    verifier: Arc<dyn TaskVerifier>,
    nesting_depth: u8,
    task_context: Arc<Mutex<String>>,  // NEW: 8th param
) -> Self {
    Self { services, stream_sink, image_budget, cancel_token, sandbox, verifier, nesting_depth, task_context }
}
```

Add a helper method for convenience:

```rust
/// Append a line to the task_context summary string.
pub fn append_task_context(&self, line: &str) {
    if let Ok(mut ctx) = self.task_context.lock() {
        ctx.push_str(line);
        ctx.push('\n');
    }
}

/// Snapshot the current task_context (independent copy).
pub fn snapshot_task_context(&self) -> String {
    self.task_context.lock().map(|g| g.clone()).unwrap_or_default()
}
```

**Step G-2 — Update call sites for `ToolCallContext::new()`**

There are exactly TWO call sites. Both MUST be updated in this step or the workspace will not compile.

**Call site 1: `crates/kay-cli/src/run.rs` (~line 324)**

Find `ToolCallContext::new(` and add `Arc::new(Mutex::new(String::new()))` as the 8th argument.

**Call site 2: `crates/kay-tools/src/builtins/sage_query.rs` (~line 225)**

Find `ToolCallContext::new(` in the `sage_query` tool's inner context construction. Add `Arc::new(Mutex::new(String::new()))` as the 8th argument. Note: for sub-turns (sage_query creates a child context), a fresh empty String is correct — the child turn accumulates its own context independently.

**Step G-3 — Update `task_complete.rs` to pass task_context snapshot to verifier**

Read `crates/kay-tools/src/builtins/task_complete.rs`. Find the `ctx.verifier.verify(...)` call. Change it from:

```rust
let outcome = ctx.verifier.verify(&summary).await;
```

to:

```rust
let task_ctx_snapshot = ctx.snapshot_task_context();
let outcome = ctx.verifier.verify(&summary, &task_ctx_snapshot).await;
```

**Step G-4 — Update the real tests in context.rs to remove todo!()**

Replace the todo!() tests with real assertions that call the new API:

```rust
    fn make_test_ctx() -> ToolCallContext {
        ToolCallContext::new(
            // ... minimal stubs ...
            Arc::new(Mutex::new(String::new())),  // task_context
        )
    }

    #[test]
    fn fresh_task_context_is_empty() {
        let ctx = make_test_ctx();
        assert!(ctx.task_context.lock().unwrap().is_empty());
    }

    #[test]
    fn append_task_context_accumulates() {
        let ctx = make_test_ctx();
        ctx.append_task_context("called tool: fs_read");
        assert!(ctx.task_context.lock().unwrap().contains("fs_read"));
    }

    #[test]
    fn snapshot_is_independent() {
        let ctx = make_test_ctx();
        ctx.append_task_context("line1");
        let snap = ctx.snapshot_task_context();
        ctx.append_task_context("line2");
        assert!(!snap.contains("line2"), "snapshot must be independent");
    }
```

**Step G-5 — Confirm GREEN**

```bash
cargo test --workspace 2>&1 | tail -20
cargo clippy --workspace -- -D warnings 2>&1 | tail -10
```

All tests must pass. Zero clippy warnings. Pay attention to: no orphaned `ToolCallContext::new()` calls anywhere in the workspace.

**Step G-6 — GREEN commit**

```
git add crates/kay-tools/src/runtime/context.rs \
        crates/kay-tools/src/builtins/task_complete.rs \
        crates/kay-tools/src/builtins/sage_query.rs \
        crates/kay-cli/src/run.rs
git commit -m "feat(verifier): W-4 GREEN — ToolCallContext::task_context + task_complete snapshot

ToolCallContext gains task_context: Arc<Mutex<String>> (8th field).
new() updated to 8 params; both call sites updated (run.rs + sage_query.rs).
task_complete.invoke() snapshots task_context and passes to verifier.verify().

VERIFY-01

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test --workspace 2>&1 | tail -20</automated>
  </verify>
  <done>
    - ToolCallContext has task_context: Arc&lt;Mutex&lt;String&gt;&gt; as the 8th field
    - ToolCallContext::new() takes 8 positional params
    - Both call sites updated: kay-cli/src/run.rs and kay-tools/src/builtins/sage_query.rs
    - task_complete.invoke() calls ctx.verifier.verify(&summary, &snapshot)
    - T1-06a/b/c and T1-08a/b/c tests pass
    - cargo test --workspace exits 0 (no regressions)
    - Two commits: W-4 RED then W-4 GREEN
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 5: Re-work loop + TurnResult enum
     Goal-backward → VERIFY-02 (bounded re-work loop with max_retries).
     This is the largest structural change: run_turn gains an outer retry
     wrapper and a new return type. The RED commit must reference
     TurnResult::Verified and RunTurnArgs.verifier_config which don't
     exist yet — a genuine compile-fail RED.
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-5: Re-work loop in run_turn + TurnResult enum + retry property test (RED then GREEN)</name>
  <files>
    crates/kay-core/src/loop.rs,
    crates/kay-core/tests/rework_loop.rs
  </files>
  <behavior>
    - T2-01a: Verifier returns PASS on first call → TurnResult::Verified, no retry
    - T2-01b: Verifier FAIL once then PASS → Verified after 1 retry, feedback message in stream
    - T2-01c: Verifier FAIL × max_retries → VerifierDisabled { max_retries_exhausted } emitted, returns TurnResult::VerificationFailed
    - T2-01d: FAIL × (max_retries - 1) then PASS → Verified after exactly (max_retries - 1) retries
    - T2-01e: Feedback message contains "Verification failed:" + critic's reason string
    - T2-01f: max_retries = 0 → no retries; single FAIL → immediate VerificationFailed
    - T4-02 (proptest): rework_count never exceeds max_retries for any value 0..=10
  </behavior>
  <action>
**TDD protocol: RED commit first (genuine compile-fail), then GREEN commit.**

### RED phase

**Step R-1 — Write the RED integration test file `crates/kay-core/tests/rework_loop.rs`**

This file references `RunTurnArgs { verifier_config: ... }` and `TurnResult::Verified` which do NOT yet exist. This causes a compile-fail RED — the TDD iron law for W-5.

```rust
//! W-5 RED: Integration tests for the re-work loop.
//! These reference RunTurnArgs::verifier_config and TurnResult which don't exist yet.
//! This file MUST FAIL TO COMPILE until W-5 GREEN adds them.

use kay_core::r#loop::{RunTurnArgs, TurnResult};  // TurnResult does not exist yet
use kay_verifier::VerifierConfig;

// Dummy reference to force compile-fail:
fn _ref_verifier_config(_a: RunTurnArgs) {
    // RunTurnArgs::verifier_config does not exist yet — compile error
    let _: VerifierConfig = _a.verifier_config;
}

fn _ref_turn_result() {
    // TurnResult does not exist yet — compile error
    let _ = TurnResult::Verified;
}

// T2-01a through T2-01f tests will be written in GREEN.
```

**Step R-2 — Confirm genuine compile-fail RED**

```bash
cargo test -p kay-core 2>&1 | grep "^error" | head -5
```

Expected: compilation errors for missing `TurnResult`, `verifier_config` field.

**Step R-3 — RED commit**

```
git add crates/kay-core/tests/rework_loop.rs
git commit -m "test(core): W-5 RED — compile-fail tests for TurnResult + RunTurnArgs::verifier_config

References TurnResult::Verified and RunTurnArgs::verifier_config which
do not exist yet. Genuine compile-fail per TDD iron law.

VERIFY-02

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Add `TurnResult` enum and `verifier_config` to `crates/kay-core/src/loop.rs`**

Read the full loop.rs file first. Add after the existing imports:

```rust
/// Outcome of a completed agent turn, returned from run_turn.
/// VerificationFailed means the re-work loop exhausted max_retries.
#[derive(Debug, PartialEq, Eq)]
pub enum TurnResult {
    /// All critics passed (or verifier disabled).
    Verified,
    /// Re-work loop hit max_retries; turn did not pass verification.
    VerificationFailed,
    /// Turn was cancelled via control channel.
    Aborted,
    /// Turn completed without reaching task_complete (model stopped itself).
    Completed,
}
```

Add `verifier_config` to `RunTurnArgs`:

```rust
// In RunTurnArgs struct, add as the last field:
/// Verifier configuration for the outer re-work loop.
/// Drives max_retries and cost_ceiling enforcement.
pub verifier_config: kay_verifier::VerifierConfig,
```

Add `kay-verifier` as a dependency in `crates/kay-core/Cargo.toml`:

```toml
[dependencies]
# ... existing ...
kay-verifier = { path = "../kay-verifier" }
```

**Step G-2 — Add the outer re-work loop in `run_turn`**

Read the full `run_turn` function. The existing function returns `Result<(), LoopError>`. Change to `Result<TurnResult, LoopError>`.

Wrap the existing inner event-processing loop in an outer retry loop:

```rust
pub async fn run_turn(mut args: RunTurnArgs) -> Result<TurnResult, LoopError> {
    let max_rework = args.verifier_config.max_retries;
    let mut rework_count: u32 = 0;

    loop {
        // ── existing inner select! loop runs here ──────────────────────
        // The inner loop returns one of:
        //   InnerOutcome::TaskComplete { verified: true, outcome: Pass } → break outer
        //   InnerOutcome::TaskComplete { verified: false, outcome: Fail { reason } } → retry
        //   InnerOutcome::Aborted → return TurnResult::Aborted
        //   InnerOutcome::Completed → return TurnResult::Completed (model stopped)

        let inner_outcome = run_inner_loop(&mut args).await?;

        match inner_outcome {
            InnerOutcome::TaskComplete { verified: true, .. } => {
                return Ok(TurnResult::Verified);
            }
            InnerOutcome::TaskComplete { verified: false, outcome } => {
                let reason = match &outcome {
                    VerificationOutcome::Fail { reason } => reason.clone(),
                    _ => "verification did not pass".to_string(),
                };
                if rework_count < max_rework {
                    // Inject critic feedback as a new user message
                    let feedback = format!("Verification failed: {reason}. Please address these issues.");
                    append_user_message(&mut args, feedback).await;
                    rework_count += 1;
                    continue; // restart inner loop with updated messages
                } else {
                    // Exhausted retries
                    let _ = args.event_tx.send(AgentEvent::VerifierDisabled {
                        reason: "max_retries_exhausted".into(),
                        cost_usd: 0.0,
                    }).await;
                    return Ok(TurnResult::VerificationFailed);
                }
            }
            InnerOutcome::Aborted => return Ok(TurnResult::Aborted),
            InnerOutcome::Completed => return Ok(TurnResult::Completed),
        }
    }
}
```

**Implementation notes:**
- The `InnerOutcome` is a local enum (crate-private) or the existing return shape from the inner loop. Refactor the existing inner loop to return this shape rather than `()`.
- `append_user_message` adds a message to the args' message list so the next iteration includes the feedback. Consult the existing loop.rs for how messages are stored in `RunTurnArgs` or `args.messages`.
- The loop does NOT track verifier cost — that is done inside `MultiPerspectiveVerifier` itself. The loop only sees `TaskComplete { verified, outcome }` events.
- Update all callers of `run_turn` in `kay-cli/src/run.rs` to handle `Result<TurnResult, LoopError>`.

**Step G-3 — Write the T2-01 + T4-02 tests in `crates/kay-core/tests/rework_loop.rs`**

Replace the compile-fail placeholder with real tests using a mock model + mock verifier:

```rust
// T2-01a through T2-01f as documented in 08-TEST-STRATEGY.md §T2-01
// T4-02 proptest: rework_count never exceeds max_retries
```

Use proptest for T4-02:

```rust
use proptest::prelude::*;
proptest! {
    #[test]
    fn rework_count_never_exceeds_max_retries(max in 0u32..=10u32) {
        // Build a mock verifier that always returns Fail
        // Run rework loop with max_retries = max
        // Assert: VerificationFailed returned after exactly max retries
        // Note: run this in a tokio runtime via tokio::runtime::Builder
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rework_count = rt.block_on(async {
            // ... drive the loop with always-fail mock ...
        });
        prop_assert!(rework_count <= max);
    }
}
```

**Step G-4 — Confirm GREEN**

```bash
cargo test --workspace 2>&1 | tail -20
cargo clippy --workspace -- -D warnings 2>&1 | tail -10
```

All tests must pass. Zero clippy warnings.

**Step G-5 — GREEN commit**

```
git add crates/kay-core/src/loop.rs crates/kay-core/tests/rework_loop.rs crates/kay-core/Cargo.toml
git commit -m "feat(core): W-5 GREEN — TurnResult enum + outer re-work loop + retry property test

run_turn now returns Result<TurnResult, LoopError>.
RunTurnArgs gains verifier_config: VerifierConfig.
Outer loop injects critic feedback as user message; bounded by max_retries.
On exhaustion: emits VerifierDisabled{max_retries_exhausted} + returns VerificationFailed.
T4-02 proptest: rework_count never exceeds max_retries for any value 0..=10.

VERIFY-02

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test --workspace 2>&1 | tail -20</automated>
  </verify>
  <done>
    - TurnResult enum exists with Verified / VerificationFailed / Aborted / Completed variants
    - RunTurnArgs has verifier_config: VerifierConfig field
    - run_turn returns Result&lt;TurnResult, LoopError&gt;
    - T2-01a through T2-01f pass
    - T4-02 proptest passes (rework_count bounded by max_retries)
    - Two commits: W-5 RED (compile-fail) then W-5 GREEN (passing)
    - cargo test --workspace exits 0
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 6: Cost ceiling + event ordering integration tests
     Goal-backward → VERIFY-03 (cost ceiling disables verifier gracefully),
     VERIFY-04 (Verification events emitted in canonical order).
     Uses wiremock for mock HTTP (already in dev-deps from W-0).
     TDD iron law: RED commit must precede GREEN commit.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-6: Cost ceiling + event ordering integration tests (RED then GREEN)</name>
  <files>
    crates/kay-verifier/tests/cost_ceiling.rs,
    crates/kay-verifier/tests/event_order.rs,
    crates/kay-verifier/tests/integration_verifier.rs
  </files>
  <behavior>
    - T2-02a: cost_ceiling=$0.001, first critic costs $0.002 → VerifierDisabled{cost_ceiling_exceeded} emitted; returns Pass
    - T2-02b: ceiling=$0.10, 3 critics each $0.03 → all 3 Verification events; total $0.09 &lt; ceiling → Pass
    - T2-02c: ceiling=$0.10, critic 2 breaches ($0.06 cumulative) → VerifierDisabled after critic 2; critic 3 NOT called; returns Pass
    - T2-02d: both verifier_cost AND cost_cap show same cumulative value after critics
    - T2-02e: after ceiling breach, zero additional critics (no more Verification events after VerifierDisabled)
    - T2-03a: Benchmark mode event order: Verification(TestEngineer) then Verification(QAEngineer) then Verification(EndUser)
    - T2-03b: TaskComplete follows all Verification events in stream
    - T4-01 (proptest): CriticResponse parser never panics on arbitrary input
  </behavior>
  <action>
**TDD protocol: RED commit (stubs that compile but fail assertions), then GREEN commit.**

**Note on wiremock:** These integration tests use `wiremock` to mock the OpenRouter HTTP endpoint. The `AgentEvent::TextDelta` field is named `content` (NOT `text`) — critical for W-6 GREEN when constructing mock SSE responses.

**Import disambiguation required in test files:**
```rust
use kay_tools::events::AgentEvent as KayAgentEvent;
```
Do NOT import `kay_provider_openrouter::event::AgentEvent` in the same file without aliasing.

### RED phase

**Step R-1 — Create `crates/kay-verifier/tests/cost_ceiling.rs` with failing tests**

Write the test file with T2-02a through T2-02e. For the RED phase, write the test structure but have the mock OpenRouter server not yet configured to return cost-exceeding responses. The tests will fail due to wrong event counts or wrong return values.

Alternatively, use a simpler RED: write the assertions against the expected behavior and use a MockVerifier that returns Pass always — the assertions about VerifierDisabled events will fail.

**Step R-2 — Create `crates/kay-verifier/tests/event_order.rs` with failing tests**

Write T2-03a and T2-03b. For RED: write tests that assert event ordering but use a mock provider that returns events in wrong order. The ordering assertions will fail.

**Step R-3 — Add T4-01 proptest to `crates/kay-verifier/src/critic.rs`**

```rust
#[cfg(test)]
mod proptest_parser {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn critic_response_parser_never_panics(s in "\\PC*") {
            // Any arbitrary string → parse attempt → no panic
            // ParseError is acceptable; panic is not
            let _ = CriticResponse::from_json(&s);
        }
    }
}
```

**Step R-4 — Confirm RED**

```bash
cargo test -p kay-verifier -- cost_ceiling 2>&1 | tail -10
cargo test -p kay-verifier -- event_order 2>&1 | tail -10
cargo test -p kay-verifier -- proptest_parser 2>&1 | tail -5
```

Expected: cost_ceiling and event_order tests fail. proptest test should pass (CriticResponse::from_json is already total — Err is acceptable, panic is not).

**Step R-5 — RED commit**

```
git add crates/kay-verifier/tests/cost_ceiling.rs \
        crates/kay-verifier/tests/event_order.rs \
        crates/kay-verifier/src/critic.rs
git commit -m "test(verifier): W-6 RED — failing cost ceiling + event ordering integration tests

T2-02a-e: cost ceiling breach tests (failing — wrong mock setup).
T2-03a-b: event ordering tests (failing — wrong ordering in mock).
T4-01: proptest parser never-panics (should already pass).

VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```

### GREEN phase

**Step G-1 — Implement proper wiremock server setup in `cost_ceiling.rs`**

Each test:
1. Starts a `wiremock::MockServer`
2. Configures it to return canned SSE responses with specific cost values in the `usage` field
3. Builds an `OpenRouterProvider` pointed at the mock server's URL
4. Builds a `MultiPerspectiveVerifier` with the mock provider + configured cost ceiling
5. Collects events via a `Vec<(String, String)>` sink (role + verdict tuples extracted before move)
6. Asserts on returned `VerificationOutcome` and event sequence

**IMPORTANT for wiremock SSE responses:** The mock server must return valid SSE format that `OpenRouterProvider`'s parser accepts. Use the existing SSE cassettes from Phase 2 tests as templates (`crates/kay-provider-openrouter/tests/fixtures/*.sse`). The critic response body must include a JSON `choices[0].message.content` field containing `{"verdict":"pass","reason":"..."}`.

**Step G-2 — Implement proper ordering assertions in `event_order.rs`**

Use a synchronous event collector sink (`Arc<Mutex<Vec<String>>>` where each element is the critic_role extracted before the move). Assert the order matches `["test_engineer", "qa_engineer", "end_user"]` for Benchmark mode.

**Step G-3 — Confirm GREEN**

```bash
cargo test -p kay-verifier 2>&1 | tail -20
cargo clippy -p kay-verifier -- -D warnings 2>&1 | tail -5
```

All T2-02a through T2-02e, T2-03a/b, and T4-01 must pass. Zero clippy warnings.

**Step G-4 — GREEN commit**

```
git add crates/kay-verifier/tests/cost_ceiling.rs \
        crates/kay-verifier/tests/event_order.rs \
        crates/kay-verifier/tests/integration_verifier.rs
git commit -m "feat(verifier): W-6 GREEN — cost ceiling + event ordering integration tests pass

T2-02a-e: wiremock-backed cost ceiling breach scenarios all pass.
T2-03a-b: Verification event order (TestEngineer→QAEngineer→EndUser) verified.
T4-01 proptest: CriticResponse::from_json is total (10k cases, zero panics).

VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test -p kay-verifier 2>&1 | tail -20</automated>
  </verify>
  <done>
    - T2-02a through T2-02e all pass
    - T2-03a and T2-03b pass
    - T4-01 proptest passes (10k cases, zero panics)
    - Two commits: W-6 RED then W-6 GREEN
    - cargo clippy -p kay-verifier -- -D warnings passes
  </done>
</task>


<!-- ──────────────────────────────────────────────────────────────────
     WAVE 7: CLI wiring + E2E tests + backlog 999.6/999.7 + CI gate
     Goal-backward → VERIFY-01..04 (all requirements closed by E2E tests
     proving the full stack works end-to-end), backlog 999.6
     (context_smoke.rs rename), backlog 999.7 (tracing::warn!),
     and VERIFY-03/04 CI cost gate.
     ────────────────────────────────────────────────────────────────── -->
<task type="auto" tdd="true">
  <name>Task W-7: CLI wiring + E2E tests + backlog 999.6/999.7 + CI cost gate</name>
  <files>
    crates/kay-cli/src/run.rs,
    crates/kay-cli/tests/verifier_e2e.rs,
    crates/kay-cli/tests/context_smoke.rs,
    crates/kay-context/src/symbol_store.rs,
    .github/workflows/ci.yml,
    .planning/phases/08-multi-perspective-verification/cost-baseline.json
  </files>
  <behavior>
    - T3-01: Full turn in Benchmark mode with 3 PASS critics → TurnResult::Verified; event stream has exactly 3 Verification events; final event is TaskComplete { verified: true }; no VerifierDisabled event
    - T3-02: Interactive mode, max_retries=1; first task_complete → FAIL; second → PASS; event stream has 1 Verification(fail) then 1 Verification(pass); session transcript (SQLite read-back) shows injected user message; final TurnResult::Verified
    - T3-03: context_smoke.rs exists (renamed from context_e2e.rs); module doc clarifies scope; existing 2 compilation tests still pass; engine_wired_into_run_turn_with_real_verifier test added
    - SymbolKind::from_kind_str: unknown arm emits tracing::warn! (not silent)
  </behavior>
  <action>
**Step 1 — Wire MultiPerspectiveVerifier into `crates/kay-cli/src/run.rs`**

Read run.rs to find where `NoOpVerifier` is constructed. Replace:

```rust
// Before:
Arc::new(NoOpVerifier)

// After:
Arc::new(MultiPerspectiveVerifier::new(
    provider.clone(),           // Arc<OpenRouterProvider>
    cost_cap.clone(),           // Arc<CostCap>
    verifier_config.clone(),    // VerifierConfig — see below
    stream_sink.clone(),        // Arc<dyn Fn(AgentEvent) + Send + Sync>
))
```

Where `verifier_config` is obtained from CLI args or uses `VerifierConfig::default()`. If a `--benchmark` flag exists, set `mode: VerifierMode::Benchmark`. If `--no-verify` exists, set `mode: VerifierMode::Disabled`.

Also add `verifier_config: verifier_config.clone()` to the `RunTurnArgs { ... }` construction.

Add imports at top of run.rs:
```rust
use kay_verifier::{MultiPerspectiveVerifier, VerifierConfig, VerifierMode};
```

**Step 2 — Rename `context_e2e.rs` → `context_smoke.rs` (Backlog 999.6)**

```bash
git mv crates/kay-cli/tests/context_e2e.rs crates/kay-cli/tests/context_smoke.rs
```

Update `crates/kay-cli/tests/context_smoke.rs` — add a module-level doc comment as the first lines:

```rust
//! Compilation smoke checks for Phase 7 context DI seams.
//!
//! These tests verify that the context engine compiles correctly with Phase 7
//! type shapes. Behavioral E2E tests (full agent turn with real context retrieval
//! and MultiPerspectiveVerifier) live in `verifier_e2e.rs` (Phase 8, T3-01/T3-02).
//!
//! Backlog 999.6: renamed from context_e2e.rs (2026-04-22).
```

Move the existing 2 compilation tests unchanged. Add:

```rust
#[test]
fn engine_wired_into_run_turn_with_real_verifier() {
    // Verifies that KayContextEngine and MultiPerspectiveVerifier are used
    // together in RunTurnArgs — compile-only check of type compatibility.
    use kay_context::engine::KayContextEngine;
    use kay_verifier::{MultiPerspectiveVerifier, VerifierConfig};
    // Type existence check — if these types are importable and the test compiles,
    // the wiring is structurally correct.
    fn _type_check(
        _engine: std::sync::Arc<KayContextEngine>,
        _config: VerifierConfig,
    ) {}
    // Test passes at compile time.
}
```

**Step 3 — Add `tracing::warn!` to SymbolKind::from_kind_str (Backlog 999.7)**

Find `SymbolKind::from_kind_str` in `crates/kay-context/src/symbol_store.rs` (or wherever it lives — search with `grep -rn "fn from_kind_str" crates/`).

In the `_` (unknown) arm of the match, add:

```rust
_ => {
    tracing::warn!(kind = s, "unknown symbol kind in database; treating as FileBoundary");
    SymbolKind::FileBoundary
}
```

**Step 4 — Write E2E tests in `crates/kay-cli/tests/verifier_e2e.rs`**

Write T3-01 and T3-02 as documented in 08-TEST-STRATEGY.md §T3:

**T3-01:** Mock OpenRouter server (wiremock) configured to:
1. Return model response that calls `task_complete`
2. Return 3 PASS critic responses for the 3 Benchmark-mode critics

Assert: `TurnResult::Verified`, exactly 3 `Verification` events, final `TaskComplete { verified: true }`.

**T3-02:** Mock OpenRouter server configured to:
1. First `task_complete` invocation: FAIL critic response (1 critic, Interactive mode)
2. Second `task_complete` invocation (after re-work): PASS critic response

Assert: 2 total `Verification` events (1 fail + 1 pass), injected user message in SQLite session transcript, `TurnResult::Verified`.

Use a real SQLite session store for T3-02 (same as Phase 6 tests). Read Phase 6 SUMMARY for the correct `SessionStore::open()` API.

**Step 5 — Create CI cost regression gate**

Create `.planning/phases/08-multi-perspective-verification/cost-baseline.json`:

```json
{
  "fixture_summary": "Fixed 5-sentence task summary + 3-tool transcript",
  "baseline_cost_usd": null,
  "tolerance_pct": 30,
  "note": "baseline_cost_usd populated on first green CI run. null = create baseline, don't fail."
}
```

Add a `cost_regression_gate` integration test in `crates/kay-verifier/tests/integration_verifier.rs`:

```rust
#[tokio::test]
async fn cost_regression_gate() {
    const FIXTURE_SUMMARY: &str = "Fixed 5-sentence task summary for cost regression. \
        The task was to implement a function that parses CSV files. \
        The implementation reads the file line by line. \
        Each line is split on commas. The result is a Vec<Vec<String>>.";
    const FIXTURE_CONTEXT: &str = "called tool: fs_read\ncalled tool: fs_write\ncalled tool: execute_commands\n";

    // Load baseline from file (or skip if null)
    let baseline_path = ".planning/phases/08-multi-perspective-verification/cost-baseline.json";
    let baseline: serde_json::Value = std::fs::read_to_string(baseline_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let baseline_cost = baseline.get("baseline_cost_usd").and_then(|v| v.as_f64());

    // Run verifier with Benchmark mode + wiremock (or mock provider)
    // Measure verifier_cost accumulator value
    let actual_cost: f64 = 0.0; // TODO: drive real mock and read verifier_cost

    if let Some(baseline) = baseline_cost {
        let tolerance = baseline * 1.30;
        assert!(
            actual_cost <= tolerance,
            "cost regression: actual ${actual_cost:.4} exceeds 130% of baseline ${baseline:.4}"
        );
    } else {
        // No baseline — create it
        let mut val = baseline.clone();
        val["baseline_cost_usd"] = serde_json::json!(actual_cost);
        let _ = std::fs::write(baseline_path, serde_json::to_string_pretty(&val).unwrap());
        eprintln!("cost-baseline.json created with baseline ${actual_cost:.4}");
    }
}
```

Update `.github/workflows/ci.yml` — add a cost gate job (or step) after the main `cargo test` step:

```yaml
      - name: Verifier cost regression gate
        run: cargo test -p kay-verifier -- cost_regression_gate --nocapture
```

**Step 6 — Confirm full workspace GREEN**

```bash
cargo test --workspace 2>&1 | tail -30
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -10
cargo fmt --workspace --check 2>&1 | tail -5
```

All 4 test tiers must pass (unit, integration, E2E, property). Zero clippy warnings. Zero fmt diffs.

**Step 7 — Commit W-7**

```
git add crates/kay-cli/src/run.rs \
        crates/kay-cli/tests/verifier_e2e.rs \
        crates/kay-cli/tests/context_smoke.rs \
        crates/kay-context/src/symbol_store.rs \
        .github/workflows/ci.yml \
        .planning/phases/08-multi-perspective-verification/cost-baseline.json
git commit -m "feat(cli): W-7 — wire MultiPerspectiveVerifier + E2E tests + backlog 999.6/999.7 + CI gate

kay-cli/src/run.rs: NoOpVerifier removed; MultiPerspectiveVerifier wired.
RunTurnArgs gains verifier_config; run_turn result handled as TurnResult.
T3-01/T3-02: E2E tests prove full verification stack end-to-end.
Backlog 999.6: context_e2e.rs → context_smoke.rs (doc comment clarifies scope).
Backlog 999.7: SymbolKind::from_kind_str emits tracing::warn! on unknown arm.
CI: cost_regression_gate step added to ci.yml.

VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
```
  </action>
  <verify>
    <automated>cargo test --workspace 2>&1 | tail -30 && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5 && cargo fmt --workspace --check 2>&1 | tail -5</automated>
  </verify>
  <done>
    - NoOpVerifier is NOT used in kay-cli/src/run.rs; MultiPerspectiveVerifier is
    - T3-01 and T3-02 E2E tests pass
    - crates/kay-cli/tests/context_smoke.rs exists; context_e2e.rs does not
    - context_smoke.rs has module-level doc comment + engine_wired_into_run_turn_with_real_verifier test
    - SymbolKind::from_kind_str emits tracing::warn! on unknown arm
    - cost-baseline.json exists in .planning/phases/08-multi-perspective-verification/
    - CI cost regression gate step added to .github/workflows/ci.yml
    - cargo test --workspace exits 0
    - cargo clippy --workspace --all-targets -- -D warnings exits 0
    - cargo fmt --workspace --check exits 0
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| agent loop → MultiPerspectiveVerifier | Task summary and context strings assembled by the agent loop are passed to critics. If an attacker can inject into `task_context`, they could craft prompts that bias critics toward Pass. |
| MultiPerspectiveVerifier → OpenRouter API | Critic prompts are sent to an external LLM. Response parsing must be strict — invalid JSON or unexpected verdicts must not produce Pass. |
| critic JSON response → CriticResponse::from_json | OpenRouter may return unexpected JSON shapes (null fields, extra fields, wrong types). The parser must be total and strict. |
| re-work loop → message injection | The agent loop injects critic feedback as a user message. This string comes from critic responses and is injected into the conversation. Improperly sanitized feedback could confuse the model in future turns. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-08-01 | Tampering | `task_context` accumulation in `ToolCallContext` | mitigate | `append_task_context` is called only from loop.rs (internal to kay-core); no external input path. Tool names and output snippets are controlled by the tool registry, not user input. |
| T-08-02 | Spoofing | CriticResponse verdict field | mitigate | `CriticResponseWire` uses `#[serde(deny_unknown_fields)]` and `CriticVerdict` enum only accepts "pass"/"fail". Any other string is a hard parse error (not a default). |
| T-08-03 | Denial of Service | Unbounded cost accumulation | mitigate | `cost_ceiling_usd` default $1.00; breach emits `VerifierDisabled` and returns `Pass` — does not halt the agent, just disables further critic calls gracefully. |
| T-08-04 | Information Disclosure | Critic prompts include `task_context` | accept | `task_context` contains tool names and truncated output already visible to OpenRouter via the main prompt. No new information surface. Low risk. |
| T-08-05 | Elevation of Privilege | re-work feedback injection into conversation | mitigate | Feedback string is prefixed with `"Verification failed: "` and sourced from CriticResponse.reason (a string parsed from OpenRouter, not user input). Injection from a compromised OpenRouter response is accepted risk (OpenRouter is a trusted provider boundary per PROV-03). The feedback does not grant tool access. |
| T-08-06 | Repudiation | Verification events missing from session transcript | mitigate | `AgentEvent::Verification` and `VerifierDisabled` events are emitted through the same `stream_sink` as other events and are appended to the JSONL transcript by Phase 6 session store fan-out. CI event_order tests lock the ordering invariant. |
| T-08-07 | Denial of Service | Infinite re-work loop | mitigate | Outer loop is bounded by `max_retries` (default 3). Exhaustion returns `TurnResult::VerificationFailed`, NOT an infinite loop. T4-02 proptest locks this invariant for all values 0..=10. |
| T-08-08 | Tampering | Non-Clone AgentEvent bypassed via Clone derive | mitigate | `static_assertions::assert_not_impl_any!(AgentEvent: Clone)` in W-1 RED tests. Any future `#[derive(Clone)]` addition will cause CI compile failure. |
</threat_model>

<verification>
Phase 8 is complete when ALL of the following are true:

**Functional gates:**
1. `cargo test --workspace` passes — all 4 tiers: unit (~20 tests), integration (6 tests), E2E (2 tests), property (2 proptest suites)
2. `cargo clippy --workspace --all-targets -- -D warnings` exits 0
3. `cargo fmt --workspace --check` exits 0

**Structural gates:**
4. `AgentEvent::Verification` and `AgentEvent::VerifierDisabled` variants exist in `crates/kay-tools/src/events.rs`
5. `MultiPerspectiveVerifier` is wired in `crates/kay-cli/src/run.rs` — `NoOpVerifier` is NOT used
6. `TaskVerifier::verify` takes `(task_summary: &str, task_context: &str)` — two params
7. `ToolCallContext::new()` takes 8 positional params (task_context is the 8th)
8. `RunTurnArgs` has `verifier_config: VerifierConfig` field
9. `TurnResult` enum exists in `crates/kay-core/src/loop.rs`

**Re-work loop gate:**
10. `run_turn` returns `Result<TurnResult, LoopError>`
11. Outer loop in `run_turn` is bounded by `max_retries` (default 3)
12. `VerifierDisabled { reason: "max_retries_exhausted" }` is emitted on exhaustion

**Cost ceiling gate:**
13. `VerifierDisabled { reason: "cost_ceiling_exceeded" }` is emitted when `verifier_cost > cost_ceiling_usd`
14. Verifier returns `Pass` (not `Fail`) after ceiling breach — does not block completion
15. Both `verifier_cost` AND `cost_cap` accumulators are updated on every critic call

**Backlog gates:**
16. `crates/kay-cli/tests/context_smoke.rs` exists; `context_e2e.rs` does NOT exist
17. `context_smoke.rs` has module-level doc comment explaining Phase 8 behavioral coverage lives in `verifier_e2e.rs`
18. `SymbolKind::from_kind_str` unknown arm emits `tracing::warn!(kind = s, "unknown symbol kind...")`

**CI gates:**
19. `.planning/phases/08-multi-perspective-verification/cost-baseline.json` exists
20. `.github/workflows/ci.yml` has a `cost_regression_gate` step

**TDD discipline gate:**
21. Each wave (W-1 through W-7) has at least one RED commit preceding the GREEN commit — visible in `git log --oneline`

**Run this verification command set:**
```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked"
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error"
cargo fmt --workspace --check 2>&1 | grep "^Diff"
grep -n "NoOpVerifier" crates/kay-cli/src/run.rs && echo "FAIL: NoOpVerifier still wired" || echo "OK: NoOpVerifier removed"
grep -n "Verification\|VerifierDisabled" crates/kay-tools/src/events.rs | grep "^crates" | head -5
ls crates/kay-cli/tests/context_smoke.rs && echo "OK: smoke renamed" || echo "FAIL: smoke file missing"
ls crates/kay-cli/tests/context_e2e.rs && echo "FAIL: old file still exists" || echo "OK: old file removed"
grep -n "tracing::warn!" crates/kay-context/src/symbol_store.rs | head -3
```
</verification>

<success_criteria>
Phase 8 is done when:

1. **VERIFY-01 closed:** Critics run inside `task_complete.invoke()` via `ctx.verifier.verify(summary, ctx_snapshot)` before any turn is accepted as finished. `MultiPerspectiveVerifier` never returns `Pending`.

2. **VERIFY-02 closed:** Re-work loop in `kay-core/src/loop.rs` is bounded by `verifier_config.max_retries` (default 3). On exhaustion: `VerifierDisabled { reason: "max_retries_exhausted" }` emitted, `TurnResult::VerificationFailed` returned. T4-02 proptest locks this for all values 0..=10.

3. **VERIFY-03 closed:** When `verifier_cost > cost_ceiling_usd`, verifier emits `VerifierDisabled { reason: "cost_ceiling_exceeded" }` and returns `VerificationOutcome::Pass` — graceful degradation. CI cost regression gate blocks >30% token cost increase vs baseline.

4. **VERIFY-04 closed:** Every critic verdict emits `AgentEvent::Verification { critic_role, verdict, reason, cost_usd }`. Benchmark mode emits 3 events per turn (TestEngineer → QAEngineer → EndUser in that order). T2-03 locks event ordering.

5. **Backlog 999.6 closed:** `crates/kay-cli/tests/context_e2e.rs` renamed to `context_smoke.rs`; module doc comment added; behavioral E2E tests live in `verifier_e2e.rs`.

6. **Backlog 999.7 closed:** `SymbolKind::from_kind_str` unknown arm emits `tracing::warn!` — no silent `FileBoundary` mapping.

7. **`cargo test --workspace` green** — all 4 test tiers.

8. **`cargo clippy --workspace --all-targets -- -D warnings` clean.**

9. **`cargo fmt --workspace --check` clean.**

10. **DCO on every commit** — `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>` present.

11. **All work on branch `phase/08-multi-perspective-verification`** — no direct commits to `main`.
</success_criteria>

<output>
After all waves complete, create:
`.planning/phases/08-multi-perspective-verification/08-01-SUMMARY.md`

Include:
- Wave-by-wave commit log (wave, commit SHA, message)
- Any Rule-1/2/3 deviations encountered and how they were resolved
- Actual crate dependency graph added (verify no circular deps: `cargo tree -p kay-verifier`)
- Test counts per tier (unit / integration / E2E / property)
- Cost baseline value captured in cost-baseline.json
- Files created vs files modified
- Any blocking issues encountered with resolutions
</output>

---

## Source Coverage Audit

| Source | Item | Covered By |
|--------|------|------------|
| GOAL (Phase 8 ROADMAP) | Critics run before task_complete accepts turn | W-3 (MPV core), W-4 (task_complete update), W-7 (CLI wiring) |
| GOAL | Bounded re-work loop | W-5 (outer loop) |
| GOAL | Cost ceilings | W-3 (ceiling check), W-6 (cost ceiling tests) |
| GOAL | AgentEvent::Verification | W-1 (variants), W-3 (emit logic) |
| REQ VERIFY-01 | Critics run inside task_complete.invoke() via TaskVerifier seam | W-3, W-4 |
| REQ VERIFY-02 | Re-work loop bounded by max_retries (default 3); exhaustion → TurnResult::VerificationFailed | W-5 |
| REQ VERIFY-03 | Cost ceiling; breach emits VerifierDisabled; graceful pass | W-3 (impl), W-6 (tests) |
| REQ VERIFY-04 | Per-critic verdict emits AgentEvent::Verification | W-1 (variant), W-3 (emit), W-6 (order tests) |
| BACKLOG 999.6 | Rename context_e2e.rs → context_smoke.rs + Phase 8 E2E tests | W-7 |
| BACKLOG 999.7 | SymbolKind::from_kind_str unknown arm → tracing::warn! | W-7 |
| DECISION | New crate crates/kay-verifier/ | W-0 |
| DECISION | TaskVerifier::verify signature adds task_context: &str | W-3 |
| DECISION | ToolCallContext gains task_context: Arc&lt;Mutex&lt;String&gt;&gt; (8th param) | W-4 |
| DECISION | Re-work loop in kay-core/src/loop.rs | W-5 |
| DECISION | TDD iron law: RED commit before GREEN per wave | W-1 through W-7 |
| DECISION | DCO on every commit | W-0 through W-7 |
| DECISION | Branch phase/08-multi-perspective-verification | W-0 (first commit) |
</output>
