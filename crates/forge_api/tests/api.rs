/// Test that the API trait is object-safe (can be used as dyn API)
#[test]
fn api_trait_is_object_safe() {
    // Static assertion: compile-time check that &dyn API is valid
    fn _assert_object_safe(_: &dyn forge_api::API) {}
    // If this compiles, the trait is object-safe
    _assert_object_safe;
}
