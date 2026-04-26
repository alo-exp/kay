//! Theme and styling for markdown output.

/// ANSI styling helpers
pub struct Theme;

impl Theme {
    /// Get terminal width
    pub fn terminal_width() -> usize {
        terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80)
    }
}

/// Apply bold styling
pub fn bold(s: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", s)
}

/// Apply dimmed styling
pub fn dimmed(s: &str) -> String {
    format!("\x1b[2m{}\x1b[0m", s)
}

/// Apply italic styling
pub fn italic(s: &str) -> String {
    format!("\x1b[3m{}\x1b[0m", s)
}

/// Apply bright (for code)
pub fn bright(s: &str) -> String {
    format!("\x1b[1;97m{}\x1b[0m", s)
}

/// Apply cyan for links
pub fn cyan(s: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", s)
}

/// Apply yellow for warnings
pub fn yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

/// Apply red for errors
pub fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

/// Apply green for success
pub fn green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

/// Reset all styling
pub fn reset() -> &'static str {
    "\x1b[0m"
}

/// Spinner characters for progress indication
pub static SPINNER_CHARS: [&str; 10] = [
    "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
];
