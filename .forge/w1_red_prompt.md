OBJECTIVE: Write Wave 1 RED (failing) integration test stubs for forge_* batch 1 crates. This is a TDD phase — tests must exist and compile but intentionally fail (todo!() panic) before any implementation.

CONTEXT:
- Project: Kay coding agent (Rust workspace)
- Branch: phase/09.1-test-coverage (check out or create from main)
- Working directory: /Users/shafqat/Documents/Projects/opencode/vs-others
- Phase: 9.1 Comprehensive Test Coverage, Wave 1
- Plan: .planning/phases/09.1-test-coverage/09.1-PLAN.md (Wave 1 section)
- Crate locations: crates/<crate_name>/ (e.g., crates/forge_app/)
- All workspace dev-deps should go in root Cargo.toml [workspace.dependencies], then crates reference with .workspace = true

DESIRED STATE:
Create the following files (all NEW — do not modify any existing source files):

1. crates/forge_app/tests/app.rs:
```rust
#[test]
fn app_lifecycle() {
    todo!("W-1 RED: implement app lifecycle test")
}
```

2. crates/forge_config/tests/config.rs:
```rust
#[test]
fn config_defaults() {
    todo!("W-1 RED: implement config defaults test")
}

#[test]
fn config_from_env() {
    todo!("W-1 RED: implement config from env test")
}
```

3. crates/forge_display/tests/display.rs:
```rust
#[test]
fn render_event_snapshot() {
    todo!("W-1 RED: implement render snapshot test")
}
```

4. crates/forge_domain/tests/domain.rs:
```rust
#[test]
fn task_round_trips_serde() {
    todo!("W-1 RED: implement domain serde round-trip test")
}
```

5. crates/forge_fs/tests/fs.rs:
```rust
#[test]
fn read_write_file() {
    todo!("W-1 RED: implement fs read/write test")
}
```

6. crates/forge_infra/tests/infra.rs:
```rust
#[test]
fn health_check_ok() {
    todo!("W-1 RED: implement infra health check test")
}
```

7. crates/forge_json_repair/tests/repair.rs:
```rust
#[test]
fn repair_invalid_json() {
    todo!("W-1 RED: implement json repair test")
}

#[test]
fn repair_valid_json_is_identity() {
    todo!("W-1 RED: implement json identity test")
}
```

8. crates/forge_main/tests/main_integration.rs:
```rust
#[test]
fn help_flag_exits_zero() {
    todo!("W-1 RED: implement help flag test")
}
```

9. crates/forge_spinner/tests/spinner.rs:
```rust
#[test]
fn spinner_start_stop() {
    todo!("W-1 RED: implement spinner test")
}
```

For each crate, also add [[test]] section to the crate's Cargo.toml if it does not already have one. Check the existing Cargo.toml first before adding. Example to add:

[[test]]
name = "app"
path = "tests/app.rs"

(Use the test file name without .rs as the name field.)

Add these dev-dependencies to root Cargo.toml [workspace.dependencies] ONLY IF they don't already exist there:
- tempfile (for fs tests, wave 1 only needs it potentially)
- assert_cmd (for main_integration test)
- proptest (for domain test)
- insta (for display test)

Then in crates/forge_main/Cargo.toml [dev-dependencies]:
assert_cmd = { workspace = true }

In crates/forge_domain/Cargo.toml [dev-dependencies]:
proptest = { workspace = true }

In crates/forge_display/Cargo.toml [dev-dependencies]:
insta = { workspace = true }

In crates/forge_fs/Cargo.toml [dev-dependencies]:
tempfile = { workspace = true }

After creating all files, run: cargo check -p forge_app -p forge_config -p forge_display -p forge_domain -p forge_fs -p forge_infra -p forge_json_repair -p forge_main -p forge_spinner --tests 2>&1 | tail -20

Then commit with:
git add crates/forge_app/tests/ crates/forge_config/tests/ crates/forge_display/tests/ crates/forge_domain/tests/ crates/forge_fs/tests/ crates/forge_infra/tests/ crates/forge_json_repair/tests/ crates/forge_main/tests/ crates/forge_spinner/tests/ Cargo.toml crates/forge_app/Cargo.toml crates/forge_config/Cargo.toml crates/forge_display/Cargo.toml crates/forge_domain/Cargo.toml crates/forge_fs/Cargo.toml crates/forge_infra/Cargo.toml crates/forge_json_repair/Cargo.toml crates/forge_main/Cargo.toml crates/forge_spinner/Cargo.toml
git commit -m "[RED] W-1: scaffold failing test stubs for forge_* batch 1 (9 crates)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

SUCCESS CRITERIA:
- All 9 test files created under crates/<crate>/tests/<name>.rs
- cargo check --tests compiles without errors (todo!() panics are runtime, not compile errors)
- Commit message starts with [RED] and includes DCO Signed-off-by trailer
- No existing source files modified (tests/ directories are new)
- Output ends with: STATUS: success

INJECTED SKILLS: testing-strategy, quality-gates
