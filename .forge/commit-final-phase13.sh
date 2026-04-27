#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/help.rs
git add crates/kay-cli/src/diff.rs
git add crates/kay-cli/src/main.rs
git add crates/kay-core/src/retry.rs
git add crates/kay-core/Cargo.toml
git add crates/kay-template/
git add crates/kay-json-repair/
git add Cargo.toml

git commit -m "feat(kay-cli): add help system, diff highlighting, and utilities

Phase 13: Complete low-priority gaps.

New modules:
- kay-cli/src/help.rs: Help system with contextual help
- kay-cli/src/diff.rs: Diff highlighting with ANSI colors
- kay-core/src/retry.rs: Retry logic with exponential backoff
- kay-template: Simple template engine
- kay-json-repair: JSON repair utility

Features:
- Help for all commands (kay help run, kay help session, etc.)
- Diff computation with green/red ANSI highlighting
- Async retry with configurable backoff
- Template rendering with variable substitution
- JSON repair for malformed input

This completes the Phase 13 feature parity implementation.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"