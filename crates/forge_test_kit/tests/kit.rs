/// Test that the fixture! macro loads a test fixture file correctly
#[tokio::test]
async fn fixture_macro_loads_file() {
    // Load a fixture file using the fixture! macro
    // The macro expects a path relative to the crate's manifest directory
    let content = forge_test_kit::fixture!("tests/fixtures/sample.txt");

    // Verify the content matches what we expect
    assert_eq!(content.trim(), "hello");
}
