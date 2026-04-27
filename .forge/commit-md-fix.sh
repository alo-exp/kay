#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-cli/src/markdown.rs

git commit -m "fix(kay-cli): improve markdown heading support

- Support h1-h6 headings (###### style)
- h1: bold + underline
- h2: bold
- h3+: bold + italic

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/10-multi-session-manager
echo "PUSH_OK"
