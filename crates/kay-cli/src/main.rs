//! kay-cli — headless CLI for Kay
//!
//! Phase 1 scaffolded `eval tb2 --dry-run`. Phase 3 Wave 4 adds
//! `tools list` — the single runtime path that proves the Phase 3
//! registry wires up end-to-end (TOOL-06: emits OpenAI-shape tool
//! definitions for all 7 tools).

use clap::Parser;

mod boot;
mod eval;

#[derive(Parser)]
#[command(
    name = "kay",
    version,
    about = "Kay — open-source terminal coding agent (headless CLI)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
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
        Command::Eval { target } => eval::run(target),
        Command::Tools { action } => match action {
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
    }
}
