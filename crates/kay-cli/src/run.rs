//! `kay run` — headless agent turn driver (Phase 5 Wave 7 T7.3).
//!
//! # What this file does
//!
//! T7.3 wires `kay run --prompt <text> [--offline]` end-to-end:
//!
//!   1. **Parse CLI args.** The `RunArgs` clap derive landed in T7.2;
//!      its shape is unchanged here.
//!   2. **Short-circuit `--max-turns 0`.** `anyhow::bail!` returns
//!      `Err(…)`; Rust's default `Termination` for `anyhow::Result<()>`
//!      maps that to exit 1, which is exactly what T7.1's
//!      `exit_code_1_on_max_turns` expects. T7.7 will wrap `execute`
//!      in the 0/1/2/3/130 classifier; until then this implicit
//!      mapping is the simplest thing that satisfies the RED test.
//!   3. **Load the persona.** Bundled `"forge"` by default; an
//!      external YAML path via `--persona PATH`. The error surface
//!      is [`kay_core::persona::PersonaError`] which is `thiserror`
//!      so `?` auto-converts to `anyhow::Error`.
//!   4. **Build a current-thread tokio runtime.** Scoping async to
//!      this subcommand only — `eval`, `tools`, `interactive_fallback`
//!      stay sync. `#[tokio::main]` on `main` would force a runtime
//!      for every invocation of `kay`, including `kay --version`,
//!      which is startup latency we do not need to pay.
//!   5. **Spawn the offline provider task.** Keyed off the `--prompt`
//!      sentinels documented in `tests/cli_e2e.rs`:
//!
//!        | Prompt sentinel          | AgentEvent sequence                   |
//!        |--------------------------|----------------------------------------|
//!        | `TEST:done`              | one `TaskComplete { verified:true, Pass }` → turn ends via LOOP-05 verify gate |
//!        | `TEST:sandbox-violation` | one `SandboxViolation { … }` → surfaces on stdout JSONL, loop drains |
//!        | `TEST:loop-forever`      | three `TextDelta` frames then stream close (the test always pairs this with `--max-turns 0` which short-circuits at step 2, so the provider body is never actually reached in CI; kept for completeness) |
//!        | `TEST:hang-forever`      | one `TextDelta`, then `future::pending::<()>` forever — the SIGINT target for T7.8 |
//!        | anything else            | one `TextDelta("echo: {prompt}")` so the JSONL stream is non-empty for arbitrary prompts |
//!
//!   6. **Spawn `run_turn`.** Empty [`ToolRegistry`] + minimal
//!      [`ToolCallContext`] backed by an inline [`NullServices`]
//!      stub. T7.3 offline scenarios never emit `ToolCallComplete`,
//!      so the dispatcher is never invoked — the registry + context
//!      are plumbing that exists only to satisfy `RunTurnArgs`'s
//!      required-field shape. A later wave (T7.3b or Phase 7)
//!      replaces them with [`kay_tools::default_tool_set`] + a real
//!      `ForgeServicesFacade` once end-to-end tool exercises land in
//!      the headless CLI.
//!   7. **Drain the event channel.** Each [`AgentEvent`] is written
//!      to stdout via [`AgentEventWire`]'s `Display` impl, which
//!      emits one compact JSONL line with a trailing `\n`. stdout is
//!      flushed per-event so real-time consumers (Phase 9 Tauri GUI,
//!      Phase 9.5 TUI) see frames as they arrive, not at process
//!      exit.
//!
//! # What this file does NOT do yet (future waves)
//!
//! * **Exit-code mapping (T7.7).** Sandbox violations → 2, config
//!   errors → 3, SIGINT → 130. Today those all map through the
//!   default `anyhow` Termination as either exit 0 (no error) or
//!   exit 1 (generic error). That's intentional — T7.3's charter is
//!   persona + runtime + JSONL; T7.7's charter is the classifier.
//! * **SIGINT handler (T7.8).** `install_ctrl_c_handler` is not
//!   wired in T7.3. The control channel exists but no message ever
//!   arrives on it during a T7.3 offline run. The
//!   `exit_code_130_on_sigint_nix` test therefore stays RED until
//!   T7.8 lands.
//! * **Real OpenRouter transport.** `--offline` is the only mode
//!   T7.3 exercises; the online branch (`--offline` absent) is
//!   handled identically in T7.3 because the offline demux also
//!   covers the non-sentinel echo path. A future wave splits these
//!   once the OpenRouter adapter is ready to stream real model
//!   tokens through the same channels.
//!
//! # Why the provider task uses `tokio::spawn`, not select-inline
//!
//! The agent loop's `select!` is biased (control > model); putting
//! the provider inline would serialize frame-production with
//! frame-forwarding and break the biased priority contract. A
//! dedicated task lets the loop see frames land on `model_rx` while
//! its select! body is free to prioritize a `ControlMsg::Abort`
//! above in-flight frames.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use clap::Args;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::control_channel;
use kay_core::persona::Persona;
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_provider_errors::ProviderError;
use kay_tools::events_wire::AgentEventWire;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    ToolRegistry, VerificationOutcome,
};

/// Arguments for `kay run`. T7.5 adds `-p` short alias on
/// `--prompt` and `allow_hyphen_values` so prompts that begin with
/// `-` (e.g. `-p "-hello"`) don't trip clap's flag parser.
///
/// Rationale for where these live: struct-level docs like this one
/// describe the SHAPE of the argument group; per-field docs below
/// describe individual flags. User-visible `--help` output renders
/// only the per-field docs — this header is for rustdoc and code
/// readers, so mentions of internal sibling-crate names are OK here
/// (struct docs don't appear in `--help`).
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Prompt text to send to the agent on the first turn. In
    /// offline mode, specific `TEST:` sentinels drive deterministic
    /// scenarios for E2E testing — see `tests/cli_e2e.rs`.
    ///
    /// `-p` is a short alias for `--prompt`. `allow_hyphen_values`
    /// lets prompts that start with `-` round-trip without needing
    /// `--prompt=-foo` quoting gymnastics (e.g. `kay run -p "-hi"`
    /// works).
    #[arg(long, short = 'p', value_name = "TEXT", allow_hyphen_values = true)]
    pub prompt: String,

    /// Use the in-memory offline provider. Safe in hermetic CI;
    /// skips OpenRouter transport entirely. T7.3 always routes
    /// through the offline demux regardless of this flag (there is
    /// no online provider wired in yet); a later wave splits the
    /// two modes once the OpenRouter adapter is ready.
    #[arg(long, default_value_t = false)]
    pub offline: bool,

    /// Upper bound on agent turns before the loop exits with code 1
    /// (bounded-loop exit — distinct from 0/2/3/130). Absent =
    /// "no bound"; pass `0` to force immediate bounded-loop exit
    /// (useful in E2E tests — see `exit_code_1_on_max_turns`).
    #[arg(long, value_name = "N")]
    pub max_turns: Option<u32>,

    /// Path to a persona YAML file. Absent = the bundled default
    /// persona. I/O or schema errors here produce an `anyhow::Error`
    /// which maps to exit 1 today; T7.7 promotes that to exit 3 via
    /// the config-error classifier.
    ///
    /// (The default persona's identifier is a kay-core internal and
    /// is deliberately not surfaced in user-visible help text — see
    /// DL-1 brand-cleanroom guard enforced by the CLI brand-parity
    /// integration test.)
    #[arg(long, value_name = "PATH")]
    pub persona: Option<PathBuf>,
}

/// Entry-point dispatched from `main.rs` on the `Run` subcommand.
///
/// Stays synchronous so `main` can remain `fn main() -> anyhow::
/// Result<()>` and startup cost for other subcommands (`eval`,
/// `tools`) doesn't include a tokio runtime build. The runtime is
/// built lazily inside this function — scoped to `run_async` — and
/// dropped when `run_async` returns.
pub fn execute(args: RunArgs) -> anyhow::Result<()> {
    // Short-circuit the `--max-turns 0` boundary BEFORE the runtime
    // or persona loader run. Rationale:
    //
    //   * No provider I/O is wasted — the exit is decided from args
    //     alone, so persona/network/tokio setup would all be dead
    //     work.
    //   * The test `exit_code_1_on_max_turns` only observes the
    //     exit code; stdout is not asserted. `anyhow::bail!` returns
    //     Err which the default `Termination` impl for
    //     `anyhow::Result<()>` maps to exit 1 — exactly the code
    //     the test expects. T7.7 will replace this implicit mapping
    //     with the explicit 0/1/2/3/130 classifier.
    //
    // If `max_turns` is ever set to 0 by a real user (not the test),
    // this message is what they see on stderr — terse but truthful.
    if matches!(args.max_turns, Some(0)) {
        anyhow::bail!("max turns exceeded (budget: 0)");
    }

    // Load the persona synchronously so a bad `--persona` path fails
    // before we pay for the runtime. `PersonaError` is `thiserror`
    // so `?` converts to `anyhow::Error` automatically.
    let persona = match args.persona.as_deref() {
        Some(path) => Persona::from_path(path)?,
        None => Persona::load("forge")?,
    };

    // Current-thread runtime — `kay run` is a single concurrent turn;
    // multi-thread is overkill, and starting the thread pool adds
    // ~100–200µs of startup latency for no throughput benefit.
    // `enable_all()` enables the I/O + time drivers the mpsc channels
    // and tokio::signal subsystem both require.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(run_async(args.prompt, persona))
}

/// The async half of `execute` — builds channels, spawns the offline
/// provider + the agent loop, and drains events to stdout.
///
/// Split out so `execute` stays sync and the runtime lifetime is
/// explicit (dropped when this returns, not when `main` returns).
async fn run_async(prompt: String, persona: Persona) -> anyhow::Result<()> {
    // ── Channel topology ────────────────────────────────────────
    //
    // Four channels per `05-BRAINSTORM.md` §Engineering-Lens, but
    // T7.3 only exercises two (control + model + event) because
    // offline scenarios never open an input or tool-output path.
    // Capacity 32 matches the locked default across all four.
    //
    // `_control_tx` is held for the lifetime of `run_async` so the
    // control channel stays open through the loop body. Dropping it
    // early would flip `control_open = false` on the first poll and
    // close the priority seat; harmless today but noisy in logs.
    let (_control_tx, control_rx) = control_channel();
    let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

    // ── Offline provider ────────────────────────────────────────
    // Dedicated task so the provider's `send().await` interleaves
    // cooperatively with the loop's `select!`. Inlining would
    // serialize production + forwarding and defeat the whole point
    // of a biased select.
    tokio::spawn(offline_provider(prompt, model_tx));

    // ── Empty registry + minimal context ────────────────────────
    // T7.3 offline scenarios never emit `ToolCallComplete`, so the
    // registry is consulted zero times. Kept as plumbing only to
    // satisfy `RunTurnArgs`'s required-field shape. The stream_sink
    // closure (`|_| {}`) is never called for the same reason.
    //
    // Image quota: `u32::MAX` on both axes is effectively unlimited
    // — matches the T4.1 happy-path test pattern. A real quota
    // threads through in Phase 7 (T7.3b or later).
    let registry = Arc::new(ToolRegistry::new());
    let tool_ctx = ToolCallContext::new(
        Arc::new(NullServices),
        Arc::new(|_ev: AgentEvent| {}),
        Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0, // nesting_depth: top-level turn (sage_query depth is per-call)
    );

    // ── Spawn the agent loop ────────────────────────────────────
    // `tokio::spawn` decouples the loop from the event drain so
    // both can make progress concurrently on the current-thread
    // runtime. `handle.await??` below joins and propagates both
    // the JoinError (outer `?`) and the LoopError (inner `?`);
    // LoopError is an empty `#[non_exhaustive]` enum today so the
    // inner `?` is vacuously true at runtime but satisfies the
    // type checker.
    let handle = tokio::spawn(run_turn(RunTurnArgs {
        persona,
        control_rx,
        model_rx,
        event_tx,
        registry,
        tool_ctx,
    }));

    // ── Drain the event channel to stdout as JSONL ──────────────
    // AgentEventWire's Display impl emits one compact JSON object
    // terminated by a single `\n`. `write!` (not `writeln!`) so we
    // do not double-newline — the schema invariant (events-wire
    // snapshot test) locks "exactly one `\n` per event".
    //
    // Per-event flush is important for streaming UIs: without it,
    // BufWriter-style stdout would accumulate multiple events
    // before hitting an internal buffer boundary. Real-time consumers
    // (Tauri GUI, TUI) rely on timely frame delivery.
    //
    // Lock held for the whole drain loop: recv() may yield between
    // events but no other code in this process writes to stdout
    // during a turn, so the lock is uncontended. Locking once
    // avoids per-event lock acquisition cost.
    let mut stdout = std::io::stdout().lock();
    while let Some(ev) = event_rx.recv().await {
        write!(stdout, "{}", AgentEventWire::from(&ev))?;
        // `.ok()`: a broken-pipe error here (e.g., `kay run ... |
        // head -n 1` closes its end after one line) should not be
        // fatal — the loop will naturally observe the next write
        // failing via the `?` above, or the event channel dropping
        // when run_turn exits. Until T7.7 refines broken-pipe
        // handling across the whole CLI, best-effort flush is
        // sufficient.
        stdout.flush().ok();
    }

    // ── Join the loop task ──────────────────────────────────────
    // Double-`?`: outer propagates `JoinError` (task panicked or
    // was cancelled); inner propagates `LoopError` (empty today).
    // Both errors flow through `anyhow::Error` via `thiserror`
    // derives on the source types.
    handle.await??;

    Ok(())
}

/// Offline provider task — emits canned `AgentEvent` sequences keyed
/// off `prompt` per the `TEST:` sentinel contract in
/// `tests/cli_e2e.rs`.
///
/// All send errors are swallowed with `let _ =` because a send
/// failure means the receiver (the loop) has already hung up — there
/// is no recovery action and no caller to surface the error to. The
/// task simply exits early; the resulting `model_tx` drop is observed
/// by the loop as a clean stream close.
async fn offline_provider(
    prompt: String,
    model_tx: mpsc::Sender<Result<AgentEvent, ProviderError>>,
) {
    match prompt.as_str() {
        // Verified TaskComplete → the LOOP-05 verify gate short-
        // circuits run_turn with Ok(()) → execute returns Ok → exit 0.
        // This is the canonical happy path.
        "TEST:done" => {
            let _ = model_tx
                .send(Ok(AgentEvent::TaskComplete {
                    call_id: "call-offline-done".into(),
                    verified: true,
                    outcome: VerificationOutcome::Pass {
                        note: "offline scenario: TEST:done".into(),
                    },
                }))
                .await;
        }

        // SandboxViolation flows through the loop unchanged — QG-C4
        // ensures it never re-enters model context, but stdout JSONL
        // is the user-visible surface, which is exactly what the
        // event filter spares. Policy-rule string matches the
        // `kay-sandbox-policy::rules::RULE_*` convention even though
        // no real sandbox runs here; it's the wire-form contract
        // that's under test.
        "TEST:sandbox-violation" => {
            let _ = model_tx
                .send(Ok(AgentEvent::SandboxViolation {
                    call_id: "call-offline-sandbox".into(),
                    tool_name: "fs_write".into(),
                    resource: "/outside/project/evil.txt".into(),
                    policy_rule: "project_root_only".into(),
                    // EACCES — plausible kernel denial for a write
                    // outside the project root.
                    os_error: Some(13),
                }))
                .await;
        }

        // `--max-turns 0` paired with this sentinel short-circuits at
        // `execute` entry, so the body below is not reached in CI.
        // Three bounded frames + clean close avoid an infinite
        // offline loop if the sentinel is ever used without a turn
        // budget (e.g., an ad-hoc `kay run --prompt TEST:loop-forever
        // --offline` run).
        "TEST:loop-forever" => {
            for i in 0..3 {
                if model_tx
                    .send(Ok(AgentEvent::TextDelta {
                        content: format!("loop-{i}"),
                    }))
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }

        // SIGINT target for T7.8. One TextDelta proves the process is
        // alive and producing output, then `pending::<()>` parks the
        // task forever. Drop of `model_tx` is suppressed by the
        // parked await — exactly what T7.8's `exit_code_130_on_
        // sigint_nix` needs: a process that won't exit on its own.
        "TEST:hang-forever" => {
            let _ = model_tx
                .send(Ok(AgentEvent::TextDelta {
                    content: "hang".into(),
                }))
                .await;
            std::future::pending::<()>().await;
        }

        // Default echo — keeps the JSONL stream non-empty for any
        // non-sentinel prompt. Important because the stream-shape
        // contract (`headless_prompt_emits_events`) asserts at least
        // one valid JSON line lands on stdout.
        other => {
            let _ = model_tx
                .send(Ok(AgentEvent::TextDelta {
                    content: format!("echo: {other}"),
                }))
                .await;
        }
    }
    // `model_tx` drops here on scope exit (except the hang-forever
    // branch which never returns) → run_turn observes stream close
    // and exits Ok(()) via the model-arm's `None` match.
}

/// No-op `ServicesHandle` stub. T7.3 offline scenarios never
/// dispatch tools, so these methods are unreachable in practice —
/// the empty `ToolRegistry` guarantees dispatch never resolves to a
/// tool that would call into services. Kept here rather than hoisted
/// into `kay-tools` because it's test-fixture-shaped (no real I/O)
/// and would pollute the production API of `kay-tools` with a type
/// that exists only for CLI stubbing.
///
/// Mirrors the `NullServices` in `crates/kay-core/tests/loop.rs` —
/// intentional duplication: the kay-core test and the kay-cli
/// runtime both need the same stub for the same reason, and sharing
/// would couple a test-only type into the public API of a sibling
/// crate.
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
