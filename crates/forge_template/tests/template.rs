/// Integration test for forge_template
use forge_template::Element;

#[test]
fn element_constructs_without_panic() {
    // Construct a simple Element and verify its debug representation is non-empty
    let e = Element::new("div");
    let debug_str = format!("{:?}", e);
    assert!(
        !debug_str.is_empty(),
        "Element Debug should not be empty, got: {}",
        debug_str
    );
}

#[test]
fn element_with_text_renders() {
    let html = Element::new("span").text("hello");
    let rendered = html.render();
    assert!(
        rendered.contains("hello"),
        "rendered element should contain text 'hello', got: {}",
        rendered
    );
}
