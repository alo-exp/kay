//! kay-tui — full-screen terminal UI frontend for Kay.
//!
//! Phase 1 placeholder. The actual ratatui-based TUI is built in the
//! dedicated TUI phase (see ROADMAP.md). Kay's `kay-cli` (rebranded
//! `forge_main`) already provides interactive CLI features (completer,
//! editor, banner, stream renderer); this crate extends that surface
//! with a distinct full-screen TUI when user preference calls for it.

fn main() {
    eprintln!(
        "kay-tui: not yet implemented — full-screen TUI lands in the TUI phase. Use `kay` (CLI) for now."
    );
    // EX_UNAVAILABLE from sysexits.h (69) — "service unavailable". Matches the
    // semantic: the binary exists but the feature isn't implemented yet. Exit 2
    // would signal "usage error" and conflicts with POSIX conventions.
    std::process::exit(69);
}
