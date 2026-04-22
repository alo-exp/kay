//! Phase 5 Wave 5 T5.1 RED — sage_query sub-tool unit coverage.
//!
//! LOOP-03 / CLI-07. Pins the contract of the `sage_query` tool: a
//! first-class builtin that lets `forge` (the planner persona) and
//! `muse` (the docs/reading persona) delegate a focused research
//! question to `sage` (the fast-context persona) as a nested agent
//! turn, while:
//!
//!   1. threading a `nesting_depth` counter through `ToolCallContext`
//!      so runaway sub-queries (`sage_query` → `sage_query` → ...) are
//!      detected at runtime (belt + suspenders — sage's YAML
//!      `tool_filter` already excludes `sage_query`, but the depth
//!      guard catches any future drift),
//!
//!   2. flowing the nested turn's events back through the parent's
//!      `stream_sink` so the user sees one continuous event stream
//!      (no "mystery sub-turn" blackout),
//!
//!   3. inheriting the parent's `sandbox` / `verifier` / `services`
//!      / `image_budget` so the sub-turn operates under the same
//!      policy envelope as the caller (no privilege escalation via
//!      sub-query).
//!
//! ## The five tests
//!
//! | # | Name                                      | Contract under test                                       |
//! |---|-------------------------------------------|-----------------------------------------------------------|
//! | 1 | `sage_query_invokes_inner_agent`          | The tool's `invoke` calls the injected `InnerAgent::run`. |
//! | 2 | `sage_query_threads_nesting_depth`        | Inner ctx's `nesting_depth == parent.nesting_depth + 1`.  |
//! | 3 | `sage_query_rejects_depth_gte_2`          | Parent depth ≥ 2 → `ToolError::NestingDepthExceeded`.     |
//! | 4 | `sage_query_emits_nested_events`          | Inner-agent events reach parent's `stream_sink`.          |
//! | 5 | `sage_query_respects_parent_sandbox`      | Inner ctx's `sandbox` is the SAME Arc as parent's.        |
//!
//! ## Expected RED state (T5.1)
//!
//! This file compile-fails on three surfaces, each added by a
//! specific GREEN task in Wave 5:
//!
//!   1. `kay_tools::builtins::sage_query::{SageQueryTool, InnerAgent}`
//!      do not exist yet → `E0432 unresolved import`. Added by T5.3.
//!
//!   2. `ToolCallContext::new` takes 6 args today; the tests call it
//!      with 7 (the 7th is `nesting_depth: u8`) → `E0061 this function
//!      takes 6 arguments but 7 were supplied`. Added by T5.2.
//!
//!   3. `ToolError::NestingDepthExceeded { depth, limit }` variant
//!      does not exist yet → `E0599 no variant named
//!      NestingDepthExceeded`. Added by T5.3 along with the tool.
//!
//! That combination is deliberate: a caller who only does the T5.2
//! field addition without writing the tool would still fail to compile
//! because `SageQueryTool` is unresolved, and vice versa — each GREEN
//! task can advance independently without the other silently turning
//! the test green.
//!
//! ## Why a stub `InnerAgent` seam (and not a full `run_turn` spawn)
//!
//! Unit tests must not require a live OpenRouter connection, a real
//! `run_turn` task, or a model stream — those are integration-test
//! concerns covered by T5.4 (`forge_calls_sage_via_sage_query_end_to_
//! end`). The `InnerAgent` trait is the seam that lets sage_query be
//! unit-tested:
//!
//!   - Production impl: wraps the OpenRouter client + `Persona::load
//!     ("sage")` + `kay_core::r#loop::run_turn`, providing a real
//!     nested turn.
//!   - Test impl (below): a `RecordingAgent` that snapshots the
//!     received prompt + inner-context shape and emits canned
//!     `TextDelta`s through `ctx.stream_sink` — enough surface to
//!     pin all five contracts above without a runtime.
//!
//! This mirrors the `Sandbox`/`TaskVerifier` pattern already used in
//! `ToolCallContext` (D-12 in `03-CONTEXT.md`): injectable trait
//! objects with no-op production defaults and recording test stubs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use kay_tools::builtins::sage_query::{InnerAgent, SageQueryTool};
use kay_tools::contract::Tool;
use kay_tools::error::ToolError;
use kay_tools::events::AgentEvent;
use kay_tools::runtime::context::{ServicesHandle, ToolCallContext};
use kay_tools::{ImageQuota, NoOpSandbox, NoOpVerifier, Sandbox};

// ── Null services stub ─────────────────────────────────────────────
//
// Mirrors the duplicated stub in `crates/kay-core/tests/loop.rs` and
// `tests/loop_property.rs` — each integration-test file is a separate
// compilation unit, so extracting to a shared helper requires the
// `common.rs` mod-import pattern which couples the suites. The ~30 LOC
// cost is cheaper than the coupling cost.
struct NullServices;

#[async_trait]
impl ServicesHandle for NullServices {
    async fn fs_read(&self, _: FSRead) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_write(&self, _: FSWrite) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_search(&self, _: FSSearch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn net_fetch(&self, _: NetFetch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
}

// ── Recording InnerAgent test stub ─────────────────────────────────
//
// Captures (prompt, inner-ctx.nesting_depth, inner-ctx.sandbox-Arc)
// from whatever sage_query decides to pass through. Also emits a
// configurable list of `TextDelta` strings through the inner ctx's
// `stream_sink` so test 4 can prove events flow upward.
//
// We record only the fields we assert on — not the full
// `ToolCallContext` — because `ToolCallContext` is `#[non_exhaustive]`
// and shape-matching the whole struct would make this file a
// maintenance burden every time a new context field lands.
//
// Using `Arc<Mutex<Option<T>>>` for each recorded field rather than a
// single `Mutex<Recorded>` struct: each test reads only the fields it
// cares about, and per-field cells make the intent self-documenting
// at the read site (`recorded_depth.lock()`).
struct RecordingAgent {
    recorded_prompt: Arc<Mutex<Option<String>>>,
    recorded_depth: Arc<Mutex<Option<u8>>>,
    recorded_sandbox: Arc<Mutex<Option<Arc<dyn Sandbox>>>>,
    /// Canned `TextDelta` content strings. `run()` emits each as an
    /// `AgentEvent::TextDelta` through `ctx.stream_sink`. Kept as
    /// `Vec<String>` rather than `Vec<AgentEvent>` because
    /// `AgentEvent` does not derive `Clone` — a struct storing it
    /// would force a one-shot move.
    text_deltas_to_emit: Vec<String>,
}

impl RecordingAgent {
    fn new(text_deltas_to_emit: Vec<String>) -> Self {
        Self {
            recorded_prompt: Arc::new(Mutex::new(None)),
            recorded_depth: Arc::new(Mutex::new(None)),
            recorded_sandbox: Arc::new(Mutex::new(None)),
            text_deltas_to_emit,
        }
    }
}

#[async_trait]
impl InnerAgent for RecordingAgent {
    async fn run(&self, prompt: String, ctx: ToolCallContext) -> Result<ToolOutput, ToolError> {
        *self.recorded_prompt.lock().unwrap() = Some(prompt);
        *self.recorded_depth.lock().unwrap() = Some(ctx.nesting_depth);
        *self.recorded_sandbox.lock().unwrap() = Some(ctx.sandbox.clone());
        for content in &self.text_deltas_to_emit {
            (ctx.stream_sink)(AgentEvent::TextDelta { content: content.clone() });
        }
        Ok(ToolOutput::text("inner-done"))
    }
}

// ── Parent-side spy: context builder + event sink recorder ─────────
//
// `ParentSpy::ctx(...)` produces a `ToolCallContext` that (a) has the
// caller-supplied `nesting_depth` and `sandbox`, and (b) routes every
// `TextDelta` event emitted through its `stream_sink` into a shared
// `Vec<String>` the test can inspect after invoke returns.
//
// Recording ONLY `TextDelta.content` (not the full event) keeps the
// assertion concise — test 4 cares about "did the inner agent's
// deltas reach my sink?", not "exact event shape". If a future test
// needs full events, extend to `Vec<AgentEvent>` (but mind Clone).
struct ParentSpy {
    events: Arc<Mutex<Vec<String>>>,
}

impl ParentSpy {
    fn new() -> Self {
        Self { events: Arc::new(Mutex::new(Vec::new())) }
    }

    fn ctx(&self, nesting_depth: u8, sandbox: Arc<dyn Sandbox>) -> ToolCallContext {
        let events = self.events.clone();
        let sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev| {
            if let AgentEvent::TextDelta { content } = ev {
                events.lock().unwrap().push(content);
            }
        });
        ToolCallContext::new(
            Arc::new(NullServices),
            sink,
            Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
            CancellationToken::new(),
            sandbox,
            Arc::new(NoOpVerifier),
            nesting_depth,
            Arc::new(Mutex::new(String::new())),
        )
    }
}

// ── Test 1: invokes inner agent ────────────────────────────────────
#[tokio::test]
async fn sage_query_invokes_inner_agent() {
    let agent = Arc::new(RecordingAgent::new(vec![]));
    let tool = SageQueryTool::new(agent.clone() as Arc<dyn InnerAgent>);
    let spy = ParentSpy::new();
    let ctx = spy.ctx(0, Arc::new(NoOpSandbox));

    let result = tool
        .invoke(json!({ "prompt": "what is 2+2?" }), &ctx, "cid-1")
        .await;

    assert!(result.is_ok(), "sage_query invocation failed: {result:?}");
    assert_eq!(
        agent.recorded_prompt.lock().unwrap().as_deref(),
        Some("what is 2+2?"),
        "inner agent must receive the prompt passed in args",
    );
}

// ── Test 2: threads nesting_depth + 1 ──────────────────────────────
#[tokio::test]
async fn sage_query_threads_nesting_depth() {
    let agent = Arc::new(RecordingAgent::new(vec![]));
    let tool = SageQueryTool::new(agent.clone() as Arc<dyn InnerAgent>);
    let spy = ParentSpy::new();
    // Parent at depth 0 → inner ctx MUST be at depth 1.
    let ctx = spy.ctx(0, Arc::new(NoOpSandbox));

    tool.invoke(json!({ "prompt": "x" }), &ctx, "cid-2")
        .await
        .expect("invoke ok");

    assert_eq!(
        *agent.recorded_depth.lock().unwrap(),
        Some(1),
        "inner nesting_depth must be parent (0) + 1",
    );
}

// ── Test 3: rejects when parent depth ≥ 2 ──────────────────────────
#[tokio::test]
async fn sage_query_rejects_depth_gte_2() {
    // Agent that would panic if called — reaching it means the depth
    // guard didn't trip. This is stronger than a quiet `assert!`
    // because any GREEN implementation that forgets the guard but
    // still returns Ok on the happy path would leak into test 1/2/4/5
    // only as a false-pass there; here we'd get a crash.
    struct PanickingAgent;
    #[async_trait]
    impl InnerAgent for PanickingAgent {
        async fn run(&self, _: String, _: ToolCallContext) -> Result<ToolOutput, ToolError> {
            panic!("inner agent invoked despite depth >= 2 — guard missing");
        }
    }
    let tool = SageQueryTool::new(Arc::new(PanickingAgent) as Arc<dyn InnerAgent>);
    let spy = ParentSpy::new();
    // Parent at depth 2 → sage_query MUST reject before invoking.
    let ctx = spy.ctx(2, Arc::new(NoOpSandbox));

    let result = tool.invoke(json!({ "prompt": "x" }), &ctx, "cid-3").await;

    assert!(
        matches!(result, Err(ToolError::NestingDepthExceeded { .. })),
        "expected NestingDepthExceeded at depth 2, got {result:?}",
    );
}

// ── Test 4: nested events flow to parent sink ──────────────────────
#[tokio::test]
async fn sage_query_emits_nested_events() {
    let agent = Arc::new(RecordingAgent::new(vec![
        "sage-says-hi".to_string(),
        "sage-says-bye".to_string(),
    ]));
    let tool = SageQueryTool::new(agent as Arc<dyn InnerAgent>);
    let spy = ParentSpy::new();
    let ctx = spy.ctx(0, Arc::new(NoOpSandbox));

    tool.invoke(json!({ "prompt": "x" }), &ctx, "cid-4")
        .await
        .expect("invoke ok");

    let events = spy.events.lock().unwrap();
    // Order and multiplicity matter: the parent sink must see exactly
    // the inner agent's two deltas, FIFO. No dedupe, no drops.
    assert_eq!(
        events.as_slice(),
        &["sage-says-hi".to_string(), "sage-says-bye".to_string()],
        "parent sink did not receive inner agent's deltas in order",
    );
}

// ── Test 5: parent sandbox propagates by-reference into inner ctx ──
#[tokio::test]
async fn sage_query_respects_parent_sandbox() {
    let agent = Arc::new(RecordingAgent::new(vec![]));
    let tool = SageQueryTool::new(agent.clone() as Arc<dyn InnerAgent>);
    let spy = ParentSpy::new();

    // Build ONE Arc and keep a clone for the assertion. `Arc::ptr_eq`
    // compares the underlying allocation pointer, not the contents —
    // which is exactly the invariant we want: sage_query MUST NOT
    // construct a fresh `Arc::new(NoOpSandbox)` (that would match
    // behavior but would break policy inheritance once non-NoOp
    // sandboxes land in Phase 4+: a per-turn policy object with state
    // would be replaced by a fresh no-state default, silently). Arc
    // sharing is the only way to prove inheritance structurally.
    let parent_sandbox: Arc<dyn Sandbox> = Arc::new(NoOpSandbox);
    let ctx = spy.ctx(0, parent_sandbox.clone());

    tool.invoke(json!({ "prompt": "x" }), &ctx, "cid-5")
        .await
        .expect("invoke ok");

    let recorded = agent.recorded_sandbox.lock().unwrap();
    let recorded_arc = recorded.as_ref().expect("sandbox arc recorded");
    assert!(
        Arc::ptr_eq(&parent_sandbox, recorded_arc),
        "inner ctx's sandbox must be the SAME Arc as parent's (policy inheritance)",
    );
}
