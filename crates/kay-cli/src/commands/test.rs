//! `kay test` command - Run tests

use anyhow::Context;
use std::process::Command;

/// Run tests
#[derive(Debug, clap::Args)]
pub struct TestArgs {
    /// Crate to test (default: entire workspace)
    #[arg(short = 'p', long)]
    pub crate_name: Option<String>,

    /// Test filter (substring match on test name)
    #[arg(short = 'f', long)]
    pub filter: Option<String>,

    /// Run only ignored tests
    #[arg(long)]
    pub ignored: bool,

    /// Include doc tests
    #[arg(long)]
    pub include_doc: bool,

    /// Don't fail on test warnings
    #[arg(long)]
    pub no_fail_fast: bool,
}

pub fn run(args: TestArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if args.ignored {
        cmd.arg("--ignored");
    }

    if args.include_doc {
        cmd.arg("--doc");
    }

    if args.no_fail_fast {
        cmd.arg("--no-fail-fast");
    }

    if let Some(ref filter) = args.filter {
        cmd.arg(filter);
    }

    if let Some(ref crate_name) = args.crate_name {
        cmd.arg("-p").arg(crate_name);
    }

    let status = cmd.status().context("Failed to execute cargo test")?;

    if status.success() {
        println!("✓ All tests passed");
    } else {
        anyhow::bail!("Tests failed with exit code: {}", status);
    }

    Ok(())
}
