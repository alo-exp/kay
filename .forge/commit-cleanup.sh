#!/bin/sh
set -e
git add -A 2>/dev/null || true
git commit -m "chore: clean up numbered temp files from crash sessions

Remove 83 numbered temp files (* 2.md, * 3.md, etc.) that were
created by crash sessions. These files are artifacts from interrupted
sessions and do not belong in the tree.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "COMMIT_OK"