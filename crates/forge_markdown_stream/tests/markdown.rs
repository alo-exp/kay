/// Integration test for forge_markdown_stream
use forge_markdown_stream::StreamdownRenderer;

#[test]
fn render_plain_text_does_not_panic() {
    let mut buffer = Vec::new();
    let mut renderer = StreamdownRenderer::new(&mut buffer, 80);

    // Push a simple token followed by newline to trigger line rendering
    renderer.push("hello").expect("push should not fail");
    renderer.push("\n").expect("push should not fail");
    renderer.finish().expect("finish should not fail");
}

#[test]
fn render_multiline_text() {
    let mut buffer = Vec::new();
    let mut renderer = StreamdownRenderer::new(&mut buffer, 80);

    renderer.push("line one\n").expect("push should not fail");
    renderer.push("line two\n").expect("push should not fail");
    renderer.finish().expect("finish should not fail");
}
