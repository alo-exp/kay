#!/bin/sh
set -e
git add \
  crates/kay-context/src/budget.rs \
  crates/kay-context/src/retriever.rs \
  crates/kay-context/src/language.rs \
  crates/kay-context/src/store.rs \
  crates/kay-context/src/event_filter.rs \
  crates/kay-provider-openrouter/tests/allowlist_gate.rs \
  crates/kay-provider-openrouter/tests/minimax_live.rs \
  ai-models-ranking.md
git commit -m "fix(tests): resolve test failures in kay-context and kay-provider-openrouter

- kay-context/budget.rs: fix Symbol::new() call in estimate_tokens test
- kay-context/retriever.rs: fix sym() helper to include required fields
- kay-provider-openrouter/minimax_live.rs: fix mock server endpoint URLs,
  Message struct fields, and SSE mock format to match protocol requirements
- kay-provider-openrouter/allowlist_gate.rs: update expected allowlist length

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
