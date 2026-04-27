#!/bin/sh
set -e
git add .forge.toml AGENTS.md
git commit -m "chore(forge): fix tool_timeout_secs key + update AGENTS.md with Forge guidance

- .forge.toml: rename tool_timeout -> tool_timeout_secs (correct TOML key
  per Forge schema; tool_timeout was silently ignored, causing 5-min timeouts)
  Increase limit to 1200s (20 min) for cold Rust workspace compilation.
  Add max_requests_per_turn=80 and max_tool_failure_per_turn=5.
  Add schema reference for IDE validation.
- AGENTS.md: document the three critical Forge patterns for this project:
  (1) Never run cargo inline — always use .forge/ scripts
  (2) Multi-line commit messages must use .forge/ scripts (quoting stall)
  (3) Use stable cargo fmt before every commit

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
