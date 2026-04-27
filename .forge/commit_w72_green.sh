#!/bin/sh
set -e
git add crates/kay-tui/src/lib.rs \
       crates/kay-tui/src/widgets/mod.rs \
       crates/kay-tui/src/widgets/session_view.rs \
       crates/kay-tui/src/widgets/tool_call_inspector.rs \
       crates/kay-tui/tests/render.rs
git commit -m "[GREEN] W-7: kay-tui render tests passing

GREEN phase Wave 7: Implement SessionView and ToolCallInspector widgets
in kay_tui::widgets module, and update tests/render.rs with real
TestBackend render tests (replacing todo!() stubs).

Files:
- crates/kay-tui/src/lib.rs (new lib module)
- crates/kay-tui/src/widgets/mod.rs (widget module)
- crates/kay-tui/src/widgets/session_view.rs (SessionView widget)
- crates/kay-tui/src/widgets/tool_call_inspector.rs (ToolCallInspector widget)
- crates/kay-tui/tests/render.rs (real render tests)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
