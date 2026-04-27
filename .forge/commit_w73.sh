#!/bin/sh
set -e
git add scripts/coverage-gate.sh
git commit -m "[GREEN] W-7: coverage-gate.sh script added

Create scripts/coverage-gate.sh that:
- Iterates all 28 gap-list crates
- Checks for tests/ directory existence
- Counts #[test] functions per crate
- Fails CI if any crate has zero tests

The script correctly identifies that forge_fs, forge_stream, forge_walker,
and forge_test_kit have empty tests/ directories - those will be filled in
by Waves 1-5 of the test-coverage phase.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
