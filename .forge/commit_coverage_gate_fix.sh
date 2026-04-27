#!/bin/sh
set -e
git add scripts/coverage-gate.sh
git commit -m "fix(coverage-gate): count #[tokio::test] as valid test functions

Coverage gate was only counting #[test] but some crates use
#[tokio::test] for async integration tests (forge_stream, forge_walker,
forge_fs, forge_test_kit). This caused false failures for crates that
do have valid async tests.

Fix: also count #[tokio::test] in total test count.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/09.1-test-coverage 2>&1
echo "PUSH_OK"
