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

/// Install a `Ctrl-C` / SIGINT listener whose delivery forwards a single
/// [`ControlMsg::Abort`] into `tx`.
///
/// Synchronous-install contract: by the time this function returns `Ok`,
/// the POSIX `sigaction` (Unix) / `SetConsoleCtrlHandler` (Windows)
/// registration has **already taken effect**. This is load-bearing —
/// the CLI-03 SIGINT E2E test (`kay-cli::tests::cli_e2e::
/// exit_code_130_on_sigint_nix`) waits a short settle window and then
/// `kill -INT <pid>`s the child; if the handler were armed lazily on
/// first `.await` (as with the bare `tokio::signal::ctrl_c()` future),
/// the scheduler's polling order could let the SIGINT arrive before
/// the task is polled, falling back to the default-disposition
/// termination and breaking the test.
///
/// The implementation binds a `Signal` stream via the platform-specific
/// `tokio::signal` constructor (`unix::signal(SignalKind::interrupt())`
/// on Unix, `windows::ctrl_c()` on Windows) — both of which install the
/// OS-level handler eagerly at construction time — and then spawns a
/// background task that does the `recv().await` + forward dance.
///
/// ## Task body
///
/// 1. `recv().await` on the pre-installed signal stream.
/// 2. On `Some(())`, best-effort `tx.send(ControlMsg::Abort).await`.
///    If the receiver has already been dropped (loop exited), the
///    `SendError` is silently discarded — there is nothing meaningful
///    to do at that point.
/// 3. On `None` (the stream ended unexpectedly — e.g. a platform
///    anomaly that unregisters the handler mid-flight), the task exits
///    without sending. The agent loop terminates via its other exit
///    conditions (stdin EOF, model end-of-stream, explicit UI `Abort`).
///
/// # Errors
///
/// Returns the `std::io::Error` from the platform-level handler
/// installation if it fails. In practice this only fails if the tokio
/// runtime was not built with `.enable_all()` / the signal driver, or
/// on extremely constrained platforms — both of which would be
/// configuration bugs caught in CI, not user-facing errors.
///
/// # Why this is not unit-tested here
///
/// `kill -INT` cannot be safely raised from inside a `cargo test`
/// subprocess without terminating the test harness itself. The
/// install-returns-Ok smoke path is exercised by kay-cli's
/// `exit_code_130_on_sigint_nix` E2E test, which spawns `kay` as a
/// separate process and sends it SIGINT. The channel-delivery semantics
/// are unit-tested in `tests/control.rs::control_abort_cooperative_
/// grace` by calling `tx.send(ControlMsg::Abort)` directly from a
/// spawned task — a hermetic substitute that exercises the same
/// receiver-side contract without raising a real signal.
pub fn install_ctrl_c_handler(tx: mpsc::Sender<ControlMsg>) -> std::io::Result<()> {
    #[cfg(unix)]
    let mut sig = {
        use tokio::signal::unix::{SignalKind, signal};
        // `signal(SignalKind::interrupt())` registers the sigaction
        // SYNCHRONOUSLY and returns an mpsc-like `Signal` stream. Any
        // SIGINT delivered after this line routes into `sig.recv()`.
        signal(SignalKind::interrupt())?
    };

    #[cfg(windows)]
    let mut sig = {
        // `ctrl_c()` on the Windows tokio::signal submodule installs
        // the console Ctrl-C handler via `SetConsoleCtrlHandler`
        // synchronously and returns a `CtrlC` stream.
        tokio::signal::windows::ctrl_c()?
    };

    tokio::spawn(async move {
        if sig.recv().await.is_none() {
            // Stream ended — platform anomaly (very unusual). Fall
            // through silently; other loop-exit conditions still apply.
            return;
        }
        // Best-effort: if the loop has already exited the receiver is
        // dropped and `SendError` is meaningless.
        let _ = tx.send(ControlMsg::Abort).await;
    });

    Ok(())
}

// M12-Task 6: Inline unit tests for kay-core control module.
// These complement the integration tests in tests/control.rs with
// pure synchronous assertions on the ControlMsg type and channel capacity.

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn control_msg_is_copy() {
        // ControlMsg must be Copy so the select! arm can match without ownership.
        fn assert_copy<T: Copy>() {}
        assert_copy::<ControlMsg>();
    }

    #[test]
    fn control_msg_variants() {
        assert!(matches!(ControlMsg::Pause, ControlMsg::Pause));
        assert!(matches!(ControlMsg::Resume, ControlMsg::Resume));
        assert!(matches!(ControlMsg::Abort, ControlMsg::Abort));
    }

    #[test]
    fn control_msg_partial_eq() {
        assert_eq!(ControlMsg::Pause, ControlMsg::Pause);
        assert_ne!(ControlMsg::Pause, ControlMsg::Resume);
        assert_ne!(ControlMsg::Abort, ControlMsg::Pause);
    }

    #[test]
    fn control_channel_capacity_is_32() {
        assert_eq!(CONTROL_CHANNEL_CAPACITY, 32);
    }

    #[test]
    fn control_channel_constructs_sender_receiver() {
        let (tx, rx) = control_channel();
        assert_eq!(tx.capacity(), CONTROL_CHANNEL_CAPACITY);
        // rx doesn't have a capacity() method; just assert construction doesn't panic
        let _ = rx;
    }

    #[tokio::test]
    async fn control_channel_send_and_receive_pause() {
        let (tx, mut rx) = control_channel();
        tx.send(ControlMsg::Pause).await.expect("send must succeed");
        let received = rx.recv().await;
        assert_eq!(received, Some(ControlMsg::Pause));
    }

    #[tokio::test]
    async fn control_channel_send_and_receive_resume() {
        let (tx, mut rx) = control_channel();
        tx.send(ControlMsg::Resume).await.expect("send must succeed");
        let received = rx.recv().await;
        assert_eq!(received, Some(ControlMsg::Resume));
    }

    #[tokio::test]
    async fn control_channel_send_and_receive_abort() {
        let (tx, mut rx) = control_channel();
        tx.send(ControlMsg::Abort).await.expect("send must succeed");
        let received = rx.recv().await;
        assert_eq!(received, Some(ControlMsg::Abort));
    }

    #[tokio::test]
    async fn control_channel_closes_on_drop() {
        let (tx, mut rx) = control_channel();
        drop(tx);
        assert_eq!(rx.recv().await, None);
    }

    #[test]
    fn install_ctrl_c_handler_returns_ok() {
        // install_ctrl_c_handler must not panic — it installs the signal handler.
        // The actual SIGINT delivery is tested by kay-cli's
        // exit_code_130_on_sigint_nix E2E subprocess test.
        let (tx, _rx) = control_channel();
        let result = install_ctrl_c_handler(tx);
        assert!(result.is_ok(), "install_ctrl_c_handler must succeed: {:?}", result);
    }
}
