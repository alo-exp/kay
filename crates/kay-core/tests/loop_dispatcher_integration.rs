//! Phase 5 Wave 4 T4.3 RED — loop + dispatcher integration.
//!
//! LOOP-01. This is the second integration test pinning the
//! behavior of [`kay_core::r#loop::run_turn`]. Contract under test:
//!
//!   Given a model-event stream that yields one
//!   `AgentEvent::ToolCallComplete` naming a tool registered in the
//!   provided [`kay_tools::ToolRegistry`], and no other inputs, the
//!   loop:
//!
//!     1. Forwards the `ToolCallComplete` to the event sink (so the
//!        UI can show "the model called tool X").
//!     2. Invokes [`kay_tools::runtime::dispatcher::dispatch`] with
//!        the registry, the tool name, arguments, the supplied
//!        [`kay_tools::ToolCallContext`], and the `call_id`.
//!     3. The tool emits one `AgentEvent::ToolOutput` via
//!        `ctx.stream_sink` (here wired to forward into `event_tx`),
//!        which the test observes alongside the forwarded
//!        `ToolCallComplete`.
//!     4. On model-stream close, the loop returns `Ok(())`.
//!
//! ## Why the loop must dispatch, not the caller
//!
//! `05-BRAINSTORM.md` §E2 locks the agent loop as the single owner
//! of model→tool routing. If the CLI had to pattern-match
//! `ToolCallComplete` itself and call `dispatch`, every future
//! frontend (Phase 9 GUI, Phase 9.5 TUI) would have to reimplement
//! that routing with identical semantics (priority, cancellation,
//! sandbox denial surface). Centralising the dispatch in `run_turn`
//! is what makes the JSONL `AgentEvent` contract (Phase 5 Wave 7)
//! the only integration surface frontends need.
//!
//! ## Expected RED state (T4.3)
//!
//! `RunTurnArgs` currently carries only `persona`, `control_rx`,
//! `model_rx`, and `event_tx`. This test adds two fields —
//! `registry` (an `Arc<ToolRegistry>`) and `tool_ctx` (an owned
//! [`ToolCallContext`]) — so compilation fails with
//! E0063 "missing fields `registry` and `tool_ctx` in initializer
//! of `RunTurnArgs`", or E0560 "no field `registry` on type
//! `RunTurnArgs`".
//!
//! T4.4 GREEN extends `RunTurnArgs` with those two fields and wires
//! `dispatch()` into the `ToolCallComplete` branch of the model-arm,
//! making this test pass.
//!
//! ## Why `ToolOutput` is the signal we assert on
//!
//! The test EchoTool emits a single `AgentEvent::ToolOutput` from
//! inside `invoke()` via `(ctx.stream_sink)(...)`. That event is the
//! only piece of evidence proving:
//!
//!   (a) the loop actually called `dispatch()`,
//!   (b) `dispatch()` resolved the tool from the registry, and
//!   (c) the tool's `invoke()` ran with the provided context.
//!
//! Asserting only on the dispatcher's return value (the `ToolOutput`
//! struct from `forge_domain`) would miss (c): the tool could be
//! registered, looked up, but its sink call could be dropped without
//! us noticing. Asserting on the streamed `ToolOutput` *event* —
//! which must physically flow through `event_tx` — closes that hole.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolName, ToolOutput};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::control_channel;
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::contract::Tool;
use kay_tools::error::ToolError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    ToolOutputChunk, ToolRegistry,
};

// ── Test tool: emits exactly one ToolOutput event, then returns ─────
//
// Using a purpose-built `EchoTool` instead of one of the built-ins
// (`execute_commands`, `fs_read`, …) keeps this test hermetic: no
// disk I/O, no process spawning, no sandbox surface beyond the
// `NoOpSandbox` that the context already supplies. The only side
// effect is a single sink call — which is exactly what we assert on.
struct EchoTool {
    name: ToolName,
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        "T4.3 test fixture — emits one ToolOutput Stdout chunk containing the args JSON"
    }
    fn input_schema(&self) -> Value {
        // Minimal object schema so `schemars::Schema::try_from` accepts
        // it if `tool_definitions()` is ever called — not required for
        // this test, but keeps EchoTool usable in any later extension.
        json!({"type": "object", "additionalProperties": true})
    }
    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        (ctx.stream_sink)(AgentEvent::ToolOutput {
            call_id: call_id.to_string(),
            chunk: ToolOutputChunk::Stdout(args.to_string()),
        });
        Ok(ToolOutput::text(args.to_string()))
    }
}

// ── Null services for ToolCallContext ──────────────────────────────
//
// EchoTool never calls any `ctx.services.*` method, so these stubs
// are unreachable in this test. They exist because `ToolCallContext`
// requires an `Arc<dyn ServicesHandle>` and the trait has four
// required methods; `unimplemented!()` would be equivalent, but
// returning empty `ToolOutput::text("")` mirrors the pattern in
// `kay_tools::runtime::context::for_test`.
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

#[tokio::test]
async fn loop_plus_dispatcher_invokes_registered_tool() {
    // ── Channels (capacity 32 per BRAINSTORM §Engineering-Lens) ─────
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // ── Registry with one EchoTool under name "echo" ────────────────
    // Wrapped in Arc because production callers (kay-cli) build the
    // registry once at startup and share it across every turn. The
    // test locks that shape in now so T4.4 cannot regress it.
    let mut registry = ToolRegistry::new();
    let echo_name = ToolName::new("echo");
    registry.register(Arc::new(EchoTool { name: echo_name.clone() }) as Arc<dyn Tool>);
    let registry: Arc<ToolRegistry> = Arc::new(registry);

    // ── ToolCallContext wiring: stream_sink forwards to event_tx ────
    // `stream_sink` is synchronous (`Fn(AgentEvent)`), but `event_tx`
    // is a tokio mpsc. `try_send` is the right adapter here: it is
    // synchronous, non-blocking, and with capacity 32 + only one tool
    // event expected, it cannot overflow. Drop-on-full would be
    // acceptable because the event_rx drain loop runs concurrently,
    // but we prefer an assert to catch capacity regressions early.
    let sink_tx = event_tx.clone();
    let stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev| {
        sink_tx
            .try_send(ev)
            .expect("event channel has capacity; test emits exactly one tool event");
    });
    // nesting_depth = 0: this integration test is a top-level turn.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        stream_sink,
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
    );

    // ── Mock provider: one ToolCallComplete frame, then close ───────
    // Buffered send before spawn — same rationale as the T4.1 test:
    // guarantees the select! sees the event before the stream-close.
    let call_id = "call-echo-1";
    model_tx
        .send(Ok(AgentEvent::ToolCallComplete {
            id: call_id.to_string(),
            name: echo_name.as_str().to_string(),
            arguments: json!({"msg": "hi"}),
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    drop(model_tx);

    // ── Persona (forge, bundled YAML) ───────────────────────────────
    let persona = Persona::load("forge").expect("bundled forge persona loads");

    // ── Spawn run_turn with the extended RunTurnArgs ────────────────
    // T4.3 RED: the struct-literal adds `registry` and `tool_ctx` —
    // both fields do NOT yet exist on `RunTurnArgs`. Compilation
    // fails here. T4.4 GREEN adds them and wires dispatch in.
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

    // ── Assert: exactly one ToolOutput event with matching call_id ──
    // We do NOT assert on the forwarded ToolCallComplete here — that
    // the loop forwards *every* model event is locked by T4.1's
    // happy-path test. T4.3's unique contract is that dispatch runs
    // AND its streamed output reaches the sink. Scoping the assertion
    // to just `ToolOutput` keeps the two tests independent.
    let tool_outputs: Vec<&AgentEvent> = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolOutput { .. }))
        .collect();
    assert_eq!(
        tool_outputs.len(),
        1,
        "expected exactly one ToolOutput event from EchoTool dispatch; \
         got {} total events: {:?}",
        events.len(),
        events,
    );
    assert!(
        matches!(
            tool_outputs[0],
            AgentEvent::ToolOutput {
                call_id: cid,
                chunk: ToolOutputChunk::Stdout(s),
            } if cid == call_id && s.contains("hi")
        ),
        "expected ToolOutput {{ call_id: {call_id:?}, chunk: Stdout(…contains 'hi'…) }}; \
         got: {:?}",
        tool_outputs[0],
    );
}
