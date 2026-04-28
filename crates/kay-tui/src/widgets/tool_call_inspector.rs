//! ToolCallInspector widget — renders detailed tool call information

use ratatui::{
    widgets::Block,
    Frame,
    layout::Rect,
};

/// A simple tool call inspector widget that displays tool call details.
///
/// In the full implementation this would show the tool name, arguments,
/// result, and duration. For now it's a minimal widget that renders a
/// bordered block.
#[derive(Debug, Default)]
pub struct ToolCallInspector;

impl ToolCallInspector {
    /// Create a new ToolCallInspector widget
    pub fn new() -> Self {
        Self
    }
}

impl ratatui::widgets::Widget for ToolCallInspector {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::bordered()
            .title(" Tool Call Inspector ");
        block.render(area, buf);
    }
}