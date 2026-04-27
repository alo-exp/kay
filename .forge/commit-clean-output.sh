#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/render.rs
git add crates/kay-cli/src/run.rs
git add crates/kay-cli/src/interactive.rs
git add crates/kay-cli/src/main.rs

git commit -m "feat(kay-cli): clean streaming output via render module

Phase 12: Renders text deltas as clean output instead of JSONL.

New module: render.rs with StreamingWriter for interactive output.
- text_delta() prints directly to stdout (no buffering)
- task_complete() moves to new line
- Accumulated text for potential follow-up

Updated run.rs:
- live mode: uses StreamingWriter for clean text output
- non-live/special events: still emit JSONL for compatibility

Updated interactive.rs:
- Uses StreamingWriter for REPL output

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
