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
//! - `ControlMsg::Abort` → emits `AgentEvent::Aborted { reason:
//!   "user_abort" }` and returns `Ok(())` (T4.10).
//!
//! ## Not yet handled (tracked in PLAN.md later waves)
//!
//! - **2s grace period on Abort**. `05-BRAINSTORM.md` §Product-Lens
//!   specifies a cooperative-then-force two-stage cancel, but the
//!   in-flight tool registry that lets the loop "wait for tools to
//!   settle" lands in a later wave. T4.10 ships the minimum-correct
//!   shape: observe Abort, signal the UI, exit immediately.
//! - **Dispatch + verify gate during Pause.** A `ToolCallComplete`
//!   that arrives while paused is buffered but its dispatch is
//!   deferred until Resume replays it — and on replay, dispatch is
//!   NOT re-run (the buffered event is a historical record, not a
//!   live invocation). A future wave can add "defer-then-replay"
//!   dispatch semantics; T4.10 tests exercise only `TextDelta`
//!   during Pause so the simpler semantic is sufficient.
//!
//! ## Completed in prior tasks
//!
//! - **Tool dispatch** (T4.3/T4.4). `ToolCallComplete` flows through
//!   `dispatch()` with the configured registry + tool context.
//! - **Verify gate** (T4.5/T4.6). `TaskComplete { verified: true,
//!   outcome: VerificationOutcome::Pass, .. }` short-circuits the
//!   loop; any other outcome (Pending, Fail, inconsistent flags) is
//!   treated as "continue".
//! - **Pause/Resume/Abort** (T4.9/T4.10). LOOP-06 buffer-and-replay
//!   pause semantics backed by a `VecDeque<AgentEvent>`; Abort emits
//!   exactly one `AgentEvent::Aborted { reason: "user_abort" }` and
//!   exits `Ok(())` — idempotent under double-Abort because the
//!   second message is never received after the first exit drops
//!   `control_rx`.

use std::collections::VecDeque;
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

    /// Context engine consulted at turn start (before the event loop).
    /// NoOpContextEngine is the default until Phase 8 wires real retrieval.
    pub context_engine: Arc<dyn kay_context::engine::ContextEngine>,

    /// Token budget for context assembly this turn.
    pub context_budget: kay_context::budget::ContextBudget,

    /// User's prompt for this turn, passed to context_engine.retrieve().
    pub initial_prompt: String,

    /// Verifier configuration for the outer re-work loop.
    /// Drives max_retries and cost_ceiling enforcement.
    pub verifier_config: kay_verifier::VerifierConfig,
}

/// Reason string carried on [`AgentEvent::Aborted`] when the loop
/// exits because it received a [`ControlMsg::Abort`] (Ctrl-C, menu
/// cancel, etc.). Phase 9 frontends match on this literal to render
/// "user cancelled" vs. future values like `"policy_violation"` or
/// `"context_exhausted"` — locking the string in one place keeps
/// the JSONL wire contract grep-visible and prevents drift if the
/// Aborted branch ever needs an additional emit site.
const ABORT_REASON_USER: &str = "user_abort";

/// What the control arm tells the outer `run_turn` loop to do after
/// handling a [`ControlMsg`]. Pulling this out as a named enum
/// instead of encoding the three outcomes with a `Result<bool, ()>`
/// or three `&mut bool` parameters makes every call site a one-line
/// `match` whose branches map 1:1 to the caller's top-level actions.
enum ControlOutcome {
    /// Continue to the next `select!` iteration. Pause and Resume
    /// both land here — they mutate state but do not change the
    /// loop's lifetime.
    Continue,
    /// Abort observed; exit `run_turn` cleanly with `Ok(())`.
    Exit,
    /// The control sender has been dropped. The outer loop should
    /// disable the control arm (flip `control_open = false`) so
    /// biased `select!` does not busy-poll a perpetually-None
    /// receiver.
    Closed,
}

/// Outcome of processing one model-event frame. Same rationale as
/// [`ControlOutcome`]: named outcomes give the outer `run_turn`
/// loop a clean dispatch shape, and the single "Exit" variant
/// captures both "consumer hung up on event_tx" and "verify gate
/// terminated the turn" without the caller needing to know which.
enum ModelOutcome {
    /// Continue to the next `select!` iteration.
    Continue,
    /// Exit `run_turn` cleanly with `Ok(())`. Triggered by a
    /// consumer hang-up on `event_tx` OR a verified `TaskComplete
    /// { Pass }` observed via the LOOP-05 gate.
    Exit,
}

/// Handle one [`ControlMsg`] (or the sender-dropped sentinel). Splits
/// the ~80-line nested match out of the `select!` body so `run_turn`
/// reads as orchestration only.
///
/// Mutates `paused` and `pause_buffer` in place because the state
/// needs to survive across loop iterations. Passing both as `&mut`
/// through an async boundary is safe: the helper is awaited
/// synchronously from within the outer `select!` arm, so there is
/// no aliasing risk with the model arm (biased `select!` runs at
/// most one arm body per iteration).
async fn handle_control(
    msg: Option<ControlMsg>,
    paused: &mut bool,
    pause_buffer: &mut VecDeque<AgentEvent>,
    event_tx: &mpsc::Sender<AgentEvent>,
) -> ControlOutcome {
    match msg {
        Some(ControlMsg::Pause) => {
            // Emit a single Paused event so the UI can show the
            // paused state, then flip the buffering flag. Order
            // matters: emitting Paused AFTER setting `paused=true`
            // would race a model frame arriving between the
            // flag-flip and the send — the model frame would be
            // buffered before the UI knew the agent was paused,
            // surfacing as "events arrive out of nowhere on resume".
            //
            // On event_tx hangup we exit cleanly: no consumer means
            // nothing to pause FOR.
            if event_tx.send(AgentEvent::Paused).await.is_err() {
                return ControlOutcome::Exit;
            }
            *paused = true;
            ControlOutcome::Continue
        }
        Some(ControlMsg::Resume) => {
            // Drain the buffer in FIFO order. `pop_front` loop is
            // O(n) where n is the buffer depth; while draining,
            // NEW model events continue to arrive on `model_rx`
            // but will not be selected over this control-arm body —
            // inside a `select!` arm, execution is synchronous
            // until the arm returns, so the flush is atomic w.r.t.
            // the outer select.
            //
            // `*paused = false` goes AFTER the flush so any
            // interleaving caused by `send().await` yielding to the
            // runtime cannot race a model frame into the "post-
            // resume" path before the queued ones are out.
            while let Some(buffered) = pause_buffer.pop_front() {
                if event_tx.send(buffered).await.is_err() {
                    return ControlOutcome::Exit;
                }
            }
            *paused = false;
            ControlOutcome::Continue
        }
        Some(ControlMsg::Abort) => {
            // Emit Aborted once and signal Exit. The second Abort
            // in the idempotency test lands here only if it races
            // the receiver drop — once `run_turn` returns, the
            // outer caller drops `args.control_rx`, so the second
            // sender write either races the drop (Ok) or fails
            // (SendError). Either way the second message is never
            // received → never re-emits `Aborted`.
            //
            // No 2s grace period here: that's the shape a future
            // wave adds once there's an in-flight tool registry to
            // drain. The T4.10 MVP semantics are the minimum
            // correct: observe Abort, tell the UI, exit.
            //
            // `let _ =` on the send: if the consumer has already
            // hung up, we still want to exit cleanly. The Aborted
            // event is a courtesy signal, not a must-deliver.
            // Propagating a SendError here would turn a benign
            // hangup into a loop error for no user benefit.
            let _ = event_tx
                .send(AgentEvent::Aborted { reason: ABORT_REASON_USER.into() })
                .await;
            ControlOutcome::Exit
        }
        None => ControlOutcome::Closed,
    }
}

/// Handle one well-formed [`AgentEvent`] pulled off `model_rx`.
/// Encapsulates the pause gate, the tool-call dispatch, and the
/// LOOP-05 verify gate into a single reusable unit — `run_turn`
/// calls this from the model-arm with the already-unwrapped event
/// value.
///
/// Takes `ev` by value because `AgentEvent` does not derive `Clone`
/// (the `Error` variant holds non-Clone `ProviderError` types) and
/// this function ultimately forwards it via `send(ev)` which moves
/// it. Any field-level snapshots we need (tool call, verify gate
/// discriminant) happen at the top before the move.
async fn handle_model_event(
    ev: AgentEvent,
    paused: bool,
    pause_buffer: &mut VecDeque<AgentEvent>,
    event_tx: &mpsc::Sender<AgentEvent>,
    registry: &Arc<ToolRegistry>,
    tool_ctx: &ToolCallContext,
) -> ModelOutcome {
    // ── Pause gate (LOOP-06 / DL-2) ─────────────
    // While paused, queue the event for FIFO replay on Resume.
    // Skip forward, dispatch, and the verify gate: buffering BOTH
    // the event AND its side effects is the only shape that makes
    // "pause" mean what the user thinks it means ("the agent stops
    // doing things"). A paused ToolCallComplete whose dispatch
    // fires immediately would mutate files AFTER the user said
    // pause — a worse UX than deferring the whole event.
    //
    // Known limitation: a verified `TaskComplete { Pass }` that
    // arrives during Pause will NOT terminate the loop until
    // Resume flushes the buffer and the NEXT model frame arrives
    // to re-enter this arm. A future wave can re-check the
    // terminates-turn flag on each buffer pop; for Wave 4 MVP the
    // simpler "only forwarded events can trigger the verify gate"
    // semantic is sufficient (the tests exercise only TextDelta
    // during Pause).
    if paused {
        pause_buffer.push_back(ev);
        return ModelOutcome::Continue;
    }

    // For `ToolCallComplete`, snapshot the three fields needed by
    // `dispatch` BEFORE forwarding the event. `AgentEvent` is not
    // `Clone`, so we cannot touch `ev` after `send(ev)` moves it.
    // The snapshot is cheap: two `String::clone` + one
    // `serde_json::Value::clone`.
    let tool_call = match &ev {
        AgentEvent::ToolCallComplete { id, name, arguments } => {
            Some((id.clone(), name.clone(), arguments.clone()))
        }
        _ => None,
    };

    // LOOP-05 verify gate (T4.6 GREEN). A `TaskComplete` event
    // with BOTH `verified: true` AND `outcome: Pass { .. }` is the
    // turn-terminating signal: the verifier has confirmed the
    // agent's reported success, so the loop must exit rather than
    // keep draining `model_rx`. Any other combination (`Pending`,
    // `Fail`, or the inconsistent `verified: false + Pass` / `true
    // + Pending`) is treated as "continue" — Phase 8 may add
    // retry/branch behavior on `Fail`, but the minimum-correct
    // T4.6 semantics are "only Pass terminates". Requiring the
    // boolean AND the outcome to agree closes the door on a buggy
    // tool mis-setting one but not the other.
    //
    // Captured BEFORE `send(ev)` moves the event (same reason as
    // `tool_call` above). The `matches!` expression is allocation-
    // free; it compiles down to a tag discriminant compare.
    let terminates_turn = matches!(
        &ev,
        AgentEvent::TaskComplete {
            verified: true,
            outcome: VerificationOutcome::Pass { .. },
            ..
        }
    );

    // Forward first, dispatch second. Order matters: the UI expects
    // `ToolCallComplete` to appear in the event stream BEFORE any
    // `ToolOutput` the tool emits via `ctx.stream_sink`. Reversing
    // the order would surface "a tool produced output" before "the
    // model called the tool", which is nonsense to render in a
    // transcript.
    //
    // Forwarding also runs BEFORE the verify-gate short-circuit:
    // the terminating `TaskComplete` event MUST reach the sink so
    // the UI can render the final verdict before the stream closes.
    // Exiting without forwarding would drop the signal the whole
    // turn exists to produce.
    if event_tx.send(ev).await.is_err() {
        // Consumer hung up — no point executing the tool if no one
        // will see its output.
        return ModelOutcome::Exit;
    }

    // Dispatch the tool call (if this was one). T4.4 GREEN ignores
    // the dispatcher's return value (`Result<ToolOutput,
    // ToolError>`) — the tool emits its streamed output via
    // `ctx.stream_sink` during `invoke`, which is the surface the
    // loop cares about. Error→event mapping (e.g. `ToolError::
    // NotFound` → a surfaced error event) is a Wave-4-later task;
    // a silent drop here is the minimum T4.3-green semantics.
    if let Some((id, name, arguments)) = tool_call {
        let _ = dispatch(registry, &ToolName::new(&name), arguments, tool_ctx, &id).await;
    }

    // Verify gate closes the turn. Placed AFTER forward+dispatch so
    // a verified `TaskComplete` that somehow carried a `call_id`
    // referencing a tool still lets that tool run (degenerate case —
    // the event is emitted by `task_complete` itself which has no
    // side-effect dispatch target on the registry — but the
    // ordering is what makes the loop's behavior composable:
    // "always forward + dispatch, then decide whether to continue".
    if terminates_turn {
        return ModelOutcome::Exit;
    }

    ModelOutcome::Continue
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
    // `_` prefix on persona — Wave 4 scope does not consume the
    // persona fields. Wave 7 wires `persona.tool_filter` and
    // `persona.model` into tool dispatch and the OpenRouter client.
    let _persona = args.persona;

    // `control_open` gates the control-channel select arm. Once the
    // sender drops (recv() returns None), we flip this to false so
    // the biased select does not busy-poll the closed receiver every
    // iteration. Idiomatic tokio pattern; see `tokio::select!` docs
    // §"Disabling branches".
    let mut control_open = true;

    // LOOP-06 control state (T4.10 GREEN).
    //
    // `paused`: while true, the model arm pushes incoming events
    // into `pause_buffer` instead of forwarding them. A `VecDeque`
    // is the natural fit: `push_back` on buffering, `pop_front`
    // during Resume gives FIFO replay with O(1) amortized per op.
    // `Vec` + `drain(..)` would also work but loses the explicit
    // front/back vocabulary. No upper bound on the buffer —
    // `05-BRAINSTORM.md` §DL-2 locks this as "unbounded; the user
    // chose to pause and the provider's own capacity-32 mpsc
    // provides natural backpressure upstream". A future wave can
    // add a soft cap + drop-oldest policy if an adversarial
    // provider is suspected.
    //
    // `pause_buffer`: only AgentEvents, never the raw Result from
    // `model_rx`. Provider errors during Pause still exit the loop
    // (current Wave 4 semantics) because a transport failure cannot
    // be "un-paused". When a later wave maps ProviderError →
    // AgentEvent::Error, the error-events can also flow through
    // this buffer.
    let mut paused: bool = false;
    let mut pause_buffer: VecDeque<AgentEvent> = VecDeque::new();

    // Context retrieval at turn start (Phase 7 DL-9).
    // _ctx_packet unused in Phase 7 — Phase 8+ injects into OpenRouter request.
    #[allow(unused)]
    let _ctx_packet = args
        .context_engine
        .retrieve(&args.initial_prompt, &args.registry.schemas())
        .await
        .unwrap_or_default();

    loop {
        tokio::select! {
            biased;

            // Arm 1: control — highest priority so Abort can outrun
            // a model chunk (Product-Lens 500ms Ctrl-C budget in
            // `05-BRAINSTORM.md`).
            ctl = args.control_rx.recv(), if control_open => {
                match handle_control(
                    ctl,
                    &mut paused,
                    &mut pause_buffer,
                    &args.event_tx,
                ).await {
                    ControlOutcome::Continue => {}
                    ControlOutcome::Exit => return Ok(()),
                    ControlOutcome::Closed => control_open = false,
                }
            }

            // Arm 2: model — lowest priority. Produces the forward
            // progress of the turn (text + tool calls + lifecycle).
            frame = args.model_rx.recv() => {
                match frame {
                    Some(Ok(ev)) => {
                        match handle_model_event(
                            ev,
                            paused,
                            &mut pause_buffer,
                            &args.event_tx,
                            &args.registry,
                            &args.tool_ctx,
                        ).await {
                            ModelOutcome::Continue => {}
                            ModelOutcome::Exit => return Ok(()),
                        }
                    }
                    Some(Err(_provider_err)) => {
                        // A later wave will convert ProviderError
                        // into AgentEvent::Error and forward. For
                        // Wave 4 we exit cleanly — the happy-path
                        // tests never exercise this branch.
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

/// Outcome of a completed agent turn, returned from run_turn.
/// VerificationFailed means the re-work loop exhausted max_retries.
#[derive(Debug, PartialEq, Eq)]
pub enum TurnResult {
    /// All critics passed (or verifier disabled).
    Verified,
    /// Re-work loop hit max_retries; turn did not pass verification.
    VerificationFailed,
    /// Turn was cancelled via control channel.
    Aborted,
    /// Turn completed without reaching task_complete (model stopped itself).
    Completed,
    /// Pass after one or more retry cycles.
    PassAfterRetry,
}

/// Error surface for [`run_turn`]. T4.2 lands the enum with zero
/// variants so callers' `Result<(), LoopError>` typing is stable
/// even before any concrete failure mode is implemented. Later
/// waves (provider-error mapping, dispatch errors) add variants as
/// failure modes come online.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoopError {}

/// Run a single agent turn with bounded retry re-work loop.
///
/// This function wraps `run_turn` with an outer retry loop that:
/// - Checks for `TaskComplete` with `verified: true` and `Pass` outcome → returns `TurnResult::Verified`
/// - On `Fail` outcome: injects feedback as user message and retries up to `max_retries` times
/// - On max_retries exhausted: emits `VerifierDisabled { max_retries_exhausted }` and returns `TurnResult::VerificationFailed`
///
/// # Errors
///
/// Returns [`LoopError`] on unrecoverable failures.
pub async fn run_with_rework(args: RunTurnArgs) -> Result<TurnResult, LoopError> {
    todo!("W-5 RED: run_with_rework not yet implemented")
}
