//! execute_commands — KIRA marker-protocol streaming shell tool.
//! REQ TOOL-02 + SHELL-01..05. See 03-RESEARCH §4-§6.
//!
//! # Scaffold deviations from 03-04-PLAN text
//!
//! - `ToolCallContext.stream_sink` is `Arc<dyn Fn(AgentEvent) + Send + Sync>`
//!   (per 03-01 scaffold — frozen B7/VAL-007), NOT a `tokio::sync::mpsc::Sender`.
//!   Events are emitted via `(ctx.stream_sink)(ev)` — synchronous callback.
//! - `ToolCallContext` has no `timeout` or `project_root` field (plan text
//!   asserted they exist; 03-01 scaffold did not add them). Rule-3 adaptation:
//!   both are stored on `ExecuteCommandsTool` and supplied at construction
//!   time (`new(project_root, timeout)`).
//! - `AgentEvent` lives in `kay_tools::events` (source of truth per
//!   03-03 Wave 2 E1 — provider crate re-exports).
//! - `ToolError::Timeout` carries `elapsed: Duration` (03-02 scaffold), not
//!   `seconds: u64`.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use async_trait::async_trait;
use forge_domain::{ToolName, ToolOutput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::events::{AgentEvent, ToolOutputChunk};
#[cfg(unix)]
use crate::markers::shells::wrap_unix_sh;
#[cfg(windows)]
use crate::markers::shells::wrap_windows_ps;
use crate::markers::{MarkerContext, ScanResult, scan_line};
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

/// Default timeout — conservative upper bound for TB2.0 command envelopes.
/// Per D-05 the runtime value is resolved via `ForgeConfig.tool_timeout_secs`;
/// Phase 3 hardcodes 300s since Forge config integration is Wave 4.
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Grace window between SIGTERM and SIGKILL in the Unix cascade (D-05).
const SIGTERM_GRACE_SECS: u64 = 2;

/// First-token denylist — forces PTY when user does not set tty:true.
/// Per D-04: non-PTY is default. Minimum set per 03-BRAINSTORM E-spec:
/// htop, vim, nano, less, top, tmux, screen. Extended with common
/// interactive tools (ssh, sudo, docker, nvim, watch, psql, mysql, sqlite3).
const PTY_REQUIRING_FIRST_TOKENS: &[&str] = &[
    "ssh", "sudo", "docker", "less", "more", "vim", "nvim", "nano", "top", "htop", "watch",
    "psql", "mysql", "sqlite3", "tmux", "screen",
];

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExecuteCommandsInput {
    /// Shell command to execute. Multi-line scripts supported (wrapped in a
    /// subshell).
    pub command: String,
    /// Working directory. When relative, resolved against project_root.
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Force PTY allocation. Default false — heuristic may still engage.
    #[serde(default)]
    pub tty: bool,
    /// Additional environment `KEY=VALUE` pairs. Merged over process env.
    #[serde(default)]
    pub env: Option<Vec<String>>,
    /// Human-readable description for UI display (not passed to shell).
    #[serde(default)]
    pub description: Option<String>,
}

pub struct ExecuteCommandsTool {
    name: ToolName,
    description: String,
    input_schema: Value,
    project_root: PathBuf,
    timeout: Duration,
    seq_counter: AtomicU64,
}

impl ExecuteCommandsTool {
    /// Construct with default timeout (`DEFAULT_TIMEOUT_SECS`).
    pub fn new(project_root: PathBuf) -> Self {
        Self::with_timeout(project_root, Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Construct with an explicit timeout. Used by integration tests and
    /// by Wave 4 when Forge config threads `tool_timeout_secs` in.
    pub fn with_timeout(project_root: PathBuf, timeout: Duration) -> Self {
        let name = ToolName::new("execute_commands");
        let description = "Execute a shell command with streamed output.".to_string();

        let raw_schema = schemars::schema_for!(ExecuteCommandsInput);
        let mut schema: Value = serde_json::to_value(&raw_schema)
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        let hints = TruncationHints {
            output_truncation_note: Some(
                "Long outputs are truncated — narrow the command or use grep.".to_string(),
            ),
        };
        harden_tool_schema(&mut schema, &hints);

        Self {
            name,
            description,
            input_schema: schema,
            project_root,
            timeout,
            seq_counter: AtomicU64::new(0),
        }
    }

    fn resolve_cwd(&self, input_cwd: Option<&Path>) -> PathBuf {
        match input_cwd {
            Some(p) if p.is_absolute() => p.to_path_buf(),
            Some(p) => self.project_root.join(p),
            None => self.project_root.clone(),
        }
    }
}

/// Heuristic: which commands should transparently engage a PTY? True when
/// `tty_flag` is set explicitly OR the first-token basename is in the
/// denylist (SHELL-02). Public only at `pub(crate)` — exposed for unit tests.
pub(crate) fn should_use_pty(command: &str, tty_flag: bool) -> bool {
    if tty_flag {
        return true;
    }
    let Some(first) = command.split_whitespace().next() else {
        return false;
    };
    let basename = Path::new(first)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(first);
    PTY_REQUIRING_FIRST_TOKENS.contains(&basename)
}

#[async_trait]
impl Tool for ExecuteCommandsTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        let args = if args.is_null() {
            serde_json::json!({})
        } else {
            args
        };
        let input: ExecuteCommandsInput =
            serde_json::from_value(args).map_err(|e| ToolError::InvalidArgs {
                tool: self.name.clone(),
                reason: e.to_string(),
            })?;

        // SHELL-05 defense-in-depth: reject commands that carry a marker
        // substring. The model cannot rationally need this.
        if input.command.contains("__CMDEND_") {
            return Err(ToolError::InvalidArgs {
                tool: self.name.clone(),
                reason: "command contains reserved marker substring".to_string(),
            });
        }

        // M-01: propagate RNG failure as ToolError::Io rather than silently
        // falling back to a zero nonce (would let a prompt-injection attacker
        // pre-compute the valid marker and force early stream closure —
        // SHELL-05 / NN#7).
        let marker = MarkerContext::new(&self.seq_counter).map_err(ToolError::Io)?;
        let cwd = self.resolve_cwd(input.cwd.as_deref());

        if should_use_pty(&input.command, input.tty) {
            run_pty(self, &input, &marker, &cwd, ctx, call_id).await
        } else {
            run_piped(self, &input, &marker, &cwd, ctx, call_id).await
        }
    }
}

// --- tokio::process piped path -------------------------------------------

async fn run_piped(
    tool: &ExecuteCommandsTool,
    input: &ExecuteCommandsInput,
    marker: &MarkerContext,
    cwd: &Path,
    ctx: &ToolCallContext,
    call_id: &str,
) -> Result<ToolOutput, ToolError> {
    #[cfg(unix)]
    let wrapped = wrap_unix_sh(&input.command, marker);
    #[cfg(windows)]
    let wrapped = wrap_windows_ps(&input.command, marker);

    #[cfg(unix)]
    let mut cmd = {
        let mut c = Command::new("sh");
        c.arg("-c").arg(&wrapped);
        // process_group(0) so killpg(pgid, SIGTERM) hits the whole group.
        c.process_group(0);
        c
    };
    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-Command", &wrapped]);
        c
    };

    cmd.current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .kill_on_drop(true);

    if let Some(env_vec) = &input.env {
        for kv in env_vec {
            if let Some((k, v)) = kv.split_once('=') {
                cmd.env(k, v);
            }
        }
    }

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().ok_or_else(|| {
        ToolError::Io(std::io::Error::other("no stdout pipe"))
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ToolError::Io(std::io::Error::other("no stderr pipe"))
    })?;

    // stream_sink is a sync Fn; we clone the Arc into each reader task.
    let sink_stdout = Arc::clone(&ctx.stream_sink);
    let sink_stderr = Arc::clone(&ctx.stream_sink);
    let call_id_stdout = call_id.to_string();
    let call_id_stderr = call_id.to_string();
    let marker_clone = marker.clone();

    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        let mut marker_exit: Option<i32> = None;
        while let Ok(Some(line)) = reader.next_line().await {
            match scan_line(&line, &marker_clone) {
                ScanResult::Marker { exit_code } => {
                    marker_exit = Some(exit_code);
                    break;
                }
                ScanResult::NotMarker | ScanResult::ForgedMarker => {
                    (sink_stdout)(AgentEvent::ToolOutput {
                        call_id: call_id_stdout.clone(),
                        chunk: ToolOutputChunk::Stdout(format!("{line}\n")),
                    });
                }
            }
        }
        marker_exit
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            (sink_stderr)(AgentEvent::ToolOutput {
                call_id: call_id_stderr.clone(),
                chunk: ToolOutputChunk::Stderr(format!("{line}\n")),
            });
        }
    });

    let timeout_result = tokio::time::timeout(tool.timeout, async {
        let marker_exit = stdout_task.await.ok().flatten();
        let _ = stderr_task.await;
        let status = child.wait().await?;
        Ok::<(Option<i32>, std::process::ExitStatus), ToolError>((marker_exit, status))
    })
    .await;

    match timeout_result {
        Ok(Ok((marker_exit, status))) => {
            let (exit_code, marker_detected) = match marker_exit {
                Some(code) => (Some(code), true),
                None => (status.code(), false),
            };
            (ctx.stream_sink)(AgentEvent::ToolOutput {
                call_id: call_id.to_string(),
                chunk: ToolOutputChunk::Closed {
                    exit_code,
                    marker_detected,
                },
            });
            Ok(ToolOutput::text(format!(
                "exit_code={} marker_detected={}",
                exit_code
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "none".into()),
                marker_detected,
            )))
        }
        Ok(Err(e)) => Err(e),
        Err(_elapsed) => {
            terminate_with_grace(&mut child, SIGTERM_GRACE_SECS).await;
            (ctx.stream_sink)(AgentEvent::ToolOutput {
                call_id: call_id.to_string(),
                chunk: ToolOutputChunk::Closed {
                    exit_code: None,
                    marker_detected: false,
                },
            });
            Err(ToolError::Timeout {
                tool: tool.name.clone(),
                elapsed: tool.timeout,
            })
        }
    }
}

async fn terminate_with_grace(child: &mut Child, grace_secs: u64) {
    #[cfg(unix)]
    {
        // Per D-05: use killpg (positive pgid) to target the process group.
        // Spawned with process_group(0) so child.id() == pgid; shell-spawned
        // descendants inherit the pgid and receive SIGTERM.
        //
        // H-01 (SHELL-04): capture the pgid BEFORE wait() so we can still
        // issue the SIGKILL escalation even if the shell LEADER died from
        // SIGTERM while grandchildren in the group ignored it. Without the
        // unconditional SIGKILL on the pgid below, orphaned descendants are
        // reparented to PID 1 and survive the cascade.
        let pgid = child.id().map(|p| p as i32);
        send_sigterm_to_group_with_pgid(pgid);
        let leader_exited =
            tokio::time::timeout(Duration::from_secs(grace_secs), child.wait())
                .await
                .is_ok();
        // Always escalate to SIGKILL on the pgid to sweep up any grandchild
        // that ignored SIGTERM — even if the leader is already dead. This
        // is a no-op when the group is fully drained (ESRCH is ignored).
        escalate_sigkill_on_pgid(pgid);
        if !leader_exited {
            // Leader still hanging — reap it now that SIGKILL has landed.
            let _ = child.wait().await;
        }
    }
    #[cfg(windows)]
    {
        // Windows MVP: TerminateProcess via start_kill targets the single
        // process. Child-spawned processes are not reaped here (OK for MVP
        // per D-05; Phase 4 adds Job Objects).
        let _ = child.start_kill();
        let _ = tokio::time::timeout(Duration::from_secs(grace_secs), child.wait()).await;
    }
}

/// Split-helper (a): SIGTERM the process group via `killpg`.
/// Spawned with `process_group(0)` so `child.id() == pgid`.
#[cfg(unix)]
fn send_sigterm_to_group_with_pgid(pgid: Option<i32>) {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid;
    if let Some(p) = pgid {
        let _ = killpg(Pid::from_raw(p), Signal::SIGTERM);
    }
}

/// Split-helper (b): SIGKILL escalation on the process group.
///
/// Mirrors `send_sigterm_to_group_with_pgid`: we deliver SIGKILL to the
/// process GROUP (killpg with the positive pgid), not just the shell
/// leader. Calling `child.start_kill()` would only target the leader's
/// PID, leaving any grandchildren that ignored SIGTERM alive and
/// reparented to PID 1 (SHELL-04 requirement — see H-01 regression test).
#[cfg(unix)]
fn escalate_sigkill_on_pgid(pgid: Option<i32>) {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid;
    if let Some(p) = pgid {
        // ESRCH (no such process group) is expected when the group is
        // fully drained — we swallow it. Any other errno is also swallowed
        // because there's no meaningful recovery at this point.
        let _ = killpg(Pid::from_raw(p), Signal::SIGKILL);
    }
}

// --- PTY path ------------------------------------------------------------

async fn run_pty(
    tool: &ExecuteCommandsTool,
    input: &ExecuteCommandsInput,
    marker: &MarkerContext,
    cwd: &Path,
    ctx: &ToolCallContext,
    call_id: &str,
) -> Result<ToolOutput, ToolError> {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| ToolError::Io(std::io::Error::other(format!("openpty: {e}"))))?;

    #[cfg(unix)]
    let wrapped = wrap_unix_sh(&input.command, marker);
    #[cfg(windows)]
    let wrapped = wrap_windows_ps(&input.command, marker);

    #[cfg(unix)]
    let mut builder = {
        let mut b = CommandBuilder::new("sh");
        b.args(["-c", &wrapped]);
        b
    };
    #[cfg(windows)]
    let mut builder = {
        let mut b = CommandBuilder::new("powershell");
        b.args(["-NoProfile", "-Command", &wrapped]);
        b
    };
    builder.cwd(cwd);
    if let Some(env_vec) = &input.env {
        for kv in env_vec {
            if let Some((k, v)) = kv.split_once('=') {
                builder.env(k, v);
            }
        }
    }

    let mut child = pair.slave.spawn_command(builder).map_err(|e| {
        ToolError::Io(std::io::Error::other(format!("spawn_command: {e}")))
    })?;
    // M-03: capture the child's PID before clone_killer / wait so the
    // timeout path can deliver a SIGTERM grace window before SIGKILL
    // (parity with the run_piped cascade). portable-pty calls setsid()
    // in the child, making its PID == its own PGID on Unix.
    let pty_pid: Option<i32> = child.process_id().map(|p| p as i32);
    let mut killer = child.clone_killer();
    let mut reader = pair.master.try_clone_reader().map_err(|e| {
        ToolError::Io(std::io::Error::other(format!("clone_reader: {e}")))
    })?;
    // M-04: drop master IMMEDIATELY so the slave's stdin sees EOF right
    // after spawn — Kay's agent-driven PTY path does not feed interactive
    // input today (no stdin producer). For stdin-free commands (`ssh -V`,
    // `echo`, `ls`) this is fine; for genuinely interactive commands
    // (`sudo` prompting for a password, `vim`, `psql`) the child sees a
    // closed stdin immediately. Real-interactive PTY support is out of
    // scope for Phase 3 — tracked for Phase 5 if agent use-cases require
    // it. See 03-REVIEW.md M-04 / L-02 notes.
    drop(pair.master);

    let (line_tx, mut line_rx) = mpsc::channel::<String>(64);
    let _reader_task = tokio::task::spawn_blocking(move || {
        use std::io::BufRead;
        let mut buf = std::io::BufReader::new(&mut reader);
        let mut line = String::new();
        loop {
            line.clear();
            match buf.read_line(&mut line) {
                Ok(0) => return,
                Ok(_) => {
                    if line_tx.blocking_send(line.clone()).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });

    let marker_clone = marker.clone();
    let sink = Arc::clone(&ctx.stream_sink);
    let call_id_stream = call_id.to_string();

    let consumer = async move {
        let mut marker_exit: Option<i32> = None;
        while let Some(line) = line_rx.recv().await {
            match scan_line(&line, &marker_clone) {
                ScanResult::Marker { exit_code } => {
                    marker_exit = Some(exit_code);
                    break;
                }
                _ => {
                    (sink)(AgentEvent::ToolOutput {
                        call_id: call_id_stream.clone(),
                        chunk: ToolOutputChunk::Stdout(line),
                    });
                }
            }
        }
        marker_exit
    };

    let timed = tokio::time::timeout(tool.timeout, consumer).await;

    let (exit_code, marker_detected) = match timed {
        Ok(marker_exit) => {
            // portable-pty's Child::wait is blocking — run on the blocking pool.
            let wait = tokio::task::spawn_blocking(move || child.wait()).await;
            match (marker_exit, wait) {
                (Some(code), _) => (Some(code), true),
                (None, Ok(Ok(status))) => (i32::try_from(status.exit_code()).ok(), false),
                _ => (None, false),
            }
        }
        Err(_elapsed) => {
            // M-03: SIGTERM grace window before SIGKILL — parity with
            // run_piped's cascade. PTY-run commands (ssh, vim, psql) are
            // exactly the ones most likely to benefit from graceful
            // shutdown (state flush, tty restore). portable-pty's
            // ChildKiller::kill sends SIGHUP on Unix, so we issue our own
            // killpg(SIGTERM) → grace → killpg(SIGKILL) directly. On
            // Windows we fall back to the ChildKiller since job objects
            // are a Phase 4 item (same MVP stance as terminate_with_grace).
            #[cfg(unix)]
            {
                use nix::sys::signal::{Signal, killpg};
                use nix::unistd::Pid;
                if let Some(p) = pty_pid {
                    let _ = killpg(Pid::from_raw(p), Signal::SIGTERM);
                }
                tokio::time::sleep(Duration::from_secs(SIGTERM_GRACE_SECS)).await;
                if let Some(p) = pty_pid {
                    let _ = killpg(Pid::from_raw(p), Signal::SIGKILL);
                }
            }
            // Always ask portable-pty to tear down its master/slave state
            // (idempotent with killpg on unix, the only path on windows).
            let _ = killer.kill();
            (ctx.stream_sink)(AgentEvent::ToolOutput {
                call_id: call_id.to_string(),
                chunk: ToolOutputChunk::Closed {
                    exit_code: None,
                    marker_detected: false,
                },
            });
            return Err(ToolError::Timeout {
                tool: tool.name.clone(),
                elapsed: tool.timeout,
            });
        }
    };

    (ctx.stream_sink)(AgentEvent::ToolOutput {
        call_id: call_id.to_string(),
        chunk: ToolOutputChunk::Closed {
            exit_code,
            marker_detected,
        },
    });

    Ok(ToolOutput::text(format!(
        "exit_code={} marker_detected={}",
        exit_code
            .map(|n| n.to_string())
            .unwrap_or_else(|| "none".into()),
        marker_detected,
    )))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn pty_heuristic_respects_tty_flag() {
        assert!(should_use_pty("whatever", true));
    }

    #[test]
    fn pty_heuristic_engages_for_ssh() {
        assert!(should_use_pty("ssh user@host", false));
        assert!(should_use_pty("/usr/bin/ssh user@host", false));
    }

    #[test]
    fn pty_heuristic_skips_plain_echo() {
        assert!(!should_use_pty("echo hi", false));
        assert!(!should_use_pty("ls -la", false));
    }

    #[test]
    fn construct_produces_hardened_schema() {
        let t = ExecuteCommandsTool::new(PathBuf::from("/tmp"));
        let obj = t.input_schema.as_object().expect("schema is object");
        assert_eq!(obj.get("additionalProperties"), Some(&serde_json::json!(false)));
        assert!(obj.get("required").is_some());
    }
}
