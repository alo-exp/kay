// Trybuild compile test — verify the proc-macro compiles on a simple struct
// This uses trybuild to verify that #[derive(ToolDescription)] works correctly

use std::path::Path;

/// Test that #[derive(ToolDescription)] compiles on basic structs
#[test]
fn derive_tool_description_compiles() {
    let t = trybuild::TestCases::new();

    // Test 1: Basic struct with doc comment derives ToolDescription
    t.pass(Path::new("tests/pass/derive_basic.rs"));

    // Test 2: Struct with generics
    t.pass(Path::new("tests/pass/derive_generics.rs"));

    // Run the tests
    t.run();
}
