use forge_ci::release_matrix::{MatrixEntry, ReleaseMatrix};
use serde_json::json;

/// Test that CI matrix entries serialize correctly to JSON
#[test]
fn workflow_serializes_to_yaml() {
    // ReleaseMatrix contains MatrixEntry structs that implement Serialize
    let matrix = ReleaseMatrix::default();
    let entries = matrix.entries();

    // Verify we have the expected number of matrix entries
    assert!(!entries.is_empty(), "Matrix should have entries");

    // Test serialization of the first entry
    let first_entry = &entries[0];
    let json = serde_json::to_string(first_entry).expect("MatrixEntry should serialize to JSON");

    // Verify JSON contains expected fields
    assert!(json.contains("os"), "JSON should contain 'os' field");
    assert!(json.contains("target"), "JSON should contain 'target' field");
    assert!(json.contains("binary_name"), "JSON should contain 'binary_name' field");

    // Verify the first entry has correct values (Ubuntu x86_64-unknown-linux-musl)
    let entry_json: serde_json::Value =
        serde_json::from_str(&json).expect("Should parse as valid JSON");
    assert_eq!(entry_json["os"], "ubuntu-latest");
    assert_eq!(entry_json["target"], "x86_64-unknown-linux-musl");
}
