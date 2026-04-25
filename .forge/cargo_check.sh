#!/bin/sh
set -e
cargo check -p kay-tauri 2>&1 | grep -E "error|warning:|Compiling|Checking|Finished" | head -30
