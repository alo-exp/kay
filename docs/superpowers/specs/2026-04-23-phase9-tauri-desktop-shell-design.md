# Design: Phase 9 — Tauri Desktop Shell

**Date:** 2026-04-23  
**Phase:** 9  
**Requirements:** TAURI-01..06, UI-01  
**Status:** Rev 4 — build.rs → test-based binding generation; = RC version pins; canary test API note

---

## 1. Problem Statement

Kay's backend is complete through Phase 8. Phase 9 adds the first native desktop window — a Tauri 2.x GUI streaming agent events in real time, making Kay accessible to desktop developers and first-time users without requiring terminal literacy.

**Non-goal (Phase 10):** Multi-session management, settings panel, API key management, model picker.

---

## 2. Scope

### In scope (Phase 9)

- `crates/kay-tauri/` Rust backend: `#[tauri::command]` IPC handlers, `IpcAgentEvent` mirror type, flush task, cancellation via `CancellationToken`
- `crates/kay-tauri/ui/` frontend: React 19 + TypeScript + Vite scaffold
- `ipc::Channel<IpcAgentEvent>` streaming from agent loop to frontend
- tauri-specta v2 TypeScript binding generation with CI drift gate (via `build.rs`)
- Session view: live agent trace, tool-call timeline, token/cost meter
- CodeMirror 6 diff viewer for `edit_file` / `write_file` tool output
- Basic session start (prompt input) and stop
- 4-hour IPC memory canary (nightly CI, macOS + Linux, uses `sysinfo` crate)
- Merged binary build (no `externalBin` sidecar — Tauri #11992)

### Out of scope (Phase 10)

Multi-session manager, settings panel, API key management, OS keychain binding, model allowlist picker.

---

## 3. Architecture

### 3.1 Binary Layout

`kay-tauri` compiles all dependency crates directly into the Tauri app bundle. No `externalBin` sidecar. Tauri #11992 blocks macOS notarization on sidecars as of Tauri 2.10.x; verify this constraint against the final pinned Tauri version at execution time.

```
Kay.app/Contents/MacOS/kay-tauri  ← single merged binary
  includes: kay-core, kay-tools, kay-session, kay-context,
            kay-verifier, kay-provider-openrouter,
            kay-provider-errors, kay-sandbox-{macos,linux,windows}
```

### 3.2 Why `IpcAgentEvent` (not `AgentEvent` directly)

`AgentEvent` (in `crates/kay-tools/src/events.rs`) **cannot** implement `serde::Serialize` or `specta::Type` because:
- `AgentEvent::Error { error: ProviderError }` — `ProviderError` wraps `reqwest::Error` and `serde_json::Error`, neither of which is `Serialize`.
- `AgentEvent::ImageRead { path, bytes }` — `Vec<u8>` serializes as a JSON integer array; a 1 MB image becomes 5 MB of JSON, which is IPC-unsafe.
- `AgentEvent::Retry { reason: RetryReason }` — `RetryReason` is not `Serialize`.

`ipc::Channel<T>` requires `T: Serialize + specta::Type`. Therefore `kay-tauri` defines a separate `IpcAgentEvent` type that:
- Is fully `Serialize + specta::Type + Clone`
- Converts from `AgentEvent` via `From<AgentEvent>`
- Maps `Error` to a human-readable `String` (deliberate Phase 9 simplification)
- Maps `ImageRead` bytes to a base64 data-URL (MIME inferred via `infer` crate)
- Maps `Retry.reason` to `format!("{:?}", reason)`

`AgentEvent` is **never modified**. `IpcAgentEvent` lives entirely in `crates/kay-tauri/src/ipc_event.rs`.

### 3.3 `IpcAgentEvent` Type (reconciled against actual `events.rs`)

```rust
// crates/kay-tauri/src/ipc_event.rs
use serde::Serialize;
use specta::Type;
use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;

/// IPC-safe mirror of AgentEvent. All fields are JSON-serializable.
/// `Clone` is safe — no non-Clone types here (unlike AgentEvent).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum IpcAgentEvent {
    // Phase 2 variants
    TextDelta          { content: String },
    ToolCallStart      { id: String, name: String },
    ToolCallDelta      { id: String, arguments_delta: String },
    ToolCallComplete   { id: String, name: String, arguments: serde_json::Value },
    ToolCallMalformed  { id: String, raw: String, error: String },
    Usage              { prompt_tokens: u64, completion_tokens: u64, cost_usd: f64 },
    Retry              { attempt: u32, delay_ms: u64, reason: String }, // RetryReason → Debug string
    Error              { message: String },                              // ProviderError → .to_string()

    // Phase 3 variants
    ToolOutput         { call_id: String, chunk: IpcToolOutputChunk },
    TaskComplete       { call_id: String, verified: bool, outcome: IpcVerificationOutcome },
    ImageRead          { path: String, data_url: String },  // bytes → base64 data-URL; see §3.4
    SandboxViolation   { call_id: String, tool_name: String, resource: String, policy_rule: String, os_error: Option<i32> },

    // Phase 5 variants
    Paused,
    Aborted            { reason: String },

    // Phase 7 variants
    ContextTruncated   { dropped_symbols: usize, budget_tokens: usize },
    IndexProgress      { indexed: usize, total: usize },

    // Phase 8 variants
    Verification       { critic_role: String, verdict: String, reason: String, cost_usd: f64 },
    VerifierDisabled   { reason: String, cost_usd: f64 },

    // Catch-all: future #[non_exhaustive] variants become Unknown
    Unknown            { event_type: String },
}

/// IPC-safe mirror of ToolOutputChunk (Clone-safe; no non-Clone payloads).
#[derive(Debug, Clone, Serialize, Type)]
pub enum IpcToolOutputChunk {
    Stdout(String),
    Stderr(String),
    Closed { exit_code: Option<i32>, marker_detected: bool },
}

/// IPC-safe mirror of VerificationOutcome (already Serialize + Clone;
/// mirrored to add specta::Type without touching kay-tools).
#[derive(Debug, Clone, Serialize, Type)]
pub enum IpcVerificationOutcome {
    Pending { reason: String },
    Pass    { note: String },
    Fail    { reason: String },
}

impl From<VerificationOutcome> for IpcVerificationOutcome {
    fn from(v: VerificationOutcome) -> Self {
        match v {
            VerificationOutcome::Pending { reason } => Self::Pending { reason },
            VerificationOutcome::Pass { note }      => Self::Pass { note },
            VerificationOutcome::Fail { reason }    => Self::Fail { reason },
            // Future variants via #[non_exhaustive]
            _ => Self::Pending { reason: "unknown_outcome_variant".to_string() },
        }
    }
}

impl From<ToolOutputChunk> for IpcToolOutputChunk {
    fn from(chunk: ToolOutputChunk) -> Self {
        match chunk {
            ToolOutputChunk::Stdout(s)                           => Self::Stdout(s),
            ToolOutputChunk::Stderr(s)                           => Self::Stderr(s),
            ToolOutputChunk::Closed { exit_code, marker_detected } =>
                Self::Closed { exit_code, marker_detected },
            // Future variants via #[non_exhaustive]
            _ => Self::Stdout("[unknown chunk variant]".to_string()),
        }
    }
}

impl From<AgentEvent> for IpcAgentEvent {
    fn from(event: AgentEvent) -> Self {
        match event {
            AgentEvent::TextDelta { content }
                => Self::TextDelta { content },
            AgentEvent::ToolCallStart { id, name }
                => Self::ToolCallStart { id, name },
            AgentEvent::ToolCallDelta { id, arguments_delta }
                => Self::ToolCallDelta { id, arguments_delta },
            AgentEvent::ToolCallComplete { id, name, arguments }
                => Self::ToolCallComplete { id, name, arguments },
            AgentEvent::ToolCallMalformed { id, raw, error }
                => Self::ToolCallMalformed { id, raw, error },
            AgentEvent::Usage { prompt_tokens, completion_tokens, cost_usd }
                => Self::Usage { prompt_tokens, completion_tokens, cost_usd },
            AgentEvent::Retry { attempt, delay_ms, reason }
                => Self::Retry { attempt, delay_ms, reason: format!("{:?}", reason) },
            AgentEvent::Error { error }
                => Self::Error { message: error.to_string() },
            AgentEvent::ToolOutput { call_id, chunk }
                => Self::ToolOutput { call_id, chunk: IpcToolOutputChunk::from(chunk) },
            AgentEvent::TaskComplete { call_id, verified, outcome }
                => Self::TaskComplete { call_id, verified, outcome: IpcVerificationOutcome::from(outcome) },
            AgentEvent::ImageRead { path, bytes }
                => Self::ImageRead { path: path.clone(), data_url: bytes_to_data_url(&path, &bytes) },
            AgentEvent::SandboxViolation { call_id, tool_name, resource, policy_rule, os_error }
                => Self::SandboxViolation { call_id, tool_name, resource, policy_rule, os_error },
            AgentEvent::Paused
                => Self::Paused,
            AgentEvent::Aborted { reason }
                => Self::Aborted { reason },
            AgentEvent::ContextTruncated { dropped_symbols, budget_tokens }
                => Self::ContextTruncated { dropped_symbols, budget_tokens },
            AgentEvent::IndexProgress { indexed, total }
                => Self::IndexProgress { indexed, total },
            AgentEvent::Verification { critic_role, verdict, reason, cost_usd }
                => Self::Verification { critic_role, verdict, reason, cost_usd },
            AgentEvent::VerifierDisabled { reason, cost_usd }
                => Self::VerifierDisabled { reason, cost_usd },
            // Future #[non_exhaustive] variants
            _ => Self::Unknown { event_type: "future_variant".to_string() },
        }
    }
}
```

### 3.4 `ImageRead` MIME Inference

`AgentEvent::ImageRead` has no MIME type field. Infer from file bytes using the `infer` crate (already in workspace at `infer = "0.19.0"`):

```rust
fn bytes_to_data_url(path: &str, bytes: &[u8]) -> String {
    use base64::Engine;
    let mime = infer::get(bytes)
        .map(|t| t.mime_type())
        .unwrap_or_else(|| {
            // Fallback: infer from path extension
            match path.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
                "png"  => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif"  => "image/gif",
                "webp" => "image/webp",
                _      => "application/octet-stream",
            }
        });
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime, b64)
}
```

---

## 4. IPC Command Handlers

### 4.1 AppState

```rust
// crates/kay-tauri/src/state.rs
use tokio_util::sync::CancellationToken;
use dashmap::DashMap;

pub struct AppState {
    /// Maps session_id → cancellation token for stop_session()
    pub sessions: DashMap<String, CancellationToken>,
}
```

`stop_session` calls `token.cancel()` — the agent loop receives cancellation via `token.cancelled()` and emits `AgentEvent::Aborted`. This is correct even if the agent holds its own `mpsc::tx` clone (no sender-drop race).

### 4.2 IPC Commands

```rust
// crates/kay-tauri/src/commands.rs
#[tauri::command]
#[specta::specta]
pub async fn start_session(
    prompt: String,
    persona: String,
    channel: Channel<IpcAgentEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let token = CancellationToken::new();
    state.sessions.insert(session_id.clone(), token.clone());

    let (tx, rx) = tokio::sync::mpsc::channel::<AgentEvent>(1024);

    // Flush task: AgentEvent → IpcAgentEvent, batch to 16ms, send via channel
    tokio::spawn(flush_task(rx, channel));

    // Agent loop: emits to tx; respects cancellation token
    tokio::spawn(run_agent_loop(prompt, persona, session_id.clone(), tx, token));

    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub async fn stop_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some((_, token)) = state.sessions.remove(&session_id) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_status(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<SessionStatus, String> {
    match state.sessions.contains_key(&session_id) {
        true  => Ok(SessionStatus::Running),
        false => Ok(SessionStatus::Complete),
    }
}

// Errors are String in Phase 9 — deliberate simplification.
// Phase 10 will introduce typed IPC errors.

#[derive(Debug, Clone, Serialize, Type)]
pub enum SessionStatus {
    Running,
    Complete,
    // Aborted is Phase 10 — removing from Phase 9 scope to avoid dead code.
}
```

### 4.3 Flush Task (corrected final drain)

```rust
// crates/kay-tauri/src/flush.rs
pub async fn flush_task(mut rx: mpsc::Receiver<AgentEvent>, channel: Channel<IpcAgentEvent>) {
    let mut ticker = tokio::time::interval(Duration::from_millis(16));
    let mut buffer: Vec<AgentEvent> = Vec::with_capacity(64);

    loop {
        tokio::select! {
            maybe = rx.recv() => {
                match maybe {
                    Some(event) => {
                        buffer.push(event);
                        if buffer.len() >= 64 { do_flush(&channel, &mut buffer); }
                    }
                    None => {
                        // Sender side dropped — flush remaining then exit
                        do_flush(&channel, &mut buffer);
                        break;
                    }
                }
            }
            _ = ticker.tick() => { do_flush(&channel, &mut buffer); }
        }
    }
}

fn do_flush(channel: &Channel<IpcAgentEvent>, buffer: &mut Vec<AgentEvent>) {
    for event in buffer.drain(..) {
        if let Err(e) = channel.send(IpcAgentEvent::from(event)) {
            tracing::warn!("ipc channel send error: {e:?}"); // never fatal
        }
    }
}
```

---

## 5. Frontend Design

### 5.1 Component Hierarchy

```
<App>
└── <SessionView>
    ├── <SessionHeader>            session ID, model, persona, elapsed time
    │   └── <CostMeter>           tokens in/out (live) + session USD cost
    ├── <AgentTrace>              auto-scroll log; pause on user scroll
    │   └── <EventRow>            dispatches on event.type (see §5.2)
    │       ├── <TextRow>         TextDelta
    │       ├── <ToolCallCard>    ToolCallComplete + ToolOutput (collapsed by default)
    │       │   └── <DiffViewer> CodeMirror 6 for edit_file/write_file diffs
    │       ├── <VerificationCard>  Verification
    │       ├── <UsageRow>        Usage (per-turn cost summary)
    │       ├── <SandboxAlertRow> SandboxViolation
    │       └── <UnknownEventRow> all other types (required fallback)
    ├── <ToolCallTimeline>        horizontal bar; one entry per ToolCallComplete, color by name
    └── <PromptInput>             textarea + "Run" / "Stop" button
```

### 5.2 `#[non_exhaustive]` Safety — TypeScript `never` check

```typescript
// src/components/EventRow.tsx
import type { IpcAgentEvent } from '../bindings';

function EventRow({ event }: { event: IpcAgentEvent }) {
  switch (event.type) {
    case 'TextDelta':        return <TextRow event={event} />;
    case 'ToolCallStart':    return null; // buffered; shown when ToolCallComplete arrives
    case 'ToolCallDelta':    return null; // streaming; buffered in state
    case 'ToolCallComplete': return <ToolCallCard event={event} />;
    case 'ToolCallMalformed': return <ErrorRow message={`Malformed tool call: ${event.data.raw}`} />;
    case 'ToolOutput':       return null; // streamed into active ToolCallCard
    case 'TaskComplete':     return <TaskCompleteRow event={event} />;
    case 'ImageRead':        return <ImageRow dataUrl={event.data.data_url} path={event.data.path} />;
    case 'SandboxViolation': return <SandboxAlertRow event={event} />;
    case 'Paused':           return <PausedRow />;
    case 'Aborted':          return <AbortedRow reason={event.data.reason} />;
    case 'Usage':            return <UsageRow event={event} />;
    case 'Retry':            return <RetryRow event={event} />;
    case 'Error':            return <ErrorRow message={event.data.message} />;
    case 'ContextTruncated': return <ContextTruncatedRow event={event} />;
    case 'IndexProgress':    return null; // rendered in status bar, not trace
    case 'Verification':     return <VerificationCard event={event} />;
    case 'VerifierDisabled': return <VerifierDisabledRow event={event} />;
    case 'Unknown':          return <UnknownEventRow type={event.data.event_type} />;
    default: {
      const _exhaustiveCheck: never = event; // compile error if new IpcAgentEvent variant unhandled
      return <UnknownEventRow type={(event as { type: string }).type} />;
    }
  }
}
```

### 5.3 State Management

```typescript
type Action =
  | { type: 'SESSION_STARTED'; sessionId: string }
  | { type: 'EVENTS_BATCH'; events: IpcAgentEvent[] }
  | { type: 'SESSION_ENDED' };

interface SessionState {
  sessionId: string | null;
  events: IpcAgentEvent[];
  totalCostUsd: number;
  totalTokensIn: number;
  totalTokensOut: number;
  status: 'idle' | 'running' | 'complete';
}

function reducer(state: SessionState, action: Action): SessionState {
  switch (action.type) {
    case 'EVENTS_BATCH': {
      let { totalCostUsd, totalTokensIn, totalTokensOut } = state;
      for (const ev of action.events) {
        if (ev.type === 'Usage') {
          totalCostUsd    += ev.data.cost_usd;
          totalTokensIn   += ev.data.prompt_tokens;
          totalTokensOut  += ev.data.completion_tokens;
        }
        if (ev.type === 'Verification') {
          totalCostUsd += ev.data.cost_usd;
        }
      }
      return { ...state, events: [...state.events, ...action.events],
               totalCostUsd, totalTokensIn, totalTokensOut };
    }
    // ...
  }
}
```

### 5.4 Diff Viewer

CodeMirror 6 via `@uiw/react-codemirror` + `@codemirror/merge`. Shown for `ToolCallComplete` events where `event.data.name === 'edit_file'` or `'write_file'`, extracting `{ original, modified }` from the tool args and subsequent `TaskComplete` outcome. The viewer is lazy-loaded (dynamic import) to keep initial bundle < 300 KB.

---

## 6. tauri-specta v2 Binding Generation

### 6.1 Version Pins (add to `[workspace.dependencies]`)

| Package | Version | Notes |
|---------|---------|-------|
| `tauri` | `"2.3"` | Tauri 2.x stable; verify latest 2.x patch at execution time |
| `tauri-build` | `"2.3"` | Must match `tauri` minor |
| `tauri-specta` | `"=2.0.0-rc.21"` | RC; `=` pin prevents accidental upgrades. Verify specta compatibility at execution time. |
| `specta` | `"=2.0.0-rc.20"` | Must use `=` pin and match tauri-specta's declared specta requirement exactly. |
| `specta-typescript` | `"0.0.7"` | Stable |
| `tokio-util` | `"0.7"` | Already in workspace; needed for `CancellationToken` |

**Compatibility rule:** Check `tauri-specta/Cargo.toml` at execution time to confirm the exact `specta` version it depends on. They must use the same RC sub-version. Pin with `=` if needed.

### 6.2 Binding Generation — Integration Test (NOT `build.rs` or `main.rs`)

`collect_commands!` is a proc-macro that requires function items to be in scope during compilation. Build scripts (`build.rs`) are compiled separately from the main crate and cannot access `crate::commands::*` items. Integration tests in `tests/` ARE compiled with access to the library crate's public API, making them the correct place for this.

```rust
// crates/kay-tauri/build.rs  — minimal, no binding generation
fn main() {
    tauri_build::build();
}
```

```rust
// crates/kay-tauri/tests/gen_bindings.rs
#[test]
fn export_tauri_bindings() {
    tauri_specta::Builder::<tauri_specta::Typescript>::new()
        .commands(tauri_specta::collect_commands![
            kay_tauri::commands::start_session,
            kay_tauri::commands::stop_session,
            kay_tauri::commands::get_session_status,
        ])
        .export(
            specta_typescript::Typescript::default(),
            // Use CARGO_MANIFEST_DIR for absolute path — integration test working
            // directory is not guaranteed to be the package root.
            concat!(env!("CARGO_MANIFEST_DIR"), "/ui/src/bindings.ts"),
        )
        .expect("export tauri-specta bindings");
}
```

**Requirements:** `commands` module and its three functions must be `pub` in `lib.rs` / `main.rs`. `tauri-specta` must be in `[dev-dependencies]` of `crates/kay-tauri/Cargo.toml` as well as `[dependencies]` (it is used at both test and runtime).

### 6.3 CI Drift Gate

```bash
#!/usr/bin/env bash
# scripts/check-bindings.sh
set -euo pipefail
COMMITTED="crates/kay-tauri/ui/src/bindings.ts"
cp "$COMMITTED" /tmp/bindings-committed.ts
# Re-run the test that exports bindings (integration test, not build.rs)
cargo test -p kay-tauri --test gen_bindings export_tauri_bindings 2>/dev/null
if ! diff -q "$COMMITTED" /tmp/bindings-committed.ts > /dev/null; then
  echo "ERROR: bindings.ts is out of sync. Run: cargo test -p kay-tauri --test gen_bindings && git add crates/kay-tauri/ui/src/bindings.ts"
  exit 1
fi
echo "bindings.ts: OK"
```

---

## 7. Frontend Scaffold

### 7.1 Package Manager: `pnpm`

### 7.2 `package.json`

```json
{
  "name": "kay-tauri-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev":     "vite",
    "build":   "tsc && vite build",
    "lint":    "eslint src --ext ts,tsx",
    "preview": "vite preview",
    "test":    "vitest run"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@uiw/react-codemirror": "^4",
    "@codemirror/merge": "^6",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "typescript": "^5",
    "vite": "^6",
    "@vitejs/plugin-react": "^4",
    "vitest": "^2",
    "@testing-library/react": "^16",
    "@testing-library/user-event": "^14",
    "eslint": "^9",
    "@typescript-eslint/eslint-plugin": "^8",
    "@typescript-eslint/parser": "^8"
  }
}
```

### 7.3 `vite.config.ts`

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  build: {
    target: 'chrome105',
    outDir: 'dist',
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

### 7.4 `tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Kay",
  "version": "0.1.0",
  "identifier": "dev.kay.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [{ "label": "main", "title": "Kay", "width": 1200, "height": 800, "resizable": true }],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

---

## 8. Cargo.toml Changes

### 8.1 Workspace `Cargo.toml` — add to `[workspace.dependencies]`

```toml
# Phase 9 (kay-tauri) additions
tauri             = { version = "2.3", features = ["rustls-tls"] }
tauri-build       = { version = "2.3" }
tauri-specta      = { version = "=2.0.0-rc.21", features = ["derive"] }  # = pin: RC, must match specta exactly
specta            = { version = "=2.0.0-rc.20" }                          # = pin: must match tauri-specta's declared dep
specta-typescript = "0.0.7"
# dashmap = "6" is already in workspace.dependencies — do not duplicate
```

### 8.2 `crates/kay-tauri/Cargo.toml`

```toml
[package]
name    = "kay-tauri"
version.workspace   = true
edition.workspace   = true

[build-dependencies]
tauri-build       = { workspace = true, features = ["codegen"] }
tauri-specta      = { workspace = true }  # provides collect_commands! + Builder
specta-typescript = { workspace = true }

[dependencies]
tauri             = { workspace = true }
tauri-specta      = { workspace = true }  # both build and runtime — collect_commands! used in build.rs;
                                           # Channel<T> + generate_handler! used at runtime
specta            = { workspace = true }
serde             = { workspace = true }
serde_json        = { workspace = true }
tokio             = { workspace = true }
tokio-util        = { workspace = true }
uuid              = { workspace = true }
base64            = { workspace = true }
infer             = { workspace = true }
dashmap           = { workspace = true }
tracing           = { workspace = true }
kay-core          = { path = "../kay-core" }
kay-tools         = { path = "../kay-tools" }
kay-provider-errors = { path = "../kay-provider-errors" }
kay-session       = { path = "../kay-session" }
kay-context       = { path = "../kay-context" }
kay-verifier      = { path = "../kay-verifier" }
kay-provider-openrouter = { path = "../kay-provider-openrouter" }
kay-sandbox-policy = { path = "../kay-sandbox-policy" }

[target.'cfg(target_os = "macos")'.dependencies]
kay-sandbox-macos   = { path = "../kay-sandbox-macos" }
[target.'cfg(target_os = "linux")'.dependencies]
kay-sandbox-linux   = { path = "../kay-sandbox-linux" }
[target.'cfg(target_os = "windows")'.dependencies]
kay-sandbox-windows = { path = "../kay-sandbox-windows" }

[dev-dependencies]
tauri    = { workspace = true, features = ["test"] }
sysinfo  = { workspace = true }
```

---

## 9. 4-Hour Memory Canary

### 9.1 RSS Measurement via `sysinfo` (already in workspace)

```rust
fn process_rss_bytes() -> u64 {
    use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory())
    );
    sys.refresh_processes();
    sys.process(Pid::from(std::process::id() as usize))
        .map(|p| p.memory())
        .unwrap_or(0)
}
```

### 9.2 Canary Test

> **Implementation note (WARN-01):** Verify the exact Tauri 2.3 test API surface before writing this test. `tauri::test::mock_builder()`, `tauri::test::mock_context()`, and `tauri::generate_assets!()` existed in Tauri 1.x but may have different names or signatures in Tauri 2.3. Create a minimal compile check first:
> ```rust
> use tauri::test::{mock_builder, mock_context}; // verify these exist
> ```
> If the API differs, update the canary test accordingly. The _concept_ (RSS measurement every 60s for 4h) is stable; the _scaffolding_ is implementation-time detail.

```rust
// crates/kay-tauri/tests/memory_canary.rs
// Requires tauri.conf.json + ui/dist to exist (built by CI before this test runs).
// VERIFY Tauri 2.3 test API names before finalizing — see note above.
#[test]
#[ignore] // cargo test -p kay-tauri --test memory_canary -- --ignored --nocapture
fn four_hour_ipc_canary() {
    let app = tauri::test::mock_builder()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![start_session, stop_session, get_session_status])
        .build(tauri::test::mock_context(tauri::generate_assets!()))
        .unwrap();

    let baseline = process_rss_bytes();
    let deadline = Instant::now() + Duration::from_secs(4 * 3600);

    while Instant::now() < deadline {
        hammer_ipc_channel(&app, 1000); // 1000 IpcAgentEvent per burst
        std::thread::sleep(Duration::from_secs(60));
        let delta_mb = process_rss_bytes().saturating_sub(baseline) / (1024 * 1024);
        assert!(delta_mb < 50, "RSS leak: +{delta_mb}MB");
    }
}
```

### 9.3 CI Job (`canary.yml`)

```yaml
name: Memory Canary (nightly)
on:
  schedule:
    - cron: '0 2 * * *'
jobs:
  canary:
    strategy:
      matrix: { os: [macos-14, ubuntu-latest] }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with: { version: 9 }
      - name: Build frontend
        working-directory: crates/kay-tauri/ui
        run: pnpm install && pnpm build
      - name: Run 4h canary
        run: cargo test -p kay-tauri --test memory_canary -- --ignored --nocapture
        timeout-minutes: 260
      - uses: actions/upload-artifact@v4
        with: { name: canary-report-${{ matrix.os }}, path: canary-report.json }
```

---

## 10. Test Strategy

### Tier 1: Unit (Rust + TypeScript)
- `IpcAgentEvent::from(AgentEvent::Error { .. })` → `IpcAgentEvent::Error { message }`
- `IpcAgentEvent::from(AgentEvent::ImageRead { .. })` → valid base64 data-URL (using `infer`)
- `IpcAgentEvent::from(AgentEvent::Retry { .. })` → `reason` is `format!("{:?}", RetryReason::RateLimited)` = `"RateLimited"`
- `IpcToolOutputChunk::from(ToolOutputChunk::Closed { .. })` preserves `exit_code`/`marker_detected`
- `flush_task`: 16ms timer flushes; 64-event size cap triggers early flush; final flush on `None` (no event loss)
- TypeScript: `<EventRow>` snapshot for each known `IpcAgentEvent` type; `default:` arm renders `<UnknownEventRow>`; TypeScript `never` check catches missing cases at compile time

### Tier 2: Integration (Rust — `tauri::test`)
- `start_session` → N events → verify ordering and completeness via mock channel
- `stop_session` cancels token → agent loop emits `Aborted` → flush_task drains and exits
- Memory canary (4h, nightly)

### Tier 3: E2E / Smoke (macOS host, `mcp__computer-use__*`)
- Launch `Kay.app`, verify window renders
- Enter prompt in `<PromptInput>`, click Run, verify `<AgentTrace>` receives events
- `mcp__Claude_in_Chrome__*` for React DOM inspection

### Tier 4: Property (proptest)
- `IpcAgentEvent` JSON serialization round-trip for every variant
- `flush_task`: proptest event sequences verify zero events dropped

---

## 11. Non-Negotiable Constraints

1. **No `externalBin` sidecar** — Tauri #11992. All crates compile into the main binary.
2. **`AgentEvent` is additive only** — `IpcAgentEvent` mirrors without modifying.
3. **`IpcAgentEvent` owns all IPC concerns** — `Serialize`, `Type`, base64, MIME inference live in `kay-tauri`, not `kay-tools`.
4. **Binding generation in `build.rs`** — never in `main.rs`.
5. **`stop_session` uses `CancellationToken`** — not sender-drop (unreliable with cloned senders).
6. **DCO on every commit** — `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`.
7. **Branch: `phase/09-tauri-desktop-shell`** — PR only, no direct main commits.
8. **100% TDD** — RED commit before GREEN commit per wave.
