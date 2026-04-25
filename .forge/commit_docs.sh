#!/bin/sh
set -e
git add docs/CHANGELOG.md docs/ARCHITECTURE.md docs/knowledge/INDEX.md
git commit -m "docs: fill Phase 2-8 CHANGELOG entries + update ARCHITECTURE + fix INDEX

- CHANGELOG.md: add 7 missing entries (Phases 2, 2.5, 3, 4, 5, 6, 7, 8)
  per doc-scheme.md finalization requirement; only Phase 1 existed before
- ARCHITECTURE.md: update Current State from 'Phase 8 in progress' to
  'Phase 8 COMPLETE'; add kay-verifier to Core Components; pin sqlite-vec
  version; fix docs/ path reference
- knowledge/INDEX.md: fix stale paths (docs/sessions/ -> removed,
  docs/specs/ -> docs/superpowers/specs/); update Phase count to 8;
  add superpowers/plans entry

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
git push origin main
echo "DOCS_PUSHED_OK"
