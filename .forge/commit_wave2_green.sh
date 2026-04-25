#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add \
  crates/forge_embed/tests/ \
  crates/forge_markdown_stream/tests/ \
  crates/forge_repo/tests/ \
  crates/forge_services/tests/ \
  crates/forge_snaps/tests/ \
  crates/forge_stream/tests/ \
  crates/forge_template/tests/ \
  crates/forge_tracker/tests/ \
  crates/forge_walker/tests/

git commit -m "[GREEN] test(wave-2): forge_* batch 2 — 9 integration tests pass

Wave 2 GREEN phase. Real assertions replacing todo!() stubs.

Crates: forge_embed, forge_markdown_stream, forge_repo, forge_services,
        forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "GREEN_COMMIT_OK"
