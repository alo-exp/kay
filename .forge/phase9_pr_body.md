## Summary

Phase 9 delivers the Tauri 2.x desktop shell for Kay — a production-quality native GUI that streams AgentEvent to a React 19 frontend via ipc::Channel<IpcAgentEvent>.

**Requirements covered:** TAURI-01 TAURI-02 TAURI-03 TAURI-04 TAURI-05 TAURI-06 UI-01

### Rust backend (crates/kay-tauri/)

- **IpcAgentEvent** — IPC-safe mirror of AgentEvent (18 variants). AgentEvent is NOT modified (additive-only constraint preserved). Conversions: Error to String, ImageRead.bytes to base64 data-URL (via infer crate), Retry.reason to Debug string, unknown future variants to Unknown.
- **flush_task** — 16ms interval batch flush; 64-event size cap for heavy PTY output; final drain on sender-drop (zero event loss).
- **AppState** — DashMap<session_id, CancellationToken> for reliable stop_session cancellation.
- **Three IPC commands** — start_session, stop_session, get_session_status with #[specta::specta] annotations.
- **build.rs** — minimal tauri_build::build() (no binding generation).
- **tests/gen_bindings.rs** — tauri-specta export via integration test (NOT build.rs — build scripts cannot access main crate items).
- **tests/memory_canary.rs** — 4h RSS canary; process_rss_is_nonzero smoke test always runs; 4h test runs --ignored in CI.

### Frontend (crates/kay-tauri/ui/)

- React 19 + TypeScript + Vite; dark theme via CSS custom properties (no external UI lib)
- SessionView: AgentTrace auto-scroll (pauses on user scroll), ToolCallTimeline (horizontal dot bar), CostMeter (tokens + USD)
- EventRow: full switch dispatch over all 19 IpcAgentEvent variants + TypeScript never exhaustiveness check
- VerificationCard: critic role + pass/fail badge + reason text (KIRA multi-perspective verification)
- DiffViewer: lazy-loaded CodeMirror 6 for edit_file/write_file diffs; initial bundle < 300 KB
- PromptInput: textarea + persona picker + Cmd+Enter to run + Esc to stop

### CI

- .github/workflows/canary.yml — nightly 4h memory canary (macOS-14 + ubuntu-latest)
- scripts/check-bindings.sh — drift gate for committed bindings.ts

### Architecture constraints honored

- No externalBin sidecar (Tauri #11992 — macOS notarization)
- AgentEvent not modified (additive-only)
- stop_session uses CancellationToken (not sender-drop)
- Workspace RC pins: tauri-specta =2.0.0-rc.21, specta =2.0.0-rc.20
- DCO on every commit

## Test plan

- [ ] cargo check -p kay-tauri — compiles clean on macOS + Linux
- [ ] cargo test -p kay-tauri — unit tests (ipc_event conversions, flush, canary smoke)
- [ ] cargo test -p kay-tauri --test gen_bindings — generates ui/src/bindings.ts
- [ ] scripts/check-bindings.sh — exits 0 (bindings in sync)
- [ ] cd crates/kay-tauri/ui && pnpm install && pnpm build — TypeScript compiles + Vite bundles
- [ ] cargo test -p kay-tauri --test memory_canary -- --ignored — 4h canary (nightly CI)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
