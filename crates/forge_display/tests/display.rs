use forge_display::SyntaxHighlighter;

#[test]
fn syntax_highlighter_default_constructs() {
    // SyntaxHighlighter has a Default impl — verifying it constructs without panic
    let highlighter = SyntaxHighlighter::default();
    // Verify highlight() produces non-empty output for valid syntax
    let result = highlighter.highlight("fn main() {}", "rust");
    assert!(!result.is_empty(), "highlighted output should not be empty");
}

#[test]
fn syntax_highlighter_highlight_unknown_lang() {
    let highlighter = SyntaxHighlighter::default();
    // Unknown language should fall back to plain text (non-empty)
    let result = highlighter.highlight("hello world", "unknownlang123");
    assert!(!result.is_empty(), "fallback output should be non-empty");
}
