#!/bin/sh
set -e
ISSUE_URL="https://github.com/alo-exp/silver-bullet/issues/33"
ITEM_ID=$(gh project item-add 4 --owner alo-exp --url "$ISSUE_URL" --format json --jq .id)
echo "ITEM_ID=$ITEM_ID"
gh project item-edit \
  --project-id PVT_kwDOA5OQY84BU8tb \
  --id "$ITEM_ID" \
  --field-id PVTSSF_lADOA5OQY84BU8tbzhMcRXE \
  --single-select-option-id 7e62dc72
echo "BACKLOG_OK"
