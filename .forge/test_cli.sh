#!/bin/sh
set -e
cargo test -p kay-cli --no-run 2>&1 | tail -5
echo "BUILD_OK"
