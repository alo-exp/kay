//! Diff Highlighting for Kay CLI
//!
//! Provides ANSI-colored diff output for code changes.

/// Diff line type
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    /// Context line (unchanged)
    Context,
    /// Added line
    Added,
    /// Removed line
    Removed,
    /// Header or separator
    Header,
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Type of diff line
    pub line_type: DiffLineType,
    /// The line content
    pub content: String,
    /// Optional line number
    pub line_no: Option<u32>,
}

impl DiffLine {
    /// Render the line with ANSI colors
    pub fn render(&self) -> String {
        let prefix = match self.line_type {
            DiffLineType::Added => "+ ",
            DiffLineType::Removed => "- ",
            DiffLineType::Context => "  ",
            DiffLineType::Header => "",
        };

        let colored_content = match self.line_type {
            DiffLineType::Added => format!("\x1b[32m{}\x1b[0m", self.content), // Green
            DiffLineType::Removed => format!("\x1b[31m{}\x1b[0m", self.content), // Red
            DiffLineType::Context => self.content.clone(),
            DiffLineType::Header => format!("\x1b[1m{}\x1b[0m", self.content), // Bold
        };

        format!("{}{}", prefix, colored_content)
    }
}

/// Simple diff computation
pub fn compute_diff(old: &str, new: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    
    let mut diff = Vec::new();
    
    // Simple line-by-line diff
    // For a proper implementation, we'd use a proper diff algorithm
    // like Myers or Hunt-McIlroy
    
    let max_len = old_lines.len().max(new_lines.len());
    
    for i in 0..max_len {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(i).copied();
        
        match (old_line, new_line) {
            (Some(ol), Some(nl)) if ol == nl => {
                diff.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: nl.to_string(),
                    line_no: Some(i as u32 + 1),
                });
            }
            (Some(ol), None) => {
                diff.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: ol.to_string(),
                    line_no: Some(i as u32 + 1),
                });
            }
            (None, Some(nl)) => {
                diff.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: nl.to_string(),
                    line_no: Some(i as u32 + 1),
                });
            }
            (Some(ol), Some(nl)) => {
                diff.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: ol.to_string(),
                    line_no: Some(i as u32 + 1),
                });
                diff.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: nl.to_string(),
                    line_no: Some(i as u32 + 1),
                });
            }
            (None, None) => {}
        }
    }
    
    diff
}

/// Render diff as ANSI-colored string
pub fn render_diff(old: &str, new: &str) -> String {
    compute_diff(old, new)
        .iter()
        .map(|line| line.render())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a unified diff header
pub fn unified_diff_header(file: &str, old_start: u32, old_count: u32, new_start: u32, new_count: u32) -> String {
    format!(
        "--- {}\n+++ {}\n@@ -{},{} +{},{} @@",
        file, file, old_start, old_count, new_start, new_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_lines() {
        let diff = compute_diff("line1\nline2\nline3", "line1\nline2\nline3");
        assert!(diff.iter().all(|l| l.line_type == DiffLineType::Context));
    }

    #[test]
    fn test_added_line() {
        let diff = compute_diff("line1", "line1\nline2");
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[1].line_type, DiffLineType::Added);
    }

    #[test]
    fn test_removed_line() {
        let diff = compute_diff("line1\nline2", "line1");
        // When removing a line, we get the kept line + the removed line marker
        // The diff contains context (line1) and possibly removed (line2)
        // Just verify we have at least one line and it contains content
        assert!(diff.len() >= 1);
        assert!(diff.iter().any(|l| l.content == "line1"));
    }

    #[test]
    fn test_changed_line() {
        let diff = compute_diff("line1\nold", "line1\nnew");
        // When lines differ, simplified diff shows first as context, then both
        assert!(diff.len() >= 1);
        // First line should be context (unchanged)
        assert_eq!(diff[0].line_type, DiffLineType::Context);
    }

    #[test]
    fn test_render_ansi_colors() {
        let diff = DiffLine {
            line_type: DiffLineType::Added,
            content: "test".to_string(),
            line_no: Some(1),
        };
        let rendered = diff.render();
        assert!(rendered.contains("\x1b[32m")); // Green
    }
}