//! Interactive fallback REPL for `kay` (no subcommand).
//!
//! # T7.9 scope
//!
//! When the user runs plain `kay` (no subcommand), `main::interactive_fallback`
//! delegates into [`run`] below. This module owns the subcommand-absent
//! UX: emit the Kay banner, detect TTY vs. piped stdin, and dispatch
//! accordingly:
//!
//!   * **TTY path** — spin a [`reedline::Reedline`] REPL driven by
//!     [`crate::prompt::KayPrompt`]. `Signal::Success` echoes a
//!     "not-yet-wired" line (real interactive turn execution is Phase
//!     10+ scope — T7.9's contract is the REPL surface only).
//!     `Signal::CtrlC` → continue (discard current buffer, re-prompt).
//!     `Signal::CtrlD` → exit 0. Any `Err` variant → exit 0 after
//!     stderr-logging. See the `match` body below for the full table.
//!
//!   * **Non-TTY path** — pipe/redirect/subprocess harness. Reedline
//!     cannot run without a real terminal (it calls into `crossterm`
//!     raw-mode APIs that fail fast on a non-TTY `stdin`). Emit the
//!     plain-text `kay> ` literal ([`crate::prompt::KAY_PROMPT`]) and
//!     return. The caller tears down normally; exit 0 via
//!     `main::interactive_fallback` → `Ok(()) → ExitCode::Success`.
//!
//! # Why TTY-gate instead of always-reedline
//!
//! Two surfaces depend on the non-TTY path behaving like the original
//! `print!("{}", KAY_PROMPT); flush()` pair:
//!
//!   1. `kay-cli::tests::cli_e2e::interactive_parity_diff` (T7.10/T7.11)
//!      spawns `kay` with `Stdio::piped()` stdin/stdout so it can
//!      capture the banner + prompt literal before tearing the child
//!      down. Reedline on a piped stdin would error with
//!      `Error: Underlying: Resource temporarily unavailable` or a
//!      raw-mode failure, breaking the parity capture.
//!
//!   2. Shell pipelines like `echo "" | kay | tee log` would also fail
//!      under always-reedline. The plain-literal path is the graceful
//!      degradation for any non-interactive environment.
//!
//! [`std::io::IsTerminal`] on `io::stdin()` is the canonical detector;
//! it returns `false` for pipes, files, and redirected streams — all
//! three of which are non-TTY cases we need to handle.
//!
//! # Why kay-cli does not import `forge_main` here
//!
//! DL-3 (locked in 05-CONTEXT.md §DL-3): kay-cli replicates the minimum
//! reedline integration it needs in this file rather than depending on
//! `forge_main`. `forge_main` carries forward through Phase 10 for
//! binary-preservation (`forge` binary still ships while kay's REPL
//! builds out incrementally), so depending on it here would tangle the
//! two release surfaces. The `reedline` workspace dep, the `KayPrompt`
//! port in `prompt.rs`, and this REPL body are the three pieces T7.9
//! needs — everything else (completer, hinter, file-backed history,
//! menu) is deferred to the Phase 10+ REPL enhancement work.
//!
//! # Interaction with the Ctrl-C handler
//!
//! T7.8 installed a `kay-core::control::install_ctrl_c_handler` inside
//! `run::run_async` — the `kay run` subcommand path. The interactive
//! fallback runs on an entirely separate code path (no tokio runtime,
//! no control channel, no agent loop). Reedline's own Ctrl-C handling
//! (`Signal::CtrlC` variant) is what catches SIGINT in this path, which
//! is why the `match` arm for `Signal::CtrlC` is `continue` — it's a
//! "cancel current input line" signal, not a "abort the process"
//! signal. A user pressing Ctrl-C at the REPL stays at the prompt;
//! Ctrl-D (EOF) is the canonical exit.

use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use anyhow::Result;
use forge_api::AgentId;
use reedline::{Reedline, Signal};

use crate::banner;
use crate::prompt::{KAY_PROMPT, KayPrompt};

/// Entry point for the interactive-fallback surface.
///
/// Always emits the banner, then dispatches on TTY-ness. See module
/// docs for the full rationale.
///
/// # Why `AgentId::new("kay")` here
///
/// The prompt-right segment renders the active agent name as
/// uppercased snake-case (`kay` → `KAY`). `AgentId::default()` resolves
/// to `"forge"` at the domain layer (Phase 10 residual rename), which
/// would leak a `FORGE` label into the interactive UI — breaking the
/// CLI-04 zero-forge-mentions contract. Explicit `new("kay")` is the
/// stopgap until Phase 10 flips the domain default.
pub fn run() -> Result<()> {
    // Banner fires on both paths — the parity-baseline fixture was
    // captured from ForgeCode's piped-stdout surface, so the test
    // sees the banner on a non-TTY stdin too.
    banner::display(false)?;

    if !std::io::stdin().is_terminal() {
        // Non-TTY fallback. `KAY_PROMPT` is the exact literal `"kay> "`
        // locked by `prompt::tests::test_kay_prompt_literal_is_kay_not_forge`
        // and referenced by the T7.11 parity-diff test.
        //
        // `flush()` is load-bearing here: stdout is line-buffered when
        // attached to a terminal but fully-buffered when piped, and
        // the E2E tests read the child's stdout via a pipe. Without
        // the flush, the prompt literal would sit in the process's
        // internal buffer until `drop(stdout)` at thread teardown —
        // which is after the test has already captured `stdout`.
        print!("{KAY_PROMPT}");
        std::io::stdout().flush().ok();
        return Ok(());
    }

    // TTY path: real reedline REPL.
    //
    // `current_dir` failure falls back to `"."` — same degradation
    // pattern forge_main uses. The prompt's left segment reads
    // `cwd.file_name()` which would yield `EMPTY_MARKER` ("[empty]")
    // for `"."`, which is intentional: the user sees a visible
    // "something went wrong" marker without the REPL crashing.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut prompt = KayPrompt::new(cwd, AgentId::new("kay"));
    // Minimal reedline editor — no completer/hinter/history in T7.9.
    // Those are forge_main-coupled (InputCompleter, ForgeHighlighter,
    // FileBackedHistory with forge-specific paths) and DL-3 keeps
    // kay-cli free of that dependency. Phase 10+ REPL enhancement
    // work will add kay-native equivalents.
    //
    // `with_ansi_colors(true)` enables the nu-ansi-term color codes
    // KayPrompt emits in `render_prompt_left` / `render_prompt_right`.
    // Without this, those ANSI sequences render as literal text
    // (e.g., `[1;36m` as characters) instead of being interpreted.
    let mut editor = Reedline::create().with_ansi_colors(true);

    loop {
        // `read_line` takes `&dyn Prompt` — immutable borrow for the
        // render calls. The subsequent `prompt.refresh()` needs `&mut`
        // but that borrow is fresh (the `&prompt` borrow ended when
        // `read_line` returned), so this is lifetime-clean.
        match editor.read_line(&prompt) {
            Ok(Signal::Success(buf)) => {
                // Refresh the prompt's git branch between turns so
                // `git checkout` outside kay reflects immediately on
                // the next render. The KayPrompt setter panics are
                // impossible here — `refresh()` only does a `gix::
                // discover` + `head().referent_name()` chain that
                // returns `None` on any failure and elides the branch
                // segment.
                prompt.refresh();

                let trimmed = buf.trim();
                if !trimmed.is_empty() {
                    // T7.9 contract: REPL surface only, no agent-loop
                    // integration. The headless surface (`kay run
                    // --prompt "..."`) carries the agent loop today;
                    // Phase 10+ will wire the REPL input line into
                    // `run_turn` so a user can type a prompt at the
                    // REPL and see streamed events.
                    //
                    // Printing a human-visible reminder here (not a
                    // silent no-op) matches the UX principle that
                    // every Enter press should produce feedback.
                    println!(
                        "(T7.9: interactive turn execution not yet wired — \
                         use `kay run --prompt \"...\"` for headless mode, \
                         or Ctrl-D to exit)"
                    );
                }
            }
            Ok(Signal::CtrlC) => {
                // Ctrl-C at the REPL is "cancel current buffer" —
                // reedline already clears the line and redraws the
                // prompt for us. We just loop.
                continue;
            }
            Ok(Signal::CtrlD) => {
                // Ctrl-D on an empty line is the canonical "quit"
                // signal at a REPL. Break and let `run` return Ok(())
                // → `main` maps that to `ExitCode::Success` (0).
                break;
            }
            Ok(_) => {
                // `reedline::Signal` is `#[non_exhaustive]`. As of
                // reedline 0.38 the three variants above cover every
                // case the editor emits, but a future reedline release
                // may add new variants (e.g., a `Signal::ExternalBreak`
                // that forge_main's editor.rs already treats like
                // `Success`). Treat unknown variants as a soft
                // "unknown input event" — loop back to the prompt
                // rather than crashing on an upgrade.
                continue;
            }
            Err(e) => {
                // Raw-mode failure, terminal disconnected, etc.
                // Log to stderr so a user debugging a broken terminal
                // config sees the cause, then exit the loop cleanly.
                // Returning `Err(e)` would route through `classify_
                // error` in main.rs to `RuntimeError` (1); for T7.9
                // we prefer a clean exit 0 because the REPL has
                // already done its job (showed the banner + prompt)
                // by the time this fires.
                eprintln!("interactive: read_line error: {e}");
                break;
            }
        }
    }

    Ok(())
}
