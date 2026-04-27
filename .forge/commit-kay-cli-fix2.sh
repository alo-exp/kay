#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/run.rs
git commit -m "fix(kay-cli): import Provider trait and fix ChatRequest fields

Import Provider trait at module level so .chat() method is found.
Add missing max_tokens and temperature fields to ChatRequest.
Change tools: None to tools: vec![] to match Vec<ToolSchema> type.
Also add Allowlist to local use statement and remove stale minimax::Allowlist path.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/10-multi-session-manager
echo "PUSH_OK"
