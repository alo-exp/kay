//! Phase 5 Wave 2 T2.6 RED — unit tests for `kay_core::control`:
//! `ControlMsg` enum shape + `control_channel()` mpsc helper.
//!
//! ## What this covers
//!
//! The control channel is the single pipe through which the agent
//! loop (Wave 4) receives `Pause` / `Resume` / `Abort` signals. It is
//! the LOOP-06 primitive — on its own, it does nothing interesting;
//! Wave 4's `tokio::select!` body is where Pause triggers the
//! buffer-and-replay (DL-2) and Abort triggers cooperative
//! cancellation with a 2s grace window (BRAINSTORM §Engineering-Lens).
//!
//! These tests lock the *shape of the primitive*, not the semantics
//! of the consumer — because the consumer lives in Wave 4 and tests
//! of the consumer appear there. Tests here:
//!
//! 1. **`control_msg_variant_shapes`** — the enum has exactly three
//!    unit variants (`Pause`, `Resume`, `Abort`), and the derives
//!    required for `tokio::sync::mpsc` ergonomics are present
//!    (`Debug + Clone + Copy + PartialEq + Eq`). `Copy` in particular
//!    is load-bearing: the `select!` arm in Wave 4 may match on the
//!    message without consuming it.
//!
//! 2. **`control_channel_pair_split`** — `control_channel()` returns
//!    a mpsc (sender, receiver) pair; sending produces FIFO-ordered
//!    deliveries on the receiver; dropping the sender closes the
//!    receiver (`recv() -> None`). This is the shutdown signal the
//!    agent-loop observes when the CLI exits cleanly.
//!
//! 3. **`control_abort_cooperative_grace`** — simulates a
//!    Ctrl-C-style handler task that fires `Abort` into the channel;
//!    asserts the receiver observes it within the 500ms Product-Lens
//!    success metric ("Ctrl-C abort cleanup < 500ms from signal →
//!    turn aborted + events flushed + exit 130" — 05-BRAINSTORM.md
//!    §Product-Lens). The *2s grace window* mentioned in
//!    BRAINSTORM's abort-mid-write risk row is a SEPARATE thing — it
//!    applies to in-flight tool cleanup inside the loop body, not to
//!    the channel delivery itself. The channel's budget is 500ms;
//!    the loop body's budget is 2s.
//!
//! ## Expected RED state (T2.6)
//!
//! `kay_core::control` does not yet exist. `cargo test -p kay-core
//! --test control` fails at compile with E0433 / E0432 ("unresolved
//! import `kay_core::control`"). T2.7 GREEN creates
//! `crates/kay-core/src/control.rs` + module decl in `lib.rs`.
//!
//! `install_ctrl_c_handler` is intentionally NOT exercised here —
//! that function binds `tokio::signal::ctrl_c()` which cannot be
//! raised safely inside a `cargo test` subprocess without terminating
//! the test harness. Its smoke path (`install → returns Ok`) is
//! adequately covered by Wave 4's loop-level integration tests that
//! call it during startup.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use kay_core::control::{self, ControlMsg};
use tokio::time::timeout;

// -----------------------------------------------------------------------------
// T2.6.a — variant shape + required derives
// -----------------------------------------------------------------------------

#[test]
fn control_msg_variant_shapes() {
    let p = ControlMsg::Pause;
    let r = ControlMsg::Resume;
    let a = ControlMsg::Abort;

    // PartialEq reflexivity
    assert_eq!(p, ControlMsg::Pause);
    assert_eq!(r, ControlMsg::Resume);
    assert_eq!(a, ControlMsg::Abort);

    // PartialEq discrimination — three distinct variants
    assert_ne!(p, r);
    assert_ne!(p, a);
    assert_ne!(r, a);

    // Copy — the select! arm in Wave 4 must be able to look at the
    // incoming message without taking ownership.
    let p_copied = p;
    assert_eq!(p, p_copied);

    // Debug format names the variant — used in tracing logs.
    assert_eq!(format!("{p:?}"), "Pause");
    assert_eq!(format!("{r:?}"), "Resume");
    assert_eq!(format!("{a:?}"), "Abort");

    // Clone — orthogonal to Copy but still asserted for API stability.
    #[allow(clippy::clone_on_copy)]
    let p_cloned = p.clone();
    assert_eq!(p, p_cloned);
}

// -----------------------------------------------------------------------------
// T2.6.b — mpsc pair: FIFO delivery + drop semantics
// -----------------------------------------------------------------------------

#[tokio::test]
async fn control_channel_pair_split() {
    let (tx, mut rx) = control::control_channel();

    // FIFO preservation across all three variants
    tx.send(ControlMsg::Pause).await.unwrap();
    tx.send(ControlMsg::Resume).await.unwrap();
    tx.send(ControlMsg::Abort).await.unwrap();

    assert_eq!(rx.recv().await, Some(ControlMsg::Pause));
    assert_eq!(rx.recv().await, Some(ControlMsg::Resume));
    assert_eq!(rx.recv().await, Some(ControlMsg::Abort));

    // Dropping the sender closes the channel — Wave 4's loop uses
    // this as one of the termination conditions in its `select!`.
    drop(tx);
    assert_eq!(
        rx.recv().await,
        None,
        "receiver must see None once all senders have been dropped"
    );
}

// -----------------------------------------------------------------------------
// T2.6.c — cooperative grace: abort delivery stays under 500ms budget
// -----------------------------------------------------------------------------

#[tokio::test]
async fn control_abort_cooperative_grace() {
    let (tx, mut rx) = control::control_channel();

    // Simulate a Ctrl-C handler task: in production this is
    // `tokio::signal::ctrl_c().await; tx.send(Abort).await` inside
    // `install_ctrl_c_handler`. Here we fire the abort immediately
    // so the test exercises the channel delivery path without
    // depending on OS signal plumbing.
    tokio::spawn(async move {
        tx.send(ControlMsg::Abort)
            .await
            .expect("handler task failed to enqueue Abort");
    });

    // Product-Lens success metric: < 500ms from signal → observed.
    let got = timeout(Duration::from_millis(500), rx.recv()).await;
    match got {
        Ok(Some(msg)) => assert_eq!(msg, ControlMsg::Abort),
        Ok(None) => panic!("channel closed before Abort arrived — handler task dropped tx early"),
        Err(_) => panic!(
            "Abort did not arrive within the 500ms grace window (05-BRAINSTORM.md \
             success metric); channel delivery is too slow for Wave 4's abort path"
        ),
    }
}
