#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/markdown.rs
git add crates/kay-cli/src/render.rs
git add crates/kay-cli/src/main.rs

git commit -m "feat(kay-cli): terminal markdown rendering

Phase 12: Render markdown formatting as ANSI terminal codes.

New module: markdown.rs
- Converts markdown patterns to ANSI escape codes:
  - **bold** → ANSI bold (only double asterisk)
  - \`code\` → ANSI bright/lighter
  - - item at line start → bullet point (•)
  - > text at line start → quoted text (│)
  - # text at line start → bold heading

Conservative matching avoids false positives:
- Single asterisks preserved (e.g., Apple's*)
- Only clear markdown patterns converted

Updated render.rs:
- StreamingWriter.text_delta() now uses markdown renderer
- Text is rendered before printing to stdout

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
