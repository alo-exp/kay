#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Checking forge_embed ==="
cargo check -p forge_embed --tests 2>&1
echo "EMBED_OK"
