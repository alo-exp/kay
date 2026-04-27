#!/bin/sh
set -e
export CARGO_TARGET_DIR=/tmp/kay-test
cd /Users/shafqat/Documents/Projects/opencode/vs-others

echo "=== kay-core ===" && cargo test -p kay-core 2>&1 | grep "test result"
echo "=== kay-context ===" && cargo test -p kay-context 2>&1 | grep "test result"
echo "=== kay-tools ===" && cargo test -p kay-tools 2>&1 | grep "test result"
echo "=== kay-session ===" && cargo test -p kay-session 2>&1 | grep "test result"
echo "=== kay-verifier ===" && cargo test -p kay-verifier 2>&1 | grep "test result"
echo "=== kay-provider-openrouter ===" && cargo test -p kay-provider-openrouter 2>&1 | grep "test result"
