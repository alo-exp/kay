//! Streaming markdown renderer for terminal output.
//!
//! Provides Forge-compatible markdown rendering with:
//! - Heading hierarchy (H1 uppercase bold, H2 bold, H3+ italic)
//! - Inline formatting (bold, italic, code)
//! - Bullet and numbered lists
//! - Tables with borders
//! - Code blocks
//! - Reasoning block dimming

use std::io;

use regex::Regex;

use crate::theme::{bold, bright, dimmed, italic};

/// Streaming markdown renderer.
pub struct MarkdownRenderer {
    width: usize,
    reasoning_mode: bool,
    line_buffer: String,
}

impl MarkdownRenderer {
    /// Create a new renderer with default width.
    pub fn new() -> Self {
        Self::with_width(80)
    }

    /// Create a new renderer with specified width.
    pub fn with_width(width: usize) -> Self {
        Self {
            width,
            reasoning_mode: false,
            line_buffer: String::new(),
        }
    }

    /// Enable or disable reasoning mode (dimmed output).
    pub fn set_reasoning(&mut self, enabled: bool) {
        self.reasoning_mode = enabled;
    }

    /// Push text to the renderer. Lines are rendered when complete.
    pub fn push(&mut self, text: &str) -> io::Result<()> {
        self.line_buffer.push_str(text);
        
        // Process complete lines
        while let Some(pos) = self.line_buffer.find('\n') {
            let line = self.line_buffer[..pos].to_string();
            self.line_buffer = self.line_buffer[pos + 1..].to_string();
            self.render_line(&line)?;
        }
        
        Ok(())
    }

    /// Flush any remaining buffered content.
    pub fn finish(&mut self) -> io::Result<()> {
        if !self.line_buffer.is_empty() {
            self.render_line(&self.line_buffer)?;
            self.line_buffer.clear();
        }
        Ok(())
    }

    /// Render a complete line.
    fn render_line(&self, line: &str) -> io::Result<()> {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            println!();
            return Ok(());
        }

        // Check for heading
        if let Some(level) = Self::get_heading_level(trimmed) {
            self.render_heading(level, trimmed)?;
            return Ok(());
        }

        // Check for list item
        if Self::is_list_item(trimmed) {
            self.render_list_item(trimmed)?;
            return Ok(());
        }

        // Check for table row
        if Self::is_table_row(trimmed) {
            self.render_table_row(trimmed)?;
            return Ok(());
        }

        // Regular paragraph - render inline markdown
        let rendered = self.render_inline(trimmed);
        self.write_line(&rendered)
    }

    /// Get heading level (1-6) or None.
    fn get_heading_level(line: &str) -> Option<u8> {
        if line.starts_with("######") { Some(6) }
        else if line.starts_with("#####") { Some(5) }
        else if line.starts_with("####") { Some(4) }
        else if line.starts_with("###") { Some(3) }
        else if line.starts_with("##") { Some(2) }
        else if line.starts_with('#') { Some(1) }
        else { None }
    }

    /// Check if line is a list item.
    fn is_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with('-') || trimmed.starts_with('*') || 
        trimmed.starts_with('+') ||
        Regex::new(r"^\d+\.\s").unwrap().is_match(trimmed)
    }

    /// Check if line is a table row.
    fn is_table_row(line: &str) -> bool {
        line.contains('|')
    }

    /// Render a heading.
    fn render_heading(&self, level: u8, line: &str) -> io::Result<()> {
        let content = line.trim_start_matches(|c: char| c == '#').trim();
        let prefix = "#".repeat(level as usize);
        
        let rendered = match level {
            1 => {
                // H1: UPPERCASE, bold, with dimmed prefix
                let upper = content.to_uppercase();
                format!("\n{}  {}", dimmed(&format!("{prefix} ")), bold(&upper))
            }
            2 => {
                // H2: Bold, with dimmed prefix
                format!("\n{}  {}", dimmed(&format!("{prefix} ")), bold(content))
            }
            _ => {
                // H3+: Italic, with dimmed prefix
                format!("\n{}  {}", dimmed(&format!("{prefix} ")), italic(content))
            }
        };
        
        println!("{}", rendered);
        Ok(())
    }

    /// Render a list item.
    fn render_list_item(&self, line: &str) -> io::Result<()> {
        let trimmed = line.trim();
        
        // Determine bullet character and content
        let (bullet_str, content_str): (&str, &str) = if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            ("•", &trimmed[2..])
        } else if let Some(cap) = Regex::new(r"^(\d+)\.\s+(.*)").unwrap().captures(trimmed) {
            let num = cap.get(1).map(|m| m.as_str()).unwrap_or("1");
            let rest = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            ("", &format!("{}. {}", num, rest))
        } else {
            ("•", &trimmed[1..].trim())
        };

        let rendered = self.render_inline(content_str);
        let bullet = bullet_str;
        
        if self.reasoning_mode {
            println!("  {} {}", dimmed(bullet), dimmed(&rendered));
        } else {
            println!("  {} {}", dimmed(bullet), rendered);
        }
        
        Ok(())
    }

    /// Render a table row.
    fn render_table_row(&self, line: &str) -> io::Result<()> {
        let cells: Vec<&str> = line.split('|')
            .filter(|s| !s.trim().is_empty() || s.is_empty())
            .map(|s| s.trim())
            .collect();
        
        if cells.is_empty() || cells.iter().all(|s| s.chars().all(|c| c == '-' || c == ':')) {
            // Separator row
            println!("{}", dimmed(&"─".repeat(self.width.min(60))));
            return Ok(());
        }

        let rendered_cells: Vec<String> = cells.iter()
            .map(|c| {
                let rendered = self.render_inline(c);
                if self.reasoning_mode {
                    format!(" {}", dimmed(&rendered))
                } else {
                    format!(" {}", rendered)
                }
            })
            .collect();

        let border: String = if self.reasoning_mode {
            dimmed("│")
        } else {
            "│".to_string()
        };

        println!("{}", format!("{}{}{}", border, rendered_cells.join(" │"), border));
        Ok(())
    }

    /// Render inline markdown (bold, italic, code, links).
    fn render_inline(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Bold: **text** or __text__
        let bold_re = Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap();
        result = bold_re.replace_all(&result, |caps: &regex::Captures| {
            let content = caps.get(1).or(caps.get(2)).map(|m| m.as_str()).unwrap_or("");
            bold(content)
        }).to_string();

        // Italic: *text* (simplified - not inside bold, which was already processed)
        // Use a simpler regex that matches *word* pattern
        let italic_re = Regex::new(r"\*([^*]+)\*").unwrap();
        result = italic_re.replace_all(&result, |caps: &regex::Captures| {
            let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            italic(content)
        }).to_string();

        // Inline code: `code`
        let code_re = Regex::new(r"`([^`]+)`").unwrap();
        result = code_re.replace_all(&result, |caps: &regex::Captures| {
            let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            bright(content)
        }).to_string();

        // Links: [text](url) -> text (dimmed)
        let link_re = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
        result = link_re.replace_all(&result, |caps: &regex::Captures| {
            let text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            dimmed(text)
        }).to_string();

        // Apply reasoning mode to entire line if enabled
        if self.reasoning_mode {
            result = dimmed(&result);
        }

        result
    }

    /// Write a line to stdout.
    fn write_line(&self, line: &str) -> io::Result<()> {
        if self.reasoning_mode {
            println!("{}", dimmed(line));
        } else {
            println!("{}", line);
        }
        Ok(())
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_level() {
        assert_eq!(MarkdownRenderer::get_heading_level("# Hello"), Some(1));
        assert_eq!(MarkdownRenderer::get_heading_level("## Hello"), Some(2));
        assert_eq!(MarkdownRenderer::get_heading_level("### Hello"), Some(3));
        assert_eq!(MarkdownRenderer::get_heading_level("Plain text"), None);
    }

    #[test]
    fn test_is_list_item() {
        assert!(MarkdownRenderer::is_list_item("- item"));
        assert!(MarkdownRenderer::is_list_item("* item"));
        assert!(MarkdownRenderer::is_list_item("1. item"));
        assert!(!MarkdownRenderer::is_list_item("plain text"));
    }

    #[test]
    fn test_inline_bold() {
        let r = MarkdownRenderer::new();
        assert_eq!(r.render_inline("**bold**"), "\x1b[1mbold\x1b[0m");
    }

    #[test]
    fn test_inline_code() {
        let r = MarkdownRenderer::new();
        assert_eq!(r.render_inline("`code`"), "\x1b[1;97mcode\x1b[0m");
    }
}
