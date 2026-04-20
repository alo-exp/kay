//! SHELL-04 integration: timeout cascade SIGTERM → 2s grace → SIGKILL → reap.
#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support/mod.rs"]
mod support;

use std::time::{Duration, Instant};

use kay_tools::{AgentEvent, ExecuteCommandsTool, Tool, ToolError, ToolOutputChunk};
use serde_json::json;
use support::{EventLog, make_ctx};

/// Collect all Stdout chunks from an EventLog snapshot as a single String
/// (used to recover the grandchild PID that the shell printed).
#[cfg(not(windows))]
fn stdout_concat(events: &[AgentEvent]) -> String {
    let mut out = String::new();
    for ev in events {
        if let AgentEvent::ToolOutput { chunk: ToolOutputChunk::Stdout(s), .. } = ev {
            out.push_str(s);
        }
    }
    out
}

/// Unix-only probe: does PID `pid` exist? Uses `kill(pid, 0)` which
/// returns 0 for a live process, -1 with ESRCH for a dead one.
#[cfg(unix)]
fn pid_is_alive(pid: i32) -> bool {
    use nix::errno::Errno;
    use nix::sys::signal::kill;
    use nix::unistd::Pid;
    match kill(Pid::from_raw(pid), None) {
        Ok(()) => true,
        Err(Errno::ESRCH) => false,
        // EPERM means the process exists but we lack permissions — still "alive".
        Err(Errno::EPERM) => true,
        Err(_) => false,
    }
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(windows))]
async fn timeout_sigterm_then_sigkill() {
    let tool = ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_millis(500));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    let start = Instant::now();
    let err = tool
        .invoke(json!({"command": "sleep 30"}), &ctx, "call_t")
        .await
        .expect_err("must time out");
    let elapsed = start.elapsed();

    assert!(
        matches!(err, ToolError::Timeout { .. }),
        "expected Timeout, got {err:?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "cascade must complete within 5s; took {elapsed:?}"
    );

    let events = log.drain();
    let saw_closed_none = events.iter().any(|ev| {
        matches!(
            ev,
            AgentEvent::ToolOutput {
                chunk: ToolOutputChunk::Closed { exit_code: None, marker_detected: false },
                ..
            }
        )
    });
    assert!(
        saw_closed_none,
        "Closed{{None,false}} must be emitted on timeout; events={events:?}"
    );
}

/// H-01 regression: a grandchild that ignores SIGTERM must still be killed
/// by the cascade — the SIGKILL escalation MUST target the process group,
/// not just the shell leader. The shell spawns a backgrounded grandchild
/// that traps TERM (so SIGTERM has no effect) and sleeps; we capture its
/// PID from stdout, wait for the timeout + cascade, then probe the PID.
/// If SIGKILL only hit the shell leader, the grandchild would be reparented
/// to PID 1 and still be alive — the assertion below catches that.
#[tokio::test(flavor = "multi_thread")]
#[cfg(unix)]
async fn timeout_cascade_kills_grandchild_that_ignores_sigterm() {
    // Timeout short so the cascade fires quickly. The 2s SIGTERM grace
    // then elapses; SIGKILL should hit the whole process group.
    let tool = ExecuteCommandsTool::with_timeout(std::env::temp_dir(), Duration::from_millis(500));
    let log = EventLog::new();
    let ctx = make_ctx(log.clone());

    // Shell: spawn a grandchild in a subshell that ignores TERM and
    // sleeps long. Print its PID on a line by itself so we can recover
    // it from the stream, then the shell itself sleeps (also long) so
    // the outer invoke hits the timeout.
    let script = "( trap '' TERM; echo GPID=$$; exec sleep 60 ) & \
                  printf 'GPID_LINE=%d\\n' $!; sleep 60";
    let _err = tool
        .invoke(json!({"command": script}), &ctx, "call_gc")
        .await
        .expect_err("must time out");

    // Allow the cascade + reparented-process cleanup a moment to settle.
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = log.drain();
    let combined = stdout_concat(&events);
    // Extract the grandchild PID from "GPID_LINE=<pid>".
    let gpid: i32 = combined
        .lines()
        .find_map(|l| l.strip_prefix("GPID_LINE="))
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| panic!("failed to parse grandchild PID from stdout: {combined:?}"));

    // The grandchild should be dead: SIGKILL delivered to the pgid
    // cannot be ignored. If `pid_is_alive` returns true we have the
    // exact H-01 regression.
    let alive = pid_is_alive(gpid);
    if alive {
        // Clean up so the test sandbox doesn't leak a sleep(60) for 60s.
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;
        let _ = kill(Pid::from_raw(gpid), Signal::SIGKILL);
    }
    assert!(
        !alive,
        "H-01 regression: grandchild PID {gpid} survived the cascade — \
         SIGKILL must target the process group, not just the shell leader"
    );
}
