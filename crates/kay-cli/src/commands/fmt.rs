//! `kay fmt` command - Format code

use anyhow::Context;
use std::process::Command;

/// Format code
#[derive(Debug, clap::Args)]
pub struct FmtArgs {
    /// Crate to format (default: entire workspace)
    #[arg(short = 'p', long)]
    pub crate_name: Option<String>,

    /// Check only (don't modify)
    #[arg(short, long)]
    pub check: bool,

    /// Apply formatting
    #[arg(long)]
    pub apply: bool,
}

pub fn run(args: FmtArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");

    if let Some(ref crate_name) = args.crate_name {
        cmd.arg("-p").arg(crate_name);
    }

    if args.check {
        cmd.arg("--check");
    }

    let status = cmd.status().context("Failed to execute cargo fmt")?;

    if status.success() {
        if args.check {
            println!("✓ Formatting is correct");
        } else {
            println!("✓ Formatting applied");
        }
    } else {
        if args.check {
            println!("✗ Formatting issues found");
        }
        anyhow::bail!("Format failed with exit code: {}", status);
    }

    Ok(())
}
