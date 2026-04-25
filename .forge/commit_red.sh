#!/bin/sh
set -e
git add crates/kay-tauri/tests/gen_bindings.rs crates/kay-tauri/tests/memory_canary.rs
git commit -m "RED(kay-tauri): gen_bindings + memory_canary stubs

gen_bindings.rs: export_tauri_bindings test asserts bindings.ts exists
memory_canary.rs: rss_measurement_works + short_ipc_canary stubs

Both are RED-phase TDD stubs — tests fail until GREEN impl lands.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "RED commit created"
