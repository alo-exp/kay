#!/bin/sh
set -e
git checkout main
git pull origin main
git cherry-pick 77ec83d
git push origin main
echo "DOCS_MAIN_OK"
