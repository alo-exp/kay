#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add crates/kay-tauri/ui/package.json crates/kay-tauri/ui/pnpm-lock.yaml crates/kay-tauri/ui/wdio.conf.ts crates/kay-tauri/ui/e2e/smoke.ts
git status

git commit -m "[RED] W-6: scaffold WDIO e2e smoke suite stubs (5 cases, all skipped)

Wave 6 RED phase: Install @wdio/cli + webdriverio v9, create wdio.conf.ts
for tauri-driver, and add 5 it.skip() smoke tests covering:
- app window opens
- session view renders
- start session button exists
- stop session button exists
- cost meter visible

Uses @crabnebula/tauri-driver-* binaries for cross-platform support
(macOS arm64/x64, Linux, Windows). The application path points to
../../target/debug/kay-tauri (requires \`cargo build -p kay-tauri\` first).

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
