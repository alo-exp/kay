#!/usr/bin/env bash
set -euo pipefail
cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Make check-bindings.sh executable
chmod +x scripts/check-bindings.sh

git add \
  .planning/WORKFLOW.md \
  docs/.planning/WORKFLOW.md \
  .planning/phases/09-tauri-desktop-shell/09-PLAN.md \
  Cargo.toml \
  Cargo.lock \
  crates/kay-tauri/Cargo.toml \
  crates/kay-tauri/build.rs \
  crates/kay-tauri/src/lib.rs \
  crates/kay-tauri/src/ipc_event.rs \
  crates/kay-tauri/src/state.rs \
  crates/kay-tauri/src/flush.rs \
  crates/kay-tauri/src/commands.rs \
  crates/kay-tauri/src/main.rs \
  crates/kay-tauri/src-tauri/tauri.conf.json \
  crates/kay-tauri/ui/package.json \
  crates/kay-tauri/ui/tsconfig.json \
  crates/kay-tauri/ui/vite.config.ts \
  crates/kay-tauri/ui/index.html \
  crates/kay-tauri/ui/.gitignore \
  crates/kay-tauri/ui/src/bindings.ts \
  crates/kay-tauri/ui/src/main.tsx \
  crates/kay-tauri/ui/src/styles.css \
  crates/kay-tauri/ui/src/components/App.tsx \
  crates/kay-tauri/ui/src/components/SessionView.tsx \
  crates/kay-tauri/ui/src/components/CostMeter.tsx \
  crates/kay-tauri/ui/src/components/ToolCallTimeline.tsx \
  crates/kay-tauri/ui/src/components/AgentTrace.tsx \
  crates/kay-tauri/ui/src/components/EventRow.tsx \
  crates/kay-tauri/ui/src/components/DiffViewer.tsx \
  crates/kay-tauri/ui/src/components/PromptInput.tsx \
  crates/kay-tauri/tests/gen_bindings.rs \
  crates/kay-tauri/tests/memory_canary.rs \
  scripts/check-bindings.sh \
  .github/workflows/canary.yml

git commit -m "$(cat <<'EOF'
feat(phase9): Tauri desktop shell — IpcAgentEvent, flush task, React UI, memory canary

Wave 1-7: complete Tauri 2.x desktop shell for Kay.

Rust backend (crates/kay-tauri/):
- IpcAgentEvent: IPC-safe mirror of AgentEvent (18 variants, all From<> impls)
  - Error → message string, ImageRead → base64 data-URL (infer crate),
    Retry.reason → Debug string, unknown future variants → Unknown
- flush_task: 16ms interval, 64-event size cap, final drain on sender-drop
- AppState: DashMap<session_id, CancellationToken> for stop_session
- start_session / stop_session / get_session_status IPC commands
- Offline provider for Phase 9 (Phase 10 adds OpenRouter key management)
- build.rs: minimal tauri_build::build()
- tests/gen_bindings.rs: tauri-specta export via integration test (not build.rs)
- tests/memory_canary.rs: 4h RSS canary (run --ignored in CI)

Frontend (crates/kay-tauri/ui/):
- React 19 + TypeScript + Vite scaffold
- Dark theme via CSS custom properties (no external UI lib)
- SessionView: AgentTrace auto-scroll, ToolCallTimeline, CostMeter
- EventRow: full switch dispatch + TypeScript `never` exhaustiveness check
- VerificationCard: critic_role + verdict badge + reason
- DiffViewer: lazy-loaded CodeMirror 6 (keeps initial bundle < 300 KB)
- PromptInput: textarea + persona picker + Cmd+Enter / Esc keyboard shortcuts

CI:
- .github/workflows/canary.yml: nightly 4h memory canary (macOS + Linux)
- scripts/check-bindings.sh: drift gate for committed bindings.ts

Workspace deps added: tauri 2.3, tauri-build 2.3,
  tauri-specta =2.0.0-rc.21, specta =2.0.0-rc.20, specta-typescript 0.0.7

TAURI-01 TAURI-02 TAURI-03 TAURI-04 TAURI-05 TAURI-06 UI-01

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
