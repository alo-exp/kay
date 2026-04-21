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

use kay_core::control::control_channel;
use kay_core::persona::Persona;
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_provider_errors::ProviderError;
use kay_tools::AgentEvent;
use tokio::sync::mpsc;

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
        .send(Ok(AgentEvent::TextDelta {
            content: "hello kay".into(),
        }))
        .await
        .expect("buffered send into capacity-32 channel");
    drop(model_tx);

    // ── Load forge persona from bundled YAML ────────────────────
    let persona = Persona::load("forge").expect("bundled forge persona loads");

    // ── Spawn the loop ──────────────────────────────────────────
    // `tokio::spawn` decouples the loop task from the test thread
    // so we can drain `event_rx` concurrently with loop execution.
    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        model_rx,
        control_rx,
        event_tx,
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
