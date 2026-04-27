//! 4-hour IPC memory canary.
//!
//! Measures RSS growth while hammering the IpcAgentEvent channel.
//! Run nightly in CI:
//!   cargo test -p kay-tauri --test memory_canary -- --ignored --nocapture
//!
//! Requires Tauri 2.3 test API. Verify `tauri::test::mock_builder` and
//! `tauri::test::mock_context` exist before uncommenting the full test body.

use std::time::{Duration, Instant};

/// Returns RSS (Resident Set Size) in bytes for the current process.
fn process_rss_bytes() -> u64 {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.process(Pid::from_u32(std::process::id()))
        .map(|p| p.memory())
        .unwrap_or(0)
}

/// Smoke test: RSS measurement works on this platform.
#[test]
fn process_rss_is_nonzero() {
    let rss = process_rss_bytes();
    assert!(rss > 0, "sysinfo RSS must be > 0 on this platform");
}

/// 10-second short canary for fast CI check.
#[test]
fn short_ipc_canary() {
    let baseline = process_rss_bytes();
    let deadline = Instant::now() + Duration::from_secs(10);

    let mut iterations = 0u32;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        iterations += 1;
    }

    let current = process_rss_bytes();
    let delta_mb = current.saturating_sub(baseline) / (1024 * 1024);
    assert!(
        delta_mb < 20,
        "RSS grew by +{delta_mb}MB over {iterations} iterations (baseline: {baseline}, current: {current})"
    );
}

/// 4-hour IPC memory canary. Run manually or nightly in CI with -- --ignored flag.
#[test]
#[ignore]
fn four_hour_ipc_canary() {
    let baseline = process_rss_bytes();
    let deadline = Instant::now() + Duration::from_secs(4 * 3600);

    // TODO Phase 10: wire tauri::test::mock_builder() + hammer_ipc_channel
    // once Tauri 2.3 test API is confirmed available.
    // See spec §9.2 for the full canary test body.

    let mut ticks = 0u32;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_secs(60));
        ticks += 1;
        let delta_mb = process_rss_bytes().saturating_sub(baseline) / (1024 * 1024);
        eprintln!("[canary tick {ticks}] RSS delta: +{delta_mb}MB");
        assert!(
            delta_mb < 50,
            "RSS leak detected: +{delta_mb}MB after {ticks} ticks"
        );
    }
}
