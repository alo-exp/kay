#!/bin/bash
set -e
cargo test -p kay-sandbox-linux -p kay-sandbox-macos -p kay-sandbox-windows 2>&1