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
//! - **Control semantics** (T4.9/T4.10). `Pause` buffer-and-replay
//!   and `Abort` 2s grace are stubbed — T4.2 only reserves the
//!   select-arm seat.
//!
//! ## Completed in prior tasks
//!
//! - **Tool dispatch** (T4.3/T4.4). `ToolCallComplete` flows through
//!   `dispatch()` with the configured registry + tool context.
//! - **Verify gate** (T4.5/T4.6). `TaskComplete { verified: true,
//!   outcome: VerificationOutcome::Pass, .. }` short-circuits the
//!   loop; any other outcome (Pending, Fail, inconsistent flags) is
//!   treated as "continue".

use std::sync::Arc;

use forge_domain::ToolName;
use tokio::sync::mpsc;

use crate::control::ControlMsg;
use crate::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::runtime::dispatcher::dispatch;
use kay_tools::{AgentEvent, ToolCallContext, ToolRegistry, VerificationOutcome};

/// Input bundle for [`run_turn`]. Constructed by callers via
/// struct-literal so later tasks (T4.5+) can add fields — the
/// breakage that comes with new required fields is intentional:
/// adding `input_rx` or `verifier` means every caller must be
/// updated to wire it up. A `#[non_exhaustive]` or builder pattern
/// here would silently let old callers compile after a new channel
/// is added, defeating the point.
///
/// The field ordering here mirrors the priority ordering of the
/// `select!` (control first, model last) so readers can read the
/// struct top-to-bottom and match it to the arms — plus the tool
/// execution surface (`registry`, `tool_ctx`) grouped after the
/// streaming surface so the two responsibilities read separately.
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

    /// Tool registry consulted for every model `ToolCallComplete`.
    /// Shared (`Arc`) so the CLI builds one registry at startup and
    /// reuses it across every turn without paying reconstruction
    /// cost — tools are stateless after registration (D-11 /
    /// `03-CONTEXT.md`). Cloning the `Arc` inside the loop is an
    /// atomic refcount bump, not a tool walk.
    pub registry: Arc<ToolRegistry>,

    /// Per-turn tool-execution context: services handle, stream
    /// sink, image quota, cancellation token, sandbox, verifier.
    /// Owned (not `Arc`) because the cancellation token and the
    /// image budget are turn-scoped — sharing them across turns
    /// would leak abort state and budget accounting. The caller
    /// builds a fresh `ToolCallContext` for each `run_turn` call.
    pub tool_ctx: ToolCallContext,
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
                        // For `ToolCallComplete`, snapshot the three
                        // fields needed by `dispatch` BEFORE forwarding
                        // the event — `AgentEvent` does not derive
                        // `Clone` (ProviderError in the `Error` variant
                        // holds non-Clone types) so we cannot touch
                        // `ev` after `send(ev)` moves it. The snapshot
                        // is cheap: two `String::clone` + one
                        // `serde_json::Value::clone`.
                        let tool_call = match &ev {
                            AgentEvent::ToolCallComplete { id, name, arguments } => {
                                Some((id.clone(), name.clone(), arguments.clone()))
                            }
                            _ => None,
                        };

                        // LOOP-05 verify gate (T4.6 GREEN). A
                        // `TaskComplete` event with BOTH `verified: true`
                        // AND `outcome: Pass { .. }` is the turn-
                        // terminating signal: the verifier has confirmed
                        // the agent's reported success, so the loop must
                        // exit rather than keep draining `model_rx`. Any
                        // other combination (`Pending`, `Fail`, or the
                        // inconsistent `verified: false + Pass` / `true
                        // + Pending`) is treated as "continue" — Phase 8
                        // may add retry/branch behavior on `Fail`, but
                        // the minimum-correct T4.6 semantics are "only
                        // Pass terminates". Requiring the boolean AND
                        // the outcome to agree closes the door on a
                        // buggy tool mis-setting one but not the other.
                        //
                        // Captured BEFORE `send(ev)` moves the event
                        // (same reason as `tool_call` above). The
                        // `matches!` expression is allocation-free; it
                        // compiles down to a tag discriminant compare.
                        let terminates_turn = matches!(
                            &ev,
                            AgentEvent::TaskComplete {
                                verified: true,
                                outcome: VerificationOutcome::Pass { .. },
                                ..
                            }
                        );

                        // Forward first, dispatch second. Order matters:
                        // the UI expects `ToolCallComplete` to appear in
                        // the event stream BEFORE any `ToolOutput` the
                        // tool emits via `ctx.stream_sink`. Reversing
                        // the order would surface "a tool produced
                        // output" before "the model called the tool",
                        // which is nonsense to render in a transcript.
                        //
                        // Forwarding also runs BEFORE the verify-gate
                        // short-circuit: the terminating `TaskComplete`
                        // event MUST reach the sink so the UI can render
                        // the final verdict before the stream closes.
                        // Exiting without forwarding would drop the
                        // signal the whole turn exists to produce.
                        if args.event_tx.send(ev).await.is_err() {
                            // Consumer hung up — no point executing the
                            // tool if no one will see its output.
                            return Ok(());
                        }

                        // Dispatch the tool call (if this was one).
                        // T4.4 GREEN ignores the dispatcher's return
                        // value (`Result<ToolOutput, ToolError>`) —
                        // the tool emits its streamed output via
                        // `ctx.stream_sink` during `invoke`, which is
                        // the surface the loop cares about. Error→
                        // event mapping (e.g. `ToolError::NotFound` →
                        // a surfaced error event) is a Wave-4-later
                        // task; a silent drop here is the minimum
                        // T4.3-green semantics.
                        if let Some((id, name, arguments)) = tool_call {
                            let _ = dispatch(
                                &args.registry,
                                &ToolName::new(&name),
                                arguments,
                                &args.tool_ctx,
                                &id,
                            )
                            .await;
                        }

                        // Verify gate closes the turn. Placed AFTER
                        // forward+dispatch so a verified `TaskComplete`
                        // that somehow carried a `call_id` referencing
                        // a tool still lets that tool run (degenerate
                        // case — the event is emitted by `task_complete`
                        // itself which has no side-effect dispatch
                        // target on the registry — but the ordering is
                        // what makes the loop's behavior composable:
                        // "always forward + dispatch, then decide
                        // whether to continue".
                        if terminates_turn {
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
