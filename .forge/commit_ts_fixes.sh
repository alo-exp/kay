#!/bin/sh
set -e
git add \
  crates/kay-tauri/ui/src/bindings.ts \
  crates/kay-tauri/ui/src/components/App.tsx \
  crates/kay-tauri/ui/src/components/EventRow.tsx

git commit -m "fix(kay-tauri): TypeScript build errors — Value type, App.tsx, EventRow exhaustiveness

Value type: Add specta::Value mapping to bindings.ts so recursive
references in IpcAgentEvent::ToolCallDelta arguments resolve.

App.tsx: handle startSession returning typedError result { status, data }.
Without this, TypeScript error TS2322 because raw string != { status, data }.

EventRow.tsx: Replace unused-variable _exhaustiveCheck with void cast
to satisfy TS6133 (noUnusedLocals). Keep exhaustiveness via never cast
on the event in the default case.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "Commit created"
