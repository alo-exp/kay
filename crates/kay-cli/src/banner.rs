//! Startup banner for `kay` (ported from `forge_main::banner`).
//!
//! # T7.4 scope
//!
//! Renders the Kay wordmark + version + command-tip table on
//! interactive-fallback startup (and on `kay run` when we later decide
//! to mirror ForgeCode's "banner before stream" behavior). The code is
//! a brand-swapped port of `crates/forge_main/src/banner.rs`:
//!
//!   * ASCII wordmark is Kay (not Forge) — `include_str!("banner")`
//!     loads the sibling asset file.
//!   * `FORGE_BANNER` env var → `KAY_BANNER`.
//!   * Version label reads `CARGO_PKG_VERSION` directly rather than
//!     going through `forge_tracker::VERSION` — `forge_tracker` is
//!     intentionally NOT in the kay-cli dep graph (DL-2: telemetry is
//!     an allowlisted ForgeCode surface we don't carry forward).
//!   * Zsh-plugin encouragement dropped — Kay has no zsh plugin today;
//!     re-introducing it would lie to users. If/when we ship one the
//!     tip can come back.
//!   * Agent-switch tips use `:kay or :muse` — the `:forge` legacy
//!     persona identifier is banned from user-visible strings per the
//!     DL-1 clean-room guard (internal `Persona::load("forge")` in
//!     `run.rs` is unchanged; the rename to `Persona::load("kay")` is
//!     tracked as a Phase 10 residual).
//!
//! # Parity-diff contract (T7.11)
//!
//! `tests/cli_e2e.rs::interactive_parity_diff` reads the baseline
//! forgecode banner fixture (T7.10) and applies these normalizations
//! before comparing to Kay's actual stdout:
//!
//!   * "ForgeCode" → "Kay"
//!   * "forgecode" → "kay"
//!   * "forge" → "kay"
//!
//! That means: every `forge` / `ForgeCode` substring the ForgeCode
//! banner emitted must have a `kay` / `Kay` counterpart at the exact
//! same structural position in Kay's output. Lines we REMOVE (like
//! the zsh tip) must also be absent in the fixture after normalization
//! — T7.10 fixture capture is the source of truth for the baseline
//! shape.

use std::io;

use colored::Colorize;

/// Kay ASCII wordmark asset. File lives as a sibling to this source
/// file (no extension) so `include_str!` reads it verbatim without
/// adding a .txt suffix in the tree.
const BANNER: &str = include_str!("banner");

// Note: ForgeCode's `banner.rs` also defined a `DisplayBox` helper
// that rendered a bordered message box — used for the zsh-plugin
// encouragement line. Kay has no zsh plugin (yet), so we dropped the
// call site AND the helper to keep the port lean. T7.9 (reedline
// REPL) may reintroduce a similar box util for in-REPL help/error
// rendering; at that point re-port from `forge_main::banner` with no
// brand changes (the box characters are neutral).

/// Displays the Kay startup banner: wordmark + version + command
/// tips.
///
/// # Arguments
///
/// * `cli_mode` — if `true`, show only CLI-relevant commands (used
///   on `kay run` when we mirror ForgeCode's banner-before-stream
///   behavior). If `false`, show the full interactive-mode tip set.
///
/// # Env
///
/// * `KAY_BANNER` — optional override wordmark. Empty string is
///   ignored (falls back to the bundled asset). Set to any non-empty
///   string to replace the wordmark block (tips are always appended
///   after, regardless of override).
pub fn display(cli_mode: bool) -> io::Result<()> {
    // Custom banner override — empty string treated as "unset" to match
    // ForgeCode behavior (users who accidentally export `KAY_BANNER=`
    // shouldn't get a blank wordmark).
    let mut banner = std::env::var("KAY_BANNER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| BANNER.to_string());

    // Version label always shows. Sourced directly from Cargo manifest
    // — avoids forge_tracker dependency (DL-2: telemetry surface not
    // carried forward).
    let version_label = ("Version:", env!("CARGO_PKG_VERSION"));

    // Command tips. `:` is the canonical prefix in both modes.
    // The `:kay or :muse` tip is a DL-1 rebrand of ForgeCode's
    // `:forge or :muse` — persona registry still exposes the legacy
    // identifier internally (Phase 10 residual: rename
    // `Persona::load("forge")` → `Persona::load("kay")` once schema
    // migration is safe).
    let tips: Vec<(&str, &str)> = if cli_mode {
        // `kay run` — only commands meaningful in a one-shot context.
        vec![
            ("New conversation:", ":new"),
            ("Get started:", ":info, :conversation"),
            ("Switch model:", ":model"),
            ("Switch provider:", ":provider"),
            ("Switch agent:", ":<agent_name> e.g. :kay or :muse"),
        ]
    } else {
        // `kay` (no args) → interactive REPL — full command set.
        vec![
            ("New conversation:", ":new"),
            ("Get started:", ":info, :usage, :help, :conversation"),
            ("Switch model:", ":model"),
            ("Switch agent:", ":kay or :muse or :agent"),
            ("Update:", ":update"),
            ("Quit:", ":exit or <CTRL+D>"),
        ]
    };

    // Version row is always first; tips follow. Chained in one array
    // so right-alignment uses the same `max_width` for every row.
    let labels: Vec<(&str, &str)> = std::iter::once(version_label).chain(tips).collect();

    // Right-align label keys. Using .len() (byte count) is safe here
    // because all label keys are ASCII — if we add non-ASCII labels in
    // the future, switch to `console::measure_text_width` for both
    // max_width computation and formatting.
    let max_width = labels.iter().map(|(key, _)| key.len()).max().unwrap_or(0);

    for (key, value) in &labels {
        banner.push_str(
            format!(
                "\n{}{}",
                format!("{key:>max_width$} ").dimmed(),
                value.cyan()
            )
            .as_str(),
        );
    }

    println!("{banner}\n");

    Ok(())
}
