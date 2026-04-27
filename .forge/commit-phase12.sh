#!/bin/sh
set -e
git add .planning/ROADMAP.md README.md .planning/phases/12-terminal-bench-submission/12-PLAN.md
git commit -m "docs: add Phase 12 plan + update ROADMAP status

- Add .planning/phases/12-terminal-bench-submission/12-PLAN.md with 5 waves:
  W1: EVAL-01a parity baseline (≥80% TB 2.0)
  W2: kay eval tb2 command
  W3: Held-out task subset validation
  W4: Real-repo eval (Rails, React+TS, Rust, Python, monorepo)
  W5: v1.0.0 release

- Update ROADMAP.md progress table with Phase 1-11 completion dates
- Update README.md status to v0.5.0 (Phase 1-11 shipped, Phase 12 planned)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"