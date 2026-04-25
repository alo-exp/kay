#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add \
  crates/forge_embed/Cargo.toml \
  crates/forge_embed/tests/ \
  crates/forge_markdown_stream/Cargo.toml \
  crates/forge_markdown_stream/tests/ \
  crates/forge_repo/Cargo.toml \
  crates/forge_repo/tests/ \
  crates/forge_services/Cargo.toml \
  crates/forge_services/tests/ \
  crates/forge_snaps/Cargo.toml \
  crates/forge_snaps/tests/ \
  crates/forge_stream/Cargo.toml \
  crates/forge_stream/tests/ \
  crates/forge_template/Cargo.toml \
  crates/forge_template/tests/ \
  crates/forge_tracker/Cargo.toml \
  crates/forge_tracker/tests/ \
  crates/forge_walker/Cargo.toml \
  crates/forge_walker/tests/

git commit -m "[RED] test(wave-2): forge_* batch 2 — 9 integration test stubs (todo!())

Wave 2 of Phase 9.1 comprehensive test coverage. Nine forge_* crates get
integration test stubs that compile but panic at runtime (TDD RED phase).

Crates: forge_embed, forge_markdown_stream, forge_repo, forge_services,
        forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "RED_COMMIT_OK"
