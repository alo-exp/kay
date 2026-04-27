#!/bin/sh
set -e
# Check what macros are available from tauri
cat > /tmp/test_macro.rs << 'TESTEOF'
use tauri::command;
fn main() {}
TESTEOF
# Let's see what __cmd__ macros exist
echo "Checking tauri crate for __cmd__..."
