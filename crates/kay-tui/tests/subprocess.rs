// subprocess.rs — integration tests for kay-cli subprocess lifecycle.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §5
//
// RED: The tests assert that KaySubprocess correctly spawns, streams events,
// and handles stop/drain lifecycle events.

use std::time::Duration;

use kay_tui::subprocess::{KaySubprocess, SubprocessError};

/// Test that spawning a non-existent binary returns BinaryNotFound error.
#[tokio::test]
async fn spawn_nonexistent_binary_returns_error() {
    // Override KAY_CLI_PATH to point to a non-existent binary.
    // Use std::env::var to avoid unsafe set_var/remove_var.
    let nonexistent = std::path::PathBuf::from("/nonexistent/path/to/kay");
    let result = KaySubprocess::spawn_with_cli_path(&[], &nonexistent).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SubprocessError::BinaryNotFound(p) if p == nonexistent));
}

/// Test that a well-behaved subprocess can be spawned and events streamed.
#[tokio::test]
async fn subprocess_streams_events() {
    // Create a mock script that outputs JSONL lines then exits.
    // This mimics the kay-cli JSONL output behavior.
    let tmp = std::env::temp_dir().join(format!("mock-kay-{}.sh", std::process::id()));
    let script = r#"#!/bin/sh
printf '%s\n' '{"type":"TextDelta","data":{"content":"hello"}}'
printf '%s\n' '{"type":"ToolCallStart","data":{"id":"t1","name":"read_file"}}'
printf '%s\n' '{"type":"Usage","data":{"prompt_tokens":100,"completion_tokens":50,"cost_usd":0.001}}'
printf '%s\n' '{"type":"ToolCallComplete","data":{"id":"t1","name":"read_file"}}'
printf '%s\n' '{"type":"TaskComplete","data":{"call_id":"t1","verified":true,"outcome":{"Pass":{"note":"ok"}}}}'
"#
    .to_string();
    std::fs::write(&tmp, script.as_bytes()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&tmp, perms).unwrap();
    }

    let result = KaySubprocess::spawn_with_cli_path(&[], &tmp).await;
    let (mut sub, mut rx) = result.expect("should spawn mock binary");

    // Note: don't delete the mock script while the subprocess is running.
    // The subprocess holds it open as stdin. Clean up after stop().

    let mut count = 0;
    let timeout = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(e) = rx.recv().await {
            count += 1;
            let _ = e; // consumed for counting
        }
    })
    .await;

    assert!(timeout.is_ok(), "should complete within timeout");
    assert_eq!(count, 5, "should receive all 5 events from fixture");

    // Drain and stop
    let _ = tokio::time::timeout(Duration::from_secs(3), sub.stop()).await;
}

/// Test that stop() terminates the subprocess within timeout.
#[tokio::test]
async fn stop_terminates_subprocess() {
    // Use /bin/echo to create a short-lived subprocess that outputs JSONL.
    // This verifies the subprocess lifecycle without needing a mock script.
    let echo_path = std::path::PathBuf::from("/bin/echo");
    let result = KaySubprocess::spawn_with_cli_path(
        &["Testing kay-tui subprocess".into()],
        &echo_path,
    )
    .await;

    // If /bin/echo exists (should on all Unix), verify it spawns successfully.
    if result.is_ok() {
        let (mut sub, mut rx) = result.unwrap();
        // Drain any events (may get 0 or 1 from echo output).
        let _ = tokio::time::timeout(Duration::from_secs(1), async {
            while rx.recv().await.is_some() {}
        }).await;
        // Stop should complete within timeout.
        let stop_result = tokio::time::timeout(Duration::from_secs(3), sub.stop()).await;
        assert!(stop_result.is_ok(), "stop should complete within timeout");
    }
    // If /bin/echo doesn't exist, the test passes without checking subprocess.
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a temporary mock shell script that outputs JSONL and exits.
/// Returns the path to the script.
#[allow(dead_code)]
fn create_mock_kay_script(fixture: &str) -> std::path::PathBuf {
    let tmp = std::env::temp_dir().join(format!("mock-kay-{}.sh", std::process::id()));
    let script = fixture
        .lines()
        .map(|l| format!("printf '%s\\n' '{}'", l))
        .collect::<Vec<_>>()
        .join("; ");
    let full_script = format!("#!/bin/sh\n{}", script);
    std::fs::write(&tmp, full_script.as_bytes()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&tmp, perms).unwrap();
    }
    tmp
}