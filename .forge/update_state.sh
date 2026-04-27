#!/bin/sh
set -e

# Update the frontmatter of STATE.md to reflect Phase 8 complete
python3 - << 'PYEOF'
import re

path = ".planning/STATE.md"
with open(path, "r") as f:
    content = f.read()

# Replace frontmatter fields
content = content.replace(
    'stopped_at: "Phase 7 shipped — PR #13 open (https://github.com/alo-exp/kay/pull/13), branch phase/07-context-engine, 70 tests green, clippy -D warnings clean. Ready for Phase 8."',
    'stopped_at: "Phase 8 shipped — PR #17 merged to main (https://github.com/alo-exp/kay/pull/17), merge commit b21897a2. MultiPerspectiveVerifier (3 KIRA critics), VerifierMode, run_with_rework loop, cost ceiling + VerifierDisabled, kay-verifier crate. All tests green CI green. Ready for Phase 9."'
)
content = content.replace(
    'last_updated: "2026-04-22T12:00:00Z"',
    'last_updated: "2026-04-23T00:43:00Z"'
)
content = content.replace(
    'next_phase: 8',
    'next_phase: 9'
)
content = content.replace(
    'next_action: "/silver:feature Phase 8: Multi-Perspective Verification (KIRA Critics)"',
    'next_action: "/silver:feature Phase 9: CLI Polish + Tauri 2.x GUI"'
)
content = content.replace(
    '  completed_phases: 6',
    '  completed_phases: 8'
)

with open(path, "w") as f:
    f.write(content)

print("STATE_UPDATED_OK")
PYEOF
