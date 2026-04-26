//! Kay CLI Output Renderer.
//!
//! Renders streaming agent events into clean, readable output for users.
//! Handles text accumulation, partial updates, and final answer display.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

        TurnOutput {
            text: self.buffer,
            completed: true,
        }
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
    /// Atomic flag for cancellation (not yet used).
    cancelled: Arc<AtomicBool>,
}

impl StreamingWriter {
    /// Creates a new streaming writer.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            completed: false,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates with a cancellation flag.
    pub fn with_cancellation(cancelled: Arc<AtomicBool>) -> Self {
        Self {
            text: String::new(),
            completed: false,
            cancelled,
        }
    }

    /// Handles a text_delta event, printing and accumulating.
    pub fn text_delta(&mut self, content: &str) {
        if content.is_empty() {
            return;
        }

        print!("{}", content);
        io::stdout().flush().ok();
        self.text.push_str(content);
    }

    /// Handles the final task_complete event.
    pub fn task_complete(&mut self) {
        println!(); // Move to new line
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