# Architecture Research

**Domain:** Open-source terminal coding agent with native desktop UI (Rust + Tauri 2.x)
**Researched:** 2026-04-19
**Confidence:** HIGH (core components cross-verified from Codex CLI, ForgeCode, claw-code, KIRA sources)

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FRONTEND — Tauri WebView (React/TS)              │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │ Session  │  │ Timeline │  │ Token/$  │  │ Settings │             │
│  │   View   │  │   Pane   │  │  Meter   │  │  Panel   │             │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       └─────────────┴─────────────┴─────────────┘                   │
│                             │                                       │
│                    [@tauri-apps/api invoke + Channels]              │
├═════════════════════════════╪═══════════════════════════════════════┤
│                    TAURI IPC BOUNDARY                               │
│         invoke()  →  Commands        Channels  →  streaming         │
├═════════════════════════════╪═══════════════════════════════════════┤
│                             │                                       │
│                    RUST BACKEND (kay-core)                          │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      SESSION MANAGER                         │    │
│  │  (Thread → Turn → Item state machine; spawn/pause/fork)      │    │
│  └───────────┬─────────────────────────────────────────────────┘    │
│              │                                                      │
│  ┌───────────▼─────────────────────────────────────────────────┐    │
│  │                      AGENT LOOP (async, event-driven)        │    │
│  │  ┌────────┐    ┌──────────┐    ┌────────┐    ┌───────────┐   │    │
│  │  │ Prompt │───▶│  Model   │───▶│  Tool  │───▶│  Verify   │   │    │
│  │  │Compose │    │  Call    │    │ Router │    │  Pass     │   │    │
│  │  └────────┘    └──────────┘    └────────┘    └───────────┘   │    │
│  │       ▲             │               │              │         │    │
│  │       └─────────────┴───────────────┴──────────────┘         │    │
│  │                  (turn → turn loop)                          │    │
│  └────┬────────┬─────────┬───────────┬──────────┬──────────────┘    │
│       │        │         │           │          │                   │
│  ┌────▼──┐ ┌───▼────┐ ┌──▼─────┐ ┌──▼────┐ ┌───▼──────┐             │
│  │Agent  │ │ Tool   │ │Context │ │Provider│ │ Verifier │             │
│  │Switch │ │Registry│ │ Engine │ │  HAL   │ │  Critics │             │
│  │(k/s/m)│ │(trait) │ │(TS+sym)│ │(OR v1) │ │(KIRA 4.3)│             │
│  └───────┘ └────┬───┘ └────┬───┘ └───┬────┘ └──────────┘             │
│                 │          │         │                               │
│  ┌──────────────▼──────────▼─────────▼──────────────────────────┐   │
│  │                   PLATFORM LAYER                             │   │
│  │  Sandbox (sandbox-exec / Landlock+seccomp / Job Objects)     │   │
│  │  FS (git-aware) │ Shell (PTY+marker) │ HTTP (reqwest+SSE)    │   │
│  └──────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                    STORAGE (on-disk, XDG-compliant)                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │ Sessions │  │ Context  │  │ Snapshots│  │ Config + │             │
│  │ SQLite   │  │  Index   │  │  (edits) │  │ Secrets  │             │
│  │ + JSONL  │  │ (sled)   │  │          │  │(keyring) │             │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘             │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **Session Manager** | Owns the Thread/Turn/Item state machine; spawn, pause, resume, fork sessions; hands a fresh agent loop per Thread | Codex-style: SQLite index + JSONL transcript per session, inspired by claude-code's `~/.claude/projects/` layout |
| **Agent Loop** | Event-driven core that emits events (UserInput, ModelDelta, ToolCall, ToolResult, VerifyResult, TurnEnd) and advances the state machine | Rust async `tokio::select!` over input+model+tool channels; NOT a blocking REPL |
| **Agent Switch** | Routes the current turn to `kay` (write), `sage` (read-only research), or `muse` (plan) personas | Shared trait object; personas = prompt+tool-filter+model pair; switching is data not code (YAML-driven like ForgeCode `forge.yaml`) |
| **Tool Registry** | Type-erased `trait Tool` objects registered at startup; supplies JSON schemas to the model; dispatches tool calls by name | Trait object map `HashMap<&'static str, Arc<dyn Tool>>`; compile-time registration via inventory/linkme crate OR runtime via explicit `register()` calls |
| **Context Engine** | Index function signatures and module boundaries; retrieve relevant symbols on demand; assemble into prompt snippets | tree-sitter parse → symbol extraction → embedding + BM25 hybrid → sled KV store; ForgeCode-style structured context, not raw file dumps |
| **Provider HAL** | Single `trait ChatProvider` with stream, tool-call, and error taxonomy; OpenRouter impl in v1 with direct-API shims for v2 | `async_trait`-based; returns `impl Stream<Item = Result<ChatDelta, ProviderError>>`; leverages `openrouter_api` or `edgequake-llm` crate patterns |
| **Verifier (Critics)** | Multi-perspective completion verification (test engineer + QA engineer + end-user) — the KIRA 4.3 pattern; runs BEFORE `task_complete` returns success | Three sequential or parallel LLM calls against the turn trace; returns PASS / FAIL+feedback to the main loop |
| **Sandbox** | Isolated shell execution per-OS; filesystem + syscall restrictions | macOS: `sandbox-exec` (Seatbelt profile); Linux: Landlock + seccomp-bpf (Codex pattern); Windows: Job Objects + restricted token |
| **Shell Runner** | PTY-backed command execution with marker-based completion polling | `portable-pty` crate + `__CMDEND__<seq>__` sentinel (KIRA pattern) |
| **IPC Bridge** | Exposes agent events and commands to Tauri frontend | `#[tauri::command]` for request/response; `tauri::ipc::Channel` for streaming deltas |

---

## Recommended Project Structure

```
kay/                                 # Cargo workspace root
├── Cargo.toml                       # [workspace] — ForgeCode-style multi-crate
├── NOTICE                           # Apache-2.0 attribution to ForgeCode/KIRA
│
├── crates/
│   ├── kay-core/                    # Pure library — no I/O — the brain
│   │   ├── src/
│   │   │   ├── agent/               # Loop, personas (kay/sage/muse), switching
│   │   │   │   ├── loop.rs          # Main event-driven run-loop
│   │   │   │   ├── persona.rs       # Trait + kay/sage/muse impls
│   │   │   │   └── turn.rs          # Turn state machine
│   │   │   ├── session/             # Thread/Turn/Item primitives
│   │   │   ├── event.rs             # AgentEvent enum (streamed to UI)
│   │   │   └── error.rs             # Kay-wide error taxonomy
│   │   └── Cargo.toml
│   │
│   ├── kay-tools/                   # Tool trait + built-in tools
│   │   ├── src/
│   │   │   ├── tool.rs              # trait Tool { name, schema, invoke }
│   │   │   ├── registry.rs          # HashMap-based registry
│   │   │   ├── builtin/
│   │   │   │   ├── execute_commands.rs   # KIRA-style bash
│   │   │   │   ├── task_complete.rs      # triggers Verifier
│   │   │   │   ├── image_read.rs         # multimodal base64
│   │   │   │   ├── read_file.rs          # fs read
│   │   │   │   ├── edit_file.rs          # fs write + snapshot
│   │   │   │   ├── search.rs             # ripgrep wrapper
│   │   │   │   └── fetch.rs              # HTTP for docs
│   │   │   └── permissions.rs       # permission-gating (claw-code pattern)
│   │   └── Cargo.toml
│   │
│   ├── kay-context/                 # Semantic context engine
│   │   ├── src/
│   │   │   ├── parser/              # tree-sitter wrappers (rs, ts, py, go, ...)
│   │   │   ├── symbols.rs           # function sigs + module boundaries
│   │   │   ├── index.rs             # sled-backed index
│   │   │   ├── retrieval.rs         # hybrid BM25 + embedding
│   │   │   └── compose.rs           # prompt-snippet assembly
│   │   └── Cargo.toml
│   │
│   ├── kay-providers/               # LLM provider HAL
│   │   ├── src/
│   │   │   ├── provider.rs          # trait ChatProvider
│   │   │   ├── stream.rs            # ChatDelta, tool-call reassembly
│   │   │   ├── error.rs             # ProviderError taxonomy
│   │   │   └── openrouter/          # v1's only impl
│   │   │       ├── client.rs        # reqwest + SSE
│   │   │       └── schema.rs        # OpenAI-compatible request/response
│   │   └── Cargo.toml
│   │
│   ├── kay-verify/                  # Multi-perspective verification
│   │   ├── src/
│   │   │   ├── critic.rs            # trait Critic
│   │   │   ├── test_engineer.rs
│   │   │   ├── qa_engineer.rs
│   │   │   ├── end_user.rs
│   │   │   └── verdict.rs           # aggregation
│   │   └── Cargo.toml
│   │
│   ├── kay-sandbox/                 # OS-specific isolation
│   │   ├── src/
│   │   │   ├── lib.rs               # trait Sandbox
│   │   │   ├── macos.rs             # sandbox-exec profiles
│   │   │   ├── linux.rs             # landlock + seccomp
│   │   │   └── windows.rs           # Job Objects + restricted token
│   │   └── Cargo.toml
│   │
│   ├── kay-shell/                   # PTY + marker polling
│   │   └── src/lib.rs               # portable-pty + __CMDEND__ sentinel
│   │
│   ├── kay-session-store/           # SQLite + JSONL persistence
│   │   └── src/
│   │       ├── sqlite.rs            # sessions, messages, tool_invocations
│   │       └── jsonl.rs             # transcript append-only log
│   │
│   ├── kay-cli/                     # Headless CLI (ForgeCode CLI surface)
│   │   └── src/main.rs              # clap-based; wires core → stdin/stdout
│   │
│   └── kay-desktop/                 # Tauri app (src-tauri equivalent)
│       ├── src/
│       │   ├── main.rs              # tauri::Builder
│       │   ├── commands.rs          # #[tauri::command] wrappers
│       │   ├── state.rs             # Arc<Mutex<SessionManager>>
│       │   └── events.rs            # AgentEvent → Channel bridge
│       ├── tauri.conf.json
│       └── ui/                      # React/TS frontend
│           ├── src/
│           │   ├── App.tsx
│           │   ├── session/         # SessionView, Timeline, TokenMeter
│           │   ├── manager/         # Multi-session tab manager
│           │   └── settings/
│           ├── package.json
│           └── vite.config.ts
│
├── docs/                            # mdBook (ForgeCode-style)
├── .github/workflows/               # CI: cargo test, tauri build, release sign
└── README.md
```

### Structure Rationale

- **Pure `kay-core`:** No I/O dependencies; it defines traits and the agent loop. Swappable providers, tools, sandboxes. Makes unit testing the agent loop trivial (mock every boundary).
- **Per-capability crates:** Following Codex CLI's ~70-crate layout — each responsibility (tools, context, providers, verify, sandbox, shell, session-store) gets its own crate. Enables incremental compile times + enforced boundary hygiene.
- **`kay-cli` and `kay-desktop` as thin shells:** Both embed the same `kay-core` library. CLI pipes events to stdout; desktop pipes them to Tauri channels. This is the Codex CLI pattern: "rendering layer decoupled from the agent loop; TUI subscribes to events." (Codex blog, March 2026.)
- **`kay-tools/builtin/` starts with the KIRA trio:** `execute_commands`, `task_complete`, `image_read` — plus Kay's expansions (`read_file`, `edit_file`, `search`, `fetch`). Keeps the v1 surface small and the trait pattern provable before the tool catalog grows.
- **Frontend under `kay-desktop/ui/`:** Colocating React with the Tauri crate keeps the IPC contract visible. Shared TypeScript types can be generated from Rust via `ts-rs` or `specta`.

---

## Architectural Patterns

### Pattern 1: Event-Driven Agent Loop (NOT REPL)

**What:** The agent loop is a `tokio::select!` over multiple channels: user-input, model-stream, tool-results, control (pause/abort/resume). It emits a typed `AgentEvent` stream that every surface (CLI, Tauri UI, logger) subscribes to.

**When to use:** Every modern top-tier agent (Codex CLI, Claude Code, ForgeCode) uses this shape. REPL blocks the model stream and makes the UI stutter.

**Trade-offs:**
- ✅ Streaming UI updates are trivial (emit event → forward to Channel)
- ✅ Pause/resume/abort is a control message, not a thread kill
- ✅ Testable: drive the loop with a mock event source
- ⚠️ More complex than a blocking loop; requires disciplined error handling across `await` points

**Example:**
```rust
pub async fn run_turn(
    mut state: TurnState,
    provider: Arc<dyn ChatProvider>,
    tools: Arc<ToolRegistry>,
    tx: mpsc::Sender<AgentEvent>,
    mut ctrl: mpsc::Receiver<Control>,
) -> Result<TurnState, AgentError> {
    let mut stream = provider.stream(state.messages.clone(), tools.schemas()).await?;
    loop {
        tokio::select! {
            Some(delta) = stream.next() => {
                match delta? {
                    ChatDelta::Text(t)     => tx.send(AgentEvent::ModelText(t)).await?,
                    ChatDelta::ToolCall(c) => {
                        tx.send(AgentEvent::ToolCall(c.clone())).await?;
                        let result = tools.invoke(&c).await?;
                        tx.send(AgentEvent::ToolResult(result.clone())).await?;
                        state.record(c, result);
                        // model sees the result on the next turn
                    }
                    ChatDelta::Done(usage) => {
                        tx.send(AgentEvent::TurnEnd(usage)).await?;
                        return Ok(state);
                    }
                }
            }
            Some(Control::Abort) = ctrl.recv() => {
                tx.send(AgentEvent::Aborted).await?;
                return Ok(state);
            }
        }
    }
}
```

---

### Pattern 2: Trait-Based Tool Registry

**What:** A `trait Tool` with `name()`, `schema()` (JSON schema for the model), and `invoke(args) -> Result<Value>`. Tools are boxed as `Arc<dyn Tool>` and stored in a `HashMap`. The registry serializes all schemas into the `tools` parameter of the provider request (KIRA's "native tool calling" pattern).

**When to use:** Always. It's the boundary that lets you add tools without touching the loop, and it lets the permission layer live on top of the registry (claw-code's permission-gated tools).

**Trade-offs:**
- ✅ Adding a tool = new struct + `register()` line, no loop changes
- ✅ Permission gating wraps the registry (decorator pattern)
- ✅ Matches native tool-calling API of every major provider
- ⚠️ Dynamic dispatch cost is trivial compared to a 500ms LLM call
- ⚠️ Compile-time registration via `inventory`/`linkme` is tempting but adds linker-order fragility on all 3 target OSes — recommend explicit runtime `register()` in `main`.

**Example:**
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;   // JSON Schema for the model
    async fn invoke(&self, args: serde_json::Value, ctx: &ToolCtx)
        -> Result<ToolOutput, ToolError>;
}

pub struct ToolRegistry(HashMap<&'static str, Arc<dyn Tool>>);

impl ToolRegistry {
    pub fn with_defaults() -> Self {
        let mut r = Self(HashMap::new());
        r.register(Arc::new(ExecuteCommands::new()));
        r.register(Arc::new(TaskComplete::new()));
        r.register(Arc::new(ImageRead::new()));
        r.register(Arc::new(ReadFile::new()));
        r.register(Arc::new(EditFile::new()));
        r
    }
    pub fn schemas(&self) -> Vec<serde_json::Value> { /* ... */ }
    pub async fn invoke(&self, call: &ToolCall) -> Result<ToolOutput, ToolError> {
        self.0.get(call.name.as_str())
            .ok_or(ToolError::Unknown)?
            .invoke(call.arguments.clone(), &ToolCtx::current()).await
    }
}
```

---

### Pattern 3: Persona Agents as Data, Not Code

**What:** `kay` (write), `sage` (read-only research), `muse` (plan) are NOT separate processes or separate model pipelines. They are the same agent loop configured with different (a) system prompts, (b) tool filters, (c) optional model choices — loaded from a YAML file at startup (the ForgeCode `forge.yaml` pattern).

**When to use:** When you want persona-specialized behavior without triplicating the codebase. The switching logic lives in the Agent Switch component: it reads the active persona from `SessionConfig`, asks the Tool Registry to return only the tools allowed for that persona, and swaps the system prompt.

**Trade-offs:**
- ✅ One code path, three behaviors — every improvement lifts all personas
- ✅ Sage is a tool itself: `kay` and `muse` can invoke `sage(query)` as a sub-turn (ForgeCode pattern: "Sage is not user-facing; Forge and Muse call it internally")
- ✅ YAML-driven makes custom personas trivial (claw-code's "Agent Teams")
- ⚠️ If personas need radically different loop logic, YAML isn't enough — but that's a v2 problem (hierarchical multi-agent)

**Example (`~/.kay/agents.yaml`):**
```yaml
agents:
  - id: kay
    model: openrouter:anthropic/claude-opus-4.6
    system_prompt_file: prompts/kay.md
    tools: [execute_commands, read_file, edit_file, task_complete, image_read, sage]
    temperature: 0.2
  - id: sage
    model: openrouter:openai/gpt-5.4
    system_prompt_file: prompts/sage.md
    tools: [read_file, search, fetch]          # read-only
    temperature: 0.0
  - id: muse
    model: openrouter:anthropic/claude-opus-4.6
    system_prompt_file: prompts/muse.md
    tools: [read_file, search, sage]           # planning only
    temperature: 0.3
```

---

### Pattern 4: Provider HAL with Streaming + Tool-Call Reassembly

**What:** A single `trait ChatProvider` with one streaming method. The stream item is a `ChatDelta` enum (text, tool_call, usage, done). The OpenRouter impl parses SSE deltas and reassembles partial JSON for tool_call arguments (OpenAI-compatible chunking).

**When to use:** Always. Even if v1 is OpenRouter-only, defining the trait now is cheap and guarantees v2 direct-API providers slot in without breaking callers.

**Trade-offs:**
- ✅ v1→v2 provider additions are additive, not breaking
- ✅ Error taxonomy is unified (RateLimited, ContextLengthExceeded, ToolCallMalformed, ProviderUnavailable, etc.)
- ✅ Mocking the trait makes the agent loop unit-testable without hitting the network
- ⚠️ Tool-call reassembly is finicky — study `openrouter_api` or `edgequake-llm` for existing patterns before rolling your own.

**Example:**
```rust
#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<serde_json::Value>,
    ) -> Result<BoxStream<'static, Result<ChatDelta, ProviderError>>, ProviderError>;
}

pub enum ChatDelta {
    Text(String),
    ToolCall(ToolCall),        // fully reassembled, not partial
    ReasoningText(String),     // for o-series / thinking models
    Usage(Usage),
    Done,
}
```

---

### Pattern 5: Tauri IPC — Commands for Control, Channels for Streams

**What:** Short synchronous operations (start session, send user input, get settings, list sessions) use `#[tauri::command]`. Streaming (agent events, token deltas, tool output) uses `tauri::ipc::Channel<T>` — Tauri's optimized path for ordered, high-throughput data (the same mechanism used internally for child-process output).

**When to use:** Always. The Tauri docs explicitly say the event system is NOT designed for streaming — channels are. Using events for agent trace will stutter the UI at >50 msg/s.

**Trade-offs:**
- ✅ Channels are fast and ordered; perfect for ModelText deltas
- ✅ Commands are typed (with `specta`/`ts-rs`), giving TS autocomplete on the frontend
- ✅ Async commands run on `tauri::async_runtime`, not the main thread — loop never blocks the webview
- ⚠️ Channels are one-way backend→frontend; use commands for frontend→backend

**Example:**
```rust
#[tauri::command]
async fn start_session(
    state: State<'_, AppState>,
    project_root: String,
    persona: String,
    on_event: Channel<AgentEvent>,   // <-- streaming back to UI
) -> Result<SessionId, String> {
    let id = state.sessions.spawn(project_root, persona).await?;
    tokio::spawn(async move {
        let mut rx = state.sessions.subscribe(id);
        while let Some(ev) = rx.recv().await {
            let _ = on_event.send(ev);
        }
    });
    Ok(id)
}

#[tauri::command]
async fn send_input(state: State<'_, AppState>, id: SessionId, text: String)
    -> Result<(), String>
{
    state.sessions.send(id, UserInput::Text(text)).await.map_err(|e| e.to_string())
}
```

---

### Pattern 6: Per-OS Sandbox via Trait

**What:** A `trait Sandbox` with `spawn(cmd, policy) -> Child`. Three impls — `MacSandbox` (sandbox-exec), `LinuxSandbox` (landlock + seccomp), `WindowsSandbox` (Job Objects + restricted token). The agent loop is OS-agnostic; it asks for a sandboxed child and gets one.

**When to use:** v1 must ship at least filesystem-scope restriction (writes confined to the project root). System-call filtering (seccomp) can be v1.5.

**Trade-offs:**
- ✅ Codex CLI proves this works cross-platform
- ✅ Sandbox escape is structurally impossible (kernel-enforced)
- ⚠️ Landlock needs kernel ≥ 5.13; fall back to PR_SET_NO_NEW_PRIVS + process group on older kernels
- ⚠️ macOS `sandbox-exec` is technically "deprecated" but has no replacement and still ships in 15.x — Codex uses it
- ⚠️ Windows Job Objects don't restrict filesystem; add restricted token + integrity level

**Example:**
```rust
#[async_trait]
pub trait Sandbox: Send + Sync {
    async fn spawn(&self, cmd: Command, policy: SandboxPolicy) -> Result<Child, SandboxError>;
}

pub struct SandboxPolicy {
    pub writable_roots: Vec<PathBuf>,   // usually just the project root + /tmp
    pub readable_roots: Vec<PathBuf>,   // project + stdlib + cache dirs
    pub allow_network: bool,
}
```

---

## Data Flow

### Primary Flow: User Input → Verified Completion

```
[User types in Tauri UI]
    ↓ invoke("send_input", { id, text })
[Rust: Session.send_input] — pushes UserInput onto session's input channel
    ↓
[Agent Loop] — receives UserInput, composes prompt via Context Engine
    ↓
[Context Engine] — retrieves relevant symbols (tree-sitter → sled → top-K)
    ↓
[Provider HAL] — sends messages + tool schemas to OpenRouter
    ↓  (SSE stream)
[ChatDelta::Text] ─────────▶ AgentEvent::ModelText ──▶ Channel ──▶ UI (live render)
[ChatDelta::ToolCall] ─────▶ Tool Registry ──▶ Sandbox/Shell ──▶ ToolResult
    ↓
[Tool Result] ──▶ appended to messages ──▶ next turn
    ↓
[Model calls task_complete]
    ↓
[Verifier (Critics)] — 3 LLM passes (test eng / QA / end-user)
    ↓
   PASS → AgentEvent::TurnEnd
   FAIL → inject critic feedback as user message → loop continues
```

### Session State Flow

```
[Session Manager]
    │
    ├── in-memory: Arc<RwLock<SessionState>>  (hot path)
    │
    ├── append-only: JSONL per session       (source of truth for transcripts)
    │       └── ~/.kay/projects/<hash>/<session-id>.jsonl
    │
    ├── SQLite index                         (for listing / resume / fork)
    │       └── ~/.kay/kay.db
    │         tables: sessions, turns, tool_invocations, compaction_history
    │
    └── snapshots                            (pre-edit file copies for rewind)
            └── ~/.kay/snapshots/<session-id>/<turn>/<path>
```

This is the **Claude Code model** (JSONL transcripts + SQLite index + snapshot-based undo) verified via multiple community deep-dives.

### Tauri IPC Flow

```
Frontend (React)                           Backend (Rust)
─────────────────                          ──────────────
                      invoke("command", args)
[UI Action] ──────────────────────────────▶ [#[tauri::command] handler]
                                                │
                                                ├── SessionManager operation
                                                │
[State updated] ◀──── Promise resolve ─────────┘

                      Channel<AgentEvent>
[Event listener] ◀────────────────────────── [tx.send(AgentEvent::...)]
      │                                              ▲
      ▼                                              │
[Redux/Zustand store]                         [Agent Loop]
      │
      ▼
[Components re-render]
```

**Critical:** All agent-loop work (model calls, tool execution, shell, context index) runs in Rust on `tauri::async_runtime`. The webview NEVER blocks. UI thread handles only rendering + routing user gestures.

---

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1 session, 1 user | v1 default. Single process, SQLite WAL mode, in-memory session state |
| 3–10 concurrent sessions | Already handled by per-session `Arc<RwLock<SessionState>>`; SQLite handles the load; each session is its own tokio task |
| 50+ concurrent sessions | Would need session pooling + shared provider rate-limiter; defer until real pain |
| Team / multi-user | Out of scope for v1; would require a different architecture (daemon + RPC) |

### First Bottlenecks (in order)

1. **OpenRouter rate limits / tokens-per-minute** — hit long before anything else. Mitigate with a provider-level throttler + request queue.
2. **Context Engine indexing on huge monorepos** — tree-sitter parse of a 1M-LOC repo takes minutes. Mitigate with lazy / on-demand indexing (only files touched in the session) and incremental parsing.
3. **Tauri Channel serialization** — JSON-encoding every ModelText delta at >1kHz stutters. Mitigate by batching deltas (coalesce <16ms) before send.
4. **SQLite write contention** — multi-session concurrent write. Mitigate by WAL mode + per-session JSONL append (SQLite is just the index).
5. **Sandbox spawn latency on macOS** — `sandbox-exec` has ~30ms startup. Mitigate by reusing a single PTY session per shell tool invocation (KIRA's marker polling pattern already assumes this).

---

## Anti-Patterns

### Anti-Pattern 1: Blocking REPL in Rust

**What people do:** Write `loop { let input = read_line(); let resp = block_on(model.call(input)); println!(resp); }` — the "obvious" REPL shape.

**Why it's wrong:** Blocks on every `await`; streaming is impossible; cancellation requires killing the thread; UI stutters; pause/resume impossible.

**Do this instead:** Event-driven loop with `tokio::select!` over input, model, tool, and control channels. Emit `AgentEvent`s; every surface subscribes.

---

### Anti-Pattern 2: Raw File Dumps as Context

**What people do:** Read entire files into the prompt ("give the model everything").

**Why it's wrong:** Burns tokens, dilutes attention, slows every turn, and doesn't scale past tiny repos. ForgeCode's wedge is precisely NOT doing this — they ship a semantic context engine that sends function signatures and module boundaries.

**Do this instead:** tree-sitter parse → symbol extraction → hybrid BM25+embedding retrieval → top-K signatures inlined. Raw file bodies only when the model explicitly calls `read_file`.

---

### Anti-Pattern 3: Using Tauri Events for Streaming

**What people do:** `app.emit("agent-text", delta)` for every ModelText delta.

**Why it's wrong:** The Tauri event system is explicitly NOT optimized for streaming — it's JSON-only, not type-safe, always async, with no backpressure. Bursty event streams cause UI stutter.

**Do this instead:** Use `tauri::ipc::Channel<AgentEvent>` — designed for ordered, high-throughput streaming (the internal mechanism for child-process stdout).

---

### Anti-Pattern 4: ICL-Style Tool Parsing

**What people do:** Instruct the model to emit `<tool>…</tool>` tags and parse them with regex (the pre-2024 pattern).

**Why it's wrong:** Fragile, breaks on edge cases, forces the model to allocate reasoning tokens to syntax. KIRA's +4.6pp improvement on Terminal-Bench came precisely from switching from ICL parsing to native `tools=[...]` parameter.

**Do this instead:** Always use the provider's native tool-calling parameter. The Tool Registry hands its JSON schemas directly to `provider.stream(messages, tools)`.

---

### Anti-Pattern 5: Triplicating Code for Personas

**What people do:** `forge_agent/`, `sage_agent/`, `muse_agent/` as three separate implementations.

**Why it's wrong:** Every improvement requires three changes; divergence is inevitable; new personas require new code.

**Do this instead:** One agent loop, parameterized by `Persona { prompt, tool_filter, model }` loaded from YAML. The ForgeCode `forge.yaml` pattern.

---

### Anti-Pattern 6: Running Shell Without Sandbox on "Just Dev Machines"

**What people do:** "It's my machine, what could go wrong?"

**Why it's wrong:** Agents have hallucinated `rm -rf ~`, have been prompt-injected into exfiltrating `.env`, have leaked `~/.aws/credentials`. Kay's spec mandates sandboxed subprocess isolation for a reason.

**Do this instead:** OS-native sandbox from day one. Even a minimal `sandbox-exec` profile that blocks writes outside the project root eliminates 90% of blast radius.

---

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| OpenRouter API | `reqwest` + SSE; `Authorization: Bearer <key>`; chat/completions endpoint | v1's only provider. Rate limits vary by account tier. Study `openrouter_api` or `edgequake-llm` crates for streaming patterns |
| OS Keyring | `keyring` crate | Store OpenRouter API key; never plaintext in config |
| Git | `git2` crate | Read .gitignore for context indexing; detect project root; optional pre-edit snapshot |
| tree-sitter grammars | `tree-sitter-{rust,ts,py,go,java,...}` crates | Per-language; load at startup |
| Tauri updater | `tauri-plugin-updater` | Signed release tags; reproducible builds |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Core ↔ Tools | `Arc<dyn Tool>` trait objects via Registry | Tool can't see agent state except via `ToolCtx` |
| Core ↔ Providers | `Arc<dyn ChatProvider>`; Stream<ChatDelta> | v1 OpenRouter; v2 adds Anthropic/OpenAI direct |
| Core ↔ Sandbox | `Arc<dyn Sandbox>`; per-OS impl | Loop never sees `#[cfg(target_os)]` |
| Core ↔ Session Store | Async write-through: in-memory first, JSONL append, SQLite index update | Transcript is source of truth; SQLite is just an index |
| Core ↔ Context Engine | Async query: `engine.retrieve(turn_context) -> Vec<Snippet>` | Engine owns its own index lifecycle |
| Core ↔ Verifier | `verifier.check(turn_trace) -> Verdict` | Called from `task_complete` tool, not loop directly |
| CLI ↔ Core | Direct library call; stdout for events | `kay-cli` embeds `kay-core` |
| Desktop ↔ Core | Tauri commands (req/resp) + Channels (stream) | `kay-desktop` embeds `kay-core` |
| Frontend ↔ Desktop (Rust) | `@tauri-apps/api/core` invoke + Channel listeners | Typed via `specta` or `ts-rs` |

---

## Suggested Build Order (for 8–12 phase "fine" granularity)

Dependency-aware. Each phase is shippable-and-testable before the next starts.

| # | Phase | What's Built | Depends On | Why This Position |
|---|-------|-------------|------------|-------------------|
| 1 | **Fork & Foundation** | Fork ForgeCode, Apache-2.0 + NOTICE, CLA workflow, signed-tag CI, crate split skeleton | — | Can't build on an unsigned fork; governance first avoids retrofitting |
| 2 | **Provider HAL (OpenRouter)** | `trait ChatProvider`, OpenRouter impl, SSE parsing, tool-call reassembly, error taxonomy | (1) | Everything downstream needs model calls; start with the mock-testable trait |
| 3 | **Tool Registry + 3 KIRA Tools** | `trait Tool`, registry, `execute_commands`, `task_complete`, `image_read`; permission scaffolding | (2) | Smallest valid tool surface that proves the native-tool-calling path; KIRA's exact shape |
| 4 | **Agent Loop (event-driven)** | `run_turn`, `AgentEvent`, control channel, pause/abort; personas as data; YAML-driven `kay/sage/muse` | (2, 3) | Core brain; can now do one-shot tasks headlessly |
| 5 | **Sandbox (macOS first, then Linux/Windows)** | `trait Sandbox`; `sandbox-exec` impl; Landlock+seccomp impl; Job Objects impl | (3) | Required before shipping anything that writes files; macOS first (dev machine) |
| 6 | **Shell + Marker Polling** | `portable-pty` integration, `__CMDEND__<seq>__` sentinel, timeout handling | (3, 5) | Completes the KIRA harness reproduction |
| 7 | **Session Store + Transcript** | SQLite index, JSONL append, resume/fork, pre-edit snapshots | (4) | First point where state survives process restart; unlocks Tauri demos |
| 8 | **Context Engine** | tree-sitter parsers, symbol extraction, sled index, hybrid retrieval, prompt assembly | (4) | ForgeCode parity wedge; big lift; must be beatable on TB 2.0 |
| 9 | **Verifier (Multi-perspective)** | Three critics (test-eng / QA / end-user), verdict aggregation, feedback injection | (4, 8) | KIRA 4.3 technique — the score-moving improvement |
| 10 | **Tauri Desktop Shell** | `kay-desktop` crate, commands, channels, session view, timeline, token meter | (4, 7) | Can't ship without UI; build once events are stable |
| 11 | **Multi-Session Manager + Settings** | Spawn/pause/resume/fork from UI, project picker, OpenRouter key binding | (7, 10) | UI completeness; needs session primitives to exist |
| 12 | **Terminal-Bench 2.0 Submission + Hardening** | Run reference, iterate on weak tasks, publish score, signed binary releases for 4 platforms | (9, 10, 11) | The v1 acceptance gate |

### Foundations vs Features

- **Foundations (cannot be retrofitted without pain):** Phases 1, 2, 4, 5, 7. Build these right or pay later.
- **Features (can iterate):** Phases 3 (tool set grows naturally), 8 (context engine can improve after v1), 9 (critic prompts tune endlessly), 10–11 (UI polish).
- **Parallel-possible:** Phase 8 (Context Engine) can run parallel to Phase 7 (Session Store) if staffing allows — they share no code. Phase 10 (Tauri) can start scaffolding during Phase 7.

---

## Extension Points for v2 Wedges

Design v1 so these slot in without breaking changes:

| v2 Wedge | v1 Extension Point | How It Plugs In |
|----------|-------------------|-----------------|
| **ACE (self-improving context)** | Context Engine's `compose()` method + `learn()` hook on the engine trait | ACE adds a learning loop that updates symbol weights from turn outcomes. Trait already supports `update_on_verdict(trace, verdict)` as a no-op in v1 |
| **Dynamic model routing per subtask** | Provider HAL's `ChatProvider` trait — add a `Router` impl that wraps N providers and picks one per message based on heuristics | Agent Switch already calls `provider.stream()`; swap in `RoutingProvider` with zero loop changes |
| **Verification-first (deeper)** | Verifier's `trait Critic` — already an extension point | v2 adds new critics, parallelizes them, adds tool-augmented critics. Aggregator becomes smarter |
| **Re-enterable hierarchical multi-agent** | Agent Switch + Persona YAML — personas already data-driven | v2 makes personas nestable: a turn inside `muse` can spawn a sub-session with `kay`. Needs session-nesting in Session Manager (add `parent_session_id` column to SQLite now, no-op in v1) |
| **Direct-API providers (Anthropic/OpenAI/Gemini)** | Provider HAL's `trait ChatProvider` | Already abstracted. v2 adds `AnthropicProvider`, `OpenAIProvider` crates. v1 OpenRouter impl unchanged |
| **Local models (Ollama/llama.cpp)** | Provider HAL | Same as above — another impl |
| **MCP (Model Context Protocol) tools** | Tool Registry's `trait Tool` | MCP tools are just another impl (`MCPTool` wraps a remote server). Lazy discovery matches claw-code's `plugins` crate pattern |
| **Hooks / Agent Teams** | Agent Event stream + Persona YAML | Hooks subscribe to `AgentEvent` (pre-tool, post-tool, turn-end). Already an open stream — v2 adds user-configurable hooks loaded from `~/.kay/hooks/` |

**v1 must:** (a) keep all traits `async_trait` with object-safe signatures, (b) reserve `parent_session_id` in the SQLite schema, (c) keep `AgentEvent` an open `#[non_exhaustive]` enum so new event types don't break subscribers.

---

## Crate Stack Recommendations

Based on ecosystem survey:

| Capability | Recommended Crate | Notes |
|-----------|-------------------|-------|
| Async runtime | `tokio` | Already the Rust-Tauri-ecosystem default |
| HTTP + SSE | `reqwest` + `reqwest-eventsource` | Or `eventsource-stream` for manual control |
| OpenRouter client | Roll your own using `reqwest`, OR vendor `openrouter_api` (type-state) / `openrouter-rs` (shared events) | Auditing for v1 |
| Tree-sitter | `tree-sitter` + per-lang grammar crates | Standard; incremental parsing is built-in |
| KV index | `sled` | Pure Rust, async-friendly, works cross-platform |
| SQLite | `rusqlite` (sync, ergonomic) or `sqlx` (async) | `rusqlite` is simpler for the session index |
| PTY | `portable-pty` | Cross-platform, proven |
| Sandbox (Linux) | `rust-landlock` + `seccompiler` | Both in active maintenance |
| Sandbox (generic) | `sandbox-rs` or `syd` | Reference implementations |
| JSON schema for tools | `schemars` + `serde` | Schema derived from struct defs |
| Keyring | `keyring` | macOS Keychain / Windows Cred Store / Linux Secret Service |
| TS type gen | `specta` (Tauri-idiomatic) or `ts-rs` | Generates TS types from Rust structs |
| Tauri | `tauri` 2.x, `tauri-plugin-fs`, `tauri-plugin-updater`, `tauri-plugin-dialog` | 2.x is the v1 target |
| Frontend | React + Vite + TanStack Query + Zustand | Tauri-typical; React because UI ecosystem |
| Handlebars (prompts) | `handlebars` | ForgeCode uses this for system prompt templating |

---

## Sources

### Primary (HIGH confidence — direct vendor docs or source)
- [ForgeCode Agents documentation](https://forgecode.dev/docs/operating-agents/) — three-agent pattern (forge/muse/sage)
- [ForgeCode Agent Configuration](https://forgecode.dev/docs/agent-configuration/) — YAML-driven personas, handlebars prompts, compaction settings
- [Terminus-KIRA repository](https://github.com/krafton-ai/KIRA) — `execute_commands`, `task_complete`, `image_read` tools; native tool calling
- [KRAFTON AI: How We Reached 74.8% on terminal-bench with Terminus-KIRA](https://krafton-ai.github.io/blog/terminus_kira_en/) — harness techniques, marker polling, multi-perspective verification
- [codex-rs Architecture deep-dive](https://codex.danielvaughan.com/2026/03/28/codex-rs-rust-rewrite-architecture/) — 70-crate workspace, Thread/Turn/Item, decoupled rendering
- [Unrolling the Codex agent loop (OpenAI)](https://openai.com/index/unrolling-the-codex-agent-loop/) — state-machine agent loop
- [Tauri v2: Inter-Process Communication](https://v2.tauri.app/concept/inter-process-communication/) — commands vs channels vs events
- [Tauri v2: Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/) — Channel for streaming, async_runtime
- [Tauri streaming discussion #7146](https://github.com/tauri-apps/tauri/discussions/7146) — event-system throughput limits
- [Claw Code repository (clean-room rewrite)](https://github.com/7df-lab/claw-code-rust) — 9 crates, plugin-based tools, permission gating
- [mini-swe-agent on GitHub](https://github.com/SWE-agent/mini-swe-agent) — minimalist linear message list, shell-only tool
- [OpenRouter API streaming docs](https://openrouter.ai/docs/api/reference/streaming) — SSE contract

### Secondary (MEDIUM confidence — community synthesis)
- [How Codex is built — Pragmatic Engineer](https://newsletter.pragmaticengineer.com/p/how-codex-is-built) — Codex CLI architecture context
- [Claude Code session management (DeepWiki)](https://deepwiki.com/anthropics-claude/claude-code/2.3-session-management) — SQLite+JSONL hybrid, fork/resume
- [Claw Code vs Claude Code analysis](https://www.eigent.ai/blog/claw-code) — event-driven, crash recovery, namespace sandboxing
- [A deep dive on agent sandboxes — Pierce Freeman](https://pierce.dev/notes/a-deep-dive-on-agent-sandboxes) — cross-OS sandbox patterns
- [mini-swe-agent architecture overview (DeepWiki)](https://deepwiki.com/SWE-agent/mini-swe-agent/1.1-architecture-overview) — three-layer protocol, 100-line loop

### Tertiary (LOW confidence — verify before citing)
- [Codemem](https://docs.rs/crate/codemem/0.10.0) — tree-sitter-based AI memory engine (reference, not dependency)
- [openrouter_api crate](https://crates.io/crates/openrouter_api) — type-state provider client (candidate dependency)
- [edgequake-llm](https://github.com/raphaelmansuy/edgequake-llm) — unified Rust LLM provider abstraction (reference for trait design)

---

*Architecture research for: open-source terminal coding agent with native desktop UI (Kay v1)*
*Researched: 2026-04-19*
