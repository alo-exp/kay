//! `kay build` command - Build the workspace

use anyhow::Context;
use std::process::Command;

/// Build the workspace or a specific crate
#[derive(Debug, clap::Args)]
pub struct BuildArgs {
    /// Crate to build (default: entire workspace)
    #[arg(short = 'p', long)]
    pub crate_name: Option<String>,

    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,
}

pub fn run(args: BuildArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if args.release {
        cmd.arg("--release");
    }

    if let Some(ref crate_name) = args.crate_name {
        cmd.arg("-p").arg(crate_name);
    }

    let status = cmd.status().context("Failed to execute cargo build")?;

    if status.success() {
        println!("✓ Build completed successfully");
    } else {
        anyhow::bail!("Build failed with exit code: {}", status);
    }

    Ok(())
}
