//! Phase 5 Nyquist-audit LOOP-06 pin — Pause + `ToolCallComplete`
//! buffered-during-pause semantics, including the "dispatch is
//! NOT re-run on replay" invariant.
//!
//! Why this file exists
//! --------------------
//! The existing LOOP-06 suite in `tests/loop.rs` locks Pause,
//! Resume, and Abort against `TextDelta` fixtures —
//! `control_pause_buffers_then_resume_replays` walks a full
//! pre-pause → pause → buffer → resume → post-resume cycle, but
//! the only event type it buffers is a plain `TextDelta`. A text
//! delta is side-effect-free: buffering it during Pause and
//! replaying it on Resume is literally just a queue push + queue
//! pop.
//!
//! `ToolCallComplete` is different. On a live invocation (not
//! paused), the loop does two things to a ToolCallComplete:
//!
//!   1. forwards it to `event_tx` so the UI can render "the model
//!      called tool X", and
//!   2. calls `dispatch(registry, name, args, ctx, call_id)` so
//!      the tool actually executes and emits a `ToolOutput`.
//!
//! When the loop is paused, `handle_model_event` returns early
//! from the pause gate (line 305 of `src/loop.rs`) BEFORE the
//! dispatch branch runs. The event is pushed into the
//! `VecDeque<AgentEvent>` pause buffer. On Resume,
//! `handle_control` drains the buffer and `send()`s each buffered
//! event to `event_tx` — but dispatch is NEVER re-run (the replay
//! path in the control arm only calls `send`, not `dispatch`).
//!
//! The module doc for `src/loop.rs` calls this out explicitly (line
//! 59-65):
//!
//! > **Dispatch + verify gate during Pause.** A `ToolCallComplete`
//! > that arrives while paused is buffered but its dispatch is
//! > deferred until Resume replays it — and on replay, dispatch is
//! > NOT re-run (the buffered event is a historical record, not a
//! > live invocation).
//!
//! That documented invariant has NO runtime test today. The 256-
//! case proptest in `loop_property.rs` generates only `TextDelta`
//! events for the buffer. The integration tests exercise either
//! live tool dispatch (`loop_dispatcher_integration.rs`) or plain
//! pause/resume (`loop.rs`), never both at once.
//!
//! What slips through without this test
//! -------------------------------------
//! If a future wave adds "dispatch-on-replay" semantics to the
//! Resume arm — e.g. someone decides that deferred tool calls
//! should actually run when the user un-pauses — the documented
//! "NOT re-run" invariant flips silently. Worse: a naïve
//! refactor that moves the dispatch call ABOVE the pause gate
//! (to simplify handle_model_event) would cause a paused
//! ToolCallComplete to execute its tool WHILE the agent is
//! nominally paused — defeating the UX promise that "pause stops
//! side effects".
//!
//! Either regression ships as green today. This file pins both:
//! the two `#[tokio::test]`s exercise the full pause-buffer-
//! resume cycle with a `ToolCallComplete` and assert (a) no
//! ToolOutput is emitted during Pause, (b) the ToolCallComplete
//! is forwarded on Resume, and (c) no ToolOutput is emitted on
//! Resume either (because dispatch is a historical record, not
//! re-run).
//!
//! Why two tests instead of one
//! ----------------------------
//! The two invariants — "no dispatch during Pause" and "no
//! dispatch on replay" — are the two sides of the same gate but
//! probe orthogonal regressions:
//!
//!   * `tool_call_during_pause_is_buffered_no_dispatch_no_output`
//!     pins the Pause-phase contract. A regression that moves
//!     dispatch above the pause gate fails this test AT PAUSE
//!     TIME, before Resume is even sent — the signal is tight.
//!
//!   * `tool_call_resume_replay_does_not_re_run_dispatch` pins
//!     the Resume-phase contract. A future "dispatch on replay"
//!     wave would leave test 1 green (nothing runs during Pause)
//!     but flip test 2 (something runs on Resume) — so the RED
//!     diagnostic points at the Resume arm specifically.
//!
//! Both tests use the same `DispatchCountingTool` fixture so the
//! counter contract is shared and the diagnostic always reads
//! "dispatch_count = N, expected 0".
//!
//! Reference: Phase 5 Nyquist audit — LOOP-06 coverage gap #GAP-D.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolName, ToolOutput};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::{ControlMsg, control_channel};
use kay_core::r#loop::{RunTurnArgs, TurnResult, run_turn};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::contract::Tool;
use kay_tools::error::ToolError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    ToolOutputChunk, ToolRegistry,
};

// ── Test tool: counts invocations, also emits ToolOutput ────────────
//
// The tool has two failure signals by design:
//   1. `dispatch_count` atomic — increments on every `invoke` call.
//      Reading it at assertion time tells us whether dispatch ran.
//   2. Sink emission of `AgentEvent::ToolOutput` — if the test sees
//      a ToolOutput event when dispatch_count should be 0, the
//      counter was bypassed (unlikely) or someone else emitted one.
//
// Both signals are independent; a regression would need to break
// both to hide. Using a purpose-built tool (not one of the
// built-ins) keeps the fixture hermetic — no disk, no spawn, no
// sandbox surface beyond `NoOpSandbox`.
struct DispatchCountingTool {
    name: ToolName,
    dispatch_count: Arc<AtomicUsize>,
}

#[async_trait]
impl Tool for DispatchCountingTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        "GAP-D test fixture — counts invocations and emits one ToolOutput \
         so a failing assertion tells the reviewer WHICH signal fired"
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object", "additionalProperties": true})
    }
    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        self.dispatch_count.fetch_add(1, Ordering::SeqCst);
        (ctx.stream_sink)(AgentEvent::ToolOutput {
            call_id: call_id.to_string(),
            chunk: ToolOutputChunk::Stdout(args.to_string()),
        });
        Ok(ToolOutput::text(args.to_string()))
    }
}

// ── Null services (tool never calls ctx.services.*) ──────────────
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

/// The 5-tuple returned by `spawn_turn_with_counting_tool`.
///
/// Aliased to keep clippy::type_complexity quiet — the
/// signature is a deliberate test-only harness bundle, not a
/// production surface, and factoring it into a named type
/// makes call sites read as `let (handle, model_tx, ctl_tx,
/// event_rx, dispatch_count) = spawn_turn_with_counting_tool(...)`.
type TurnHarness = (
    tokio::task::JoinHandle<Result<TurnResult, kay_core::r#loop::LoopError>>,
    mpsc::Sender<Result<AgentEvent, ProviderError>>,
    mpsc::Sender<ControlMsg>,
    mpsc::Receiver<AgentEvent>,
    Arc<AtomicUsize>,
);

/// Shared turn wiring so each test reads as "set up, act, assert"
/// rather than 50 lines of boilerplate. Returns the handle to the
/// spawned `run_turn` plus the receiver side of every channel the
/// test wants to probe. The tool's dispatch_count is cloned into
/// the returned tuple so the test body can read it after the
/// loop exits.
///
/// The event channel is split into `event_rx` (what the test
/// observes) and `sink_tx` (what the tool forwards its
/// ToolOutput through) — they are the same underlying channel,
/// so both streams land in the same drain.
fn spawn_turn_with_counting_tool(tool_name: &'static str) -> TurnHarness {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (ctl_tx, control_rx) = control_channel();
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(32);

    let dispatch_count = Arc::new(AtomicUsize::new(0));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(DispatchCountingTool {
        name: ToolName::new(tool_name),
        dispatch_count: dispatch_count.clone(),
    }) as Arc<dyn Tool>);
    let registry: Arc<ToolRegistry> = Arc::new(registry);

    // stream_sink forwards into the SAME event_tx so the test
    // sees both loop-forwarded events AND tool-emitted events on
    // one drain. `try_send` is appropriate because the test
    // emits ≤ 2 events per turn and capacity is 32.
    let sink_tx = event_tx.clone();
    let stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev| {
        sink_tx
            .try_send(ev)
            .expect("event channel has capacity; test emits ≤2 tool events");
    });
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        stream_sink,
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let persona = Persona::load("forge").expect("bundled forge persona loads");

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
        verifier_config: kay_verifier::VerifierConfig { mode: kay_verifier::VerifierMode::Disabled, max_retries: 0, cost_ceiling_usd: 0.0, model: String::new() },
    }));

    (handle, model_tx, ctl_tx, event_rx, dispatch_count)
}

/// GAP-D.1 — Pause-phase invariant:
/// A `ToolCallComplete` that arrives WHILE the loop is paused is
/// buffered (not forwarded), and the tool is NOT dispatched.
///
/// If this test fails, either:
///   (a) the pause gate was bypassed — the ToolCallComplete was
///       forwarded to event_tx immediately, AND/OR
///   (b) dispatch ran above the pause gate — dispatch_count
///       went to 1 during Pause, AND/OR
///   (c) the tool's ToolOutput emission leaked to the sink.
///
/// All three signals are checked independently. Any one would
/// indicate a LOOP-06 regression.
#[tokio::test]
async fn tool_call_during_pause_is_buffered_no_dispatch_no_output() {
    let (handle, model_tx, ctl_tx, mut event_rx, dispatch_count) =
        spawn_turn_with_counting_tool("count");

    // ── Phase 1: pre-pause sanity — send a TextDelta first so
    // we know the loop is responsive before we start probing the
    // pause semantics. Catches the degenerate case where the
    // loop is broken from the start and the pause assertions
    // below would pass vacuously.
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "pre-pause".into() }))
        .await
        .expect("buffered send");
    let ev = tokio::time::timeout(Duration::from_millis(500), event_rx.recv())
        .await
        .expect("pre-pause TextDelta should forward within 500ms")
        .expect("event channel should deliver the pre-pause frame");
    assert!(
        matches!(&ev, AgentEvent::TextDelta { content } if content == "pre-pause"),
        "pre-pause TextDelta should pass through unmodified; got: {ev:?}"
    );

    // ── Phase 2: Pause ─────────────────────────────────────────
    ctl_tx
        .send(ControlMsg::Pause)
        .await
        .expect("Pause accepted");
    let ev = tokio::time::timeout(Duration::from_millis(500), event_rx.recv())
        .await
        .expect("Pause must cause Paused event within 500ms")
        .expect("event channel should deliver Paused");
    assert!(
        matches!(&ev, AgentEvent::Paused),
        "expected AgentEvent::Paused immediately after ControlMsg::Pause; got: {ev:?}"
    );

    // ── Phase 3: send ToolCallComplete while paused ────────────
    // If the loop is healthy: (a) the event goes into pause_buffer,
    // (b) handle_model_event returns early BEFORE the dispatch
    // branch, (c) dispatch_count stays 0, (d) event_rx sees nothing.
    //
    // If the loop is buggy: some combination of the above fails.
    // 50 ms is deliberately generous — the model-arm branch needs
    // to (a) recv, (b) observe paused=true, (c) VecDeque::push_back.
    // Even on a loaded CI host the round-trip is microseconds; a
    // 50 ms window catches scheduling hiccups without masking a
    // real regression.
    let call_id = "call-paused-count-1";
    model_tx
        .send(Ok(AgentEvent::ToolCallComplete {
            id: call_id.to_string(),
            name: "count".to_string(),
            arguments: json!({"phase": "during-pause"}),
        }))
        .await
        .expect("buffered send of ToolCallComplete while paused");
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Invariant 1: nothing on event_rx during Pause. The
    // ToolCallComplete was buffered, not forwarded.
    assert!(
        event_rx.try_recv().is_err(),
        "PAUSE-PHASE REGRESSION: ToolCallComplete leaked to event_tx \
         while paused. LOOP-06 pause gate in handle_model_event has \
         been bypassed — every paused frame must go to pause_buffer \
         until Resume. Likely cause: the pause check in \
         src/loop.rs:305 was removed or moved below the forward() \
         call."
    );

    // Invariant 2: dispatch_count stays 0 during Pause. If this
    // fires, the tool's invoke() ran while the agent was paused —
    // defeating the UX contract that pause halts side effects.
    assert_eq!(
        dispatch_count.load(Ordering::SeqCst),
        0,
        "PAUSE-PHASE REGRESSION: dispatch_count = {} while paused; \
         expected 0. The tool's invoke() ran during Pause, meaning \
         dispatch() was called above the pause gate in \
         handle_model_event. Likely cause: the dispatch call was \
         moved out of the 'paused == false' branch in src/loop.rs.",
        dispatch_count.load(Ordering::SeqCst)
    );

    // ── Phase 4: clean close → loop exits Ok ────────────────────
    // Before closing, send Abort rather than dropping the control
    // channel — both are valid exit paths but Abort is the one
    // the surrounding test suite uses and a clean Abort emits an
    // Aborted event we can drain, which keeps the final assertion
    // channel-deterministic.
    ctl_tx
        .send(ControlMsg::Abort)
        .await
        .expect("Abort accepted");

    // Drain any remaining events (Paused + maybe Aborted if the
    // abort arm raced the buffered frame) — they go to the sink
    // before the loop drops event_tx on exit.
    tokio::time::timeout(Duration::from_millis(500), handle)
        .await
        .expect("run_turn must exit within 500ms of Abort")
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok after Abort");

    // Final drain — confirm the channel is now closed and count
    // the events that surfaced on the way out. The BUFFERED
    // ToolCallComplete is dropped (Abort empties the buffer via
    // the loop's exit path); we ONLY care about the invariant
    // that NO ToolOutput ever emitted. See GAP-D.2 for the
    // Resume-replay path.
    let mut residual = Vec::new();
    while let Ok(ev) = tokio::time::timeout(Duration::from_millis(50), event_rx.recv()).await {
        if let Some(ev) = ev {
            residual.push(ev);
        } else {
            break;
        }
    }
    let tool_outputs: Vec<&AgentEvent> = residual
        .iter()
        .chain(std::iter::empty())
        .filter(|e| matches!(e, AgentEvent::ToolOutput { .. }))
        .collect();
    assert_eq!(
        tool_outputs.len(),
        0,
        "PAUSE-PHASE REGRESSION: a ToolOutput event surfaced on \
         the event stream despite dispatch_count = 0. The \
         counting-tool sink was invoked even though invoke() \
         reportedly did not run — this shouldn't be possible \
         unless someone else is emitting ToolOutput events. \
         Residual events: {residual:?}"
    );

    // Tie off the model channel to keep the type-level contract
    // happy (holding a dropped sender binding wouldn't compile).
    let _keep_alive_model_tx = model_tx;
    let _keep_alive_ctl_tx = ctl_tx;
}

/// GAP-D.2 — Resume-phase invariant:
/// On Resume, a buffered `ToolCallComplete` is flushed to
/// `event_tx` (historical record) but dispatch is NOT re-run
/// (the documented "buffered event is a historical record, not
/// a live invocation" invariant from `src/loop.rs:59-65`).
///
/// This is the stricter half of the pair. A regression that
/// wires dispatch into the Resume replay path — e.g. "resume
/// should run deferred tools" — leaves GAP-D.1 green (nothing
/// ran during Pause) but flips this test (something ran on
/// Resume). The failure diagnostic points at the Resume arm.
#[tokio::test]
async fn tool_call_resume_replay_does_not_re_run_dispatch() {
    let (handle, model_tx, ctl_tx, mut event_rx, dispatch_count) =
        spawn_turn_with_counting_tool("count");

    // ── Phase 1: Pause immediately ─────────────────────────────
    // Skip the pre-pause sanity frame (GAP-D.1 covers that). Get
    // straight into the pause state so the ToolCallComplete lands
    // deterministically on the buffered path.
    ctl_tx
        .send(ControlMsg::Pause)
        .await
        .expect("Pause accepted");
    let paused_ev = tokio::time::timeout(Duration::from_millis(500), event_rx.recv())
        .await
        .expect("Pause must cause Paused event")
        .expect("event channel delivers Paused");
    assert!(
        matches!(paused_ev, AgentEvent::Paused),
        "expected Paused after ControlMsg::Pause; got: {paused_ev:?}"
    );

    // ── Phase 2: send ToolCallComplete into the paused buffer ──
    let call_id = "call-buffered-count-2";
    model_tx
        .send(Ok(AgentEvent::ToolCallComplete {
            id: call_id.to_string(),
            name: "count".to_string(),
            arguments: json!({"phase": "buffered-for-replay"}),
        }))
        .await
        .expect("buffered send of ToolCallComplete while paused");
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Sanity: dispatch_count is still 0 (GAP-D.1 contract) — if
    // this is not 0, test 1 already failed and this test's
    // assertion would be vacuous. Re-pin here so both tests are
    // independently sound.
    assert_eq!(
        dispatch_count.load(Ordering::SeqCst),
        0,
        "GAP-D.2 precondition broken: dispatch_count = {} during \
         Pause (expected 0). GAP-D.1's invariant failed first; \
         run that test for the diagnostic.",
        dispatch_count.load(Ordering::SeqCst)
    );

    // ── Phase 3: Resume — buffer flushes to event_tx ───────────
    ctl_tx
        .send(ControlMsg::Resume)
        .await
        .expect("Resume accepted");

    // Expect the buffered ToolCallComplete to flush within 500 ms.
    let replayed = tokio::time::timeout(Duration::from_millis(500), event_rx.recv())
        .await
        .expect(
            "Resume must flush buffered ToolCallComplete within 500ms — \
             LOOP-06 buffer-flush regression or the Resume handler hung",
        )
        .expect("event channel delivers the buffered ToolCallComplete");
    assert!(
        matches!(
            &replayed,
            AgentEvent::ToolCallComplete { id, name, arguments }
                if id == call_id
                && name == "count"
                && arguments.get("phase") == Some(&json!("buffered-for-replay"))
        ),
        "expected the buffered ToolCallComplete to be replayed verbatim \
         to event_tx on Resume; got: {replayed:?}"
    );

    // ── Phase 4: give any accidental dispatch time to happen ──
    // This is the core GAP-D.2 assertion. We give the runtime a
    // 100 ms window — long enough for a regression that wires
    // dispatch into the Resume arm to actually complete — and
    // then assert dispatch_count is still 0 AND no ToolOutput
    // event ever surfaced.
    //
    // Why 100 ms and not 20 ms: the faulty path being probed is
    // "Resume replays the event AND calls dispatch". dispatch()
    // is an async call that may yield to the runtime mid-flight;
    // even a non-faulty path (which would complete the dispatch
    // and emit ToolOutput synchronously via try_send) wants a
    // settled moment. 100 ms is imperceptible and catches even
    // generous yields.
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        dispatch_count.load(Ordering::SeqCst),
        0,
        "RESUME-PHASE REGRESSION: dispatch_count = {} after Resume \
         replay; expected 0. The loop RAN the tool on replay — \
         violating the documented 'buffered event is a historical \
         record, not a live invocation' invariant (src/loop.rs:59-65). \
         Likely cause: dispatch was wired into handle_control's \
         Resume arm, or the Resume replay sends buffered events \
         through handle_model_event instead of bypassing it.",
        dispatch_count.load(Ordering::SeqCst)
    );

    // ── Phase 5: close the loop cleanly and drain residuals ────
    ctl_tx
        .send(ControlMsg::Abort)
        .await
        .expect("Abort accepted");
    tokio::time::timeout(Duration::from_millis(500), handle)
        .await
        .expect("run_turn must exit within 500ms of Abort")
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok after Abort");

    // Assert NO ToolOutput event ever surfaced — belt-and-suspenders
    // alongside the dispatch_count check. Drain everything the
    // channel still holds, then inspect.
    let mut residual = Vec::new();
    while let Ok(ev) = tokio::time::timeout(Duration::from_millis(50), event_rx.recv()).await {
        if let Some(ev) = ev {
            residual.push(ev);
        } else {
            break;
        }
    }
    let tool_outputs: Vec<&AgentEvent> = residual
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolOutput { .. }))
        .collect();
    assert_eq!(
        tool_outputs.len(),
        0,
        "RESUME-PHASE REGRESSION: a ToolOutput event surfaced in the \
         residual drain despite dispatch_count = 0. This points at \
         either (a) a second dispatch site that is not the counting \
         tool, or (b) the counting tool's sink being invoked outside \
         of invoke(). Either is a LOOP-06 regression. Residual: {residual:?}"
    );

    let _keep_alive_model_tx = model_tx;
    let _keep_alive_ctl_tx = ctl_tx;
}
