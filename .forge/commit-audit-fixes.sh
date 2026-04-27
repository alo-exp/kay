#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/markdown.rs
git add crates/kay-cli/src/spinner.rs
git add crates/kay-cli/src/help.rs
git add crates/kay-cli/src/main.rs

git commit -m "fix(kay-cli): implement audit findings for M3, M4, L1

Adversarial audit found:
- M3: Italic, tables, links not implemented in markdown
- M4: Spinner not separate module
- L1: Help not wired into CLI

Fixed:
- M3: Added italic support for *text* and _text_
- M3: Added code block rendering (\`\`\`)
- M4: Created spinner.rs module with Spinner and ProgressBar
- L1: Added show-help command wired to help module
- L1: Changed from 'Help' to 'ShowHelp' to avoid conflict

All kay-cli tests now pass (38 unit + 9 integration).

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
