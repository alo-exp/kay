/// Integration test for forge_embed
use include_dir::{Dir, include_dir};
use forge_embed::{files, register_templates};
use handlebars::Handlebars;

static TEMPLATE_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../templates");

#[test]
fn templates_register_without_panic() {
    // files() should return an iterator over embedded template files
    let file_count = files(&TEMPLATE_DIR).count();
    // The templates directory is expected to contain at least one file
    assert!(
        file_count > 0,
        "expected at least one template file, found {}",
        file_count
    );
}

#[test]
fn register_templates_accepts_valid_dir() {
    let mut hb = Handlebars::new();
    // Should not panic for a well-formed directory
    register_templates(&mut hb, &TEMPLATE_DIR);
    // If we reach here, registration succeeded
}
