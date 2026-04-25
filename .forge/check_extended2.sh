#!/bin/sh
set -e
cargo check -p kay-tauri --lib 2>&1 | tail -20
cargo check -p kay-tauri --bin kay-tauri 2>&1 | tail -20
