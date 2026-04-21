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
//! # T7.2 scope (this commit)
//!
//! Lands clap argument parsing for the `run` subcommand + a stub
//! interactive-fallback path that prints a placeholder banner + prompt
//! before exiting. T7.3 wires `run` to `kay_core::run_turn`; T7.4 ports
//! the real banner; T7.9 populates `interactive_fallback` with the full
//! reedline REPL.
//!
//! Why `command: Option<Command>`? clap requires distinguishing
//! "no subcommand provided" from "subcommand missing" — the `Option`
//! wrapper plus the absence of `#[command(arg_required_else_help)]`
//! means plain `kay` with no args is a valid invocation that falls
//! through to `interactive_fallback`. This matches ForgeCode's UX
//! parity contract (DL-1): running `forge` / `kay` alone drops the
//! user into an interactive session.

use std::io::Write;

use clap::Parser;

mod boot;
mod eval;
mod run;

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
}

#[derive(clap::Subcommand)]
enum ToolsAction {
    /// List registered tool names + descriptions. Used by integration
    /// tests and ops smoke-checks to verify the 7-tool default set is
    /// wired up.
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Run(args)) => run::execute(args),
        Some(Command::Eval { target }) => eval::run(target),
        Some(Command::Tools { action }) => match action {
            ToolsAction::List => {
                let reg = boot::install_tool_registry(None)?;
                let mut defs = reg.tool_definitions();
                defs.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
                for def in defs {
                    println!("{}\t{}", def.name.as_str(), def.description);
                }
                Ok(())
            }
        },
        None => interactive_fallback(),
    }
}

/// T7.2 stub for the interactive-fallback path.
///
/// Emits a placeholder banner line + `kay>` prompt to stdout, then
/// returns `Ok(())`. The `interactive_parity_diff` test in T7.1 will
/// fail against this stub (banner text doesn't match fixtures) —
/// deliberate until T7.4 ports the real banner and T7.9 installs the
/// reedline REPL.
///
/// Why print the prompt AT ALL in the stub: the RED test asserts
/// `kay>` appears in stdout; having the prompt here keeps T7.6 +
/// T7.9 independent (they can swap in reedline integration without
/// this function needing a concurrent update).
fn interactive_fallback() -> anyhow::Result<()> {
    // Placeholder banner. T7.4 replaces with `banner::emit()` port
    // from forge_main.
    println!("Kay — open-source terminal coding agent");
    print!("kay> ");
    // stdout is line-buffered in terminals but fully-buffered when
    // piped; flush so subprocess-level E2E tests see the prompt
    // before we return and Drop closes stdout.
    std::io::stdout().flush().ok();
    Ok(())
}
