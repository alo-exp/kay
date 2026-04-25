#!/bin/sh
set -e
git add crates/kay-tauri/ui/src/bindings.ts crates/kay-tauri/ui/src/components/App.tsx crates/kay-tauri/ui/src/components/EventRow.tsx
git commit -m "fix(kay-tauri): UI build errors — Value type, typedError, exhaustiveness

- bindings.ts: add exported Value type (specta::Value recursive reference)
  so TS6133 'Cannot find name Value' is resolved
- App.tsx: startSession returns typedError<{status,data}|{status,error}>
  not raw string — fix TS2322 type mismatch
- EventRow.tsx: replace unused _exhaustiveCheck with void _never
  pattern — compiles cleanly while preserving never-type exhaustiveness

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "Commit created"
