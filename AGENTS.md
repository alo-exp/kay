# Project: Kay — Rust Terminal Coding Agent

## Project Conventions
- Workspace: `crates/` with multiple Rust crates (kay-tools, kay-core, kay-cli, kay-verifier, etc.)
- Active branch: `phase/08-multi-perspective-verification`
- DCO required: every commit needs `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`
- No direct commits to `main`
- `cargo test --workspace` for full test suite; `cargo check -p <crate>` for fast feedback
- Integration test files: `crates/<crate>/tests/<name>.rs` — no spaces in filenames
- ForgeCode JSON hardening: `required`-before-`properties`, `deny_unknown_fields` on wire types

## Forge Output Format
Always output:
```
STATUS: success | partial | failed
FILES_CHANGED: <list of modified files>
ASSUMPTIONS: <any assumptions made>
PATTERNS_DISCOVERED: <codebase patterns observed>
```

## Task Patterns
- TDD iron law: RED commit (failing tests) MUST precede GREEN commit (implementation)
- `ToolCallContext::new()` currently takes 8 positional params — update ALL call sites when signature changes
- `AgentEvent` is `#[non_exhaustive]` — add variants additively only, update `events_wire.rs` match arms

## Forge Corrections

### CRITICAL: Cargo commands take 5–20 minutes — NEVER run inline

**Rule:** Do NOT run `cargo check`, `cargo build`, `cargo clippy`, `cargo test`, or `cargo fmt` as
a direct shell command in a single agent turn. These exceed the 5-minute tool timeout on the first
run (cold cache). Always delegate to a `.forge/` script.

**Pattern — write a script file first, then run it:**

```sh
# Step 1: write the script content (edit a .forge/*.sh file)
# Step 2: run it as a separate tool call
sh .forge/my_script.sh 2>&1
```

**Reason:** `tool_timeout_secs = 1200` is set in `.forge.toml` but cold Rust compilation on large
workspaces can approach 15 minutes on first run. Using a script allows the timeout to apply to the
whole script, and Forge can report partial output.

### CRITICAL: Multi-line commit messages — always use .forge/ scripts

**Rule:** Never embed multi-line git commit messages directly in shell commands inside a `forge -p`
prompt. The `\n` + single-quote combination causes the background task to hang indefinitely with
zero output.

**Pattern that works:**

```sh
#!/bin/sh
set -e
git add <files>
git commit -m "subject line

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin <branch>
echo "PUSH_OK"
```

Write this to `.forge/commit_<task>.sh`, then run: `sh .forge/commit_<task>.sh 2>&1`

**Reason:** Single quotes in heredoc syntax inside `forge -p "..."` double-quoted strings cause the
shell parser to enter an unterminated string state, hanging the background task silently.

### Cargo fmt: stable rustfmt only

**Rule:** Run `cargo fmt -p <crate>` (not `cargo +nightly fmt`) before every commit that touches
Rust source. The CI uses stable 1.95 rustfmt. The project `rustfmt.toml` has nightly-only keys
that stable rustfmt warns about but skips — this is expected, not an error.

### Branch name

Active branch: `phase/08-multi-perspective-verification`
Always push to: `git push origin phase/08-multi-perspective-verification`

### DCO signoff format

Every git commit MUST end with exactly:
```
Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```
No variations in name or email are accepted.

### clippy::too_many_arguments

When adding parameters to existing constructors, add `#[allow(clippy::too_many_arguments)]` if the
function has more than 7 parameters. The CI runs `cargo clippy -- -D warnings`.
