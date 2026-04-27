#!/bin/sh
set -e
cargo fix -p kay-tauri --bin kay-tauri --allow-dirty 2>&1 | grep -E "error|warning:|Fixed|Finished" | head -20
