//! `kay clippy` command - Linting

use anyhow::Context;
use std::process::Command;

/// Run clippy linter
#[derive(Debug, clap::Args)]
pub struct ClippyArgs {
    /// Crate to lint (default: entire workspace)
    #[arg(short = 'p', long)]
    pub crate_name: Option<String>,

    /// Treat warnings as errors
    #[arg(long)]
    pub deny_warnings: bool,

    /// Allow warnings
    #[arg(long)]
    pub allow_warnings: bool,
}

pub fn run(args: ClippyArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");

    if args.deny_warnings {
        cmd.arg("--").arg("-D");
    } else if args.allow_warnings {
        cmd.arg("--").arg("-A");
    }

    if let Some(ref crate_name) = args.crate_name {
        cmd.arg("-p").arg(crate_name);
    }

    let status = cmd.status().context("Failed to execute cargo clippy")?;

    if status.success() {
        println!("✓ Clippy passed - no warnings");
    } else {
        anyhow::bail!("Clippy found issues with exit code: {}", status);
    }

    Ok(())
}
