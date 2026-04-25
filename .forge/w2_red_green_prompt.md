OBJECTIVE:
Wave 2 of Phase 9.1 comprehensive test coverage. Add integration tests to 9
forge_* crates (batch 2): forge_embed, forge_markdown_stream, forge_repo,
forge_services, forge_snaps, forge_stream, forge_template, forge_tracker,
forge_walker. Two commits required: [RED] then [GREEN] (TDD discipline).

CONTEXT:
Branch: phase/09.1-test-coverage
Working dir: /Users/shafqat/Documents/Projects/opencode/vs-others
Wave 1 already committed (RED + GREEN for batch 1). This is Wave 2 batch 2.
Workspace dev-deps already in [workspace.dependencies]: proptest, assert_cmd,
insta, tempfile, predicates.

DESIRED STATE:
- Each crate has tests/<name>.rs with [[test]] in Cargo.toml
- [RED] commit: todo!() stubs that compile
- [GREEN] commit: real assertions that pass

--- CRATE API SUMMARY (use this to write tests) ---

forge_embed: exports files(), register_templates() — embeds Handlebars templates
forge_markdown_stream: exports StreamdownRenderer, Renderer, Parser, Theme, Style, repair_line()
forge_repo: re-exports agent defs, conversation mgmt, file snapshotting
forge_services: exports IntoDomain, FromDomain traits, context engines, service discovery
forge_snaps: exports SnapshotInfo, SnapshotId (via service:: module)
forge_stream: exports mpsc_stream utilities (MPSC channel streaming)
forge_template: exports Element (template element abstraction)
forge_tracker: exports Tracker, Event, EventKind, ToolCallPayload, init_tracing(), VERSION
forge_walker: exports Walker, File (filesystem traversal)

--- STEP 1: Read each crate ---

Before writing tests, read src/lib.rs for each crate to understand:
- Constructor signatures (what args does the main struct need?)
- Whether Default is derived
- Visibility of fields

--- STEP 2: Update Cargo.toml for each crate ---

For each of the 9 crates, add to the crate's Cargo.toml AFTER [dependencies]:

[dev-dependencies]
proptest    = { workspace = true }
insta       = { workspace = true }
tempfile    = { workspace = true }
assert_cmd  = { workspace = true }

[[test]]
name = "<short_name>"
path = "tests/<filename>.rs"

Use these test names:
- forge_embed         → name = "embed",          path = "tests/embed.rs"
- forge_markdown_stream → name = "markdown",     path = "tests/markdown.rs"
- forge_repo          → name = "repo",           path = "tests/repo.rs"
- forge_services      → name = "services",       path = "tests/services.rs"
- forge_snaps         → name = "snaps",          path = "tests/snaps.rs"
- forge_stream        → name = "stream",         path = "tests/stream.rs"
- forge_template      → name = "template",       path = "tests/template.rs"
- forge_tracker       → name = "tracker",        path = "tests/tracker.rs"
- forge_walker        → name = "walker",         path = "tests/walker.rs"

--- STEP 3: Create RED test stubs ---

Create these files with todo!() stubs:

crates/forge_embed/tests/embed.rs:
```rust
#[test]
fn templates_register_without_panic() {
    todo!("W-2 RED: verify template registration")
}
```

crates/forge_markdown_stream/tests/markdown.rs:
```rust
#[test]
fn render_plain_text_does_not_panic() {
    todo!("W-2 RED: render plain text through StreamdownRenderer")
}
```

crates/forge_repo/tests/repo.rs:
```rust
#[test]
fn snapshot_id_debug_non_empty() {
    todo!("W-2 RED: verify SnapshotId or similar type constructs")
}
```

crates/forge_services/tests/services.rs:
```rust
#[test]
fn into_domain_trait_object_safe() {
    todo!("W-2 RED: verify trait object safety or simple construction")
}
```

crates/forge_snaps/tests/snaps.rs:
```rust
#[test]
fn snapshot_id_round_trips_display() {
    todo!("W-2 RED: verify SnapshotId display/debug")
}
```

crates/forge_stream/tests/stream.rs:
```rust
#[test]
fn mpsc_channel_sends_and_receives() {
    todo!("W-2 RED: basic mpsc stream send/receive")
}
```

crates/forge_template/tests/template.rs:
```rust
#[test]
fn element_constructs_without_panic() {
    todo!("W-2 RED: verify Element construction")
}
```

crates/forge_tracker/tests/tracker.rs:
```rust
#[test]
fn version_is_non_empty() {
    todo!("W-2 RED: verify VERSION constant is defined")
}
```

crates/forge_walker/tests/walker.rs:
```rust
#[test]
fn walker_empty_dir_yields_no_files() {
    todo!("W-2 RED: walk empty directory yields empty result")
}
```

--- STEP 4: RED commit ---

git add crates/forge_embed/ crates/forge_markdown_stream/ crates/forge_repo/ \
  crates/forge_services/ crates/forge_snaps/ crates/forge_stream/ \
  crates/forge_template/ crates/forge_tracker/ crates/forge_walker/
git commit -m "[RED] test(wave-2): forge_* batch 2 — 9 integration test stubs (todo!())

Wave 2 of Phase 9.1 comprehensive test coverage. Nine forge_* crates get
integration test stubs that compile but panic at runtime (TDD RED phase).

Crates: forge_embed, forge_markdown_stream, forge_repo, forge_services,
        forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

--- STEP 5: Implement GREEN tests ---

Replace each todo!() with real assertions. Read each crate's src/lib.rs first
to get constructor signatures, then write the minimal passing test.

Key patterns for each crate:

forge_embed: Call forge_embed::files() and assert the iterator is non-empty
  OR call register_templates on a new Handlebars instance and assert no error.
  handlebars = { workspace = true } may be needed as dev-dep.

forge_markdown_stream: Construct StreamdownRenderer (check constructor args)
  and call render or process on a simple string. Assert no panic.

forge_repo: Find simplest exported type (a struct or enum) that derives Debug.
  Construct it and assert format!("{:?}", val) is non-empty.

forge_services: Find an exported type with Default or a simple constructor.
  Avoid types requiring network/database. Check src/ for config types.

forge_snaps: SnapshotId likely wraps a string. Test construction and display:
  let id = SnapshotId::new("test"); assert_eq!(id.to_string(), "test");

forge_stream: Create a tokio mpsc channel wrapper if provided, OR just verify
  the module compiles with a simple assert_eq!(1, 1) if nothing is easily testable.

forge_template: Element might be a simple enum or struct.
  Construct it: let e = Element::text("hello"); assert!(!format!("{:?}", e).is_empty());

forge_tracker: VERSION is a &'static str constant:
  assert!(!forge_tracker::VERSION.is_empty());

forge_walker: Create a tempdir, construct Walker::new(tempdir.path()), collect
  files, assert the result is empty (empty directory).

--- STEP 6: GREEN commit ---

git add crates/forge_embed/tests/ crates/forge_markdown_stream/tests/ \
  crates/forge_repo/tests/ crates/forge_services/tests/ crates/forge_snaps/tests/ \
  crates/forge_stream/tests/ crates/forge_template/tests/ crates/forge_tracker/tests/ \
  crates/forge_walker/tests/
git commit -m "[GREEN] test(wave-2): forge_* batch 2 — 9 integration tests pass

Wave 2 GREEN phase. Real assertions replacing todo!() stubs.

Crates: forge_embed, forge_markdown_stream, forge_repo, forge_services,
        forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

SUCCESS CRITERIA:
- [RED] commit exists with all 9 todo!() stubs
- [GREEN] commit exists with real assertions
- cargo check -p forge_embed --tests (and each crate) passes
- STATUS: success

INJECTED SKILLS: testing-strategy, code-review
