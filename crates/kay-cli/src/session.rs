//! `kay session *` and `kay rewind` subcommand handlers (Phase 6 W-7).
//!
//! All handlers are synchronous. Session store I/O is synchronous (no tokio dep
//! in kay-session); only the drain loop (run.rs) is async.

use clap::Args;
use std::path::PathBuf;
use uuid::Uuid;
use anyhow::Result;

// ─── Argument types ──────────────────────────────────────────────────────

#[derive(clap::Subcommand, Debug)]
pub enum SessionAction {
    /// List sessions (newest first).
    List(ListArgs),
    /// Fork a session into a new child session.
    Fork(ForkArgs),
    /// Export a session to a directory.
    Export(ExportArgs),
    /// Import a previously exported session.
    Import(ImportArgs),
    /// Replay a session's JSONL transcript to stdout.
    Replay(ReplayArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
    #[arg(long, default_value = "table", value_parser = ["table", "json"])]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct ForkArgs {
    pub session_id: Uuid,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    pub session_id: Uuid,
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub include_persona: bool,
}

#[derive(Args, Debug)]
pub struct ImportArgs {
    pub dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct ReplayArgs {
    pub dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct RewindArgs {
    #[arg(long, value_name = "SESSION_ID")]
    pub session: Option<Uuid>,
    #[arg(long, value_name = "N")]
    pub turn: Option<u64>,
    /// Skip confirmation prompt.
    #[arg(long)]
    pub force: bool,
    /// Show what would be restored without writing.
    #[arg(long)]
    pub dry_run: bool,
}

// ─── Dispatch ────────────────────────────────────────────────────────────

/// Dispatch `kay session <action>`.
pub fn dispatch(action: SessionAction) -> Result<()> {
    match action {
        SessionAction::List(args) => list(args),
        SessionAction::Fork(args) => fork(args),
        SessionAction::Export(args) => export(args),
        SessionAction::Import(args) => import(args),
        SessionAction::Replay(args) => replay(args),
    }
}

fn list(_args: ListArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement list")
}

fn fork(_args: ForkArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement fork")
}

fn export(_args: ExportArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement export")
}

fn import(_args: ImportArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement import")
}

fn replay(_args: ReplayArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement replay")
}

/// `kay rewind` handler (DL-2: most recent snapshot; DL-8: --force/--dry-run).
pub fn rewind_cmd(_args: RewindArgs) -> Result<()> {
    anyhow::bail!("W-7 GREEN: implement rewind with DL-8 confirmation")
}
