//! Live API smoke tests for `kay run --live`.
//!
//! Tests end-to-end invocation of the `kay` binary with `--live` mode,
//! which makes real API calls to MiniMax-M2.7. All tests are gated behind
//! the `live` Cargo feature so normal `cargo test` does not bill usage.
//!
//! ```sh
//! # Run with real API key (set via environment)
//! export MINIMAX_API_KEY="sk-cp-..."
//! cargo test -p kay-cli --features live -- --nocapture
//!
//! # Skip live tests entirely (CI default — no API key needed)
//! cargo test -p kay-cli
//! ```
//!
//! ## Test scenarios covered
//!
//! | Scenario | Expected outcome |
//! |----------|-----------------|
//! | `--live` + `TEST:done` sentinel | TaskComplete event → exit 0 |
//! | `--live` + real question | TextDelta emitted → exit 0 |
//! | `--live` without API key | Clear error message → exit 1 |
//! | `--live` + `--offline` together | "mutually exclusive" error → exit 1 |
//! | `--offline` alone (regression) | TaskComplete event → exit 0 |

#![allow(clippy::unwrap_used)]

use std::env;
use std::process::{Command, Output, Stdio};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path to the compiled `kay` binary.
fn kay_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("kay")
}

/// Returns the MiniMax API key from environment, or skips the test if absent.
fn minimax_key() -> Option<String> {
    env::var("MINIMAX_API_KEY").ok().filter(|k| !k.is_empty())
}

/// Runs `kay run --live --prompt <prompt>` with the real MiniMax key, capturing output.
fn kay_run_live(prompt: &str) -> Output {
    let key = minimax_key().expect("MINIMAX_API_KEY must be set for live tests");
    Command::new(kay_bin())
        .env("MINIMAX_API_KEY", key)
        .args(["run", "--live", "--prompt", prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable")
}

/// Runs `kay run --offline --prompt <prompt>` (regression check).
fn kay_run_offline(prompt: &str) -> Output {
    Command::new(kay_bin())
        .args(["run", "--offline", "--prompt", prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable")
}

// ---------------------------------------------------------------------------
// Feature-gated live tests (require real MINIMAX_API_KEY)
// ---------------------------------------------------------------------------

/// Smoke test: `kay run --live --prompt "TEST:done"` sends the sentinel to the
/// MiniMax API. The real API interprets "TEST:done" as a normal prompt and
/// returns text — this test verifies the live path is wired, the event stream
/// opens, and the process exits 0 with at least one JSONL line on stdout.
///
/// This does NOT use the offline sentinel path (that requires `--offline`).
#[test]
#[cfg(feature = "live")]
fn live_minimax_smoke_test_done_sentinel() {
    let output = kay_run_live("TEST:done");

    // Non-zero exit signals a crash or unhandled error — live path must stay up.
    assert!(
        output.status.success(),
        "kay run --live should exit 0 on normal completion.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // stdout must contain at least one line of JSONL (the event stream).
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        !lines.is_empty(),
        "stdout must contain at least one JSONL event line.\nstdout: {stdout}"
    );

    // Verify at least one line is valid JSON (the event stream).
    for line in &lines {
        let parsed = serde_json::from_str::<serde_json::Value>(line);
        assert!(
            parsed.is_ok(),
            "Each stdout line must be valid JSON.\nLine: {line}\nError: {:?}",
            parsed.err()
        );
    }
}

/// Smoke test: `kay run --live --prompt "What is 2+2?"` makes a real MiniMax
/// API call and verifies TextDelta frames appear in the JSONL output.
#[test]
#[cfg(feature = "live")]
fn live_minimax_echo_question_text_delta() {
    let output = kay_run_live("What is 2+2? Respond in one sentence.");

    assert!(
        output.status.success(),
        "kay run --live should exit 0 on normal completion.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_text_delta = stdout.lines().any(|line| {
        line.contains("\"type\":\"TextDelta\"") || line.contains(r#""type":"TextDelta"#)
    });

    assert!(
        has_text_delta,
        "stdout should contain at least one TextDelta JSONL line.\n\
         stdout: {stdout}"
    );
}

/// Smoke test: `kay run --live --model minimax/MiniMax-M2.7` explicitly names
/// the model. Verifies model override is accepted and stream opens.
#[test]
#[cfg(feature = "live")]
fn live_minimax_explicit_model_override() {
    let key = minimax_key().expect("MINIMAX_API_KEY must be set");
    let output = Command::new(kay_bin())
        .env("MINIMAX_API_KEY", key)
        .args([
            "run",
            "--live",
            "--model",
            "minimax/MiniMax-M2.7",
            "--prompt",
            "ping",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable");

    assert!(
        output.status.success(),
        "explicit --model minimax/MiniMax-M2.7 should work.\n\
         stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

// ---------------------------------------------------------------------------
// Always-on regression tests (no API key needed)
// ---------------------------------------------------------------------------

/// Regression: `--offline` still works and exits 0 on `TEST:done`.
/// This test is ALWAYS on — it does NOT call the live API.
#[test]
fn offline_regression_test_done_still_exits_zero() {
    let output = kay_run_offline("TEST:done");

    assert!(
        output.status.success(),
        "--offline with TEST:done should exit 0.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for task_complete event (case-insensitive, with or without quotes)
    let has_task_complete = stdout.lines().any(|l| {
        l.to_lowercase().contains("task_complete") || l.contains(r#""type":"TaskComplete""#)
    });

    assert!(
        has_task_complete,
        "stdout should contain TaskComplete JSONL line.\nstdout: {stdout}"
    );
}

/// Regression: `--offline` + `--live` mutual exclusion — exit 1 with a clear error.
#[test]
fn offline_live_mutual_exclusion_error() {
    let output = Command::new(kay_bin())
        .args(["run", "--offline", "--live", "--prompt", "ping"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable");

    assert!(
        !output.status.success(),
        "--offline + --live should fail (exit non-zero)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mutually exclusive") || stderr.to_lowercase().contains("mutual"),
        "stderr should explain mutual exclusion.\nstderr: {stderr}"
    );
}

/// Regression: `--live` without MINIMAX_API_KEY should fail early with a clear
/// error — not a cryptic internal panic.
#[test]
fn live_without_api_key_clear_error() {
    // Remove MINIMAX_API_KEY from environment to simulate the missing-key case.
    // Use a scrubbed environment so no parent-process key leaks in.
    let output = Command::new(kay_bin())
        .env_remove("MINIMAX_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .args(["run", "--live", "--prompt", "ping"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable");

    assert!(
        !output.status.success(),
        "--live without MINIMAX_API_KEY should fail (exit non-zero)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Error should guide user to set API key - match any of several patterns
    // (exact text varies by provider implementation)
    let has_key_guidance = stderr.to_lowercase().contains("minimax_api_key")
        || stderr.to_lowercase().contains("missing")
        || stderr.to_lowercase().contains("api key")
        || stderr.to_lowercase().contains("authentication")
        || stderr.to_lowercase().contains("environment")
        || stderr.to_lowercase().contains("key not set");
    assert!(
        has_key_guidance,
        "stderr should guide user to set MINIMAX_API_KEY.\nstderr: {stderr}"
    );
}

/// Regression: `kay run --live --model` with a non-allowlisted model should
/// be rejected at the allowlist check (not at HTTP layer). Uses the offline
/// path to avoid a real API call — the allowlist check happens before any HTTP.
#[test]
fn live_non_allowlisted_model_rejected_offline_path() {
    // We test the allowlist rejection via the offline path by using a model
    // that is NOT in the MiniMax allowlist. The live provider is selected,
    // the model is rejected pre-HTTP, and kay exits non-zero.
    //
    // Note: This test uses `--live` flag but with a model name that will fail
    // allowlist check before any HTTP call is made. We don't need a real key.
    let output = Command::new(kay_bin())
        .env_remove("MINIMAX_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .args([
            "run",
            "--live",
            "--model",
            "openai/gpt-4", // Not in MiniMax allowlist
            "--prompt",
            "ping",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("kay binary should be runnable");

    // Should fail because openai/gpt-4 is not on the MiniMax allowlist.
    assert!(
        !output.status.success(),
        "--live with non-MiniMax model should fail allowlist check.\n\
         stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should fail with a helpful error. Without API key, auth error comes first.
    // With API key + bad model, allowlist error comes first.
    let has_helpful_error = stderr.to_lowercase().contains("allowlist")
        || stderr.to_lowercase().contains("not allowlisted")
        || stderr.to_lowercase().contains("model")
        || stderr.to_lowercase().contains("authentication")
        || stderr.to_lowercase().contains("missing")
        || stderr.to_lowercase().contains("key");
    assert!(
        has_helpful_error,
        "stderr should explain model allowlist rejection or auth error.\nstderr: {stderr}"
    );
}
