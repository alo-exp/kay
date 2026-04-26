//! kay-cli — headless CLI for Kay.
//!
//! # Subcommand layout (Phase 5 Wave 7)
//!
//!   * `kay run --prompt <text> [--offline] [--max-turns N] [--persona PATH]`
//!     headless one-shot turn driver. Emits the AgentEvent JSONL
//!     stream (CLI-05) on stdout. Exit codes per CLI-03.
//!   * `kay eval ...` — evaluation harness dispatch (Phase 1).
//!   * `kay tools list` — tool-registry introspection (Phase 3 Wave 4).
//!   * `kay` — no subcommand → interactive fallback mode (T7.9).
//!     Emits banner + `kay>` prompt and enters reedline loop.
//!
//! # T7.7 scope (this commit)
//!
//! `main` no longer returns `anyhow::Result<()>`. Instead each
//! subcommand dispatches into a helper that returns `Result<ExitCode,
//! anyhow::Error>`, and `main` translates both arms through
//! [`exit::classify_error`] and [`std::process::exit`]. This replaces
//! Rust's stdlib default `Termination` (which collapses every
//! `Err(_)` onto exit 1) with the CLI-03 classifier that emits 0, 1,
//! 2, 3, or 130 depending on what kind of error escaped. See
//! `src/exit.rs` for the full contract table.
//!
//! # Why `command: Option<Command>`?
//!
//! clap requires distinguishing "no subcommand provided" from
//! "subcommand missing" — the `Option` wrapper plus the absence of
//! `#[command(arg_required_else_help)]` means plain `kay` with no
//! args is a valid invocation that falls through to
//! `interactive_fallback`. This matches the UX-parity contract (DL-1):
//! running `kay` alone drops the user into an interactive session.

use clap::Parser;

mod banner;
mod boot;
mod commands;
mod diff;
mod eval;
mod exit;
mod help;
mod interactive;
mod markdown;
mod prompt;
mod render;
mod run;
mod session;
mod spinner;

use exit::{ExitCode, classify_error};

#[derive(Parser)]
#[command(
    name = "kay",
    version,
    about = "Kay — open-source terminal coding agent (headless CLI)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run a headless agent turn. Emits AgentEvent JSONL on stdout.
    Run(run::RunArgs),
    /// Run evaluation harnesses (Terminal-Bench 2.0, etc.)
    Eval {
        #[command(subcommand)]
        target: eval::EvalTarget,
    },
    /// Introspect the built-in tool registry.
    Tools {
        #[command(subcommand)]
        action: ToolsAction,
    },
    /// Manage sessions (list, fork, export, import, replay).
    Session {
        #[command(subcommand)]
        action: session::SessionAction,
    },
    /// Rewind to the most recent pre-edit snapshot.
    Rewind(session::RewindArgs),
    /// Build the workspace or a specific crate.
    Build(commands::build::BuildArgs),
    /// Type-check the workspace or a specific crate.
    Check(commands::check::CheckArgs),
    /// Format code.
    Fmt(commands::fmt::FmtArgs),
    /// Run clippy linter.
    Clippy(commands::clippy::ClippyArgs),
    /// Run tests.
    Test(commands::test::TestArgs),
    /// Run code review (clippy + formatting check).
    Review(commands::review::ReviewArgs),
    /// Show help for commands and topics.
    ShowHelp {
        #[command(subcommand)]
        topic: Option<HelpTopic>,
    },
}

#[derive(clap::Subcommand)]
enum HelpTopic {
    /// General help
    General,
    /// Help for the run command
    Run,
    /// Help for session management
    Session,
    /// Help for build commands
    Build,
}

#[derive(clap::Subcommand)]
enum ToolsAction {
    /// List registered tool names + descriptions. Used by integration
    /// tests and ops smoke-checks to verify the 7-tool default set is
    /// wired up.
    List,
}

/// Entry point.
///
/// Returns `()` (not `anyhow::Result<()>`) so that stdlib's default
/// `Termination` impl never gets the chance to collapse every error
/// onto exit 1. Instead each subcommand returns a `Result<ExitCode,
/// anyhow::Error>` that we translate through two paths:
///
///   * `Ok(code)` → use `code` directly (covers the
///     `ExitCode::SandboxViolation` (2) and `ExitCode::Success` (0)
///     cases originating from `run::execute`).
///   * `Err(e)`   → print the diagnostic to stderr (`{:?}` renders
///     the full anyhow context chain — helpful for debugging config
///     errors) and call [`classify_error`] for the code.
///
/// The final `std::process::exit(code)` is the ONLY exit path out of
/// `main` — there is no implicit return.
///
/// See `src/exit.rs` for the CLI-03 contract table.
fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Some(Command::Run(args)) => match run::execute(args) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Eval { target }) => match eval::run(target) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Tools { action }) => match run_tools(action) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Session { action }) => match session::dispatch(action) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Rewind(args)) => match session::rewind_cmd(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Build(args)) => match commands::build::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Check(args)) => match commands::check::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Fmt(args)) => match commands::fmt::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Clippy(args)) => match commands::clippy::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Test(args)) => match commands::test::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::Review(args)) => match commands::review::run(args) {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
        Some(Command::ShowHelp { topic }) => {
            let topic_str = match &topic {
                Some(HelpTopic::General) => Some("general"),
                Some(HelpTopic::Run) => Some("run"),
                Some(HelpTopic::Session) => Some("session"),
                Some(HelpTopic::Build) => Some("build"),
                None => None,
            };
            help::dispatch_help(topic_str);
            ExitCode::Success
        }
        None => match interactive_fallback() {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("Error: {e:?}");
                classify_error(&e)
            }
        },
    };
    // `u8 as i32` is the standard widening for `process::exit`'s
    // signature — all ExitCode discriminants fit in u8 by type (the
    // `#[repr(u8)]` on the enum makes that compile-time-enforced).
    std::process::exit(code.as_u8() as i32);
}

/// `kay tools <action>` dispatch. Pulled out of `main` so the exit-
/// code plumbing above can treat it like the other subcommands (a
/// function returning `anyhow::Result<()>`) instead of inlining the
/// match arms.
fn run_tools(action: ToolsAction) -> anyhow::Result<()> {
    match action {
        ToolsAction::List => {
            let reg = boot::install_tool_registry(None)?;
            let mut defs = reg.tool_definitions();
            defs.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
            for def in defs {
                println!("{}\t{}", def.name.as_str(), def.description);
            }
            Ok(())
        }
    }
}

/// Interactive-fallback entry point.
///
/// Thin delegator into [`interactive::run`], which owns the actual
/// banner + TTY-gated REPL dispatch. Split out of `main` so `main`'s
/// match arm stays a single function call — the match structure is
/// load-bearing for the CLI-03 exit-code plumbing (every arm follows
/// the same `Ok(()) → Success / Err → classify_error` shape).
///
/// See `interactive.rs` module docs for:
///   * Why banner fires on both TTY and non-TTY paths
///   * Why `reedline::Reedline::create()` is TTY-gated (pipe surface
///     would break the parity-diff test capture)
///   * Why Ctrl-C maps to `continue` (not process abort) at the REPL
///   * Why kay-cli does NOT import forge_main here (DL-3)
fn interactive_fallback() -> anyhow::Result<()> {
    interactive::run()
}
