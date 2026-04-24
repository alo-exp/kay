//! kay-tui — full-screen terminal UI frontend for Kay.
//!
//! Phase 1 placeholder. The actual ratatui-based TUI is built in the
//! dedicated TUI phase (see ROADMAP.md). Kay's `kay-cli` (rebranded
//! `forge_main`) already provides interactive CLI features (completer,
//! editor, banner, stream renderer); this crate extends that surface
//! with a distinct full-screen TUI when user preference calls for it.

use kay_tui::ui;

fn main() {
    let app = ui::App::new();
    std::process::exit(ui::run(app).unwrap_or(1));
}
