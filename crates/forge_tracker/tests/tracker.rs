/// Integration test for forge_tracker
use forge_tracker::VERSION;

#[test]
fn version_is_non_empty() {
    assert!(
        !VERSION.is_empty(),
        "VERSION constant should not be empty, got: '{}'",
        VERSION
    );
}

#[test]
fn version_is_defined() {
    // Just referencing VERSION should compile fine
    let v: &str = VERSION;
    assert!(!v.is_empty());
}
