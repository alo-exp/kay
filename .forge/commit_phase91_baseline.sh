#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add -A

git commit -m "Phase 9.1: Test coverage baseline — all tests GREEN, coverage gate passing

Phase 9.1 comprehensive test coverage work consolidated:
- forge_* Batch 1 tests: forge_app, forge_config, forge_display, forge_domain,
  forge_embed, forge_fs, forge_infra, forge_json_repair, forge_main, forge_spinner
- forge_* Batch 2 tests: forge_markdown_stream, forge_repo, forge_services, forge_snaps,
  forge_stream, forge_template, forge_tracker, forge_walker
- forge_api, forge_ci, forge_test_kit, forge_tool_macros tests
- kay-sandbox cross-platform escape tests (linux, macos, windows)
- kay-tui render tests with widgets (session_view, tool_call_inspector)
- kay-template conditional rendering fix (string-based approach)
- kay-tauri Tauri integration tests (marked ignore - require live app context)
- kay-config, kay-json-repair, kay-repo, kay-provider-minimax fixes
- All snapshot updates for changed APIs
- Coverage gate script working with all 28 crates verified

DCO: Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "Commit created successfully"
