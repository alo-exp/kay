#!/bin/sh
set -e
git add \
  crates/kay-tauri/ui/src/components/App.tsx \
  crates/kay-tauri/ui/src/components/CommandApprovalDialog.tsx \
  crates/kay-tauri/ui/src/components/ModelPicker.tsx \
  crates/kay-tauri/ui/src/components/SessionList.tsx
git commit -m "fix(tauri-ui): resolve TypeScript build errors

- Remove unused React imports (modern JSX transform)
- Remove unused pendingApproval state variable
- Add missing CommandApprovalDialog import to App.tsx
- Remove unused showAllModels state in ModelPicker
- Suppress unused variable warnings with void statements

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"
