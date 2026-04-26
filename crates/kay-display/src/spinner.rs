//! Spinner for progress indication with thread-safe operations.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::theme::SPINNER_CHARS;

/// Shared spinner that handles thread-safe operations.
pub struct SharedSpinner {
    inner: Arc<Mutex<SpinnerState>>,
}

struct SpinnerState {
    message: Option<String>,
    current_frame: usize,
    started: Option<Instant>,
}

impl SharedSpinner {
    /// Create a new shared spinner.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SpinnerState {
                message: None,
                current_frame: 0,
                started: None,
            })),
        }
    }

    /// Start the spinner with a message.
    pub fn start(&self, message: Option<&str>) -> io::Result<()> {
        let mut state = self.inner.lock().unwrap();
        state.message = message.map(String::from);
        state.current_frame = 0;
        state.started = Some(Instant::now());
        self.render(&state)
    }

    /// Stop the spinner with an optional completion message.
    pub fn stop(&self, message: Option<String>) -> io::Result<()> {
        let mut state = self.inner.lock().unwrap();
        state.started = None;
        
        // Clear the spinner line
        print!("\r");
        io::stdout().flush()?;
        
        if let Some(msg) = message {
            println!("✓ {}", msg);
        }
        
        Ok(())
    }

    /// Clear the spinner line (for writing content).
    pub fn clear(&self) -> io::Result<()> {
        print!("\r");
        for _ in 0..80 {
            print!(" ");
        }
        print!("\r");
        io::stdout().flush()
    }

    /// Tick the spinner animation.
    pub fn tick(&self) -> io::Result<()> {
        let mut state = self.inner.lock().unwrap();
        if state.started.is_none() {
            return Ok(());
        }
        state.current_frame = (state.current_frame + 1) % SPINNER_CHARS.len();
        self.render(&state)
    }

    fn render(&self, state: &SpinnerState) -> io::Result<()> {
        if let Some(started) = state.started {
            let elapsed = started.elapsed().as_secs();
            let spinner = SPINNER_CHARS[state.current_frame];
            let msg = state.message.as_deref().unwrap_or("");
            print!("\r{} {} ({elapsed}s) ", spinner, msg);
            io::stdout().flush()?;
        }
        Ok(())
    }
}

impl Default for SharedSpinner {
    fn default() -> Self {
        Self::new()
    }
}
