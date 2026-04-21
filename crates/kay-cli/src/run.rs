//! `kay run` — headless agent turn driver (Phase 5 Wave 7 T7.2 stub).
//!
//! # Scope of this stub
//!
//! T7.2 lands the clap argument surface (`RunArgs`) and a minimal
//! JSONL stub that emits one well-formed AgentEvent-shaped line and
//! exits 0. This is enough to turn `headless_prompt_emits_events`
//! (T7.1) green from the structural side — the contract under test
//! is: (a) `kay run --prompt X --offline` is a recognized subcommand,
//! (b) stdout is valid JSONL, one JSON object per line.
//!
//! T7.3 replaces `execute_stub` with real `kay_core::run_turn`
//! wiring + offline provider dispatch. At that point this module
//! grows an `offline::provider()` builder, a JSONL serializer on
//! top of `AgentEventWire` (from kay-tools), and the scenario
//! demultiplexer keyed off `RunArgs::prompt` (`TEST:done`,
//! `TEST:loop-forever`, etc. — see `tests/cli_e2e.rs` fixture
//! contract).
//!
//! T7.7 (EXIT-CODES) later wraps `execute` in the exit-code mapping
//! layer so `Err(ConfigError)` → exit 3, `Err(SandboxViolation)` →
//! exit 2, etc.
//!
//! # Args philosophy
//!
//! Every flag is optional-with-sensible-default EXCEPT `--prompt`,
//! which is required. Rationale: a `kay run` invocation with no
//! prompt is always a user error (nothing to say to the agent);
//! making it required surfaces that at the clap layer (exit 2,
//! built-in clap usage message) rather than at runtime.

use clap::Args;
use std::path::PathBuf;

/// Arguments for `kay run`. Every field has a `long` attribute so
/// they accept both `--prompt` and shorter forms — long-only for
/// now (no `-p`) to keep the flag namespace open; short aliases can
/// land in a later wave once the command surface is stable.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Prompt text to send to the agent on the first turn. In
    /// offline mode, specific `TEST:` sentinels drive deterministic
    /// scenarios for E2E testing — see `tests/cli_e2e.rs`.
    #[arg(long, value_name = "TEXT")]
    pub prompt: String,

    /// Use the in-memory offline provider. Safe in hermetic CI;
    /// skips OpenRouter transport entirely. T7.3 wires this to the
    /// scenario demultiplexer driven by `--prompt`.
    #[arg(long, default_value_t = false)]
    pub offline: bool,

    /// Upper bound on agent turns before the loop exits with code 1
    /// (bounded-loop exit — distinct from 0/2/3/130). Absent =
    /// "no bound"; pass `0` to force immediate bounded-loop exit
    /// (useful in E2E tests).
    #[arg(long, value_name = "N")]
    pub max_turns: Option<u32>,

    /// Path to a persona YAML file. Absent = the built-in default
    /// persona. T7.3 loads this via `kay_core::persona::load_file`
    /// and surfaces a config-error (exit 3) on I/O or parse failure.
    #[arg(long, value_name = "PATH")]
    pub persona: Option<PathBuf>,
}

/// T7.2 stub entrypoint. Emits one well-formed JSONL event line
/// (an `AgentEvent::TaskComplete`-shaped object) and returns
/// `Ok(())`. Satisfies the structural half of
/// `headless_prompt_emits_events`:
///
///   - subcommand `run` is recognized (clap parsed us here),
///   - stdout emits at least one line of valid JSON,
///   - exit code is 0.
///
/// T7.3 replaces the body with:
///
/// ```ignore
/// let (control_tx, control_rx) = mpsc::channel(8);
/// let (event_tx, event_rx)     = mpsc::channel(32);
/// let (_model_tx, model_rx)    = offline::spawn_provider(&args.prompt);
/// let persona = load_persona(&args.persona)?;
/// let registry = kay_tools::default_tool_set(...);
/// let tool_ctx = build_tool_ctx(...);
/// let handle = tokio::spawn(kay_core::run_turn(RunTurnArgs { … }));
/// while let Some(ev) = event_rx.recv().await {
///     println!("{}", AgentEventWire::from(ev));
/// }
/// handle.await??;
/// Ok(())
/// ```
///
/// The stub is deliberately zero-runtime-deps so T7.2's commit
/// lands without pulling tokio / kay-core changes — those arrive
/// in T7.3 on top.
pub fn execute(args: RunArgs) -> anyhow::Result<()> {
    // T7.2 sanity: acknowledge args were parsed (future T7.7 uses
    // them for exit-code branching; stubbing them here prevents a
    // dead-code warning without `#[allow]`).
    let _ = &args.prompt;
    let _ = args.offline;
    let _ = args.max_turns;
    let _ = args.persona.as_ref();

    // One JSONL line. Shape chosen to mirror the Phase 1 AgentEvent
    // wire contract (`kind` discriminant + flat fields). T7.3
    // replaces with a real `AgentEventWire` serialization.
    println!(r#"{{"kind":"TaskComplete","verified":true,"outcome":"Pass"}}"#);

    Ok(())
}
