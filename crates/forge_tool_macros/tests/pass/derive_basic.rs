//! Basic struct with doc comment derives ToolDescription

use forge_tool_macros::ToolDescription;

/// A simple test struct
#[derive(ToolDescription)]
/// This is a test struct for the ToolDescription derive macro.
struct TestStruct {
    /// A field
    field: String,
}

fn main() {
    let _ = TestStruct {
        field: "test".to_string(),
    };
    // Verify the description method is available
    println!("TestStruct implements ToolDescription");
}
