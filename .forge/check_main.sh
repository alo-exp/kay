#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
cargo check -p forge_main --tests 2>&1 | tail -10
echo "DONE_main"
