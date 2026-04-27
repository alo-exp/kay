#!/bin/sh
set -e
# Check with fresh build
rm -rf target/debug/build/kay-tauri-*
cargo check -p kay-tauri --lib 2>&1 | tail -5
cargo check -p kay-tauri --bin kay-tauri 2>&1 | tail -20
