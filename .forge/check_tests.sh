#!/bin/sh
set -e
cargo check -p kay-tauri --tests 2>&1 | tail -20
