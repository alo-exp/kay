#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
cargo check -p forge_json_repair 2>&1 | tail -10
