#!/bin/sh
set -e
git add crates/kay-tui/src/ui.rs crates/kay-tui/tests/ui_smoke.rs
git commit -m "feat(kay-tui): implement Wave 6 - wire session manager keyboard shortcuts

- Add TuiSessionManager to App struct
- Wire keyboard shortcuts in handle_input():
  - n: spawn new session
  - p: pause current session
  - r: resume paused session
  - f: fork current session
  - x: kill current session
  - s: toggle settings (moved to KeyboardMapper)
  - ?: show help
- Update footer to show all keyboard shortcuts
- Add 7 new unit tests for session control shortcuts

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
