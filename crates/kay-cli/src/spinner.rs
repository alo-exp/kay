//! Spinner / Progress Indicator for Kay CLI
//!
//! Provides animated progress indicators for long-running operations.

use std::io::{self, Write};
use std::time::Duration;

/// Spinner animation characters
const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Progress indicator with spinner animation.
pub struct Spinner {
    message: String,
    current_frame: usize,
    running: bool,
}

impl Spinner {
    /// Create a new spinner with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            current_frame: 0,
            running: false,
        }
    }

    /// Start the spinner animation.
    pub fn start(&mut self) {
        self.running = true;
        self.tick();
    }

    /// Update the spinner frame (call this in a loop).
    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        
        // Write spinner frame
        print!("\r{} {}", SPINNER_CHARS[self.current_frame], self.message);
        io::stdout().flush().ok();
        
        // Advance frame
        self.current_frame = (self.current_frame + 1) % SPINNER_CHARS.len();
    }

    /// Stop the spinner and clear the line.
    pub fn stop(&mut self) {
        self.running = false;
        print!("\r");
        for _ in 0..60 {
            print!(" ");
        }
        print!("\r");
        io::stdout().flush().ok();
    }

    /// Update the message.
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Stop and show completion message.
    pub fn finish_with(&mut self, message: impl Into<String>) {
        self.stop();
        println!("✓ {}", message.into());
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.running {
            self.stop();
        }
    }
}

/// Create a simple progress bar.
#[allow(dead_code)]
pub struct ProgressBar {
    current: u32,
    total: u32,
    width: usize,
}

impl ProgressBar {
    /// Create a new progress bar.
    pub fn new(total: u32, width: usize) -> Self {
        Self {
            current: 0,
            total,
            width,
        }
    }

    /// Set the current progress.
    pub fn set_progress(&mut self, current: u32) {
        self.current = current.min(self.total);
    }

    /// Increment progress.
    pub fn inc(&mut self) {
        self.current = (self.current + 1).min(self.total);
    }

    /// Render the progress bar.
    pub fn render(&self) -> String {
        let filled = if self.total > 0 {
            ((self.current as f64 / self.total as f64) * self.width as f64) as usize
        } else {
            0
        };
        let empty = self.width.saturating_sub(filled);
        
        format!(
            "[{}{}] {}/{}",
            "█".repeat(filled),
            "░".repeat(empty),
            self.current,
            self.total
        )
    }

    /// Print the progress bar to stdout.
    #[allow(dead_code)]
    pub fn print(&self) {
        print!("\r{}", self.render());
        io::stdout().flush().ok();
    }

    /// Finish and print newline.
    #[allow(dead_code)]
    pub fn finish(&self) {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_new() {
        let s = Spinner::new("Working...");
        assert_eq!(s.message, "Working...");
    }

    #[test]
    fn test_progress_bar() {
        let mut pb = ProgressBar::new(100, 20);
        pb.set_progress(50);
        let rendered = pb.render();
        assert!(rendered.contains("50/100"));
    }
}
