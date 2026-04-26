//! `kay check` command - Type check the workspace

use anyhow::Context;
use std::process::Command;

/// Type check the workspace
#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Crate to check (default: entire workspace)
    #[arg(short = 'p', long)]
    pub crate_name: Option<String>,
}

pub fn run(args: CheckArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");

    if let Some(ref crate_name) = args.crate_name {
        cmd.arg("-p").arg(crate_name);
    }

    let status = cmd.status().context("Failed to execute cargo check")?;

    if status.success() {
        println!("✓ Check completed successfully");
    } else {
        anyhow::bail!("Check failed with exit code: {}", status);
    }

    Ok(())
}
