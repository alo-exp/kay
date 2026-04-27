#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Commit 1: specta dependency fix
git add Cargo.toml Cargo.lock
git commit -m "fix(deps): upgrade specta/tauri-specta/specta-typescript to rc.24

tauri-specta =2.0.0-rc.21 required specta =2.0.0-rc.22, conflicting with
specta-typescript ^0.0.7 which required specta =2.0.0-rc.20. Aligned all
three packages to the latest compatible set: 2.0.0-rc.24 + 0.0.11.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

# Commit 2: Wave 1 RED stubs
git add \
  crates/forge_app/ \
  crates/forge_config/ \
  crates/forge_display/ \
  crates/forge_domain/ \
  crates/forge_fs/ \
  crates/forge_infra/ \
  crates/forge_json_repair/ \
  crates/forge_main/ \
  crates/forge_spinner/
git commit -m "[RED] test(wave-1): forge_* batch 1 — 9 integration test stubs (todo!())

Wave 1 of Phase 9.1 comprehensive test coverage. Nine forge_* crates get
integration test stubs that compile but panic at runtime (TDD RED phase).
Each test file added with [[test]] section in Cargo.toml and dev-dependencies
(proptest, assert_cmd, insta, tempfile where needed).

Crates: forge_app, forge_config, forge_display, forge_domain, forge_fs,
        forge_infra, forge_json_repair, forge_main, forge_spinner.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "COMMIT_OK"
