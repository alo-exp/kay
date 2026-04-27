#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-session/src/budget.rs
git add crates/kay-session/src/sessions.rs
git add crates/kay-session/src/lib.rs
git add crates/kay-session/Cargo.toml
git add crates/kay-cli/src/commands/
git add crates/kay-cli/src/main.rs

git commit -m "feat(kay-cli): add CLI commands and session management

Phase 13: Implement missing CLI commands and session management.

New commands (kay CLI):
- kay build - Build workspace or crate
- kay check - Type-check workspace or crate  
- kay fmt - Format code (cargo fmt)
- kay clippy - Run linter (with strict/allow options)
- kay test - Run tests (with filter, ignored, doc options)
- kay review - Code review workflow (clippy + fmt check)

New kay-session modules:
- budget.rs: Token budget management for sessions
- sessions.rs: CLI session list/load/delete commands

Features:
- Command history in REPL (arrow keys)
- TokenBudget for tracking usage
- Session management CLI (kay session list/load/delete)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"