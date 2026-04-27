//! Kay CLI Output Renderer.
//!
//! Renders streaming agent events into clean, readable output for users.
//! Uses kay-display for Forge-compatible markdown rendering.
//!
//! # Markdown Rendering (Forge-compatible)
//!
//! Text deltas are processed through kay_display::MarkdownRenderer:
//! - `# Heading` → UPPERCASE bold (H1), Bold (H2), Italic (H3+)
//! - `**bold**` → ANSI bold
//! - `*italic*` → ANSI italic  
//! - `` `code` `` → ANSI bright
//! - `- item` → bullet point (•)
//! - `|col1|col2|` → ANSI table with borders
//! - `[link](url)` → dimmed link text
//!
//! Reasoning content is automatically dimmed when detected.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kay_display::MarkdownRenderer;

use crate::markdown::render_markdown;

/// Rendered output for a single turn.
pub struct TurnOutput {
    /// Accumulated text content.
    pub text: String,
    /// Whether the turn completed successfully.
    pub completed: bool,
}

/// Output renderer that collects streaming events and displays them nicely.
///
/// # Example
/// ```
/// let mut renderer = OutputRenderer::new();
/// renderer.add_text_delta("Hello");
/// renderer.add_text_delta(", world!");
/// renderer.finish();
/// println!("{}", renderer.output()); // prints "Hello, world!"
/// ```
pub struct OutputRenderer {
    buffer: String,
    cursor_position: usize,
    completed: bool,
    verbose: bool,
}

impl OutputRenderer {
    /// Creates a new output renderer.
    pub fn new(verbose: bool) -> Self {
        Self {
            buffer: String::new(),
            cursor_position: 0,
            completed: false,
            verbose,
        }
    }

    /// Adds a text delta event.
    /// For verbose mode, appends to buffer and redraws.
    /// For quiet mode (default), accumulates silently until finish.
    pub fn add_text_delta(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        if self.verbose {
            // Redraw mode: clear previous output and show updated text
            self.buffer.push_str(text);
            self.redraw();
        } else {
            // Quiet mode: accumulate text silently
            self.buffer.push_str(text);
        }
    }

    /// Redraws the current buffer, clearing previous line content.
    fn redraw(&self) {
        // Clear line and move cursor to beginning
        print!("\r\x1b[K");
        print!("{}", self.buffer);
        io::stdout().flush().ok();
    }

    /// Finishes the rendering, moving to a new line.
    pub fn finish(mut self) -> TurnOutput {
        if self.verbose {
            // Move to new line after rendering
            println!();
        }

        self.completed = true;

        TurnOutput { text: self.buffer, completed: true }
    }

    /// Returns the accumulated text.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Returns true if rendering is complete.
    pub fn is_complete(&self) -> bool {
        self.completed
    }
}

/// Streaming output handler that prints events to stdout and collects final text.
///
/// Use this for interactive mode to show streaming response while accumulating
/// the final text for potential follow-up.
pub struct StreamingWriter {
    /// Accumulated text from all text_delta events.
    pub text: String,
    /// Whether the stream completed successfully.
    pub completed: bool,
    /// Atomic flag for cancellation.
    cancelled: Arc<AtomicBool>,
    /// Markdown renderer for Forge-compatible output.
    renderer: MarkdownRenderer,
}

impl StreamingWriter {
    /// Creates a new streaming writer.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            completed: false,
            cancelled: Arc::new(AtomicBool::new(false)),
            renderer: MarkdownRenderer::new(),
        }
    }

    /// Creates with a cancellation flag.
    pub fn with_cancellation(cancelled: Arc<AtomicBool>) -> Self {
        Self {
            text: String::new(),
            completed: false,
            cancelled,
            renderer: MarkdownRenderer::new(),
        }
    }

    /// Enable reasoning mode (dimmed output).
    pub fn set_reasoning(&mut self, enabled: bool) {
        self.renderer.set_reasoning(enabled);
    }

    /// Handles a text_delta event, printing and accumulating.
    /// Content is rendered through kay_display MarkdownRenderer for ANSI terminal formatting.
    pub fn text_delta(&mut self, content: &str) {
        if content.is_empty() {
            return;
        }

        // Push to kay_display renderer for Forge-compatible markdown output
        self.renderer.push(content).ok();
        self.text.push_str(content);
    }

    /// Handles the final task_complete event.
    pub fn task_complete(&mut self) {
        self.renderer.finish().ok();
        self.completed = true;
    }

    /// Returns the accumulated text.
    pub fn into_text(self) -> String {
        self.text
    }

    /// Returns true if completed.
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Requests cancellation (future use).
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns true if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for StreamingWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Pretty-print a block of text with optional indentation.
pub fn print_text_block(text: &str, indent: usize) {
    let prefix = " ".repeat(indent);
    for line in text.lines() {
        println!("{}{}", prefix, line);
    }
}

/// Print a separator line.
pub fn print_separator() {
    println!("{}", "─".repeat(40));
}

/// Print a section header.
pub fn print_header(text: &str) {
    println!("\x1b[1m{}\x1b[0m", text); // Bold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_accumulates() {
        let mut r = OutputRenderer::new(false);
        r.add_text_delta("Hello");
        r.add_text_delta(", ");
        r.add_text_delta("world!");
        let out = r.finish();
        assert_eq!(out.text, "Hello, world!");
        assert!(out.completed);
    }

    #[test]
    fn test_streaming_writer() {
        let mut w = StreamingWriter::new();
        w.text_delta("Test");
        w.text_delta("ing");
        w.task_complete();
        assert_eq!(w.text, "Testing");
        assert!(w.is_completed());
    }
}
