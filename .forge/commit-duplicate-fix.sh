#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-provider-minimax/src/translator.rs

git commit -m "fix(kay-provider-minimax): prevent duplicate content from final chunk

The final chunk contains both streaming delta.content AND
complete message.content. Previously both were emitted,
causing content to appear twice.

Now final chunk's message.content is NOT emitted when
delta.content already streamed. This prevents duplication.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
