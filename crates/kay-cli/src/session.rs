//! `kay session *` and `kay rewind` subcommand handlers (Phase 6 W-7).
//!
//! All handlers are synchronous. Session store I/O is synchronous (no tokio dep
//! in kay-session); only the drain loop (run.rs) is async.

use clap::Args;
use std::io::IsTerminal;
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

fn open_store() -> Result<kay_session::SessionStore> {
    let store_dir = kay_session::kay_home().join("sessions");
    std::fs::create_dir_all(&store_dir)?;
    Ok(kay_session::SessionStore::open(&store_dir)?)
}

fn list(args: ListArgs) -> Result<()> {
    let store = open_store()?;
    let sessions = kay_session::index::list_sessions(&store, args.limit)?;

    if sessions.is_empty() {
        if args.format == "json" {
            println!("[]");
        } else {
            println!("No sessions found");
        }
        return Ok(());
    }

    if args.format == "json" {
        let json_arr: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id.to_string(),
                    "title": s.title,
                    "status": s.status,
                    "start_time": s.start_time.to_rfc3339(),
                    "turn_count": s.turn_count,
                    "cost_usd": s.cost_usd,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_arr)?);
    } else {
        println!("{:<36}  {:<8}  {:<20}  {}", "ID", "STATUS", "STARTED", "TITLE");
        for s in &sessions {
            println!(
                "{:<36}  {:<8}  {:<20}  {}",
                s.id,
                s.status,
                s.start_time.format("%Y-%m-%d %H:%M:%S"),
                s.title,
            );
        }
    }
    Ok(())
}

fn fork(args: ForkArgs) -> Result<()> {
    let store = open_store()?;
    let child = kay_session::fork::fork_session(&store, &args.session_id)?;
    println!("Forked session: {}", child.id);
    Ok(())
}

fn export(args: ExportArgs) -> Result<()> {
    let store = open_store()?;
    let out_dir = args.output.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_default()
            .join(args.session_id.to_string())
    });
    kay_session::export::export_session(&store, &args.session_id, &out_dir)?;
    println!("Exported to {}", out_dir.display());
    Ok(())
}

fn import(args: ImportArgs) -> Result<()> {
    let store = open_store()?;
    let session = kay_session::export::import_session(&store, &args.dir)?;
    println!("Imported as session: {}", session.id);
    Ok(())
}

fn replay(args: ReplayArgs) -> Result<()> {
    let jsonl_path = args.dir.join("transcript.jsonl");
    let mut stdout = std::io::stdout();
    kay_session::export::replay(&jsonl_path, &mut stdout)?;
    Ok(())
}

/// `kay rewind` handler (DL-2: most recent snapshot; DL-8: --force/--dry-run).
pub fn rewind_cmd(args: RewindArgs) -> Result<()> {
    let store = open_store()?;

    let session_id = match args.session {
        Some(id) => id,
        None => {
            let sessions = kay_session::index::list_sessions(&store, 1)?;
            sessions
                .into_iter()
                .next()
                .map(|s| s.id)
                .ok_or_else(|| anyhow::anyhow!("no sessions found for rewind"))?
        }
    };

    // DL-8: --dry-run lists files without restoring any
    if args.dry_run {
        let snap_paths = kay_session::list_rewind_paths(&store, &session_id)?;
        if snap_paths.is_empty() {
            anyhow::bail!("no snapshots available for session {session_id}");
        }
        println!("Would restore {} file(s):", snap_paths.len());
        for p in &snap_paths {
            println!("  {}", p.display());
        }
        return Ok(());
    }

    // Restore: copy snapshot files back to original locations
    let snap_paths = kay_session::snapshot::rewind(&store, &session_id)?;

    if snap_paths.is_empty() {
        anyhow::bail!("no snapshots available for session {session_id}");
    }

    // Non-interactive without --force: ConfirmationRequired (DL-8, QG-C8)
    let is_tty = std::io::stdin().is_terminal();
    if !is_tty && !args.force {
        return Err(kay_session::SessionError::ConfirmationRequired.into());
    }

    println!("Restored {} file(s).", snap_paths.len());
    Ok(())
}
