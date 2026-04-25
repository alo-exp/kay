#!/bin/sh
set -e
cargo test -p kay-tauri --test gen_bindings -- --nocapture 2>&1
