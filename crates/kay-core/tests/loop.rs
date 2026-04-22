//! Phase 5 Wave 4 T4.1 RED — agent loop single-turn happy path.
//!
//! LOOP-01. The first test pinning the behavior of
//! [`kay_core::r#loop::run_turn`]. Contract under test:
//!
//!   Given a model-event stream that yields one `TextDelta` and then
//!   closes cleanly (mock provider flushes + drops its sender), and
//!   no other inputs, the loop:
//!
//!     1. Forwards the `TextDelta` to the event sink (caller's
//!        `event_tx`) — exactly once, with content intact.
//!     2. Observes the `model_rx` close and exits cleanly.
//!     3. Returns `Ok(())` from the spawned task.
//!
//! ## Why this is the first test
//!
//! The loop has four inputs (control, input, tool, model) per
//! `05-BRAINSTORM.md` §E2. Most of Wave 4 is about priority ordering
//! and state machines (pause buffer, abort grace, verify gate) — but
//! before any of that matters, the baseline behavior is "forward model
//! frames and exit on stream close". If this test does not hold, none
//! of the later priority/state-machine tests mean anything.
//!
//! ## Expected RED state (T4.1)
//!
//! `kay_core::r#loop` does not yet exist. Compilation fails with
//! E0432 "unresolved import `kay_core::r#loop`". T4.2 GREEN creates
//! `crates/kay-core/src/loop.rs`, adds `pub mod r#loop;` to
//! `lib.rs`, and implements the minimum `run_turn` skeleton that
//! makes this test pass.
//!
//! The API shape this test pins is intentionally minimal — three
//! channels (`model_rx`, `control_rx`, `event_tx`) and a persona.
//! Later waves add `input_rx`, `tool_rx`, `registry`, `sandbox`,
//! `verifier`. The struct-literal initializer (`RunTurnArgs { … }`)
//! makes those additions non-breaking for callers that construct
//! the struct with explicit field names.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::{ControlMsg, control_channel};
use kay_core::r#loop::{RunTurnArgs, run_turn, TurnResult};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    ToolRegistry, VerificationOutcome,
};

// T4.4 GREEN rippled: `RunTurnArgs` now carries `registry` + `tool_ctx`.
// The T4.1 happy-path test never emits a `ToolCallComplete`, so neither
// the registry nor the tool-context is exercised — but both fields must
// still be supplied. Kept as local stubs rather than a shared helper
// because the stub surface is ~30 LOC and sharing it would couple two
// independent test files.
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
async fn run_turn_single_turn_happy_path() {
    // ── Channel setup ────────────────────────────────────────────
    // Capacity 32 matches the locked Wave 4 channel capacity (05-
    // BRAINSTORM.md §Engineering-Lens) — keeps backpressure uniform
    // with the three other loop channels even though this test only
    // drives the model channel.
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // ── Mock provider: one TextDelta, then close ────────────────
    // Closing the sender before run_turn starts guarantees the
    // select! sees the stream-close branch without racing — the
    // value is already buffered, and recv() returns None after.
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "hello kay".into() }))
        .await
        .expect("buffered send into capacity-32 channel");
    drop(model_tx);

    // ── Load forge persona from bundled YAML ────────────────────
    let persona = Persona::load("forge").expect("bundled forge persona loads");

    // ── Minimal tool fixtures (unused in this test) ─────────────
    // T4.4 added `registry` + `tool_ctx` to `RunTurnArgs`. This
    // test emits only a `TextDelta`, so dispatch never runs — but
    // the fields are required. An empty registry + a context with
    // a no-op sink is sufficient; the sink is never called.
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    // ── Spawn the loop ──────────────────────────────────────────
    // `tokio::spawn` decouples the loop task from the test thread
    // so we can drain `event_rx` concurrently with loop execution.
    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    // ── Drain events until the loop drops `event_tx` ────────────
    // `recv()` returns None when the loop returns (Drop runs on
    // `event_tx`). No timeout needed because the happy path is
    // bounded by a dropped `model_tx`.
    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    // ── Assert: loop returned Ok(TurnResult::Completed) ───────────
    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) on stream close");

    // ── Assert: exactly one forwarded TextDelta ─────────────────
    assert_eq!(
        events.len(),
        1,
        "exactly one AgentEvent should have flowed through; got {} events",
        events.len()
    );
    assert!(
        matches!(&events[0], AgentEvent::TextDelta { content } if content == "hello kay"),
        "expected AgentEvent::TextDelta(\"hello kay\") forwarded through the loop; got: {:?}",
        events[0]
    );
}

// ───────────────────────────────────────────────────────────────────────
// T4.5 RED — verify gate on task_complete (LOOP-05)
// ───────────────────────────────────────────────────────────────────────
//
// These two tests pin the verifier short-circuit contract:
//
//   • Pending outcome → loop MUST continue draining model_rx
//     (verification is still in flight; a stubbed NoOpVerifier must
//     not accidentally end the turn). Currently-passing regression
//     guard: if T4.6 wires the gate too eagerly this test breaks.
//
//   • Pass outcome → loop MUST terminate immediately, even if more
//     model frames are still buffered / the sender is still alive.
//     Currently-FAILING: on T4.4 the loop forwards the event and
//     keeps polling model_rx, so this test times out.
//
// Why two tests instead of one: the two outcomes are the two sides of
// the same gate, and a single test cannot express both "continue on
// Pending" AND "halt on Pass" without either racing or over-coupling
// two invariants into one assertion. Splitting them also makes RED vs.
// GREEN legibility cleaner — a failure in "Pending" is a different
// bug (over-eager gate) than a failure in "Pass" (missing gate), and
// the test name should say which.
//
// Timeout is picked at 500ms: way more than enough for the loop to
// peek a single buffered frame on any reasonable CI host, but short
// enough that a failing test does not look like it hung forever.
// A bounded timeout is the only way to express "the loop should have
// already returned by now" — polling `handle.is_finished()` races
// with the runtime scheduler and is flaky under `--test-threads=1`.

/// Pending outcome must NOT short-circuit — the loop forwards the
/// `TaskComplete` event, then continues, and exits normally when
/// `model_rx` closes. Both the TaskComplete AND the follow-up
/// TextDelta must reach the event sink.
#[tokio::test]
async fn task_complete_does_not_terminate_on_pending_verification() {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // Pending verification — this is the NoOpVerifier outcome in
    // Phase 3/5 (real verifier lands in Phase 8). The loop MUST NOT
    // treat Pending as "turn complete"; otherwise every turn would
    // short-circuit before the model ever finishes its response.
    model_tx
        .send(Ok(AgentEvent::TaskComplete {
            call_id: "call-complete-pending".into(),
            verified: false,
            outcome: VerificationOutcome::Pending {
                reason: "awaiting real verifier (Phase 8)".into(),
            },
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    // A follow-up event after the Pending TaskComplete — it MUST be
    // forwarded before the loop exits. If the loop short-circuits on
    // Pending, this TextDelta never reaches the sink and the count
    // assertion below trips.
    model_tx
        .send(Ok(AgentEvent::TextDelta {
            content: "after-pending".into(),
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    drop(model_tx); // clean close → loop exits via stream-close branch

    let persona = Persona::load("forge").expect("bundled forge persona loads");
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) on stream close");

    assert_eq!(
        events.len(),
        2,
        "Pending TaskComplete must NOT short-circuit the loop: expected \
         both TaskComplete + the follow-up TextDelta to flow; got {} events: {:?}",
        events.len(),
        events,
    );
    assert!(
        matches!(
            &events[0],
            AgentEvent::TaskComplete {
                verified: false,
                outcome: VerificationOutcome::Pending { .. },
                ..
            }
        ),
        "expected TaskComplete {{ verified: false, outcome: Pending }} as first event; \
         got: {:?}",
        events[0],
    );
    assert!(
        matches!(&events[1], AgentEvent::TextDelta { content } if content == "after-pending"),
        "expected TextDelta(\"after-pending\") as second event \
         (proves loop kept draining after a Pending TaskComplete); got: {:?}",
        events[1],
    );
}

/// Pass outcome MUST terminate the loop immediately. The follow-up
/// `TextDelta` buffered in `model_rx` must NOT leak to the sink, and
/// `run_turn` must return `Ok(())` even though `model_tx` is still
/// alive (the turn is verified-complete — no reason to keep listening).
///
/// ## Expected RED state (T4.5)
///
/// T4.4's `run_turn` has no verifier gate: on `TaskComplete` it just
/// forwards the event and loops back to `model_rx.recv()`. With
/// `model_tx` held alive, `recv()` blocks forever — the drain loop
/// never sees `event_tx` drop, and the 500 ms timeout below fires
/// with a clear LOOP-05 diagnostic. T4.6 GREEN adds the
/// `VerificationOutcome::Pass` short-circuit and this test turns green.
#[tokio::test]
async fn task_complete_on_verifier_pass_terminates_loop() {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (_ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // Verified Pass — this is the signal that the turn is genuinely
    // done. The loop must terminate without dequeuing further frames.
    model_tx
        .send(Ok(AgentEvent::TaskComplete {
            call_id: "call-complete-pass".into(),
            verified: true,
            outcome: VerificationOutcome::Pass { note: "goal reached".into() },
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    // Bait event: if the loop does NOT short-circuit, this will be
    // pulled off `model_rx` and forwarded — the assertion below will
    // catch it. The test fails loudly rather than silently swallowing
    // a gate regression.
    model_tx
        .send(Ok(AgentEvent::TextDelta {
            content: "should-not-appear".into(),
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    // Intentionally NOT dropping `model_tx`: holding the sender alive
    // proves that the loop terminated because of the Pass verdict, not
    // because the stream happened to close. A keep-alive binding with
    // a `_`-prefixed name silences the unused-variable warning while
    // retaining the live handle until scope exit.
    let _keep_alive_model_tx = model_tx;

    let persona = Persona::load("forge").expect("bundled forge persona loads");
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    // Drain events concurrently with the loop. On GREEN: `run_turn`
    // returns on Pass, drops `event_tx`, and `event_rx.recv()` yields
    // None — the drain exits cleanly well within the budget. On RED:
    // the loop blocks on `model_rx.recv()` after forwarding the two
    // buffered frames, so the sink never closes and the timeout fires.
    let drain = async {
        let mut events = Vec::new();
        while let Some(ev) = event_rx.recv().await {
            events.push(ev);
        }
        events
    };
    let events = tokio::time::timeout(std::time::Duration::from_millis(500), drain)
        .await
        .expect(
            "run_turn did not terminate within 500ms of a verified TaskComplete — \
             LOOP-05 verify gate is missing: the loop is still blocked on \
             model_rx.recv() instead of short-circuiting on VerificationOutcome::Pass",
        );

    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) on verified TaskComplete");

    assert!(
        events
            .iter()
            .all(|e| !matches!(e, AgentEvent::TextDelta { .. })),
        "the bait TextDelta buffered BEHIND TaskComplete Pass must not leak \
         through the sink — the verify gate must short-circuit before that \
         frame is dequeued. Got: {:?}",
        events,
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            AgentEvent::TaskComplete {
                verified: true,
                outcome: VerificationOutcome::Pass { .. },
                ..
            }
        )),
        "expected the verified TaskComplete to be forwarded before termination; \
         got: {:?}",
        events,
    );
}

// ───────────────────────────────────────────────────────────────────────
// T4.9 RED — Pause / Resume / Abort state machine (LOOP-06)
// ───────────────────────────────────────────────────────────────────────
//
// These three tests pin the control-channel contract specified in
// `05-BRAINSTORM.md` §Engineering-Lens and §DL-2:
//
//   • Pause  → emit `AgentEvent::Paused` once, then buffer every
//              subsequent model frame instead of forwarding it.
//   • Resume → flush the pause buffer in FIFO order to `event_tx`,
//              then return to normal forward-on-recv behavior.
//   • Abort  → emit `AgentEvent::Aborted { reason: "user_abort" }`
//              once and return `Ok(())`, even if `model_tx` is still
//              alive. Second Abort is a no-op (idempotent).
//
// Why three tests instead of one combined fixture: each control
// message has a distinct state-machine contract, and folding them into
// a single test would make RED diagnostics ambiguous — "Pause buffers
// + Resume flushes + Abort exits" is three different assertions on
// three different code paths. Splitting them also lets T4.10 GREEN
// ship Pause/Resume + Abort as independent sub-commits if the shape of
// the VecDeque-backed pause buffer surfaces a subtle bug in one path
// but not the other.
//
// ## Synchronization strategy
//
// Tests use `event_rx.recv()` as explicit sync points wherever an
// event is EXPECTED (e.g. "after Pause, the Paused event arrives").
// For the "should NOT arrive" invariant (pause-buffering), we use a
// tiny sleep + `try_recv().is_err()` — long enough that the loop has
// certainly polled `model_rx` and run the pause-buffer branch, but
// short enough that a real regression surfaces in under a second.
//
// Timeout-bounded recvs (`tokio::time::timeout`) wrap the places where
// a missing GREEN implementation would otherwise cause the test to
// hang indefinitely: Pause-emission, Abort-emission, and the final
// `handle.await` on Abort. The timeout message is the RED diagnostic —
// it names the specific LOOP-06 sub-contract that is unimplemented.
//
// ## Expected RED state (T4.9)
//
// T4.6's `run_turn` has a no-op control-arm handler: it accepts
// `ControlMsg::{Pause, Resume, Abort}` and continues. Consequently:
//
//   • Test 1 hangs on the first timeout waiting for `Paused`.
//   • Test 2 hangs on the first timeout waiting for `Aborted`.
//   • Test 3 hangs identically to Test 2.
//
// T4.10 GREEN adds the three handlers and the VecDeque-backed pause
// buffer, turning all three tests green.

/// Pause buffers subsequent model events; Resume flushes them in
/// FIFO order; post-Resume model events forward directly again.
///
/// The test walks one full cycle: pre-pause, pause, buffering,
/// resume, post-resume. Each phase has a distinct sync point so a
/// regression in any transition is localized by the failure message.
#[tokio::test]
async fn control_pause_buffers_then_resume_replays() {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    let persona = Persona::load("forge").expect("bundled forge persona loads");
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    // ── Phase 1: pre-pause — event forwards normally ────────────────
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "e1".into() }))
        .await
        .expect("buffered send into capacity-32 channel");
    let ev = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect("pre-pause TextDelta should forward within 500ms")
        .expect("event channel should deliver e1");
    assert!(
        matches!(&ev, AgentEvent::TextDelta { content } if content == "e1"),
        "pre-pause TextDelta should pass through unmodified; got: {:?}",
        ev,
    );

    // ── Phase 2: pause — loop emits Paused ──────────────────────────
    ctl_tx
        .send(ControlMsg::Pause)
        .await
        .expect("control channel accepts Pause");
    let ev = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect(
            "ControlMsg::Pause must cause AgentEvent::Paused to emit within 500ms — \
             LOOP-06 pause handler missing",
        )
        .expect("event channel should deliver Paused");
    assert!(
        matches!(&ev, AgentEvent::Paused),
        "expected AgentEvent::Paused immediately after ControlMsg::Pause; got: {:?}",
        ev,
    );

    // ── Phase 3: while paused — events are buffered, not forwarded ─
    // Send two frames; neither should surface on `event_rx` while the
    // loop is paused. A 20 ms window is deliberately generous: the
    // model-arm branch must (a) poll recv(), (b) observe paused=true,
    // (c) push into VecDeque. Even on a loaded CI host the whole
    // round-trip is microseconds. If this window ever flakes, the
    // signal is "a new control path is doing I/O before checking the
    // pause flag" — not a test bug.
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "e2".into() }))
        .await
        .expect("buffered send");
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "e3".into() }))
        .await
        .expect("buffered send");
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert!(
        event_rx.try_recv().is_err(),
        "paused loop must NOT forward buffered events — pause state leaked",
    );

    // ── Phase 4: resume — buffered frames flush in FIFO order ───────
    ctl_tx
        .send(ControlMsg::Resume)
        .await
        .expect("control channel accepts Resume");
    let ev2 = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect("Resume must flush buffered events within 500ms")
        .expect("event channel should deliver e2");
    assert!(
        matches!(&ev2, AgentEvent::TextDelta { content } if content == "e2"),
        "first flushed event must be e2 (FIFO); got: {:?}",
        ev2,
    );
    let ev3 = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect("second buffered event must flush promptly")
        .expect("event channel should deliver e3");
    assert!(
        matches!(&ev3, AgentEvent::TextDelta { content } if content == "e3"),
        "second flushed event must be e3 (FIFO); got: {:?}",
        ev3,
    );

    // ── Phase 5: post-resume — forwarding is back to normal ────────
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "e4".into() }))
        .await
        .expect("buffered send");
    let ev4 = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect("post-resume events must forward directly again")
        .expect("event channel should deliver e4");
    assert!(
        matches!(&ev4, AgentEvent::TextDelta { content } if content == "e4"),
        "post-resume TextDelta must pass through unmodified; got: {:?}",
        ev4,
    );

    // ── Phase 6: clean close → loop exits Ok ───────────────────────
    drop(model_tx);
    drop(ctl_tx);
    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) on clean close after pause/resume cycle");
}

/// Abort emits `AgentEvent::Aborted { reason: "user_abort" }` and
/// exits with `Ok(())`, even when `model_tx` is still alive.
///
/// Holding `model_tx` alive is the critical invariant here: it proves
/// that the loop terminated because of the Abort verdict, not because
/// the stream happened to close. Without the keep-alive, a buggy
/// implementation that ignores Abort but happens to see the stream
/// close first would pass the test silently.
#[tokio::test]
async fn control_abort_emits_aborted_event_and_exits() {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    let persona = Persona::load("forge").expect("bundled forge persona loads");
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    // One pre-abort event to prove the loop is responsive before the
    // abort lands — catches the degenerate case where the loop is
    // already broken and any "abort works" assertion below would pass
    // vacuously.
    model_tx
        .send(Ok(AgentEvent::TextDelta { content: "e1".into() }))
        .await
        .expect("buffered send into capacity-32 channel");
    let ev = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect("pre-abort TextDelta should forward within 500ms")
        .expect("event channel should deliver e1");
    assert!(
        matches!(&ev, AgentEvent::TextDelta { content } if content == "e1"),
        "pre-abort TextDelta should pass through unmodified; got: {:?}",
        ev,
    );

    // Fire Abort with `model_tx` ALIVE — the exit must come from the
    // Abort verdict, not a stream-close race.
    ctl_tx
        .send(ControlMsg::Abort)
        .await
        .expect("control channel accepts Abort");

    // Expect Aborted event within 500ms (Product-Lens Ctrl-C budget).
    let ev = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
        .await
        .expect(
            "ControlMsg::Abort must cause AgentEvent::Aborted to emit within 500ms — \
             LOOP-06 abort handler missing",
        )
        .expect("event channel should deliver Aborted");
    assert!(
        matches!(&ev, AgentEvent::Aborted { reason } if reason == "user_abort"),
        "expected AgentEvent::Aborted {{ reason: \"user_abort\" }}; got: {:?}",
        ev,
    );

    // Loop must exit Ok(()) within the budget. If the loop mis-
    // handles Abort by only emitting the event without returning,
    // `handle.await` hangs until the outer test harness times out —
    // wrapping it in `timeout` gives a clear diagnostic instead.
    let join = tokio::time::timeout(std::time::Duration::from_millis(500), handle)
        .await
        .expect("run_turn must return within 500ms of Abort");
    join.expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) after Abort");

    // Keep model_tx alive until here — proves the exit was Abort-
    // driven, not close-driven.
    let _keep_alive_model_tx = model_tx;
    let _keep_alive_ctl_tx = ctl_tx;
}

/// Abort is idempotent: firing it twice emits at most one Aborted
/// event, and the loop still exits cleanly.
///
/// Why this matters: the CLI's Ctrl-C handler and the Product-Lens
/// "cooperative-then-force" two-stage cancel both converge on the
/// same `ControlMsg::Abort` message. If the second press (or a
/// retry from a nervous user) triggered a second Aborted event, the
/// JSONL event stream Phase 9 frontends consume would show a
/// spurious second abort after the turn is already dead — confusing
/// to render and worse to debug in prod.
///
/// The second `ctl_tx.send` may succeed (if the loop has not yet
/// dropped `control_rx`) or fail (if it has) — the test does not
/// assert on either outcome; the invariant under test is downstream:
/// the event stream must contain ≤ 1 Aborted event regardless.
#[tokio::test]
async fn control_double_abort_is_idempotent() {
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (ctl_tx, control_rx) = control_channel();
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    let persona = Persona::load("forge").expect("bundled forge persona loads");
    let registry = Arc::new(ToolRegistry::new());
    // nesting_depth = 0: these are top-level turns. sage_query depth
    // threading is tested in `crates/kay-tools/tests/sage_query.rs`.
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
        Arc::new(Mutex::new(String::new())),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
        context_engine: std::sync::Arc::new(kay_context::engine::NoOpContextEngine),
        context_budget: kay_context::budget::ContextBudget::default(),
        initial_prompt: String::new(),
    }));

    // First abort: expected to emit exactly one Aborted and trigger
    // the return. Use a short yield so the loop has a chance to
    // select the control arm before we send the second Abort.
    ctl_tx
        .send(ControlMsg::Abort)
        .await
        .expect("first Abort must be accepted");

    // Second abort: may succeed (if the loop hasn't dropped the
    // receiver yet) or fail (if it has). Either is allowed — the
    // only invariant is that no second Aborted event ever emits.
    let _ = ctl_tx.send(ControlMsg::Abort).await;

    // Exit must complete within the same budget as the single-abort
    // case. Extra time on double-abort would imply the loop is
    // blocking on a drained control channel, not exiting on the
    // abort it already observed.
    let join = tokio::time::timeout(std::time::Duration::from_millis(500), handle)
        .await
        .expect("run_turn must exit within 500ms even under double-Abort");
    join.expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok(TurnResult::Completed) after double-Abort");

    // Drain any remaining events (`event_tx` is now dropped by the
    // loop's return, so the channel is closed and `recv()` will
    // yield None once drained).
    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    // Idempotency invariant: exactly one Aborted event.
    let aborted_count = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::Aborted { .. }))
        .count();
    assert_eq!(
        aborted_count, 1,
        "double-Abort must emit exactly one AgentEvent::Aborted — got {}: {:?}",
        aborted_count, events,
    );

    let _keep_alive_model_tx = model_tx;
}
