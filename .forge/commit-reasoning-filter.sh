#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-provider-minimax/src/translator.rs

git commit -m "fix(kay-provider-minimax): filter out reasoning_content from output

MiniMax sends chain-of-thought in delta.reasoning_content.
Previously this was emitted as TextDelta, making output confusing.

Now only delta.content is emitted as the answer. Reasoning
content is intentionally ignored.

Added unit tests:
- ignore_reasoning_content: verifies reasoning is not emitted
- parse_done_with_message: verifies final message is emitted

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
