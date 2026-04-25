#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
echo "=== forge_json_repair ===" && cargo check -p forge_json_repair --tests 2>&1 | tail -5
echo "=== forge_domain ===" && cargo check -p forge_domain --tests 2>&1 | tail -5
echo "=== forge_fs ===" && cargo check -p forge_fs --tests 2>&1 | tail -5
echo "=== forge_display ===" && cargo check -p forge_display --tests 2>&1 | tail -5
echo "=== forge_config ===" && cargo check -p forge_config --tests 2>&1 | tail -5
echo "=== forge_spinner ===" && cargo check -p forge_spinner --tests 2>&1 | tail -5
echo "=== forge_app ===" && cargo check -p forge_app --tests 2>&1 | tail -5
echo "=== forge_infra ===" && cargo check -p forge_infra --tests 2>&1 | tail -5
echo "=== forge_main ===" && cargo check -p forge_main --tests 2>&1 | tail -5
echo "ALL_CHECK_OK"
