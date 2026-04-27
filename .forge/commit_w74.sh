#!/bin/sh
set -e
git add .github/workflows/ci.yml
git commit -m "[GREEN] W-7.4: CI matrix (3 OS) + coverage-gate CI job wired

Add coverage-gate job to .github/workflows/ci.yml as a required CI step.
The job runs scripts/coverage-gate.sh which validates that all 28 gap-list
crates have at least one #[test] function. A missing tests/ directory or
zero #[test] functions is a revert-worthy regression.

Also note: .github/workflows/ui-smoke.yml is a separate workflow (already
committed in W-6.2) that runs the WDIO smoke suite on macOS-14 + Ubuntu.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/09.1-test-coverage
echo "PUSH_OK"
