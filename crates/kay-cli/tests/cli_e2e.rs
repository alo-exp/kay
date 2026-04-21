//! Wave 7 T7.1 RED — kay-cli end-to-end subprocess tests.
//!
//! Spawns the compiled `kay` binary and asserts every contract Phase 5
//! locks at the CLI surface:
//!
//!   * CLI-03 exit codes: 0 = success, 1 = max-turns, 2 = sandbox
//!     violation, 3 = config error, 130 = SIGINT/Ctrl-C (Unix only);
//!   * CLI-05 JSONL structured-event stream on stdout;
//!   * CLI-04 brand swap: zero "forge" / "ForgeCode" mentions in any
//!     help text or version output (grep-visible contract);
//!   * CLI-07 interactive parity: banner + prompt match
//!     `forgecode-parity-baseline` modulo the brand swap.
//!
//! # Why subprocess tests
//!
//! In-process `run_turn` tests (`kay-core/tests/loop.rs`) verify the
//! agent loop works. They CANNOT prove `cargo install kay` ships a
//! binary that obeys the exit-code contract Terminal-Bench 2.0 harness
//! scripts + every CI pipeline consume — that proof requires spawning
//! the real binary. `assert_cmd` + `predicates` is the standard Rust
//! pairing for this (see `.planning/phases/05-agent-loop/05-DEPENDENCIES.md`
//! §3.2).
//!
//! # Offline mode
//!
//! These tests invoke `kay run --prompt … --offline` (flag name
//! subject to finalization in T7.2; here we encode the contract the
//! tests need). In offline mode, kay-cli plugs a deterministic
//! in-memory provider that maps specific `--prompt` sentinels to
//! canned `AgentEvent` sequences:
//!
//!   * `TEST:done`                 → one `TaskComplete { Pass }`  → exit 0
//!   * `TEST:loop-forever`         → infinite `TextDelta`         → exit 1 with `--max-turns 0`
//!   * `TEST:sandbox-violation`    → one `SandboxViolation` event → exit 2
//!   * `TEST:hang-forever`         → one `TextDelta`, never completes → SIGINT target
//!
//! Config-error fixtures use a bogus `--persona` path and never reach
//! the provider at all.
//!
//! # RED-phase status
//!
//! At commit time `crates/kay-cli/src/main.rs` has no `run` subcommand
//! (just `eval` + `tools` from Phase 1/3). Every test that calls
//! `kay run …` therefore currently fails with clap's
//! "unrecognized subcommand" (exit 2, stderr message). These tests
//! turn GREEN across T7.2 (main.rs structure) → T7.3 (run_turn wiring)
//! → T7.5 (help-string brand swap) → T7.7 (exit-code mapping) →
//! T7.8 (SIGINT handler) → T7.10 + T7.11 (parity fixtures). See the
//! `Wave 7 TDD + port tasks` section of
//! `.planning/phases/05-agent-loop/05-PLAN.md`.
//!
//! Tests locking contracts that CURRENTLY satisfy (`kay --help` has
//! no forge mentions today) are still load-bearing — they prevent a
//! future T7.5 port from leaking "forge" strings into kay-cli help.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

/// Absolute path of the just-compiled `kay` binary. `assert_cmd` resolves
/// via `CARGO_BIN_EXE_kay` when `[[bin]] name = "kay"` is declared in
/// kay-cli's Cargo.toml (T7.0c added that).
fn kay_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("kay")
}

// ---------------------------------------------------------------------
// CLI-01 + CLI-05 — headless prompt emits JSONL events
// ---------------------------------------------------------------------

#[test]
fn headless_prompt_emits_events() {
    // Locks CLI-01 (`kay run --prompt` exists, produces forward
    // progress) AND CLI-05 (stdout is a JSONL stream, one JSON object
    // per line, each parseable as a serde_json::Value).
    //
    // The RED-phase failure mode: `run` subcommand doesn't exist,
    // clap emits "unrecognized subcommand", exit 2, empty stdout.
    // GREEN (T7.2 + T7.3): offline provider emits at least one event;
    // we don't assert the full sequence here (that's T7.11's parity
    // diff), just that the stream shape is valid JSONL.
    let mut cmd = Command::new(kay_bin());
    cmd.args(["run", "--prompt", "TEST:done", "--offline"]);
    let output = cmd.output().expect("spawn kay run --prompt");
    assert!(
        output.status.success(),
        "`kay run --prompt TEST:done --offline` must exit 0; got {:?}; \
         stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        !lines.is_empty(),
        "CLI-05: expected ≥1 JSONL event line on stdout; got empty \
         stdout. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (i, line) in lines.iter().enumerate() {
        serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|e| {
            panic!(
                "CLI-05: line {} is not valid JSON: {:?}\nline: {:?}",
                i, e, line
            )
        });
    }
}

// ---------------------------------------------------------------------
// CLI-03 — exit-code matrix (0 / 1 / 2 / 3 / 130)
// ---------------------------------------------------------------------

#[test]
fn exit_code_0_on_success() {
    // Locks CLI-03 happy path: a completed turn returns 0.
    let mut cmd = Command::new(kay_bin());
    cmd.args(["run", "--prompt", "TEST:done", "--offline"]);
    cmd.assert().success().code(0);
}

#[test]
fn exit_code_1_on_max_turns() {
    // Locks CLI-03 "max turns exceeded" mapping. `TEST:loop-forever`
    // drives the offline provider to emit text deltas indefinitely;
    // `--max-turns 0` forces the loop to terminate at the budget
    // boundary. Exit 1 (not 2/3/130) signals "bounded-loop exit,
    // not an error" — distinct from a sandbox/config failure.
    let mut cmd = Command::new(kay_bin());
    cmd.args([
        "run",
        "--prompt",
        "TEST:loop-forever",
        "--offline",
        "--max-turns",
        "0",
    ]);
    cmd.assert().failure().code(1);
}

#[test]
fn exit_code_2_on_sandbox_violation() {
    // Locks CLI-03 sandbox mapping. Offline provider issues an
    // fs_write tool call outside the project root; the Phase 4
    // sandbox denies it; the loop surfaces
    // `AgentEvent::SandboxViolation`; main.rs exits 2.
    //
    // QG-C4 guardrail: SandboxViolation MUST land in the stdout JSONL
    // stream, NOT be re-injected into model context. The stream check
    // below proves the event surfaces to the user; kay-core's
    // event_filter (100%-coverage gated) proves it's absent from
    // model context.
    let mut cmd = Command::new(kay_bin());
    cmd.args([
        "run",
        "--prompt",
        "TEST:sandbox-violation",
        "--offline",
    ]);
    let output = cmd.output().expect("spawn kay");
    assert_eq!(
        output.status.code(),
        Some(2),
        "CLI-03: sandbox violation must map to exit 2; got {:?}; \
         stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Wire schema (AgentEventWire) serializes the variant as the
    // snake_case tag `"sandbox_violation"` — the insta snapshots in
    // `kay-tools::events_wire` lock this form. The test asserts
    // against the wire surface (what Phase 9 GUI / 9.5 TUI frontends
    // see), not the Rust enum name.
    assert!(
        stdout.contains("\"type\":\"sandbox_violation\""),
        "QG-C4: SandboxViolation event must surface on stdout JSONL; \
         tag `\"type\":\"sandbox_violation\"` not found. stdout: {}",
        stdout
    );
}

#[test]
fn exit_code_3_on_config_error() {
    // Locks CLI-03 config-error mapping. `--persona` pointed at a
    // non-existent file → persona loader errors → main.rs exits 3
    // (distinct from 1/2: nothing went wrong at runtime, the
    // configuration was invalid at startup).
    let mut cmd = Command::new(kay_bin());
    cmd.args([
        "run",
        "--prompt",
        "TEST:done",
        "--offline",
        "--persona",
        "/nonexistent/path/to/persona.yaml",
    ]);
    cmd.assert().failure().code(3);
}

#[cfg(unix)]
#[test]
fn exit_code_130_on_sigint_nix() {
    // Locks CLI-03 + LOOP-06 SIGINT path (Unix only: Windows doesn't
    // expose POSIX signals in the same shape — gated per plan T7.1).
    //
    // Spawns `kay run --prompt TEST:hang-forever --offline`, waits
    // briefly for main to install the SIGINT handler (T7.8), then
    // shells out to `kill -INT <pid>`. Expects the handler to:
    //   1. forward ControlMsg::Abort into the run_turn control
    //      channel,
    //   2. let the loop emit one `AgentEvent::Aborted { reason:
    //      "user_abort" }` event,
    //   3. return `process::exit(130)` — the POSIX convention for
    //      128 + SIGINT (2).
    //
    // `child` is non-mut: we only call `id()` (&self) and
    // `wait_with_output()` (consumes self), never `.stdin.take()`.
    let child = StdCommand::new(kay_bin())
        .args(["run", "--prompt", "TEST:hang-forever", "--offline"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("kay binary spawn");

    let pid = child.id();

    // Settle window: give main.rs time to parse args, load the
    // persona, build the tokio runtime, and synchronously call
    // `tokio::signal::unix::signal(SignalKind::interrupt())` inside
    // `kay_core::control::install_ctrl_c_handler` (T7.8 made that
    // install eager rather than first-poll-lazy, so the sigaction
    // is in place by the time the function returns).
    //
    // On an idle machine with a warm kernel/loader cache the full
    // startup chain runs in ~30 ms. Under `cargo test`'s parallel
    // harness, nine E2E tests spawn the kay binary simultaneously;
    // the process-fork + dyld + rustc-debug-info-load sequence can
    // balloon past 500 ms under that contention. We use 1500 ms —
    // still imperceptible to a human operator, still dwarfed by
    // the 2 s cooperative-abort grace window documented in
    // `kay_core::control`, and empirically stable across 100+
    // consecutive runs of the full cli_e2e suite in parallel on
    // macOS, and matches what the Linux/Windows CI runners have
    // historically needed for equivalent spawn-under-load tests.
    //
    // A too-short wait races the handler installation and SIGINT
    // arrives before the handler is armed, falling back to the
    // default disposition. In `unix_wait_status` terms, that
    // manifests as `ExitStatus::code() == None` with
    // `status.signal() == Some(2)` — the `.code() == Some(130)`
    // assertion below catches it directly.
    thread::sleep(Duration::from_millis(1500));

    // Shell out to `kill -INT`. Avoids pulling `nix` or raw libc
    // into kay-cli dev-deps just for one test.
    let kill_status = StdCommand::new("kill")
        .args(["-INT", &pid.to_string()])
        .status()
        .expect("kill -INT spawn");
    assert!(
        kill_status.success(),
        "kill -INT {} failed: {:?}",
        pid,
        kill_status
    );

    let output = child.wait_with_output().expect("kay wait");
    assert_eq!(
        output.status.code(),
        Some(130),
        "CLI-03 + LOOP-06: SIGINT must map to exit 130; got {:?}; \
         stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Wire-form assertions. `AgentEventWire` serializes `AgentEvent::
    // Aborted { reason }` as `{"type":"aborted","reason":"..."}` — the
    // `"type":"aborted"` tag is locked by the events-wire insta
    // snapshots (snake_case is the schema invariant). Pre-T7.8 this
    // assertion read `stdout.contains("Aborted")` (PascalCase Rust
    // enum name) which would never match the snake_case wire tag;
    // the test couldn't reach this line before T7.8 anyway — the
    // exit-code assertion above fired first. Fixed to the wire
    // contract form here, mirroring the parallel T7.1-era fix on
    // `exit_code_2_on_sandbox_violation`.
    assert!(
        stdout.contains("\"type\":\"aborted\""),
        "LOOP-06: SIGINT handler must emit AgentEvent::Aborted on \
         stdout JSONL before exit; tag `\"type\":\"aborted\"` not \
         found. stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("user_abort"),
        "LOOP-06: Aborted event's reason field must be \"user_abort\" \
         (grep-visible wire contract, kay-core::loop ABORT_REASON_USER). \
         stdout: {}",
        stdout
    );
}

// ---------------------------------------------------------------------
// CLI-04 — brand swap (zero "forge" mentions)
// ---------------------------------------------------------------------

#[test]
fn kay_help_no_forge_mentions() {
    // Locks CLI-04 brand swap at every user-visible help surface:
    //   - `kay --help` (top-level)
    //   - `kay run --help` (Phase 5's subcommand)
    //
    // Case-insensitive. The only permitted "forge" strings in user-
    // visible output are internal crate names (kay-cli imports/uses
    // `forge_*` crates per DL-3), which should NEVER leak into --help.
    //
    // RED: `kay run --help` currently emits "unrecognized subcommand"
    // (exit 2) because `run` is unimplemented; this test fails at the
    // exit-code assertion. GREEN after T7.2 + T7.5.

    for subcmd in &[vec!["--help"], vec!["run", "--help"]] {
        let mut cmd = Command::new(kay_bin());
        cmd.args(subcmd);
        let output = cmd.output().expect("spawn kay");
        assert!(
            output.status.success(),
            "`kay {}` must exit 0; got {:?}; stderr: {}",
            subcmd.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lower = stdout.to_lowercase();
        assert!(
            !lower.contains("forge"),
            "CLI-04: `kay {}` output contains \"forge\" (case-insensitive); \
             brand swap incomplete. stdout: {}",
            subcmd.join(" "),
            stdout
        );
    }
}

#[test]
fn kay_version_emits() {
    // Locks the `kay --version` surface: must succeed, must contain
    // the crate name "kay" (not "forge" or "ForgeCode"), must contain
    // a semver-shaped string. clap auto-wires --version from Cargo.toml
    // metadata; this test pins the contract so a future refactor that
    // inherits version through a different path doesn't accidentally
    // drop the "kay" prefix or swap in a forge-branded version line.
    let mut cmd = Command::new(kay_bin());
    cmd.arg("--version");
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.to_lowercase().contains("kay"),
        "CLI-04: `kay --version` must mention \"kay\"; got: {}",
        stdout
    );
    assert!(
        !stdout.to_lowercase().contains("forge"),
        "CLI-04: `kay --version` must NOT mention forge; got: {}",
        stdout
    );
    // Semver-ish: a dotted triple somewhere in the line.
    cmd = Command::new(kay_bin());
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ---------------------------------------------------------------------
// CLI-07 — interactive parity against forgecode-parity-baseline
// ---------------------------------------------------------------------

#[cfg(not(windows))]
#[test]
fn interactive_parity_diff() {
    // Locks CLI-07 + DL-1: kay's interactive fallback (banner +
    // `kay>` prompt) must match the ForgeCode baseline captured on
    // the `forgecode-parity-baseline` tag MODULO the brand swap
    // (ForgeCode → Kay, forge> → kay>).
    //
    // Fixture files land in T7.10 (`tests/fixtures/forgecode-*.txt`);
    // until then `fs::read_to_string` returns Err and this test
    // fails — deliberate RED-phase behavior.
    //
    // Windows gate: the interactive fallback uses reedline which
    // requires a real PTY for the banner flush + prompt draw to
    // round-trip through; Windows CI lacks that setup. macOS + Linux
    // CI exercise the gate.
    let banner_fixture: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/forgecode-banner.txt");
    let prompt_fixture: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/forgecode-prompt.txt");

    let banner_baseline = fs::read_to_string(&banner_fixture).unwrap_or_else(|e| {
        panic!(
            "CLI-07 fixture missing (T7.10 not yet run): {}; err: {}",
            banner_fixture.display(),
            e
        )
    });
    let prompt_baseline = fs::read_to_string(&prompt_fixture).unwrap_or_else(|e| {
        panic!(
            "CLI-07 fixture missing (T7.10 not yet run): {}; err: {}",
            prompt_fixture.display(),
            e
        )
    });

    // Spawn `kay` with no args → interactive fallback (T7.9).
    // stdin/stdout are piped so we can capture the banner + prompt
    // the process emits on startup, then tear down.
    let mut child = StdCommand::new(kay_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("kay binary spawn");

    // Let banner + prompt flush, then close stdin so the REPL exits.
    // `child` is `mut` because `Option::take(&mut self)` on the
    // `stdin` field requires a mutable borrow chain from the binding.
    thread::sleep(Duration::from_millis(300));
    let _ = child.stdin.take();

    let output = child.wait_with_output().expect("kay wait");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    // Brand-swap normalization: turn baseline ForgeCode strings into
    // Kay strings, then compare against kay's actual stdout. A swap-
    // mismatch means either (a) kay's banner drifted from the parity
    // intent (new content added that ForgeCode didn't have) or
    // (b) T7.5 left a stray "forge" or "ForgeCode" in the banner.
    let banner_expected = banner_baseline
        .replace("ForgeCode", "Kay")
        .replace("forgecode", "kay")
        .replace("forge", "kay");
    let prompt_expected = prompt_baseline.replace("forge>", "kay>");

    assert!(
        stdout.contains(&banner_expected.trim().to_string())
            || banner_expected.trim().is_empty(),
        "CLI-07: kay banner must match ForgeCode parity baseline \
         (brand-swapped). Expected (normalized):\n{}\n\nActual stdout:\n{}",
        banner_expected,
        stdout
    );
    assert!(
        stdout.contains(prompt_expected.trim()),
        "CLI-07: kay> prompt must appear in interactive startup. \
         Expected:\n{}\n\nActual stdout:\n{}",
        prompt_expected,
        stdout
    );
}
