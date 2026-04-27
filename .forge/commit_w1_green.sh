#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
git commit -m "[GREEN] test(wave-1): forge_* batch 1 — 9 integration tests pass

Wave 1 GREEN phase. Replace todo!() stubs with real assertions that call
the public API of each crate. Tests compile and pass under cargo test.

Crates: forge_app, forge_config, forge_display, forge_domain, forge_fs,
        forge_infra, forge_json_repair, forge_main, forge_spinner.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
echo "COMMIT_OK"
