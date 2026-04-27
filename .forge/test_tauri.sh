#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
cargo test -p kay-tauri --test commands 2>&1 | tee .forge/test1_output.txt
echo "TEST1_EXIT_CODE=$?" >> .forge/test1_output.txt
