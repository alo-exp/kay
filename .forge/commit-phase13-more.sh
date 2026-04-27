#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-tools/src/task.rs
git add crates/kay-tools/src/lib.rs
git add crates/kay-core/src/planner.rs
git add crates/kay-core/src/lib.rs
git add crates/kay-cli/src/interactive.rs

git commit -m "feat: add task delegation, planning system, REPL history

Phase 13: Additional Forge parity features.

Task Delegation (kay-tools/src/task.rs):
- spawn_task() for single task execution
- spawn_tasks_parallel() for parallel execution
- TaskManager for tracking delegated tasks

Planning System (kay-core/src/planner.rs):
- Requirement tracking with REQ-ID mapping
- Threat model generation per phase
- Rollback plan generation
- Quality gates (9 dimensions)
- Milestone tracking
- Phase creation with defaults

REPL History (kay-cli/interactive.rs):
- FileBackedHistory stored in ~/.kay/history
- Arrow key navigation through previous commands
- History persists across sessions

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
