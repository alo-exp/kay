//! Agent loop — the `tokio::select!`-driven turn runner (LOOP-01).
//!
//! # LOOP-01, LOOP-05, LOOP-06
//!
//! `run_turn` drives one agent turn: a provider stream is drained,
//! tool calls dispatched, user input honored, and control signals
//! (pause / resume / abort) respected — all coordinated through a
//! single biased `tokio::select!` so priority ordering is explicit
//! and deterministic.
//!
//! ## Priority ordering (biased)
//!
//! Per [`05-BRAINSTORM.md` §Engineering-Lens / §E2]:
//!
//! > control > input > tool > model
//!
//! Abort must outrun a model chunk, user input must outrun a model
//! chunk (so the user can steer mid-stream), and tool results must
//! outrun a model chunk (so tool_call → tool_result round-trips do
//! not backlog when a slow provider is mid-token). `biased;` on the
//! `select!` makes this deterministic.
//!
//! ## Channel topology (Wave 4 target — T4.2 starts minimal)
//!
//! | Channel      | Element type                                  |
//! |--------------|-----------------------------------------------|
//! | `control_rx` | [`crate::control::ControlMsg`]                |
//! | `input_rx`   | _(Wave 4 later tasks)_                        |
//! | `tool_rx`    | _(Wave 4 later tasks)_                        |
//! | `model_rx`   | `Result<AgentEvent, ProviderError>` frames    |
//!
//! T4.2 lands only the two channels the T4.1 happy-path test
//! exercises: `model_rx` (drives forward progress) and `control_rx`
//! (reserves the priority seat even though its handler is a stub
//! until T4.10). Subsequent tasks extend `RunTurnArgs` in-place —
//! struct-literal construction keeps those additions non-breaking.
//!
//! ## Exit conditions
//!
//! - `model_rx` closed cleanly → `Ok(())`. The happy path: the
//!   provider finished streaming its response, the model-adapter
//!   task dropped its sender, and the turn is over.
//! - `event_tx` closed (downstream consumer hung up) → `Ok(())`.
//!   No value is served by continuing to produce events no one
//!   reads.
//! - Provider frame is `Err(ProviderError)` → for T4.2, treated as
//!   clean exit. Error→`AgentEvent::Error` conversion is deferred
//!   to a Wave 4 later task.
//! - `ControlMsg::Abort` → Wave 4 T4.10. T4.2 stubs the handler.
//!
//! ## Not yet handled (tracked in PLAN.md Wave 4 later tasks)
//!
//! - **Tool dispatch** (T4.3/T4.4). A `ToolCallComplete` event flows
//!   through today; the dispatcher invocation comes in T4.4.
//! - **Verify gate** (T4.5/T4.6). `task_complete` is not currently
//!   gated on verifier result.
//! - **Control semantics** (T4.9/T4.10). `Pause` buffer-and-replay
//!   and `Abort` 2s grace are stubbed — T4.2 only reserves the
//!   select-arm seat.

use tokio::sync::mpsc;

use crate::control::ControlMsg;
use crate::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::AgentEvent;

/// Input bundle for [`run_turn`]. Constructed by callers via
/// struct-literal so later tasks (T4.3+) can add fields — the
/// breakage that comes with new required fields is intentional:
/// adding `input_rx` or `verifier` means every caller must be
/// updated to wire it up. A `#[non_exhaustive]` or builder pattern
/// here would silently let old callers compile after a new channel
/// is added, defeating the point.
///
/// The field ordering here mirrors the priority ordering of the
/// `select!` (control first, model last) so readers can read the
/// struct top-to-bottom and match it to the arms.
pub struct RunTurnArgs {
    /// The persona driving this turn. Used to source the system
    /// prompt, tool filter, and model id.
    pub persona: Persona,

    /// Control channel: Pause / Resume / Abort. T4.2 reserves the
    /// priority seat but does not yet act on the messages (T4.10).
    pub control_rx: mpsc::Receiver<ControlMsg>,

    /// Model-event channel. Fed by the provider adapter task; each
    /// element is either a parsed `AgentEvent` or a transport-layer
    /// `ProviderError` (rate limit, network, etc.).
    pub model_rx: mpsc::Receiver<Result<AgentEvent, ProviderError>>,

    /// Downstream event sink. Every event the loop observes on
    /// `model_rx` is forwarded here (after event_filter gating —
    /// applied in a later task). `Drop` on the sender side means
    /// the consumer has hung up and the loop should exit cleanly.
    pub event_tx: mpsc::Sender<AgentEvent>,
}

/// Run a single agent turn to completion.
///
/// See the module-level docs for priority ordering and exit
/// conditions.
///
/// # Errors
///
/// Returns [`LoopError`] on unrecoverable failures. T4.2 keeps this
/// a closed-set placeholder — later tasks add concrete variants as
/// failure modes come online.
pub async fn run_turn(mut args: RunTurnArgs) -> Result<(), LoopError> {
    // `_` prefix on persona — T4.2 does not yet consume the
    // persona fields. T4.4+ wires `persona.tool_filter` and
    // `persona.model` into tool dispatch and provider requests.
    let _persona = args.persona;

    // `control_open` gates the control-channel select arm. Once
    // the sender drops (recv() returns None), we flip this to
    // false so the biased select does not busy-poll the closed
    // receiver every iteration. Idiomatic tokio pattern; see
    // `tokio::select!` docs §"Disabling branches".
    let mut control_open = true;

    loop {
        tokio::select! {
            biased;

            // Arm 1: control — highest priority so Abort can
            // outrun a model chunk (Product-Lens 500ms Ctrl-C
            // budget in `05-BRAINSTORM.md`).
            ctl = args.control_rx.recv(), if control_open => {
                match ctl {
                    Some(_msg) => {
                        // Pause / Resume / Abort handling lands in
                        // T4.10 (DL-2 buffer-and-replay + 2s grace
                        // on Abort). For T4.2 we accept the message
                        // and continue — the priority seat is held
                        // so the arm ordering is locked in.
                    }
                    None => {
                        // Sender dropped — no further control is
                        // possible. Disable the arm to avoid a hot
                        // loop where biased `select!` keeps picking
                        // a perpetually-None `recv()`.
                        control_open = false;
                    }
                }
            }

            // Arm 2: model — lowest priority. Produces the forward
            // progress of the turn (text + tool calls + lifecycle).
            frame = args.model_rx.recv() => {
                match frame {
                    Some(Ok(ev)) => {
                        // Forward to the caller. If the caller
                        // hung up, treat it as a clean exit — no
                        // point producing events no one reads.
                        if args.event_tx.send(ev).await.is_err() {
                            return Ok(());
                        }
                    }
                    Some(Err(_provider_err)) => {
                        // T4.x later task: convert ProviderError
                        // into AgentEvent::Error and forward. For
                        // T4.2 we exit cleanly — the happy-path
                        // test never exercises this branch.
                        return Ok(());
                    }
                    None => {
                        // Model stream closed — provider finished.
                        // Turn is complete.
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Error surface for [`run_turn`]. T4.2 lands the enum with zero
/// variants so callers' `Result<(), LoopError>` typing is stable
/// even before any concrete failure mode is implemented. Later
/// Wave 4 tasks (T4.4 dispatch errors, T4.6 verifier errors) add
/// variants as failure modes come online.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoopError {}
