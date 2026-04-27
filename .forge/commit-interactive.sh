#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/interactive.rs

git commit -m "feat(kay-cli): wire interactive REPL to live MiniMax provider

Phase 12: Enable interactive mode with live API.

- Import kay_provider_minimax types (ChatRequest, Message, Provider)
- Add run_live_turn async function that:
  - Builds MiniMaxProvider with MiniMax-M2.1 model
  - Streams AgentEvents to stdout as JSONL
- REPL now calls live provider on user input instead of
  showing T7.9 placeholder message

Note: Interactive mode requires a TTY (real terminal).
Use 'kay run --live --prompt \"...\"' for headless/piped usage.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
