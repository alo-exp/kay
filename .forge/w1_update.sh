#!/bin/sh
# Don't use set -e since we want to capture the tail regardless
cargo update -p specta -p tauri-specta -p specta-typescript 2>&1 | tail -10
echo "UPDATE_DONE"
