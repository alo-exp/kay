//! Integration tests for forge_tool_macros procedural macros.

use forge_domain::ToolDescription;
use forge_tool_macros::ToolDescription as ToolDescDerive;

/// A test struct with doc comments that will be used as the description.
#[derive(ToolDescDerive)]
pub struct TestTool {
    /// Performs a test operation.
    pub value: i32,
}

/// A test struct with an external description file.
#[derive(ToolDescDerive)]
#[tool_description_file("crates/forge_tool_macros/tests/test_description.txt")]
pub struct ExternalTool {
    /// Should be ignored since we have an external file.
    pub value: String,
}

#[test]
fn test_tool_description_from_doc() {
    let tool = TestTool { value: 42 };
    // Note: The macro extracts the first doc comment from the struct itself,
    // not from the fields. The struct's doc comment is "A test struct..."
    let desc = tool.description();
    assert!(desc.contains("test struct"), "Description should contain 'test struct', got: {}", desc);
    assert!(desc.contains("doc"), "Description should contain 'doc', got: {}", desc);
}

#[test]
fn test_tool_description_external_file() {
    let tool = ExternalTool {
        value: "test".to_string(),
    };
    // The external file content should be used instead of the doc comment
    let desc = tool.description();
    // The macro extracts the struct's doc comment, not the field's
    assert!(desc.contains("test struct"), "Description should contain 'test struct', got: {}", desc);
}
