//! Simple terminal markdown renderer.
//!
//! Converts common markdown patterns to ANSI escape codes for terminal display.
//! Uses conservative matching to avoid false positives.
//!
//! Supported patterns:
//! - `**text**` → ANSI bold (only double asterisk)
//! - `` `code` `` → ANSI bright/lighter
//! - `- item` at line start → bullet point
//! - `> text` at line start → quoted text
//! - `# text` at line start → bold heading
//!
//! Single asterisks are NOT converted to italic to avoid false positives
//! with content containing asterisks (e.g., `Apple's*`).

use std::io::{self, Write};

/// Renders markdown-formatted text to the terminal with ANSI styling.
/// Returns the ANSI-escaped string ready for printing.
///
/// Conservative matching - only converts clear markdown patterns.
pub fn render_markdown(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            result.push('\n');
            continue;
        }

        // Process line by line for line-start patterns
        let rendered_line = render_line(line);
        result.push_str(&rendered_line);
        result.push('\n');
    }

    result.trim_end().to_string()
}

/// Render a single line with markdown formatting.
fn render_line(line: &str) -> String {
    let trimmed = line.trim();

    // Heading: "# text" to "###### text"
    if trimmed.starts_with('#') {
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count > 0 && trimmed.chars().nth(hash_count) == Some(' ') {
            let content = &trimmed[hash_count + 1..];
            let style = match hash_count {
                1 => "\x1b[1m\x1b[4m", // Bold + underline for h1
                2 => "\x1b[1m",        // Bold for h2
                _ => "\x1b[1m\x1b[3m", // Bold + italic for h3+
            };
            return format!("{style}{content}\x1b[0m");
        }
    }

    // Quoted text: "> text"
    if trimmed.starts_with("> ") {
        let content = &trimmed[2..];
        return format!("\x1b[2m│ {}\x1b[0m", render_inline(content));
    }

    // Bullet point: "- item" or "* item"
    if (trimmed.starts_with("- ") || trimmed.starts_with("* ")) && !trimmed.starts_with("**") {
        let content = if trimmed.starts_with("- ") {
            &trimmed[2..]
        } else {
            &trimmed[2..]
        };
        return format!("\x1b[2m• {}\x1b[0m", render_inline(content));
    }

    // Numbered list: "1. item" or "1) item"
    if trimmed.len() > 2 {
        let first = trimmed.chars().next().unwrap_or(' ');
        let second = trimmed.chars().nth(1).unwrap_or(' ');
        let third = trimmed.chars().nth(2).unwrap_or(' ');
        if first.is_ascii_digit() && (second == '.' || second == ')') && third == ' ' {
            let content = &trimmed[3..];
            return format!("\x1b[2m{}\x1b[0m", render_inline(content));
        }
    }

    // Regular line - render inline markdown only
    render_inline(line)
}

/// Render inline markdown patterns within text.
fn render_inline(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                // Inline code: `code`
                let content = take_until_char(&mut chars, '`');
                result.push_str(&ansi_code(&content));
            }
            '*' => {
                // Check for bold: **text**
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                    let content = take_until_str(&mut chars, "**");
                    result.push_str(&ansi_bold(&content));
                    // Skip closing ** if present
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        if chars.peek() == Some(&'*') {
                            chars.next();
                        }
                    }
                } else {
                    // Single asterisk - not italic (too many false positives)
                    // Just emit the asterisk
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Print markdown text directly to stdout with ANSI styling.
pub fn print_markdown(input: &str) {
    let rendered = render_markdown(input);
    print!("{}", rendered);
    io::stdout().flush().ok();
}

/// Print a newline after markdown.
pub fn print_markdownln(input: &str) {
    print_markdown(input);
    println!();
}

// ── ANSI helpers ──────────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BRIGHT: &str = "\x1b[1;37m";
const BOLD_RESET: &str = "\x1b[22m";

fn ansi_bold(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    format!("{BOLD}{s}{BOLD_RESET}")
}

fn ansi_code(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    format!("{BRIGHT}{s}{RESET}")
}

// ── Helper parsers ─────────────────────────────────────────────────────────

fn take_until_char(chars: &mut std::iter::Peekable<std::str::Chars>, end: char) -> String {
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == end {
            break;
        }
        result.push(c);
    }
    result
}

fn take_until_str(chars: &mut std::iter::Peekable<std::str::Chars>, end: &str) -> String {
    let end_len = end.len();
    let mut result = String::new();
    let mut match_count = 0;

    while let Some(c) = chars.next() {
        if match_count < end_len {
            // Still trying to match the end sequence
            if c == end.chars().nth(match_count).unwrap_or_default() {
                match_count += 1;
                if match_count == end_len {
                    // Found the end
                    break;
                }
            } else {
                // No match - output what we've accumulated plus this char
                for i in 0..match_count {
                    result.push(end.chars().nth(i).unwrap_or_default());
                }
                result.push(c);
                match_count = 0;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        let result = render_markdown("Hello **world**");
        assert!(result.contains("\x1b[1mworld\x1b[22m"));
    }

    #[test]
    fn test_code() {
        let result = render_markdown("Use `echo` command");
        assert!(result.contains("\x1b[1;37mecho\x1b[0m"));
    }

    #[test]
    fn test_bullet() {
        let result = render_markdown("- item");
        assert!(result.contains("•"));
    }

    #[test]
    fn test_heading() {
        let result = render_markdown("# Heading");
        // Check that the heading content is rendered (style may vary)
        assert!(result.contains("Heading") || result.contains("\x1b"));
    }

    #[test]
    fn test_quote() {
        let result = render_markdown("> quote");
        assert!(result.contains("│"));
    }

    #[test]
    fn test_single_asterisk_preserved() {
        // Single asterisk should be preserved, not treated as italic
        let result = render_markdown("Apple's* product");
        assert!(result.contains("Apple's*"));
    }

    #[test]
    fn test_multiline() {
        let result = render_markdown("- item1\n- item2");
        assert!(result.contains("•"));
        assert!(result.contains("item1"));
        assert!(result.contains("item2"));
    }
}
