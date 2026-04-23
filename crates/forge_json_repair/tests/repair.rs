use forge_json_repair::json_repair;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleJson {
    key: String,
}

#[test]
fn repair_invalid_json() {
    // json_repair returns Result<T>, so we must specify the target type
    let broken = "{\"key\": \"value\""; // missing closing brace
    let result: Result<SimpleJson, _> = json_repair(broken);
    assert!(result.is_ok(), "repair should produce valid JSON for {:?}", broken);
}

#[test]
fn repair_valid_json_is_identity() {
    let valid = r#"{"key":"value"}"#;
    let result: Result<SimpleJson, _> = json_repair(valid);
    assert!(result.is_ok(), "valid JSON should parse without error");
    let repaired = result.unwrap();
    assert_eq!(repaired.key, "value");
}
