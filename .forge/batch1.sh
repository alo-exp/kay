#!/bin/sh
set -e
cargo test -p kay-core -p kay-tools -p kay-context 2>&1 | tail -15
echo "BATCH1_DONE"
