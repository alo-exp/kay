//! kay-cli — headless CLI for Kay
//!
//! Phase 1 scaffolds only `eval tb2 --dry-run` per CONTEXT.md §User Amendments.
//! The actual parity run lands in follow-on task EVAL-01a.

use clap::Parser;

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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval { target } => eval::run(target),
    }
}
