#!/bin/sh
set -e
mkdir -p "$HOME/forge"
cat > "$HOME/forge/AGENTS.md" << 'AGENTS_EOF'
# Forge Global Instructions

## Code Style
- Use early returns instead of deeply nested if/else
- In Rust: prefer #[allow(clippy::too_many_arguments)] over restructuring a constructor that legitimately needs many params
- Run cargo fmt -p <crate> (stable, not nightly) before every commit; CI uses stable rustfmt

## Testing
- In Rust workspaces: always run existing tests before writing new ones
- RED commit (failing tests) MUST precede GREEN commit per TDD wave — no skipping
- For async Rust tests: use #[tokio::test] with tokio::sync::mpsc::channel receivers held in named vars (_rx not _) to avoid immediate channel close

## Git Workflow
- Commit after each logical change (RED, GREEN, REFACTOR) rather than one big commit per feature
- Multi-line commit messages with DCO sign-off: always write to a .forge/commit_<task>.sh script and run it — never embed newlines + single-quotes in inline forge -p prompts (causes silent hang)
- DCO format: Signed-off-by: Name <email> on a blank line after the body

## Forge Behavior
- cargo check/build/clippy/test commands consistently exceed the 5-minute default tool timeout on large Rust workspaces; always delegate to a .forge/ script file and run via sh .forge/script.sh 2>&1
- tool_timeout key in .forge.toml is silently ignored — the correct TOML key is tool_timeout_secs; set to 1200 for Rust workspace projects
- Background forge tasks stall silently when the prompt contains newlines + single-quote sequences; always run forge foreground for multi-line shell operations
- Forge tends to retry timed-out cargo commands without changing strategy; counteract by breaking tasks into named .forge/ scripts that can be inspected and re-run
AGENTS_EOF
echo "WRITTEN_OK"
