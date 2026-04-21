//! Control channel — the `Pause` / `Resume` / `Abort` pipe into the
//! Wave 4 agent loop's `tokio::select!`.
//!
//! # LOOP-06
//!
//! The control channel is one of four `tokio::mpsc` channels the Wave 4
//! turn loop multiplexes over (input / model / tool / control). In the
//! locked-decision table from `05-BRAINSTORM.md` §Engineering-Lens, the
//! `select!` runs with `biased;` priority — **control > input > tool >
//! model** — so a pending `Abort` always wins against in-flight model
//! frames and is observed on the very next select! tick.
//!
//! This module owns the primitives only:
//!
//! - [`ControlMsg`] — the 3-variant signal enum (Pause / Resume / Abort)
//! - [`control_channel`] — standard bounded mpsc pair constructor
//!   (capacity [`CONTROL_CHANNEL_CAPACITY`])
//! - [`install_ctrl_c_handler`] — binds `tokio::signal::ctrl_c()` so a
//!   SIGINT delivers one `ControlMsg::Abort` into the loop
//!
//! # Semantics expected by Wave 4
//!
//! The Wave 4 `loop` module is the consumer that gives these messages
//! meaning; tests for those semantics live alongside it. The shape
//! contract here is:
//!
//! - `Pause` — the loop stops forwarding events to the UI sink and
//!   starts buffering (`VecDeque<AgentEvent>`, DL-2 "buffer-and-replay")
//!   until `Resume` arrives. Model / tool channel traffic keeps flowing
//!   into the buffer; no new model turns are started.
//! - `Resume` — the loop flushes the buffer in order to the UI sink,
//!   then resumes normal select! operation.
//! - `Abort` — cooperative cancellation. The loop sets an abort flag,
//!   gives in-flight tool invocations a **2 s grace window** to observe
//!   it and unwind cleanly (the "abort mid-write" risk row in
//!   BRAINSTORM), then falls back to Phase 4's Job-Object /
//!   process-group kill for anything still alive. Final exit code is
//!   130 (SIGINT convention).
//!
//! The channel-delivery budget is separate and tighter:
//! **< 500 ms from Ctrl-C to `Abort` observed at the receiver**
//! (`05-BRAINSTORM.md` §Product-Lens success metric). The 2 s grace
//! window applies AFTER that 500 ms, inside the loop body, on
//! in-flight tool cleanup — not on the channel delivery itself.
//!
//! # Why capacity 32?
//!
//! BRAINSTORM §Engineering-Lens locked-decision table:
//! *"Buffered with bound `32` each to backpressure model stream
//! explosions."* Control traffic is user-paced (a human cannot press
//! Ctrl-C at 1 kHz), so 32 is far more than we need — the parity with
//! the other three channels keeps `select!` backpressure uniform and
//! avoids one channel accidentally starving another.

use tokio::sync::mpsc;

/// Bounded capacity for the control mpsc.
///
/// Matches the other three Wave 4 loop channels (input / model / tool)
/// per `05-BRAINSTORM.md` §Engineering-Lens locked decision.
pub const CONTROL_CHANNEL_CAPACITY: usize = 32;

/// Control signals routed from the CLI / UI into the agent loop.
///
/// `Copy` is load-bearing: the Wave 4 `select!` arm matches on the
/// message without taking ownership, so the loop can both trace-log
/// the signal and act on it without a `.clone()` dance.
///
/// The derives (`Debug + Clone + Copy + PartialEq + Eq`) are the
/// minimum surface area for mpsc + test ergonomics and are locked by
/// `tests/control.rs::control_msg_variant_shapes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMsg {
    /// Stop forwarding events to the UI sink; start buffering via
    /// `VecDeque<AgentEvent>` (DL-2) until `Resume` arrives.
    ///
    /// The loop does NOT kill in-flight tools on `Pause`; it holds
    /// them and holds any new model turn. This preserves the invariant
    /// that `Pause` is non-destructive — a user can `Pause` → inspect →
    /// `Resume` without losing context.
    Pause,

    /// Flush the pause-buffer in order to the UI sink, then resume
    /// normal `select!` operation.
    Resume,

    /// Cooperatively cancel the current turn.
    ///
    /// Sequence (Wave 4 implements; documented here for the contract):
    ///
    /// 1. Loop sets the abort flag; new model turns are rejected.
    /// 2. In-flight tool invocations get a 2 s grace window to observe
    ///    the flag and unwind.
    /// 3. After grace, Phase 4 Job-Object / process-group kill reaps
    ///    any remaining descendants.
    /// 4. Pending events flush, then exit 130.
    Abort,
}

/// Construct the bounded `(tx, rx)` pair for the control channel.
///
/// Uses [`CONTROL_CHANNEL_CAPACITY`] — see module-level docs for the
/// rationale.
///
/// # Example
///
/// ```no_run
/// use kay_core::control::{control_channel, ControlMsg};
///
/// # async fn demo() {
/// let (tx, mut rx) = control_channel();
/// tx.send(ControlMsg::Abort).await.unwrap();
/// assert_eq!(rx.recv().await, Some(ControlMsg::Abort));
/// # }
/// ```
pub fn control_channel() -> (mpsc::Sender<ControlMsg>, mpsc::Receiver<ControlMsg>) {
    mpsc::channel(CONTROL_CHANNEL_CAPACITY)
}

/// Spawn a background Tokio task that listens for `Ctrl-C` and emits a
/// single [`ControlMsg::Abort`] into `tx` when it fires.
///
/// Returns `Ok(())` after the task is spawned; the handler does NOT
/// block the caller. The spawned task:
///
/// 1. Awaits `tokio::signal::ctrl_c()`.
/// 2. On signal, best-effort sends `ControlMsg::Abort` on `tx`.
///    If the receiver is already dropped (loop already exited), the
///    send error is silently discarded — there is nothing meaningful
///    to do at that point.
/// 3. On `ctrl_c().await` error (signal subsystem failed to install),
///    the task exits without sending. The agent loop will still
///    terminate via its other exit conditions (stdin EOF, model
///    end-of-stream, explicit UI `Abort`).
///
/// # Errors
///
/// Returns the `std::io::Error` from the signal-install path if it
/// fails synchronously. In practice `tokio::signal::ctrl_c()` does
/// its subsystem setup lazily on first `.await`, so this return type
/// is here for forward-compat; the spawn itself is infallible.
///
/// # Platform notes
///
/// - Unix: binds SIGINT via `tokio::signal::ctrl_c()`.
/// - Windows: binds the console Ctrl-C handler via
///   `SetConsoleCtrlHandler` (Tokio handles the FFI).
///
/// # Why this is not unit-tested here
///
/// `tokio::signal::ctrl_c()` cannot be raised safely from inside a
/// `cargo test` subprocess without terminating the test harness itself.
/// The smoke-path ("install → Ok") is exercised by Wave 4's loop-level
/// integration tests that call this function at startup; the channel
/// delivery path is unit-tested in
/// `tests/control.rs::control_abort_cooperative_grace` by calling
/// `tx.send(ControlMsg::Abort)` directly from a spawned task.
pub fn install_ctrl_c_handler(tx: mpsc::Sender<ControlMsg>) -> std::io::Result<()> {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_err() {
            // Signal subsystem failed to install on this platform /
            // runtime; fall through silently. See function docs.
            return;
        }
        // Best-effort: if the loop has already exited the receiver is
        // dropped and SendError is meaningless.
        let _ = tx.send(ControlMsg::Abort).await;
    });
    Ok(())
}
