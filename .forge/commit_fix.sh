#!/bin/sh
set -e
git add \
  crates/kay-tui/src/events.rs \
  crates/kay-tui/src/jsonl.rs \
  crates/kay-tui/src/ui.rs

git commit -m "fix(kay-tui): serde deserializer — Unknown variant, correct field extraction, redundant from_value

events.rs:
- Add TuiEvent::Unknown catch-all variant with custom Deserialize impl
  (serde(tag) doesn't support catch-all; manual impl needed)
- Fix all match arms: extract fields directly from data (serde_json::Value)
  instead of redundant from_value(data) → inner.get()
- Add missing ToolCallComplete arm
- Remove duplicate ToolCallDelta arm
- Unknown types now route to Unknown { event_type } (passes round-trip test)

jsonl.rs:
- unknown_event_type_returns_error → unknown_event_type_routes_to_tui_event_unknown
  (reflects new TuiEvent::Unknown behavior)
- Remove unused BufRead import

ui.rs:
- Fix unreachable pattern warning from duplicate ToolCallDelta arm

Tests: 28 passed, 0 failed

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "Commit OK"
