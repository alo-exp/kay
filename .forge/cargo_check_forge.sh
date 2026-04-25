#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== Checking forge_embed --tests ==="
cargo check -p forge_embed --tests 2>&1
EMBED_RESULT=$?

echo ""
echo "=== Checking forge_markdown_stream --tests ==="
cargo check -p forge_markdown_stream --tests 2>&1
STREAM_RESULT=$?

echo ""
echo "=== RESULTS ==="
if [ $EMBED_RESULT -eq 0 ]; then
    echo "forge_embed --tests: PASSED"
else
    echo "forge_embed --tests: FAILED (exit code $EMBED_RESULT)"
fi

if [ $STREAM_RESULT -eq 0 ]; then
    echo "forge_markdown_stream --tests: PASSED"
else
    echo "forge_markdown_stream --tests: FAILED (exit code $STREAM_RESULT)"
fi

exit 0
