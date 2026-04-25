#!/bin/sh
set -e
cargo test -p kay-tauri -- --nocapture 2>&1 | tail -30
