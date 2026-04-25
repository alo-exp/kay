#!/bin/sh
set -e
cargo tree -p kay-tauri -e features 2>&1 | grep -E "tauri|TAURI" | head -20
