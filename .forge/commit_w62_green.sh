#!/bin/sh
set -e

# Stage W-6.2 GREEN files (excluding pnpm-lock.yaml which is not in changes)
git add \
  crates/kay-tauri/ui/src/components/CostMeter.tsx \
  crates/kay-tauri/ui/src/components/SessionView.tsx \
  crates/kay-tauri/ui/src/components/PromptInput.tsx \
  crates/kay-tauri/ui/e2e/smoke.ts \
  .github/workflows/ui-smoke.yml

# Commit with message
git commit -m "[GREEN] W-6: WDIO UI smoke suite passing (5 cases) + ui-smoke CI workflow

GREEN phase Wave 6: Convert it.skip() stubs to active tests.
Add data-testid attributes to React components:
- SessionView: data-testid=\"session-view\"
- CostMeter: data-testid=\"cost-meter\"
- Start button: data-testid=\"start-session-btn\"
- Stop button: data-testid=\"stop-session-btn\"

Create .github/workflows/ui-smoke.yml with:
- Runs on macOS-14 + Ubuntu (tauri-driver support)
- Builds kay-tauri binary first
- Installs pnpm + node deps
- Runs typecheck then WDIO smoke tests

Also update e2e/smoke.ts from it.skip() to active it() tests.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

echo "COMMIT_OK"