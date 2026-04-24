// subprocess.rs — kay-cli subprocess lifecycle manager.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §5
//
// WAVE 3: KaySubprocess struct with spawn/stdout channel/stderr channel.
// - Spawns kay-cli binary with JSONL output flag
// - Streams stdout lines through mpsc channel to the TUI loop
// - Stderr goes to parent process stderr (not shown in TUI)

use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

pub use crate::events::TuiEvent;

/// Path to the kay-cli binary (kay-cli crate's binary).
fn kay_cli_binary() -> PathBuf {
    // In development: ../kay-cli/target/.../kay
    // In release: the installed `kay` binary
    std::env::var("KAY_CLI_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_default()
                .join("kay")
        })
}

/// Handle to the kay-cli subprocess.
/// The stdout event receiver is returned directly from spawn() and NOT stored
/// in this struct — mpsc::Receiver is not Clone so it can't be shared across
/// multiple consumers. Callers receive it directly from spawn().
#[derive(Debug)]
pub struct KaySubprocess {
    child: Child,
}

impl KaySubprocess {
    /// Spawn a kay-cli subprocess with JSONL output streaming.
    /// Returns the handle and the event receiver.
    ///
    /// Arguments are passed directly to kay-cli (e.g., `["--session", "abc123"]`).
    pub async fn spawn(
        args: &[String],
    ) -> Result<(Self, mpsc::Receiver<TuiEvent>), SubprocessError> {
        let binary = kay_cli_binary();

        if !binary.exists() {
            return Err(SubprocessError::BinaryNotFound(binary));
        }

        let mut child = Command::new(&binary)
            .args(["--output-format", "jsonl"])
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // stderr goes to terminal (not shown in TUI)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SubprocessError::SpawnFailed(binary.clone(), e))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            SubprocessError::SpawnFailed(
                binary,
                std::io::Error::new(std::io::ErrorKind::NotConnected, "stdout not captured"),
            )
        })?;

        let (event_tx, event_rx) = mpsc::channel(1024);

        // Spawn async task to read stdout and forward JSONL lines to channel
        tokio::spawn(Self::read_stdout(stdout, event_tx));

        Ok((Self { child }, event_rx))
    }

    /// Asynchronously drain stdout lines and forward to the event channel.
    async fn read_stdout(stdout: tokio::process::ChildStdout, tx: mpsc::Sender<TuiEvent>) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut parser = crate::jsonl::JsonlParser::new();

        while let Ok(Some(line)) = lines.next_line().await {
            parser.feed(line.as_bytes());
            parser.feed(b"\n");
            while let Some(result) = parser.next_event() {
                match result {
                    Ok(event) => {
                        if tx.send(event).await.is_err() {
                            // Receiver dropped — stop reading
                            return;
                        }
                    }
                    Err(e) => {
                        // Malformed/unknown line was already logged by JsonlParser
                        tracing::debug!(error = %e, "jsonl: skipped event");
                    }
                }
            }
        }
    }

    /// Wait for the subprocess to exit and return its exit code.
    pub async fn wait(&mut self) -> Result<i32, std::io::Error> {
        self.child.wait().await.map(|s| s.code().unwrap_or(-1))
    }

    /// Send SIGINT to the subprocess (graceful stop).
    pub async fn stop(&mut self) -> Result<(), std::io::Error> {
        self.child.start_kill()?;
        self.wait().await?;
        Ok(())
    }
}

/// Errors from subprocess management.
#[derive(Debug)]
pub enum SubprocessError {
    /// kay-cli binary not found at expected path.
    BinaryNotFound(PathBuf),
    /// Failed to spawn the subprocess.
    SpawnFailed(PathBuf, std::io::Error),
}

impl std::fmt::Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubprocessError::BinaryNotFound(path) => {
                write!(f, "kay-cli binary not found at {path:?}")
            }
            SubprocessError::SpawnFailed(path, e) => {
                write!(f, "failed to spawn {path:?}: {e}")
            }
        }
    }
}

impl std::error::Error for SubprocessError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subprocess_error_display() {
        let err = SubprocessError::BinaryNotFound(PathBuf::from("/bin/kay"));
        assert!(err.to_string().contains("not found"));
    }
}
