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

use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::control_channel;
use kay_core::r#loop::{RunTurnArgs, run_turn};
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
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
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
    }));

    // ── Drain events until the loop drops `event_tx` ────────────
    // `recv()` returns None when the loop returns (Drop runs on
    // `event_tx`). No timeout needed because the happy path is
    // bounded by a dropped `model_tx`.
    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    // ── Assert: loop returned Ok(()) ────────────────────────────
    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok on stream close");

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
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
    }));

    let mut events = Vec::new();
    while let Some(ev) = event_rx.recv().await {
        events.push(ev);
    }

    handle
        .await
        .expect("run_turn task joined cleanly")
        .expect("run_turn returned Ok on stream close");

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
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
    );

    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
        registry,
        tool_ctx,
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
        .expect("run_turn returned Ok on verified TaskComplete");

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
