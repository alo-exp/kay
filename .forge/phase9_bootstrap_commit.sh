#!/usr/bin/env bash
set -euo pipefail
cd /Users/shafqat/Documents/Projects/opencode/vs-others

git add \
  .planning/WORKFLOW.md \
  .planning/ROADMAP.md \
  .planning/STATE.md \
  docs/ARCHITECTURE.md \
  "docs/superpowers/specs/2026-04-23-phase9-tauri-desktop-shell-design.md" \
  ".planning/phases/09-tauri-desktop-shell/WORKFLOW.md"

git commit -m "chore(phase9): bootstrap — spec Rev 4, WORKFLOW.md, quality gates PASS

- Fix ROADMAP.md: mark Phase 7 + 8 complete with PR refs
- Update STATE.md: advance cursor to Phase 9 Tauri Desktop Shell
- Fix ARCHITECTURE.md: correct Phase 9 = Tauri, Phase 9.5 = TUI label swap
- Add Phase 9 design spec Rev 4 (IpcAgentEvent mirror type, test-based
  binding generation, = RC version pins, 16ms flush, CancellationToken)
- Add .planning/WORKFLOW.md: silver-bullet path log (primary commit gate)
- Add Phase 9 WORKFLOW.md tracker
- All 9 silver-quality-gates dimensions PASS (design-time)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "Bootstrap commit done: $(git rev-parse --short HEAD)"
