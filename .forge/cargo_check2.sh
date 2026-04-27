#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
# Update the index once (may take time)
cargo update -p specta 2>&1 | head -5
# Now check
cargo check -p kay-tauri 2>&1 | tail -20
