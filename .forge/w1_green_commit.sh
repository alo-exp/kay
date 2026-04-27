#!/bin/bash
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
git add crates/forge_app/tests/app.rs \
        crates/forge_config/tests/config.rs \
        crates/forge_display/tests/display.rs \
        crates/forge_domain/tests/domain.rs \
        crates/forge_fs/tests/fs.rs \
        crates/forge_infra/tests/infra.rs \
        crates/forge_json_repair/tests/repair.rs \
        crates/forge_main/tests/main_integration.rs \
        crates/forge_spinner/tests/spinner.rs
git commit -m "[GREEN] test(wave-1): forge_* batch 1 — 9 integration test implementations

Implements Wave 1 GREEN tests for forge_app, forge_config, forge_display,
forge_domain, forge_fs, forge_infra, forge_json_repair, forge_main,
forge_spinner. All todo!() stubs replaced with real assertions against
the public API of each crate.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
echo "W1 GREEN commit done"
