#!/bin/sh
set -e
cargo update -p specta-typescript 2>&1 | head -10
