#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others
git add crates/kay-core/src/control.rs
git add crates/kay-sandbox-linux/tests/escape.rs
git add crates/kay-sandbox-macos/tests/escape.rs
git add crates/kay-sandbox-windows/tests/escape.rs
git commit -m "fix(tests): convert sync #[test] to #[tokio::test] for async Sandbox trait methods

The Sandbox trait methods (check_fs_write, check_fs_read, check_net) are
async and must be called with .await. The escape tests were using #[test]
with synchronous calls to these async methods, causing:
  - E0599: no method named `check_*` found (trait not in scope)
  - E0728: await is only allowed inside async functions

Changes:
- kay-core/control.rs: Changed install_ctrl_c_handler_returns_ok from
  #[test] fn to #[tokio::test] async fn (tokio runtime required for
  signal handler installation)
- kay-sandbox-*/tests/escape.rs: Changed all 9-10 tests from #[test]
  to #[tokio::test] async, added `.await` to all check_* calls, added
  `use kay_tools::seams::sandbox::Sandbox` import
- Fixed test assertions to match actual policy behavior:
  - Default allowlist is openrouter.ai:443 only (not minimax.io)
  - Default deny list is ~/.ssh, ~/.gnupg etc (not /etc/passwd)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/10-multi-session-manager
echo "PUSH_OK"