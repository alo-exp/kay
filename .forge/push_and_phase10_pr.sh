#!/bin/sh
set -e

cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Ensure we're on the right branch
git checkout phase/10-multi-session-manager

# Push the branch
git push origin phase/10-multi-session-manager 2>&1

# Create PR from main <- phase/10-multi-session-manager
gh pr create \
  --base main \
  --head phase/10-multi-session-manager \
  --title "feat: Phase 10 — Multi-Session Manager + Project Settings" \
  --body-file .forge/phase10_pr_body.md \
  --label "phase/10" 2>&1

echo "PUSH_AND_PR_OK"
