#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/run.rs
git commit -m "fix(kay-cli): use public re-exports instead of private module paths

The live_provider function was importing from private sub-modules
(kay_provider_openrouter::provider::* and
kay_provider_openrouter::openrouter_provider::*) instead of using
the public re-exports at the crate root.

Also adds missing tool_call_id: None field to Message struct init.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/10-multi-session-manager
echo "PUSH_OK"