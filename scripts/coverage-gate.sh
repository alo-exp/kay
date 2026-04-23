#!/bin/bash
# coverage-gate.sh — W-7.3
#
# Enforces that every gap-list crate has at least one test file.
# This is a hard CI failure — a gap-list crate with zero tests is
# a revert-worthy regression.
#
# Usage: ./scripts/coverage-gate.sh
# Exit codes:
#   0 — all gap-list crates have test coverage
#   1 — one or more gap-list crates have no tests

set -euo pipefail

GAP_CRATES=(
  # forge_* batch 1
  "forge_app"
  "forge_config"
  "forge_display"
  "forge_domain"
  # forge_* batch 2
  "forge_embed"
  "forge_fs"
  "forge_infra"
  "forge_json_repair"
  "forge_main"
  "forge_markdown_stream"
  "forge_repo"
  "forge_services"
  "forge_snaps"
  "forge_spinner"
  "forge_stream"
  "forge_template"
  "forge_tracker"
  "forge_walker"
  # forge_* batch 3
  "forge_api"
  "forge_ci"
  "forge_test_kit"
  "forge_tool_macros"
  # kay-sandbox
  "kay-sandbox-linux"
  "kay-sandbox-macos"
  "kay-sandbox-windows"
  # Kay crates
  "kay-tauri"
  "kay-tui"
)

FAILED=0

for crate in "${GAP_CRATES[@]}"; do
  tests_dir="crates/$crate/tests"
  src_dir="crates/$crate/src"

  # Check if tests/ directory exists
  if [ ! -d "$tests_dir" ]; then
    echo "FAIL: $crate has no tests/ directory"
    FAILED=1
    continue
  fi

  # Count #[test] functions in tests/ directory
  # grep outputs lines matching #[test]; wc -l counts them
  # || true handles grep exit code 1 (no matches) on macOS
  test_count=$(grep -rh "#\[test\]" "$tests_dir" --include="*.rs" 2>/dev/null | wc -l || true)

  if [ "$test_count" -eq 0 ]; then
    echo "FAIL: $crate has tests/ dir but zero #[test] functions"
    FAILED=1
  else
    echo "PASS: $crate — $test_count test(s) found"
  fi
done

if [ $FAILED -ne 0 ]; then
  echo ""
  echo "ERROR: One or more gap-list crates failed coverage gate."
  echo "Post-merge push with a gap-list crate missing tests is a revert-worthy regression."
  exit 1
fi

echo ""
echo "All gap-list crates have test coverage."
exit 0