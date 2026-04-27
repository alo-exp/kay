#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-provider-minimax/
git add crates/kay-cli/src/run.rs
git add crates/kay-cli/Cargo.toml
git add Cargo.toml

git commit -m "feat(kay-provider-minimax): add MiniMax native API streaming provider

Phase 12 W1: Wire MiniMax live API into kay-cli.

New crate: kay-provider-minimax with:
- MiniMaxProvider (reqwest-based) using MiniMax's streaming API
- JSON-SSE translator (parses data: {JSON} lines)
- Extracts delta.content for final answer
- Handles reasoning_content (chain-of-thought) separately
- TaskComplete on stream end

Wired into kay-cli run --live command:
- Reads MINIMAX_API_KEY from env
- Falls back to MiniMaxProvider for live mode

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
