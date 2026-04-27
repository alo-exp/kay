#!/bin/sh
set -e
git add .planning/STATE.md
git commit -m "docs: update STATE.md with Phase 12 milestone info

- Milestone: v0.5.0 — Phase 12 TB 2.0 Submission
- Progress: 73% (11 of 12 phases)
- Current position updated with Phase 12 plan summary
- Session continuity updated with today's work

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/10-multi-session-manager 2>&1
echo "PUSH_OK"