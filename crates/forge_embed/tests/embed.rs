/// Integration test for forge_embed
use include_dir::{Dir, include_dir};

static TEMPLATE_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../templates");

#[test]
fn templates_register_without_panic() {
    todo!("W-2 RED: verify template registration")
}
