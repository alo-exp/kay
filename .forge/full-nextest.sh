#!/bin/bash
# Full workspace test with nextest - saves output to files
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

LOGFILE=".forge/full-test.log"
rm -f "$LOGFILE" target/debug/.cargo-lock

echo "=== Full Test Run ===" | tee "$LOGFILE"
echo "Started: $(date)" | tee -a "$LOGFILE"
echo "System: $(uname -m), cores: $(sysctl -n hw.ncpu)" | tee -a "$LOGFILE"
echo "" | tee -a "$LOGFILE"

# Run nextest (faster than regular cargo test)
cargo nextest run --workspace --no-fail-fast 2>&1 | tee -a "$LOGFILE"

echo "" | tee -a "$LOGFILE"
echo "=== Complete: $(date) ===" | tee -a "$LOGFILE"