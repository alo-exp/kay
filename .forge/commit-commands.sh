#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/commands/
git add crates/kay-cli/src/main.rs
git add crates/kay-cli/Cargo.toml

git commit -m "feat(kay-cli): add build/check/fmt/clippy/test/review commands

Phase 13: Add Forge-matching CLI commands.

New commands:
- kay build [-p crate] [--release]
- kay check [-p crate]
- kay fmt [-p crate] [--check]
- kay clippy [-p crate] [--deny-warnings]
- kay test [-p crate] [--ignored] [--no-fail-fast]
- kay review [--strict] [--check-format]

All commands:
- Delegate to cargo equivalents
- Support -p for crate selection
- Print success/failure indicators
- Proper error handling

Fixed clap short arg conflicts across all commands.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
