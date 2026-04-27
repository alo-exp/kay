#!/bin/sh
set -e
git add \
  crates/kay-context/src/store.rs \
  crates/kay-tools/src/builtins/task_complete.rs \
  crates/kay-tools/src/events.rs \
  crates/kay-tools/src/runtime/context.rs

git commit -m "fix(ci): apply stable rustfmt to kay-tools and kay-context files

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"

git push origin phase/08-multi-perspective-verification
echo "PUSH_OK"
