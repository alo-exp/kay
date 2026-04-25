//! `kay eval` subcommand module.
//!
//! Phase 1 scaffolds `eval tb2 --dry-run` as a shim that describes what a real
//! Harbor run would invoke. Actual execution lands in follow-on task EVAL-01a
//! once an OpenRouter key + $100 budget are available.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum EvalTarget {
    /// Terminal-Bench 2.0 parity run via Harbor (scaffolded; run deferred to EVAL-01a)
    Tb2 {
        /// MiniMax model to use (MiniMax-M2.7 or MiniMax-M2.5).
        /// MiniMax is OpenRouter-compatible; no separate OpenRouter key needed.
        #[arg(long, default_value = "minimax/MiniMax-M2.7")]
        model: String,

        /// Number of Terminal-Bench 2.0 tasks to run (89 is the full suite).
        #[arg(long, default_value_t = 89)]
        tasks: u32,

        /// Directory to archive the JSONL transcript and summary into (per D-20).
        #[arg(
            long,
            default_value = ".planning/phases/01-fork-governance-infrastructure/parity-baseline"
        )]
        archive_dir: String,

        /// Opt-in to a real Harbor run (not implemented in Phase 1).
        /// Phase 1 always dry-runs because the actual parity run is deferred to EVAL-01a.
        /// Passing this flag will hard-fail with a pointer to PARITY-DEFERRED.md.
        #[arg(long, default_value_t = false)]
        run: bool,
    },
}

pub fn run(target: EvalTarget) -> anyhow::Result<()> {
    match target {
        EvalTarget::Tb2 { model, tasks, archive_dir, run } => {
            if run {
                anyhow::bail!(
                    "EVAL-01a not yet implemented — actual parity run deferred per CONTEXT.md \
                     §User Amendments (2026-04-19). In Phase 1, omit --run (dry-run is the default)."
                );
            }
            eprintln!(
                "[kay eval tb2] Parity run deferred to EVAL-01a per user amendment 2026-04-19."
            );
            eprintln!(
                "See .planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md"
            );
            eprintln!(
                "Would run: harbor run -d terminal-bench/terminal-bench-2 -m minimax/{model} -n {tasks}"
            );
            eprintln!("Archive directory: {archive_dir}");
            eprintln!("Prerequisites (when run is enabled):");
            eprintln!("  - Docker installed + running");
            eprintln!("  - uv tool install harbor  (or pip install harbor)");
            eprintln!("  - MINIMAX_API_KEY set (MiniMax key; OpenRouter key NOT required)");
            eprintln!("  - DAYTONA_API_KEY set (for --env daytona)");
            eprintln!(
                "On completion, compare the score against the existing 'forgecode-parity-baseline' \
                 tag and archive the transcript + summary.md to `archive_dir`. The tag itself was \
                 created during Phase 1 (unsigned per D-OP-04) and will be re-signed in Phase 11 \
                 once a signing key is procured."
            );
            Ok(())
        }
    }
}
