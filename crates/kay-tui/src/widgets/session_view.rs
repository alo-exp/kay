//! SessionView widget — renders the agent session timeline

use ratatui::{
    widgets::Block,
    Frame,
    layout::Rect,
};

/// A simple session view widget that displays a bordered container.
///
/// In the full implementation this would show the agent event timeline,
/// token usage, cost meter, and tool call inspector. For now it's a
/// minimal widget that renders a bordered block.
#[derive(Debug, Default)]
pub struct SessionView;

impl SessionView {
    /// Create a new SessionView widget
    pub fn new() -> Self {
        Self
    }
}

impl ratatui::widgets::Widget for SessionView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::bordered()
            .title(" Kay Session ");
        block.render(area, buf);
    }
}