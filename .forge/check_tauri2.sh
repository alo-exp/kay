#!/bin/sh
set -e
# Check what tauri v2 requires for specta
# The __cmd__start_session macro comes from the tauri crate itself
# when specta feature is enabled

# Let's check if tauri 2.x has a "specta" feature
cargo update -p tauri 2>&1 | head -5
# And check what version of tauri we're using
cargo tree -p tauri --depth 1 2>&1 | head -5
