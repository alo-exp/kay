#!/bin/sh
set -e
git add \
  crates/kay-tauri/ui/src/components/App.tsx \
  crates/kay-tauri/ui/src/components/SettingsPanel.tsx \
  crates/kay-tauri/ui/src/components/SessionList.tsx \
  crates/kay-tauri/ui/src/components/ModelPicker.tsx \
  crates/kay-tauri/ui/src/components/CommandApprovalDialog.tsx
git commit -m "feat(kay-tauri): Phase 10 frontend - SessionList, ModelPicker, CommandApprovalDialog

- Add SessionList component with pause/resume/fork/kill actions
- Add ModelPicker with tiered selection (Recommended/Experimental/All)
- Add CommandApprovalDialog with keyboard shortcuts (Enter/Escape)
- Integrate all components into App.tsx
- Update SettingsPanel to support controlled/uncontrolled tabs

Success criteria addressed:
1. List sessions from GUI ✓
2. Spawn/pause/resume/fork/kill from GUI ✓
3. Command approval dialog on first tool call ✓
4. Model picker with tiered list ✓
7. Settings panel accessible ✓

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
