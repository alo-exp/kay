//! Phase 5 Wave 5 T5.4 RED — forge + sage_query end-to-end integration.
//!
//! LOOP-03 / CLI-07. The final nesting-tier integration test for
//! Wave 5: proves the FULL stack (provider → loop → dispatcher →
//! registry → SageQueryTool → InnerAgent) routes a model-emitted
//! `ToolCallComplete { name: "sage_query", … }` through to the
//! inner agent AND that the inner agent's streamed events surface
//! in the parent's event sink.
//!
//! ## Contract under test
//!
//! Given:
//!   - the bundled forge persona (`Persona::load("forge")`),
//!   - a `ToolRegistry` built via
//!     `default_tool_set(project_root, quota, inner_agent)` — the
//!     3-arg form that registers `SageQueryTool::new(inner_agent)`
//!     under the `"sage_query"` name,
//!   - a single model-stream event
//!     `ToolCallComplete { name: "sage_query",
//!      arguments: {"prompt": "research distributed consensus"} }`,
//!
//! the loop MUST:
//!   1. Forward the `ToolCallComplete` to the event sink
//!      (locked by T4.1 for every tool, but retested here to prove
//!      sage_query doesn't short-circuit it).
//!   2. Invoke the dispatcher with the `"sage_query"` tool.
//!   3. `SageQueryTool::invoke()` bumps `nesting_depth` from 0 to 1
//!      and delegates to `InnerAgent::run`.
//!   4. The inner agent's streamed events (canned `TextDelta`
//!      chunks) reach `event_rx` via the same `stream_sink` the
//!      parent handed in — i.e. the parent and inner turn share
//!      ONE event pipeline, not two.
//!   5. On model-stream close, `run_turn` returns `Ok(())`.
//!
//! ## Why this test exists (distinct from tests/sage_query.rs)
//!
//! `crates/kay-tools/tests/sage_query.rs` pins the tool's behavior
//! in isolation: the tool is handed a pre-built context and told to
//! run. This test, in contrast, proves the FULL wiring — starting
//! from a `ToolCallComplete` frame arriving over a provider channel,
//! running through the loop's `select!`, through `dispatch()`, into
//! the registry's `sage_query` slot, down through the tool, and back
//! out through the shared sink.
//!
//! If `tests/sage_query.rs` passes but this one fails, the
//! regression is in the wiring (dispatcher, registry,
//! default_tool_set) and not the tool itself. That diagnostic split
//! is the primary reason to keep the two layers of coverage.
//!
//! ## Expected RED state (T5.4)
//!
//! `default_tool_set` currently has the 2-arg signature:
//!
//!     pub fn default_tool_set(
//!         project_root: PathBuf,
//!         quota: Arc<ImageQuota>,
//!     ) -> ToolRegistry
//!
//! This test calls it with 3 args (`project_root, quota,
//! inner_agent`). Compilation fails with E0061
//! "this function takes 2 arguments but 3 arguments were supplied".
//!
//! T5.5 GREEN extends the factory to 3 args, registers
//! `SageQueryTool::new(inner_agent)` under the `"sage_query"` name,
//! and updates the 4 other call sites (kay-cli boot.rs + the two
//! test files currently using the 2-arg form). That makes this test
//! pass.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::control_channel;
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::builtins::sage_query::InnerAgent;
use kay_tools::error::ToolError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    default_tool_set,
};

// ── Null services ───────────────────────────────────────────────────
//
// Duplicated from tests/loop.rs and tests/loop_dispatcher_integration.rs
// per the established rationale: each integration test file is its own
// crate compilation unit, so lifting to a shared helper module would
// require a `common.rs` mod-include pattern that couples formerly
// independent suites. ~30 LOC duplication is cheaper than that coupling.
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

// ── RecordingAgent: inner-agent stub with three roles ───────────────
//
// Same shape as the stub in tests/sage_query.rs; duplicated for the
// same independent-compilation-unit reason as NullServices. The stub
// captures three facts that together prove the E2E wiring:
//
//   1. `recorded_prompt` — the raw prompt string the loop→dispatcher
//      pipeline delivered to the InnerAgent. Proves the prompt body
//      round-trips unchanged from forge's JSON args all the way into
//      the inner turn.
//
//   2. `recorded_depth` — the `nesting_depth` on the `inner_ctx` at
//      invoke time. MUST be 1 (parent was 0). If it were 0, the guard
//      against recursive sage_query would silently fail: a child
//      could issue another sage_query at depth 0 indefinitely.
//
//   3. `text_deltas_to_emit` — canned events the stub streams through
//      `ctx.stream_sink`. We assert they reach `event_rx` on the
//      parent side, which is the only evidence that the parent and
//      inner turn share THE SAME sink `Arc<dyn Fn>` rather than
//      sage_query accidentally constructing a fresh one.
struct RecordingAgent {
    recorded_prompt: Arc<Mutex<Option<String>>>,
    recorded_depth: Arc<Mutex<Option<u8>>>,
    text_deltas_to_emit: Vec<String>,
}

#[async_trait]
impl InnerAgent for RecordingAgent {
    async fn run(
        &self,
        prompt: String,
        ctx: ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // Record BEFORE emitting: if a future wiring bug makes the
        // sink panic, we still have the recorded state to diagnose
        // what the tool DID deliver to the inner agent.
        *self.recorded_prompt.lock().unwrap() = Some(prompt);
        *self.recorded_depth.lock().unwrap() = Some(ctx.nesting_depth);

        // Emit canned events via the (supposedly) shared sink. Order
        // is assertion-significant: the test locks in the two chunks
        // appearing in event_rx in send order.
        for content in &self.text_deltas_to_emit {
            (ctx.stream_sink)(AgentEvent::TextDelta { content: content.clone() });
        }

        Ok(ToolOutput::text("sage-done"))
    }
}

#[tokio::test]
async fn forge_calls_sage_via_sage_query_end_to_end() {
    // ── Channels (capacity 32 per BRAINSTORM §Engineering-Lens) ─────
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // ── RecordingAgent shared state ─────────────────────────────────
    //
    // Cloning the Arcs hands out shared views: one into the
    // RecordingAgent (which mutates the inner values), and one kept
    // on the test side (which reads them for assertions after the
    // loop drains). Mutex is std::sync (not tokio) because the
    // critical section is a single assignment with no await point —
    // the rustc async-Send check on the spawned run_turn task
    // confirms std::sync is safe here.
    let recorded_prompt = Arc::new(Mutex::new(None::<String>));
    let recorded_depth = Arc::new(Mutex::new(None::<u8>));
    let canned_events = vec!["sage says ".to_string(), "hello".to_string()];
    let inner_agent: Arc<dyn InnerAgent> = Arc::new(RecordingAgent {
        recorded_prompt: recorded_prompt.clone(),
        recorded_depth: recorded_depth.clone(),
        text_deltas_to_emit: canned_events.clone(),
    });

    // ── Registry via default_tool_set's 3-arg form ──────────────────
    //
    // *** RED SURFACE — LOCKED BY T5.4 ***
    //
    // Current signature is 2-arg: (project_root, quota). This 3-arg
    // call fails to compile with E0061 until T5.5 GREEN extends the
    // factory to accept `inner_agent: Arc<dyn InnerAgent>` as the
    // third parameter AND registers `SageQueryTool::new(inner_agent)`
    // under the `"sage_query"` name. That one change makes the full
    // test pass.
    let project_root = PathBuf::from("/tmp/phase5-wave5-t5.4-e2e");
    let quota = Arc::new(ImageQuota::new(u32::MAX, u32::MAX));
    let registry = Arc::new(default_tool_set(
        project_root,
        quota.clone(),
        inner_agent,
    ));

    // ── ToolCallContext (top-level turn: nesting_depth = 0) ─────────
    //
    // `stream_sink` forwards synchronously into `event_tx`. Capacity
    // 32 >> the 3 events this test produces (1 ToolCallComplete
    // forward + 2 TextDeltas from sage), so `try_send` cannot
    // overflow. We prefer `expect` to a drop-on-full policy so any
    // future capacity regression surfaces as a test failure instead
    // of a silent event loss.
    let sink_tx = event_tx.clone();
    let stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev| {
        sink_tx
            .try_send(ev)
            .expect("event channel has capacity 32; test emits <= 3 events total");
    });
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        stream_sink,
        quota,
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0, // top-level turn; sage_query must bump this to 1
    );

    // ── Mock provider: one ToolCallComplete (sage_query), then close ─
    //
    // Buffered send into the capacity-32 channel before spawning
    // `run_turn` — same rationale as the T4.1 and T4.3 tests:
    // guarantees the select! sees the event before it sees the
    // close, keeping the test deterministic.
    let call_id = "call-sage-e2e-1";
    model_tx
        .send(Ok(AgentEvent::ToolCallComplete {
            id: call_id.to_string(),
            name: "sage_query".to_string(),
            arguments: json!({ "prompt": "research distributed consensus" }),
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    drop(model_tx);

    // ── Persona (bundled forge — sage_query is forge's delegation tool)
    let persona = Persona::load("forge").expect("bundled forge persona loads");

    // ── Spawn run_turn ──────────────────────────────────────────────
    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
    }));

    // ── Drain events until the loop drops `event_tx` ────────────────
    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    // ── Assert: loop returned Ok(()) ────────────────────────────────
    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok on stream close");

    // ── Assert: RecordingAgent received the forge-sent prompt ───────
    //
    // Proves args.prompt survives the full round-trip:
    //   JSON in model frame → serde_json::from_value → SageQueryArgs
    //   → InnerAgent::run parameter.
    // If this fails while the others pass, the regression is either
    // in SageQueryArgs deserialization or in how sage_query extracts
    // `input.prompt` from parsed args.
    let recv_prompt = recorded_prompt.lock().unwrap().clone();
    assert_eq!(
        recv_prompt.as_deref(),
        Some("research distributed consensus"),
        "inner agent must receive the prompt forge passed via sage_query args"
    );

    // ── Assert: nesting_depth bumped 0 → 1 ──────────────────────────
    //
    // This is the LOOP-03 guard invariant at the integration level.
    // tests/sage_query.rs pins it at the tool level with a hand-built
    // context; this test re-pins it at the E2E level because a future
    // refactor of `dispatch()` or `run_turn` could accidentally clone
    // the parent context without letting sage_query bump the depth,
    // and the tool-level test would not catch that.
    let recv_depth = *recorded_depth.lock().unwrap();
    assert_eq!(
        recv_depth,
        Some(1),
        "sage_query must bump nesting_depth from 0 (parent) to 1 (inner) \
         before invoking InnerAgent::run; got {:?}",
        recv_depth,
    );

    // ── Assert: canned TextDelta events flow through parent sink ────
    //
    // Extract only TextDeltas; there may also be the forwarded
    // ToolCallComplete and/or other events interleaved. Strict
    // ordering of TextDelta vs ToolCallComplete is NOT asserted here
    // — that's a biased-select property owned by
    // tests/loop_property.rs. We only lock:
    //   (a) both canned TextDeltas reached event_rx, in send order,
    //   (b) no extra TextDeltas appeared (proving the sink is not
    //       double-wired or duplicated by the tool).
    let text_deltas: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::TextDelta { content } => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        text_deltas,
        vec!["sage says ", "hello"],
        "sage's streamed TextDeltas must reach the parent event_rx via the \
         shared stream_sink in send order; got {:?}",
        text_deltas,
    );

    // ── Assert: ToolCallComplete was forwarded ──────────────────────
    //
    // Sanity: sage_query must not accidentally consume the
    // ToolCallComplete frame (e.g. by pattern-matching out of the
    // normal forwarding branch). The T4.1 happy-path test already
    // locks this for arbitrary tools, but we double-lock here so a
    // future "optimize sage_query path" refactor cannot regress it
    // silently.
    let forwarded = events
        .iter()
        .find(|e| matches!(e, AgentEvent::ToolCallComplete { name, .. } if name == "sage_query"));
    assert!(
        forwarded.is_some(),
        "sage_query ToolCallComplete must be forwarded to event_rx; \
         got events: {:?}",
        events,
    );
}
