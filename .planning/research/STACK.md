# Stack Research

**Domain:** Rust + Tauri 2.x desktop coding-agent (Kay — ForgeCode fork)
**Researched:** 2026-04-19
**Confidence:** HIGH (core) / MEDIUM (context indexer, sandbox) / LOW (SSE crate selection)

---

## Executive Recommendation (one-line-per-layer)

- **Language / workspace:** Rust stable (≥ 1.85), multi-crate Cargo workspace forked from ForgeCode — **HIGH**
- **Async runtime:** `tokio` 1.51.x (LTS until March 2027) — **HIGH**
- **HTTP client:** `reqwest` 0.13.x + `rustls-tls` + `stream` + `json` features — **HIGH**
- **SSE streaming:** `reqwest-eventsource` 0.6.x (pairs with reqwest 0.13) — **MEDIUM**
- **OpenRouter client:** write **thin in-tree client** over `reqwest`+`reqwest-eventsource` (NOT the third-party `openrouter_api` crate — still pinned to reqwest 0.11) — **HIGH**
- **JSON / schema:** `serde` 1.0.x, `serde_json` 1.0.x, `schemars` 0.8.x + manual post-processing for ForgeCode-style flattening — **HIGH**
- **CLI parsing:** `clap` 4.5.x with `derive` feature — **HIGH**
- **Subprocess execution:** `tokio::process::Command` for headless shell; `portable-pty` 0.9.x for PTY-attached flows; `__CMDEND__<seq>__` sentinel for completion polling — **HIGH**
- **Sandbox (per-platform):** macOS `sandbox-exec` (Seatbelt profiles), Linux `landlock` + `seccompiler`, Windows Job Objects + `RestrictedToken` — mirror codex-rs layout — **MEDIUM**
- **Code indexing:** `tree-sitter` 0.24.x + language grammars; symbol store in SQLite via `rusqlite` + `sqlite-vec` extension for local vector search — **MEDIUM**
- **Tauri desktop shell:** `tauri` 2.10.x, `wry` 0.55.x, `tao` 0.35.x — **HIGH**
- **Frontend (inside Tauri):** **React 19 + TypeScript 5.x + Vite 6**. Not Leptos/Yew/Dioxus for v1. — **HIGH**
- **Typed bridge (Rust ↔ TS):** `tauri-specta` v2 + `specta` v2 — **HIGH**
- **IPC for streams:** Tauri 2 `ipc::Channel<T>` (NOT events) for tokens, subprocess output, trace frames — **HIGH**
- **Logging:** `tracing` 0.1.x + `tracing-subscriber` 0.3.x + `tracing-opentelemetry` 0.27+ — **HIGH**
- **Telemetry export:** `opentelemetry` 0.27+, `opentelemetry-otlp` 0.27+ (opt-in; default off) — **MEDIUM**
- **Packaging:** Tauri bundler (macOS `.app`/`.dmg`, Windows `.msi`/`.exe`, Linux `.deb`/`.AppImage`) + `cargo install kay` for headless CLI — **HIGH**
- **Signing:** Tauri's built-in signing pipeline (macOS notarization + `codesign`; Windows Azure Key Vault/KMS-style flow) — **HIGH**

---

## Recommended Stack (full tables)

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| Rust | stable ≥ 1.85 (MSRV 1.82) | Language | ForgeCode, Codex CLI, claw-code all Rust; single-binary distribution; parity with top-10 TB2 entrants |
| `tokio` | **1.51.x** (LTS → Mar 2027) | Async runtime | Pin LTS. Gate features tightly (`rt-multi-thread`, `macros`, `io-util`, `process`, `signal`, `sync`, `fs`). `full` bloats binary and compile time. |
| `tauri` | **2.10.3** | Desktop shell | 2.x is stable since Oct 2024; Kay's v1 thesis literally depends on Tauri desktop UI — no alternative considered |
| `wry` / `tao` | 0.55 / 0.35 | Tauri webview/window plumbing | Transitive — don't pin directly, let Tauri control |
| `reqwest` | **0.13.x** | HTTP client to OpenRouter | de-facto standard; built on hyper 1.x; features: `rustls-tls`, `stream`, `json`, `gzip`, `brotli`. Disable default `native-tls` to avoid OpenSSL linkage on Linux. |
| `serde` + `serde_json` | 1.0.x | (De)serialization | Ubiquitous; schema-compatible with `schemars` |
| `schemars` | **0.8.x** (consider 1.0.0-alpha once stable) | JSON Schema derivation for tool definitions | Respects serde attributes; direct path to OpenAI-shape tool specs. Layer a custom post-processor that (a) hoists required before properties and (b) flattens nested required arrays — the exact ForgeCode hardening technique that took TB2 from ~74% to 81.8%. |
| `clap` | **4.5.x** | CLI parsing | Industry-standard; derive macros; matches Rust-CLI book recommendation |
| `tracing` | 0.1.x | Structured logging / spans | Native OpenTelemetry bridge; span ↔ OTel span mapping |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---|---|---|---|
| `reqwest-eventsource` | 0.6.x | SSE parsing over reqwest streams | Only path that keeps us on reqwest 0.13 without pulling a second HTTP stack (rules out `eventsource-client` which bundles hyper v1 separately) |
| `eventsource-stream` | 0.2.x | Lower-level SSE parser | Fall-back if `reqwest-eventsource` can't surface per-provider retry semantics we need |
| `tower` / `tower-http` | tower 0.5 / tower-http 0.6 | Middleware: retry, timeout, rate-limit around reqwest | Retry with jittered exponential backoff for OpenRouter 429/5xx |
| `backoff` or `backon` | backon 1.x | Retry policy primitives | `backon` is actively maintained in 2026; `backoff` crate is in soft-maintenance |
| `futures` / `futures-util` | 0.3.x | Stream combinators | Required for SSE token assembly and tool-call accretion |
| `tokio-util` | 0.7.x | `Codec`, `LinesCodec`, `CancellationToken` | Cancellation propagation from UI into long-running agent turns |
| `portable-pty` | 0.9.x | PTY allocation for shell-sensitive commands | When subprocess needs an ISATTY (npm, cargo, docker buildx) — wezterm-sourced, genuinely portable |
| `which` | 7.x | Resolve executables from PATH | Before spawning shell commands |
| `shell-escape` / `shlex` | shlex 1.3 | Argv quoting/parsing | Render shell commands for the trace view without injection risk |
| `tree-sitter` | 0.24.x | Incremental parser for semantic indexing | Context engine: function signatures, module boundaries. Matches ForgeCode's indexer ethos. |
| `tree-sitter-rust`, `-typescript`, `-javascript`, `-python`, `-go`, `-java` | latest compat | Grammars | Ship 6–8 grammars in the default build; others via dynamic-lib path |
| `rusqlite` | 0.32.x | Session + symbol storage | SQLite is embedded, zero-ops, fits single-binary constraint |
| `sqlite-vec` (via `rusqlite` extension) | 0.1.x | On-disk vector search for symbol embeddings | Avoids shipping a separate DB daemon; stays embedded |
| `tauri-plugin-shell` | 2.x | UI-triggered sidecar spawning | For the "run command" button in the GUI trace |
| `tauri-plugin-dialog` | 2.x | Directory picker | Project/workspace selection |
| `tauri-plugin-fs` | 2.x | Scoped filesystem access from the frontend | Read session logs, export transcripts |
| `tauri-plugin-store` | 2.x | Persistent UI prefs / API-key storage | Pair with OS keyring for secrets |
| `tauri-plugin-updater` | 2.x | In-app auto-update with signature verification | Matches the "signed releases from v0.0.1" constraint |
| `keyring` | 3.x | Cross-platform OS keyring wrapper | OpenRouter API key storage (macOS Keychain, Windows Credential Manager, Linux Secret Service) |
| `tauri-specta` | 2.x | Typed commands + events → TypeScript bindings | Kill class of "frontend-backend drift" bugs at compile time |
| `specta` | 2.x | Type introspection | Transitive via tauri-specta |
| `tracing-subscriber` | 0.3.x | Subscriber: fmt + env filter + JSON output | Dual output: pretty for dev, JSON for session replay |
| `tracing-appender` | 0.2.x | Non-blocking rolling file logs | Agent traces can be hundreds of MB per session |
| `tracing-opentelemetry` | 0.27.x+ | `tracing` → OTel span bridge | Opt-in only; default build has telemetry compiled out via feature flag |
| `opentelemetry` / `opentelemetry-otlp` | 0.27.x+ | OTel SDK + OTLP exporter | For users who want Jaeger/Tempo/Honeycomb export |
| `uuid` | 1.x | Session/turn/tool-call IDs | v7 time-ordered IDs make session-log sorting free |
| `time` | 0.3.x | Timestamp handling | `chrono` also fine; pick one and stay consistent |
| `directories` / `etcetera` | directories 5.x | XDG / AppData / Application Support paths | Cross-platform config/cache/data dirs |
| `anyhow` + `thiserror` | anyhow 1.x / thiserror 2.x | Error types | `anyhow` at binary boundary, `thiserror` in library crates |
| `dashmap` | 6.x | Concurrent map for session registry | Cheaper than `RwLock<HashMap>` for hot paths |
| `parking_lot` | 0.12.x | Faster mutex/rwlock | Tokio-compatible via feature flag; used by ForgeCode-lineage projects |
| `base64` | 0.22.x | For multimodal `image_read` tool (KIRA technique) | Required for sending terminal screenshots to vision models |
| `image` | 0.25.x | Screenshot capture/encode for `image_read` | Pair with `screenshots` crate on the harness side |
| `screenshots` | 0.8.x | Capture terminal state as PNG | For the KIRA-style multimodal `image_read` loop |

### Development Tools

| Tool | Purpose | Notes |
|---|---|---|
| `cargo-nextest` | Faster, parallel test runner | Standard in 2026 Rust CI |
| `cargo-deny` | License + vuln + duplicate-dep gate | Required by "signed releases + reproducibility" constraint |
| `cargo-audit` | RustSec vuln scanning | Run on every release; wire into CI |
| `cargo-insta` | Snapshot testing | Codex CLI precedent — required for TUI/UI Rust-side snapshots |
| `cargo-machete` | Detect unused deps | Keep the binary lean (Tauri already 10–20 MB; don't bloat) |
| `cargo-release` | Coordinated workspace release | Matches multi-crate workspace with signed tags |
| `tauri-cli` (`cargo tauri`) | Tauri dev + build + bundle | v2.10.1 |
| `vite` | Frontend dev server/build | Pairs with Tauri's `beforeDevCommand` hook |
| `biome` (or eslint + prettier) | TS/JS lint + format | Biome is faster and single-binary — matches Kay's own philosophy |
| `pnpm` | Frontend package manager | Faster, strict, disk-efficient vs npm |

---

## Installation

```bash
# Workspace: Cargo.toml
[workspace]
members = ["crates/*", "src-tauri"]
resolver = "2"

[workspace.dependencies]
tokio         = { version = "1.51", features = ["rt-multi-thread", "macros", "io-util", "process", "signal", "sync", "fs", "time"] }
reqwest       = { version = "0.13", default-features = false, features = ["rustls-tls", "json", "stream", "gzip", "brotli"] }
reqwest-eventsource = "0.6"
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
schemars      = "0.8"
clap          = { version = "4.5", features = ["derive", "env"] }
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender   = "0.2"
anyhow        = "1"
thiserror     = "2"
uuid          = { version = "1", features = ["v7", "serde"] }
futures       = "0.3"
tokio-util    = { version = "0.7", features = ["codec"] }
tower         = "0.5"
tower-http    = { version = "0.6", features = ["timeout", "trace"] }
backon        = "1"
portable-pty  = "0.9"
tree-sitter   = "0.24"
rusqlite      = { version = "0.32", features = ["bundled"] }
keyring       = "3"
directories   = "5"

# Tauri crate (src-tauri/Cargo.toml)
tauri         = { version = "2.10", features = ["devtools"] }
tauri-specta  = { version = "2", features = ["derive", "typescript"] }
specta        = "2"
tauri-plugin-shell   = "2"
tauri-plugin-dialog  = "2"
tauri-plugin-fs      = "2"
tauri-plugin-store   = "2"
tauri-plugin-updater = "2"
```

```bash
# Frontend (src-tauri/../ui/package.json via pnpm)
pnpm add react@19 react-dom@19
pnpm add -D typescript@5 vite@6 @vitejs/plugin-react
pnpm add @tauri-apps/api@2 @tauri-apps/plugin-shell @tauri-apps/plugin-dialog \
         @tauri-apps/plugin-fs @tauri-apps/plugin-store @tauri-apps/plugin-updater
pnpm add -D @biomejs/biome
```

```bash
# Dev tooling
cargo install --locked cargo-nextest cargo-deny cargo-audit cargo-insta \
                       cargo-machete cargo-release tauri-cli@2.10.1
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|---|---|---|
| `reqwest` 0.13 | `hyper` 1.x directly | Only if we need sub-millisecond overhead or custom connection pooling. For OpenRouter (network-bound) it's overkill. |
| `reqwest` 0.13 | `ureq` | Headless/blocking-only use. Rules itself out by Tauri's tokio-everywhere architecture. |
| In-tree OpenRouter client | `openrouter_api` 0.6 | Only if we accept reqwest **0.11** (three majors behind) and rustls-native-roots churn. Not worth the dep. |
| In-tree OpenRouter client | `async-openai` | If we wanted only OpenAI-shape endpoints. But OpenRouter-specific headers (`HTTP-Referer`, `X-Title`), provider preferences, and error shapes reward a thin bespoke client. |
| React + TypeScript (Tauri frontend) | Leptos | If we were Rust-native zealots and TB2 gave points for signal purity. Mature, but smaller ecosystem for agent-UI components (Markdown renderers, code highlighters, diff viewers). |
| React + TypeScript | Dioxus 0.7 | If we ever wanted mobile parity without porting a web stack. For v1 desktop-only, adds Rust compile-time tax with no user-visible win. |
| React + TypeScript | SolidJS / Svelte 5 | Defensible; lose ecosystem mass (Monaco/CodeMirror wrappers, shadcn/ui, Lucide). |
| `tree-sitter` indexer | `rust-analyzer` as a library | Only for Rust-only indexing. Tree-sitter is the only multi-language path that matches ForgeCode's breadth. |
| `portable-pty` | `pty-process` / `tokio-pty-process` | `pty-process` is fine; `tokio-pty-process` is thin-maintenance. `portable-pty` wins on WezTerm production battle-testing + dynamic implementation selection. |
| Tauri 2 Channels | Tauri 2 Events | Events are JSON-only and not designed for high-throughput ordered streams. Use events for "session-created" style notifications; use channels for token/tool-output streams. |
| `tauri-bundler` (signing) | `cargo-dist` | `cargo-dist` is great for pure CLI crates but still weak on macOS notarization as of early 2026. Because Kay ships a Tauri app, `tauri-bundler` wins by default. Use `cargo-dist` **only** for the CLI-only distribution channel (`cargo install kay` adjunct tarballs). |
| `sqlite-vec` | `lancedb` | LanceDB is excellent but adds ~15–25 MB to the bundle and a Parquet-lineage dependency graph. `sqlite-vec` keeps us one-file-on-disk and zero-daemon. |
| `sqlite-vec` | `qdrant` (embedded) | Qdrant is server-oriented; embedded mode is newer and adds gRPC surface. Overkill for v1. |
| `rustyline` / `reedline` | neither | Kay's "TUI" is explicitly out of scope — headless CLI is `clap`-driven one-shots and the interactive surface is Tauri. REPL crates are unnecessary. |
| `tracing` | `log` + `env_logger` | Kay needs spans and structured fields for session replay. `log` is insufficient. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|---|---|---|
| `openrouter_api` crate | Pinned to `reqwest` 0.11 (April 2026); dragging it in forces a second HTTP stack or a downgrade. | Thin in-tree OpenRouter client over `reqwest` 0.13 + `reqwest-eventsource`. |
| `async-openai` (for Kay's main path) | Targets OpenAI-native endpoint shape; OpenRouter's provider-preferences / cost metering / credits APIs don't map cleanly. Adds a provider-abstraction layer we don't want for a v1 single-provider scope. | In-tree client. Revisit in v2 when direct providers come online. |
| `reqwest` 0.11 / `hyper` 0.14 | Pre-hyper-1.0; EOL maintenance; TLS configuration gotchas with rustls. | `reqwest` 0.13 + `hyper` 1.x stack. |
| `openssl`-based TLS | System OpenSSL linkage = Linux distribution pain + reproducible-build headache. | `rustls` via `reqwest`'s `rustls-tls` feature (or `rustls-tls-native-roots` if users demand OS trust store). |
| `tokio` feature `full` | Bloats binary size and compile time; pulls unused net/signal components into every crate. | Explicit feature list (see workspace.dependencies above). |
| `log` + `env_logger` | No spans, no structured fields, no OTel bridge. | `tracing` stack. |
| `chrono` `default-features = true` | Pulls `time` 0.1-era timezone code with known soundness warnings. | `chrono` with `default-features = false, features = ["std", "clock", "serde"]` **or** the `time` crate outright. |
| Raw `std::process::Command` for agent shell | Blocking; can't cancel; breaks TB2-style long-running builds and test suites. | `tokio::process::Command` with `kill_on_drop(true)` + `CancellationToken`; `portable-pty` when ISATTY needed. |
| Tauri 2 `emit`/events for token streams | JSON-only, not ordered under load, double-listen footguns. | `tauri::ipc::Channel<T>` + typed enum events. |
| Running agent shell commands in the Tauri main process | Blocks UI thread on any panic/hang; mixes UI capabilities with shell capabilities in the same trust boundary. | Separate `kay-core` long-lived process (sidecar) launched by the Tauri app; communicate over `tauri::ipc::Channel`. Matches codex-rs's `codex-app-server` split. |
| `openrouter_api`'s "MCP support" feature | MCP is v2 territory for Kay; adoption now creates abstraction debt. | Defer MCP until v2; hand-roll tool execution for v1. |
| Unsigned / ad-hoc release tags | Explicit project constraint (Silver-Bullet#28 cited in PROJECT.md). | Signed tags from v0.0.1; Tauri updater's built-in signature verification; Apple notarization + Windows Authenticode from the first nightly. |
| Python-in-sidecar (claw-code pattern) | Claw-code runs Python for orchestration → adds runtime + packaging complexity. Kay wants pure-Rust single binary. | Keep orchestration in `kay-core` (Rust). Python only appears as a subject language the agent edits, never as runtime infrastructure. |
| `cargo-dist` for the desktop bundle | No first-class macOS notarization as of April 2026. | `tauri-bundler` for the Tauri build; `cargo-dist` **only** for headless CLI tarballs. |
| Global static `tokio::runtime::Runtime::new()` outside Tauri | Competes with Tauri's own runtime; double-scheduling. | Use the runtime Tauri provides; spawn with `tauri::async_runtime::spawn` or pass handles through. |

---

## Stack Patterns by Variant

**If the user runs `kay --headless` (CI mode):**
- Build `kay-cli` crate: `clap` → `kay-core` directly, no Tauri.
- Same JSON session-log format as the GUI (swap terminal output for stdout/stderr streams).
- Distribute via `cargo install kay` and a plain tarball through cargo-dist.

**If the user runs `Kay.app` (desktop GUI):**
- Tauri launches → spawns `kay-core` as a sidecar subprocess via `tauri-plugin-shell`.
- UI ↔ core via `tauri::ipc::Channel<AgentEvent>` for streaming and typed `invoke` commands for control.
- Secrets via OS keyring (`keyring` crate); never via `localStorage`.

**If a tool invocation needs a PTY (ISATTY-sensitive programs):**
- Spawn via `portable-pty`; read child output through a `tokio::sync::mpsc` + the `__CMDEND__<seq>__` marker.
- Send through the same `AgentEvent::ShellOutput { turn_id, chunk }` channel.

**If a tool invocation is "pure" (no ISATTY):**
- `tokio::process::Command` with `kill_on_drop(true)`, `stdin/stdout/stderr: Stdio::piped()`.
- Still use marker-based polling for completion parity with the KIRA technique.

**If sandboxing is requested (default ON after Phase X):**
- macOS: `sandbox-exec -f <profile.sb>` wrapping the child. Profiles shipped in `crates/kay-sandbox/profiles/`.
- Linux: `landlock` + `seccompiler` applied from a pre-exec hook; fall back to pass-through if kernel < 5.13.
- Windows: Job Object + `RestrictedToken` via `windows-rs`.

**If the user builds from source (`cargo install kay`):**
- Headless-only; bundle script prints "For the desktop app, download Kay.app from releases."
- Cuts compile time by ~60% (no webview, no Tauri plugins).

**If telemetry is opted in:**
- Compile with `--features telemetry` → links `tracing-opentelemetry` + `opentelemetry-otlp`.
- Default build ships with telemetry feature off; zero code path, zero dep drag.

---

## Version Compatibility

| Package A | Compatible With | Notes |
|---|---|---|
| `tauri` 2.10.x | `tao` 0.35.x, `wry` 0.55.x | Let Tauri own these transitives; don't pin independently. |
| `tauri` 2.10.x | `tauri-specta` 2.x (requires Specta 2) | tauri-specta v2 **does not** support Tauri v1 — relevant only for migration-from-v1 sanity checks. |
| `reqwest` 0.13.x | `hyper` 1.1+, `rustls` 0.23 | `reqwest` 0.13 requires hyper 1.x stack; mixing with 0.12 is harmless but wastes binary space — pin to 0.13. |
| `reqwest-eventsource` 0.6.x | `reqwest` 0.13 | Earlier versions (0.5) bound to reqwest 0.12. Match majors. |
| `tokio` 1.51.x (LTS) | Everything current | LTS until March 2027. `1.47` LTS expires September 2026 — don't pick it for a new project. |
| `schemars` 0.8.x | `serde` 1.0.x | `schemars` 1.0-alpha exists; wait until 1.0 stable before committing. |
| `tree-sitter` 0.24.x | Grammar crates matching 0.24 ABI | Each grammar crate bumps in lockstep with the parser; a mismatched grammar = runtime panic. Pin all grammars together. |
| `rusqlite` 0.32 | `sqlite-vec` 0.1 | `sqlite-vec` loads as a dynamic extension; use `rusqlite` `load_extension` feature. |
| `clap` 4.5 | Rust ≥ 1.74 | Comfortably below our MSRV. |
| `tauri-plugin-updater` 2.x | Tauri 2.x key format (minisign) | Generate a new keypair at v0.0.1 and publish the public key via the `tauri.conf.json` — do NOT rotate later without careful migration. |

---

## Reference-Implementation Cross-Check

| Source of Evidence | What They Chose | Kay's Alignment |
|---|---|---|
| **ForgeCode** (tailcallhq/forgecode) | Rust, multi-crate workspace, tokio 1.52.x recent bumps, reqwest-based HTTP, JSON schema hardening (required-before-properties, flattened required) | ✅ Full alignment — fork literally inherits this baseline |
| **Codex CLI (codex-rs)** | ~70-crate Cargo workspace; `codex-core` reusable library; `codex-app-server` JSON-RPC bridge (IDE); Ratatui TUI; sandboxing via Seatbelt (macOS) / Landlock (Linux) / RestrictedToken (Windows); insta snapshot tests | ✅ Mirror the workspace split (`kay-core`, `kay-cli`, `kay-tauri-bridge`, `kay-sandbox/*`); adopt insta for TUI-ish snapshot tests in `kay-tauri-bridge` command handlers |
| **Claw Code** (instructkr/claw-code) | 9-crate Rust workspace (`rusty-claude-cli`, `runtime`, `api`, `tools`, `commands`, `plugins`, `telemetry`, `mcp`, `mock-anthropic-service`); `rustyline` REPL | ⚠️ Partial — Kay rejects the Python-sidecar part of claw-code; keep the workspace-split inspiration, skip rustyline (no TUI in Kay v1) |
| **Terminus-KIRA** (KRAFTON AI) | Native LLM tool calling; marker-based `__CMDEND__<seq>__` polling; multimodal `image_read`; multi-perspective verification | ✅ Techniques are library-independent — implement inside `kay-core::tools::shell` and `kay-core::tools::vision` |

---

## Sources

### High-confidence (Context7-equivalent / official docs)

- [Tauri Release Ecosystem](https://v2.tauri.app/release/) — tauri 2.10.3, wry 0.55.0, tao 0.35.0 as of April 2026
- [Tauri: Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — channels vs events; streaming guidance
- [Tauri: Embedding External Binaries (sidecar)](https://v2.tauri.app/develop/sidecar/) — sidecar naming, target-triple convention
- [Tauri: macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) — notarization flow
- [Tauri: Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) — Authenticode / Azure Key Vault
- [tauri-specta v2 docs](https://specta.dev/docs/tauri-specta/v2) — typesafe commands, Tauri v2 only
- [tokio crates.io](https://crates.io/crates/tokio) — 1.51 LTS → March 2027, 1.52.x current
- [reqwest docs (0.13)](https://docs.rs/crate/reqwest/latest) — hyper 1.x; rustls default; stream feature
- [reqwest v0.12 release notes](https://seanmonstar.com/blog/reqwest-v012/) — hyper 1.0 upgrade lineage
- [schemars docs](https://docs.rs/schemars) — `#[derive(JsonSchema)]`, serde-compatible
- [clap derive guide](https://docs.rs/clap/latest/clap/_derive/index.html) — 4.5.x idioms
- [tracing / tracing-subscriber / tracing-opentelemetry docs](https://docs.rs/tracing-opentelemetry) — bridge stability
- [OpenTelemetry Rust](https://opentelemetry.io/docs/languages/rust/) — logs/metrics stable, traces Beta (April 2026)
- [portable-pty docs](https://docs.rs/portable-pty) — WezTerm-sourced, runtime-selectable backends
- [tree-sitter Rust bindings](https://docs.rs/tree-sitter) — 0.24.x; bindings live in main repo
- [sqlite-vec](https://github.com/asg017/sqlite-vec) — embedded vector search extension for SQLite

### Medium-confidence (reference implementations + community docs)

- [ForgeCode GitHub](https://github.com/antinomyhq/forgecode) — 93.6% Rust, workspace layout, active tokio 1.52 bumps
- [ForgeCode blog: "Benchmarks Don't Matter — Until They Do (Part 2)"](https://forgecode.dev/blog/gpt-5-4-agent-improvements/) — JSON schema hardening detail (required before properties, flattened nested required)
- [codex-rs architecture writeup](https://codex.danielvaughan.com/2026/03/28/codex-rs-rust-rewrite-architecture/) — ~70 crate workspace, codex-core reusable, sandbox modules per platform
- [Claw Code rust crate architecture (DeepWiki)](https://deepwiki.com/instructkr/claw-code/2-rust-crate-architecture) — 9-crate layout
- [openrouter_api lib.rs](https://lib.rs/crates/openrouter_api) — v0.6.0, reqwest 0.11 pin → **confirms we should not use it**
- [Leptos vs Yew vs Dioxus (2026)](https://reintech.io/blog/leptos-vs-yew-vs-dioxus-rust-frontend-framework-comparison-2026) — ecosystem maturity comparison supporting React-for-Tauri v1

### Low-confidence (single source / patterns only)

- Cursor agent-sandbox blog and [Pierce Freeman: A deep dive on agent sandboxes](https://pierce.dev/notes/a-deep-dive-on-agent-sandboxes) — sandboxing approach lineage
- Various SSE client comparison blog posts — the choice between `reqwest-eventsource` and `eventsource-client` is partly taste; flag for a Phase-level deeper validation when we first integrate streaming

---

## Flagged for Deeper Phase Research

- **Sandbox crate selection** — `landlock` crate vs hand-rolled syscall via `seccompiler`; Windows RestrictedToken ergonomics. Prototype both before committing. **Confidence: MEDIUM.**
- **Context indexer schema** — SQLite layout for function signatures + vector embeddings is a design question beyond stack selection. Flag for Phase "Context Engine". **Confidence: MEDIUM.**
- **SSE retry semantics** — OpenRouter's 429 behavior + provider-level failovers may need custom retry logic around `reqwest-eventsource`. Validate with real traces in Phase "Provider Integration". **Confidence: LOW.**
- **Tauri 2 sidecar signing** — Known open issue with notarization when using `externalBin` on macOS ([tauri#11992](https://github.com/tauri-apps/tauri/issues/11992)). Verify workaround early; may push us to in-process `kay-core` for v1 and defer sidecar split to v2. **Confidence: MEDIUM.**
- **`schemars` 0.8 vs 1.0-alpha** — schemars 1.0 is in alpha; if it stabilizes mid-milestone, migrating after lock-in could cost a week. Pin 0.8 and schedule a look at 1.0 at Phase 3 boundary. **Confidence: MEDIUM.**

---

*Stack research for: Kay — Rust + Tauri 2.x desktop coding agent forked from ForgeCode with KIRA harness techniques and OpenRouter backend.*
*Researched: 2026-04-19*
