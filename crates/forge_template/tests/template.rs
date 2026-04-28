/// Integration test for forge_template
use forge_template::Element;

#[test]
fn element_constructs_without_panic() {
    // Construct a simple Element and verify it renders without panic
    let e = Element::new("div");
    let rendered = e.render();
    assert!(
        rendered.contains("div"),
        "Element should contain tag name, got: {}",
        rendered
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
