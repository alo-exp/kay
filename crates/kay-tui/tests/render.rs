//! kay-tui render tests — W-7.2 GREEN
//!
//! These tests verify that the TUI components render without panic
//! using ratatui's TestBackend.

use kay_tui::widgets::{SessionView, ToolCallInspector};

/// W-7.2 GREEN: SessionView widget renders without panic
#[test]
fn session_view_renders_without_panic() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let widget = SessionView::new();
        f.render_widget(widget, f.size());
    }).unwrap();
    // No panic = pass
}

/// W-7.2 GREEN: ToolCallInspector widget renders without panic
#[test]
fn tool_call_inspector_renders_without_panic() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let widget = ToolCallInspector::new();
        f.render_widget(widget, f.size());
    }).unwrap();
    // No panic = pass
}