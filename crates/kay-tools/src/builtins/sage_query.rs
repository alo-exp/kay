//! `sage_query` — delegate a focused research question to the `sage`
//! persona as a nested agent turn (LOOP-03 / CLI-07).
//!
//! # Why this tool exists
//!
//! `forge` (the planner persona) and `muse` (the docs/reading persona)
//! often need a fast context lookup that does not warrant reshaping
//! their own message history: "what does `foo::Bar` do?", "find the
//! public API for X", "summarize this README". Issuing that question
//! against the planner's own context inflates every subsequent token,
//! costs more, and pollutes the main thread of thought.
//!
//! `sage_query` solves that by spawning a SUB-turn with a different
//! persona (`sage` — faster model, smaller context budget, read-only
//! tool set) and routing ONLY its streamed answer back to the caller's
//! event sink. From the parent's point of view, `sage_query` behaves
//! like any other tool: send a prompt, get a result, one event stream.
//! From sage's point of view, it's just another turn.
//!
//! # Why a runtime nesting guard
//!
//! Sage's YAML `tool_filter` already excludes `sage_query`, so a
//! well-configured deployment can never recurse. But filter-based
//! exclusions are fragile — a future YAML edit, a dynamic-tool-
//! injection feature, or a persona clone could accidentally punch a
//! hole through that guard. `NestingDepthExceeded` is the belt-and-
//! suspenders runtime check that catches any such drift before it
//! turns into a billable infinite-recursion incident.
//!
//! Two independent guards (YAML + runtime) means an attacker needs to
//! compromise both to induce recursion. That's a more useful security
//! property than "one guard at one layer".
//!
//! # Depth semantics
//!
//! - Top-level turn (user → forge): `ctx.nesting_depth == 0`
//! - forge calls `sage_query` → inner sage turn at depth 1 (allowed)
//! - If that sage turn somehow called `sage_query` → depth 2 (allowed
//!   — this is why the limit is 2, not 1; allows at most one level of
//!   indirection without being paranoid)
//! - Anything that would create depth 3+ → `NestingDepthExceeded`
//!
//! The limit is the PARENT depth at which invoke is rejected:
//! `invoke()` checks `ctx.nesting_depth >= MAX_NESTING_DEPTH` where
//! `MAX_NESTING_DEPTH = 2`. A parent at depth 2 would create a child
//! at depth 3 — that is the rejection threshold.
//!
//! # The `InnerAgent` seam
//!
//! Production `InnerAgent` impls wrap the OpenRouter client +
//! `Persona::load("sage")` + `kay_core::r#loop::run_turn`. Unit tests
//! (see `crates/kay-tools/tests/sage_query.rs`) inject a stub that
//! records the passed context and emits canned events — the same
//! dependency-injection pattern the `Sandbox` and `TaskVerifier`
//! seams already use in `ToolCallContext` (D-12 in 03-CONTEXT.md).
//!
//! `InnerAgent::run` takes the inner context by value because
//! ownership moves into the sub-turn for its full lifetime; a
//! `&ToolCallContext` would force the caller to construct and hold
//! the fresh context across an await point for a reason that has
//! nothing to do with borrowing.

use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{ToolName, ToolOutput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

/// Maximum legal value of `ctx.nesting_depth` at which `sage_query`
/// will still proceed. A parent context at depth `MAX_NESTING_DEPTH`
/// is rejected because the inner sub-turn would be at depth
/// `MAX_NESTING_DEPTH + 1`, which is one deeper than the ceiling.
///
/// Value 2 is the LOOP-03 locked ceiling: allows the common "planner
/// → sage" (depth 1) and the rare "planner → sage → sage" double-hop
/// (depth 2) patterns, rejects anything deeper. See 05-PLAN.md §Wave 5
/// and 05-BRAINSTORM.md §Engineering-Lens §E-10 for the rationale.
pub const MAX_NESTING_DEPTH: u8 = 2;

/// Seam used by `SageQueryTool` to spawn the inner agent turn.
///
/// Production: a thin wrapper around
/// `kay_core::r#loop::run_turn(RunTurnArgs { persona: sage_persona, ... })`
/// plus an OpenRouter provider adapter. Wired up in Wave 7 when the
/// CLI learns to configure sub-personas.
///
/// Tests: a recording stub that captures the prompt + passed context
/// and emits canned events through `ctx.stream_sink`. See
/// `crates/kay-tools/tests/sage_query.rs` for the reference impl.
///
/// Returning `Result<ToolOutput, ToolError>` — not a bare
/// `Result<(), _>` — so the inner agent can surface a structured
/// summary back to the caller as the tool's return value. The streamed
/// side-channel (events via `stream_sink`) is for the UI; the
/// returned `ToolOutput` is for the model's message history.
#[async_trait]
pub trait InnerAgent: Send + Sync + 'static {
    /// Execute one nested turn with the given prompt and a
    /// sage_query-prepared inner context. Implementations must:
    ///
    ///   (a) route all of their produced `AgentEvent`s through
    ///       `ctx.stream_sink` (not build a separate sink),
    ///
    ///   (b) respect `ctx.sandbox` / `ctx.verifier` / `ctx.services`
    ///       / `ctx.image_budget` as-is — sage_query has already set
    ///       these to the parent's values so policy is inherited,
    ///
    ///   (c) honor `ctx.cancel_token` so parent-turn cancellation
    ///       propagates cleanly into the sub-turn.
    async fn run(&self, prompt: String, ctx: ToolCallContext) -> Result<ToolOutput, ToolError>;
}

/// JSON-schema input for `sage_query`. One required field: `prompt`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct SageQueryArgs {
    /// The research question to delegate to sage. Should be self-
    /// contained — sage starts with no access to the caller's context
    /// history beyond this string.
    pub prompt: String,
}

/// The `sage_query` tool. Construct with an `InnerAgent` impl:
/// production wires in the real OpenRouter-backed agent, tests wire
/// in a recording stub.
pub struct SageQueryTool {
    name: ToolName,
    description: String,
    input_schema: Value,
    inner: Arc<dyn InnerAgent>,
}

impl SageQueryTool {
    /// Build a new `sage_query` tool bound to the given inner-agent
    /// implementation.
    ///
    /// The `Arc<dyn InnerAgent>` is owned rather than borrowed
    /// because `SageQueryTool` itself is typically owned by the
    /// `ToolRegistry` through `Arc<dyn Tool>`, and `Tool::invoke`
    /// is called across await points where borrowing an inner-agent
    /// reference would constrain lifetime in ways the registry's
    /// single-Arc-per-tool pattern does not support.
    pub fn new(inner: Arc<dyn InnerAgent>) -> Self {
        let name = ToolName::new("sage_query");
        let description = "Delegate a focused research question to the `sage` sub-agent. \
            Returns sage's answer as the tool result; sage's streamed events flow \
            through the caller's event sink so the UI renders one continuous stream."
            .to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(SageQueryArgs))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints { output_truncation_note: None },
        );
        Self { name, description, input_schema: schema, inner }
    }
}

#[async_trait]
impl Tool for SageQueryTool {
    fn name(&self) -> &ToolName {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        _call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        // ── Depth guard (LOOP-03) ──────────────────────────────────
        // Check BEFORE parsing args so a mal-crafted prompt that
        // violates the schema still surfaces the more important
        // error (recursion) first. A depth-exceeded recursive chain
        // with 40 layers of bad JSON would spam the caller with 40
        // `InvalidArgs` before the caller realized the real problem
        // was runaway recursion — the depth check is strictly
        // cheaper than JSON parsing and catches it at layer 1.
        if ctx.nesting_depth >= MAX_NESTING_DEPTH {
            return Err(ToolError::NestingDepthExceeded {
                depth: ctx.nesting_depth,
                limit: MAX_NESTING_DEPTH,
            });
        }

        // ── Parse args ─────────────────────────────────────────────
        let input: SageQueryArgs = serde_json::from_value(args).map_err(|e| {
            ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() }
        })?;

        // ── Build inner context ────────────────────────────────────
        // Every seam (services, sandbox, verifier, image_budget) is
        // CLONED from the parent so the sub-turn operates under the
        // same policy envelope — no privilege escalation possible via
        // sage_query recursion. The stream_sink is also inherited so
        // the sub-turn's events surface in the parent's event stream
        // (one UI, one event log).
        //
        // The cancel_token is cloned (not child-tokenized) for MVP:
        // cancelling the parent should cancel the child. A future
        // wave can switch to `child_token()` if finer-grained abort
        // semantics prove necessary, but the simpler "same token"
        // shape is sufficient until we have a concrete reason to
        // diverge.
        //
        // The ONLY field that differs from the parent is
        // `nesting_depth`, which increments by 1. That's the guard
        // state for the next recursion check, and the observable
        // field downstream tests assert on.
        let inner_ctx = ToolCallContext::new(
            ctx.services.clone(),
            ctx.stream_sink.clone(),
            ctx.image_budget.clone(),
            ctx.cancel_token.clone(),
            ctx.sandbox.clone(),
            ctx.verifier.clone(),
            ctx.nesting_depth + 1,
        );

        // ── Delegate to inner agent ────────────────────────────────
        self.inner.run(input.prompt, inner_ctx).await
    }
}

/// Placeholder inner-agent used by the default factory before the
/// production OpenRouter-backed `InnerAgent` is wired up (Wave 7).
///
/// Returns `ToolError::ExecutionFailed` when invoked — LOUDLY. We do
/// NOT return an empty `Ok` here because that would let a test or
/// early-boot `kay run` invocation silently "succeed" at a sage_query
/// call that actually did no research. An error value with a pointed
/// message makes the misuse self-documenting in logs.
///
/// The struct has zero fields (unit struct) — same shape convention
/// as `NoOpSandbox` / `NoOpVerifier` in the other two seams. Callers
/// instantiate via `Arc::new(NoOpInnerAgent)` and hand the `Arc<dyn
/// InnerAgent>` to `default_tool_set`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpInnerAgent;

#[async_trait]
impl InnerAgent for NoOpInnerAgent {
    async fn run(&self, _prompt: String, _ctx: ToolCallContext) -> Result<ToolOutput, ToolError> {
        Err(ToolError::ExecutionFailed {
            tool: ToolName::new("sage_query"),
            source: anyhow::anyhow!(
                "NoOpInnerAgent cannot execute sage_query — \
                 replace with a concrete InnerAgent impl before \
                 invoking (Wave 7 wires the OpenRouter-backed one)"
            ),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // Dummy inner agent — never called in these sanity tests, which
    // only exercise the tool's own construction surface.
    struct DummyAgent;
    #[async_trait]
    impl InnerAgent for DummyAgent {
        async fn run(&self, _: String, _: ToolCallContext) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::text("unused"))
        }
    }

    #[test]
    fn name_is_sage_query() {
        let t = SageQueryTool::new(Arc::new(DummyAgent));
        assert_eq!(t.name().as_str(), "sage_query");
    }

    #[test]
    fn construct_produces_hardened_schema() {
        // Confirms the tool goes through the same schema-hardening
        // pipeline as every other builtin (additionalProperties: false,
        // required field declared). Matches the pattern in
        // task_complete.rs's unit tests.
        let t = SageQueryTool::new(Arc::new(DummyAgent));
        let schema = t.input_schema();
        let obj = schema.as_object().expect("schema object");
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false))
        );
        let required = obj
            .get("required")
            .and_then(|v| v.as_array())
            .expect("required array present");
        assert!(
            required.iter().any(|v| v == "prompt"),
            "schema must require prompt: {required:?}"
        );
    }

    #[test]
    fn max_nesting_depth_is_two() {
        // Lock the LOOP-03 constant. If a future wave bumps this,
        // the test forces an explicit, reviewable change — not a
        // silent drift.
        assert_eq!(MAX_NESTING_DEPTH, 2);
    }
}
