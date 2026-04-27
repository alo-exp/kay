//! `kay review` command - Code review workflow

use anyhow::Context;
use std::process::Command;

/// Run code review (clippy + custom checks)
#[derive(Debug, clap::Args)]
pub struct ReviewArgs {
    /// Path to review (default: current directory)
    #[arg(short = 'd', long)]
    pub path: Option<String>,

    /// Run clippy with warnings as errors
    #[arg(short, long)]
    pub strict: bool,

    /// Run formatting checks
    #[arg(short, long)]
    pub check_format: bool,

    /// Run security checks (if available)
    #[arg(long)]
    pub security: bool,
}

pub fn run(args: ReviewArgs) -> anyhow::Result<()> {
    let mut issues_found = false;

    // 1. Run clippy
    println!("Running clippy...");
    let mut clippy_cmd = Command::new("cargo");
    clippy_cmd.arg("clippy").arg("--all");

    if args.strict {
        clippy_cmd.arg("--").arg("-D");
    }

    match clippy_cmd.status() {
        Ok(status) if status.success() => {
            println!("✓ Clippy passed");
        }
        Ok(_) => {
            println!("✗ Clippy found issues");
            issues_found = true;
        }
        Err(e) => {
            println!("✗ Clippy failed to run: {}", e);
            issues_found = true;
        }
    }

    // 2. Check formatting
    if args.check_format {
        println!("\nChecking formatting...");
        let mut fmt_cmd = Command::new("cargo");
        fmt_cmd.arg("fmt").arg("--check");

        if let Some(ref path) = args.path {
            fmt_cmd.current_dir(path);
        }

        match fmt_cmd.status() {
            Ok(status) if status.success() => {
                println!("✓ Formatting is correct");
            }
            Ok(_) => {
                println!("✗ Formatting issues found");
                issues_found = true;
            }
            Err(e) => {
                println!("✗ Formatting check failed: {}", e);
                issues_found = true;
            }
        }
    }

    // 3. Security checks (placeholder for future implementation)
    if args.security {
        println!("\nNote: Security scanning not yet implemented.");
        println!("  Consider running: cargo audit");
    }

    if issues_found {
        anyhow::bail!("Review found issues - please fix before committing");
    }

    println!("\n✓ Code review passed - no issues found");
    Ok(())
}
