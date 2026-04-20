# Phase 03 — Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 28 new / 3 modified
**Analogs found:** 27 / 28 (1 genuinely novel: marker protocol module has partial analog only)

## Summary

Phase 03 scaffolds a brand-new `crates/kay-tools/` crate. The closest and most load-bearing analog is `crates/kay-provider-openrouter/` — it establishes the canonical shape for a `kay-*` crate (Cargo.toml header, crate-root `#![deny(...)]` directive, module-per-concern layout, `#[non_exhaustive]` typed error enum, `#[async_trait]` object-safe trait + concrete impl, mockito-backed integration tests in `tests/`). Most per-tool builtins (`fs_read`, `fs_write`, `fs_search`, `net_fetch`, `image_read`) are thin adapters that wrap `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)` — the analog is any existing `forge_services::tool_services::*.rs` file (service-struct holding `Arc<I>` + `#[async_trait::async_trait]` impl). The genuinely-new code is the marker protocol (SHELL-01/05), PTY fallback (SHELL-02), and timeout cascade (SHELL-04) — no direct in-tree analog, so those files take cues from `forge_services::tool_services::shell.rs` (non-streaming parity) for struct shape only.

## Crate-level Analog

**Target crate:** `crates/kay-tools/`
**Closest analog:** `crates/kay-provider-openrouter/`
**Why:** Same `kay-*` family, same async+tokio+streaming/trait-object shape, same workspace-pinned deps (`async-trait`, `tokio`, `serde_json`, `schemars`, `thiserror`, `tracing`, `futures`), same `#[non_exhaustive]` evolution pattern on public enums, same mockito/pretty_assertions/proptest test rig. Phase 2.5 precedent; `NOTICE` also modeled on `crates/kay-core/NOTICE`.

**Excerpt — Cargo.toml header shape** (from `crates/kay-provider-openrouter/Cargo.toml:1-43`):

```toml
[package]
name = "kay-provider-openrouter"
publish = false
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "OpenRouter provider HAL (Phase 2)"

[dependencies]
forge_json_repair = { path = "../forge_json_repair" }

tokio = { workspace = true }
reqwest = { workspace = true }
# ...
thiserror = { workspace = true }
async-trait = "0.1"
async-stream = "0.3"
futures = "0.3"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "test-util"] }
mockito = "1.7"
proptest = "1.11"
pretty_assertions = "1"
```

**Excerpt — lib.rs layout** (from `crates/kay-provider-openrouter/src/lib.rs:1-40`):

```rust
//! kay-provider-openrouter — OpenRouter provider HAL (Phase 2).
//! ...

// Crate-wide lint: forbid `.unwrap()` / `.expect()` in non-test code.
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod allowlist;
mod auth;
mod client;
mod error;
mod event;
mod openrouter_provider;
mod provider;
// ...

pub use error::{AuthErrorKind, ProviderError, RetryReason};
pub use event::AgentEvent;
pub use provider::{AgentEventStream, ChatRequest, Message, Provider, ToolSchema};
```

**Excerpt — NOTICE file** (from `crates/kay-core/NOTICE:1-7`):

```
This crate incorporates source code derived from ForgeCode
(https://github.com/antinomyhq/forgecode), Copyright 2025 Tailcall,
licensed under the Apache License, Version 2.0.

Imported from ForgeCode at commit ... on 2026-04-19.

See the repository-level NOTICE for full attribution.
```

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/kay-tools/Cargo.toml` | config (manifest) | build-time | `crates/kay-provider-openrouter/Cargo.toml` | exact |
| `crates/kay-tools/NOTICE` | config (attribution) | build-time | `crates/kay-core/NOTICE` | exact |
| `crates/kay-tools/src/lib.rs` | module (crate root) | build-time | `crates/kay-provider-openrouter/src/lib.rs` | exact |
| `crates/kay-tools/src/tool.rs` | abstraction (async trait) | request-response | `crates/kay-provider-openrouter/src/provider.rs` (Provider trait) | role-match |
| `crates/kay-tools/src/registry.rs` | registry (HashMap store) | CRUD (register/get/list) | `crates/forge_app/src/tool_registry.rs` | role-match (diff: object-safe Arc<dyn> map vs generic-over-S) |
| `crates/kay-tools/src/error.rs` | error taxonomy | transform | `crates/kay-provider-openrouter/src/error.rs` | exact |
| `crates/kay-tools/src/schema.rs` | utility (schema wrap) | transform | `crates/forge_app/src/utils.rs` (enforce_strict_schema) | role-match |
| `crates/kay-tools/src/sandbox.rs` | abstraction (DI seam) | request-response | `crates/kay-provider-openrouter/src/provider.rs` (trait + concrete impl) | role-match |
| `crates/kay-tools/src/verifier.rs` | abstraction (DI seam) | request-response | `crates/kay-provider-openrouter/src/provider.rs` (trait shape) | role-match |
| `crates/kay-tools/src/quota.rs` | utility (atomics) | CRUD counter | — (no direct analog; AtomicU32 idiom) | novel (small) |
| `crates/kay-tools/src/markers/mod.rs` | utility (security primitive) | transform | — (no direct analog) | **novel** |
| `crates/kay-tools/src/markers/shells.rs` | utility (shell wrap) | transform | — (no direct analog) | **novel** |
| `crates/kay-tools/src/pty.rs` (or inside builtins/execute_commands.rs) | adapter (PTY) | streaming | `crates/forge_services/src/tool_services/shell.rs` (struct shape only) | partial |
| `crates/kay-tools/src/timeout.rs` (or inside builtins/execute_commands.rs) | utility (process control) | event-driven | `crates/forge_app/src/tool_registry.rs:45-61` (call_with_timeout) | partial |
| `crates/kay-tools/src/builtins/mod.rs` | module | build-time | `crates/forge_services/src/tool_services/mod.rs` | exact |
| `crates/kay-tools/src/builtins/execute_commands.rs` | adapter (streaming shell) | streaming | `crates/forge_services/src/tool_services/shell.rs` (non-streaming parity ref) | partial — bypasses ToolExecutor |
| `crates/kay-tools/src/builtins/task_complete.rs` | tool (verifier-gated) | request-response + event | — (no parity) | **novel** |
| `crates/kay-tools/src/builtins/image_read.rs` | adapter (quota-wrapped) | CRUD | `crates/forge_services/src/tool_services/image_read.rs` | role-match |
| `crates/kay-tools/src/builtins/fs_read.rs` | adapter (delegator) | CRUD | `crates/forge_services/src/tool_services/fs_read.rs` | exact |
| `crates/kay-tools/src/builtins/fs_write.rs` | adapter (delegator) | CRUD | `crates/forge_services/src/tool_services/fs_write.rs` | exact |
| `crates/kay-tools/src/builtins/fs_search.rs` | adapter (delegator) | CRUD | `crates/forge_services/src/tool_services/fs_search.rs` | exact |
| `crates/kay-tools/src/builtins/net_fetch.rs` | adapter (delegator) | CRUD | `crates/forge_services/src/tool_services/fetch.rs` | exact |
| `crates/kay-tools/src/default_set.rs` | factory | build-time | `crates/forge_app/src/tool_registry.rs` (new() pattern) | role-match |
| `crates/kay-tools/tests/registry_integration.rs` | integration test | request-response | `crates/kay-provider-openrouter/tests/tool_call_reassembly.rs` | role-match |
| `crates/kay-tools/tests/marker_streaming.rs` | integration test | streaming | `crates/kay-provider-openrouter/tests/streaming_happy_path.rs` | role-match |
| `crates/kay-tools/tests/marker_race.rs` | integration test | streaming | `crates/kay-provider-openrouter/tests/tool_call_malformed.rs` | role-match |
| `crates/kay-tools/tests/timeout_cascade.rs` | integration test | event-driven | `crates/kay-provider-openrouter/tests/retry_429_503.rs` | role-match |
| `crates/kay-tools/tests/pty_integration.rs` | integration test (gated) | streaming | same as marker_streaming.rs | role-match |
| `crates/kay-tools/tests/image_quota.rs` | integration test | CRUD | `crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs` | role-match |
| `crates/kay-tools/tests/schema_hardening_property.rs` | property test | transform | `crates/forge_app/src/utils.rs` test module (proptest patterns) | role-match |
| `Cargo.toml` (workspace root, MODIFIED) | config | build-time | existing file; append member + 2 deps | — |
| `crates/kay-provider-openrouter/src/event.rs` (MODIFIED) | event enum | event-driven | itself (additive `#[non_exhaustive]` variants) | exact (self-reference) |
| `crates/kay-cli/src/main.rs` (MODIFIED at Phase 5 — deferred; this phase only plans the seam) | config | build-time | — | deferred |

## Pattern Assignments

### `crates/kay-tools/Cargo.toml` (config, build-time)

**Analog:** `crates/kay-provider-openrouter/Cargo.toml`

**Copy the workspace-inheritance header pattern verbatim** (lines 1-10), then adapt the `[dependencies]` block to match the deps listed in RESEARCH §1 (forge_domain/app/services/config + portable-pty + subtle + nix/windows-sys platform-conditionals). Use `{ workspace = true }` for every dep that exists in the root workspace deps table.

**Key divergence:** target-conditional blocks needed:

```toml
[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"] }

[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true, features = ["Win32_System_Threading"] }
```

(Note: workspace root currently pins `windows-sys` with feature `Win32_System_Console` only — the planner must ADD `Win32_System_Threading` to the root feature list OR declare per-crate feature subset. See `Cargo.toml:187`.)

---

### `crates/kay-tools/src/lib.rs` (module, build-time)

**Analog:** `crates/kay-provider-openrouter/src/lib.rs` (full file, 40 lines)

**Copy pattern:**
- Line 1-13: crate-doc comment with role + Phase reference
- Line 17: `#![deny(clippy::unwrap_used, clippy::expect_used)]` — **load-bearing** per RESEARCH "Anti-Patterns" and Phase 2 precedent
- Lines 19-29: `mod X;` declarations, one per concern
- Lines 31-40: `pub use` re-export block

**New lib.rs skeleton (per RESEARCH §1):**

```rust
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod tool;
pub mod registry;
pub mod error;
pub mod schema;
pub mod sandbox;
pub mod verifier;
pub mod quota;
pub mod markers;
pub mod builtins;
mod default_set;

pub use tool::Tool;
pub use registry::ToolRegistry;
pub use error::{ToolError, CapScope};
pub use sandbox::{Sandbox, NoOpSandbox, SandboxDenial};
pub use verifier::{TaskVerifier, NoOpVerifier};
pub use default_set::default_tool_set;
```

---

### `crates/kay-tools/src/tool.rs` (abstraction, request-response)

**Analog:** `crates/kay-provider-openrouter/src/provider.rs` lines 60-75

**Imports pattern** (provider.rs:10-17):
```rust
use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

use crate::error::ProviderError;
use crate::event::AgentEvent;
```

**Trait-definition pattern** (provider.rs:62-75):
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat<'a>(
        &'a self,
        request: ChatRequest,
    ) -> Result<AgentEventStream<'a>, ProviderError>;

    async fn models(&self) -> Result<Vec<String>, ProviderError>;
}
```

**Kay Tool trait (per CONTEXT D-01, RESEARCH Pattern 1):**
```rust
#[async_trait::async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &ToolName;
    fn description(&self) -> &str;
    fn input_schema(&self) -> &schemars::Schema;

    async fn invoke(
        &self,
        args: serde_json::Value,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError>;
}
```

**Divergence from Provider trait:** Tool is object-safe (must work behind `Arc<dyn Tool>`) — so NO generic parameters, NO `<'a>` lifetime on `invoke`. The provider.rs trait has `<'a>` on `chat` but `Provider` is also object-safe; confirm by mirroring that exact pattern. Note **RESEARCH Q16 (§15)**: planner may add `call_id: &str` as a third `invoke` arg — that decision is made in plan 03-02.

---

### `crates/kay-tools/src/registry.rs` (registry, CRUD)

**Analog:** `crates/forge_app/src/tool_registry.rs` lines 28-61

**Struct + timeout helper pattern** (tool_registry.rs:28-61):
```rust
pub struct ToolRegistry<S> {
    tool_executor: ToolExecutor<S>,
    agent_executor: AgentExecutor<S>,
    mcp_executor: McpExecutor<S>,
    services: Arc<S>,
}

impl<S: Services + EnvironmentInfra<Config = forge_config::ForgeConfig>> ToolRegistry<S> {
    pub fn new(services: Arc<S>) -> Self { ... }

    async fn call_with_timeout<F, Fut>(
        &self,
        tool_name: &ToolName,
        future: F,
    ) -> anyhow::Result<ToolOutput>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<ToolOutput>>,
    {
        let tool_timeout = Duration::from_secs(self.services.get_config()?.tool_timeout_secs);
        timeout(tool_timeout, future())
            .await
            .context(Error::CallTimeout {
                timeout: tool_timeout.as_secs() / 60,
                tool_name: tool_name.clone(),
            })?
    }
}
```

**Divergence (per D-01, D-11):**
- Kay's `ToolRegistry` is NOT generic-over-S. It holds `HashMap<ToolName, Arc<dyn Tool>>` directly (object-safe trait object map).
- Immutable after construction (no runtime registration API in v1 per D-11).
- Add `pub fn tool_definitions(&self) -> Vec<ToolDefinition>` that iterates the map and emits one `ToolDefinition` per tool using `name()`, `description()`, and `input_schema()` (for provider `tools` parameter emission per TOOL-06).

**Reuse `call_with_timeout` behavior** (timeout cascade per D-05); Kay's registry wraps `Tool::invoke` in `tokio::time::timeout` identically. For the `execute_commands` marker path the timeout is handled in-tool (so the cascade can do SIGTERM → grace → SIGKILL); for all other tools the registry-level timeout is sufficient.

---

### `crates/kay-tools/src/error.rs` (error taxonomy, transform)

**Analog:** `crates/kay-provider-openrouter/src/error.rs` lines 13-77

**Pattern — `#[non_exhaustive]` + `thiserror::Error` + `Display`-first variants** (error.rs:13-51):
```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("rate limited; retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },

    #[error("authentication failed: {reason:?}")]
    Auth { reason: AuthErrorKind },

    #[error("model {requested} not allowlisted; allowed: {allowed:?}")]
    ModelNotAllowlisted {
        requested: String,
        allowed: Vec<String>,
    },

    // ...
}
```

**Sub-enum pattern for discriminant-only sub-classifications** (error.rs:57-66):
```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthErrorKind {
    Missing,
    Invalid,
    Expired,
}
```

**Kay ToolError (per D-09, RESEARCH §Decisions):**
```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{tool}: invalid arguments: {reason}")]
    InvalidArgs { tool: ToolName, reason: String },
    #[error("{tool}: timed out after {elapsed:?}")]
    Timeout { tool: ToolName, elapsed: Duration },
    #[error("{tool}: execution failed")]
    ExecutionFailed { tool: ToolName, #[source] source: anyhow::Error },
    #[error("image cap exceeded at {scope:?} (limit {limit})")]
    ImageCapExceeded { scope: CapScope, limit: u32 },
    #[error("{tool}: sandbox denied: {reason}")]
    SandboxDenied { tool: ToolName, reason: String },
    #[error("{tool}: not found")]
    NotFound { tool: ToolName },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapScope { PerTurn, PerSession }
```

**Copy test discipline from** `error.rs:83-104` — the `debug_impl_never_prints_credential_material` + `provider_error_display_includes_context` unit tests. Kay equivalents: `tool_error_display_includes_context`, `tool_error_invalidargs_roundtrips_reason`.

---

### `crates/kay-tools/src/schema.rs` (utility, transform)

**Analog:** `crates/forge_app/src/utils.rs:340-440` (enforce_strict_schema)

**Core hardening pattern — DO NOT reimplement; delegate** (utils.rs:351-440):
```rust
pub fn enforce_strict_schema(schema: &mut serde_json::Value, strict_mode: bool) {
    match schema {
        serde_json::Value::Object(map) => {
            if strict_mode {
                flatten_all_of_schema(map);
                map.remove("propertyNames");
            }
            normalize_string_format_keyword(map, strict_mode);
            let is_object = is_object_schema(map);
            if is_object && !map.contains_key("type") {
                map.insert("type".to_string(),
                    serde_json::Value::String("object".to_string()));
            }
            // ...adds properties, required, additionalProperties=false, nullable->anyOf
        }
        serde_json::Value::Array(items) => {
            for value in items { enforce_strict_schema(value, strict_mode); }
        }
        _ => {}
    }
}
```

**Kay wrapper** (per D-02, RESEARCH §Code Examples):
```rust
pub fn harden_tool_schema(schema: &mut Value, hints: &TruncationHints) {
    forge_app::utils::enforce_strict_schema(schema, true);

    if let Some(obj) = schema.as_object_mut() {
        if let Some(note) = hints.output_truncation_note.as_ref() {
            match obj.get_mut("description") {
                Some(Value::String(existing)) => {
                    existing.push_str("\n\n");
                    existing.push_str(note);
                }
                _ => {
                    obj.insert("description".into(), Value::String(note.clone()));
                }
            }
        }
    }
}
```

**Validation test pattern** — use `forge_app::utils` tests as model: every assertion is `schema["required"]`-order checks (see utils.rs lines 777-1279 for the `enforce_strict_schema` test suite; copy the assertion style for Kay's property test in `tests/schema_hardening_property.rs`).

---

### `crates/kay-tools/src/sandbox.rs` (abstraction, DI seam)

**Analog:** `crates/kay-provider-openrouter/src/provider.rs` lines 62-75 (trait shape)

**Kay surface (per D-12, RESEARCH §9):**
```rust
#[derive(Debug)]
pub struct SandboxDenial {
    pub reason: String,
    pub resource: String,
}

#[async_trait::async_trait]
pub trait Sandbox: Send + Sync {
    async fn check_shell(&self, command: &str, cwd: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_read(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_write(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_search(&self, path: &Path) -> Result<(), SandboxDenial> {
        self.check_fs_read(path).await
    }
    async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial>;
}

pub struct NoOpSandbox;

#[async_trait::async_trait]
impl Sandbox for NoOpSandbox {
    async fn check_shell(&self, _: &str, _: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    // ... all return Ok(()) in Phase 3
}
```

**Trait-plus-concrete-impl pattern:** mirrors the `Provider` trait + `OpenRouterProvider` pattern from Phase 2 (crate-level `kay-provider-openrouter` has `trait Provider` in `provider.rs` + `struct OpenRouterProvider` in `openrouter_provider.rs` — same split pattern for `Sandbox` + `NoOpSandbox`).

---

### `crates/kay-tools/src/verifier.rs` (abstraction, DI seam)

**Analog:** same as sandbox.rs — `crates/kay-provider-openrouter/src/provider.rs:62-75`

**Kay surface (per D-06):**
```rust
#[async_trait::async_trait]
pub trait TaskVerifier: Send + Sync {
    async fn verify(&self, task_summary: &str) -> VerificationOutcome;
    // Note RESEARCH §8 Option (c): drop transcript arg for Phase 3; Phase 8 adds it.
}

pub struct NoOpVerifier;

#[async_trait::async_trait]
impl TaskVerifier for NoOpVerifier {
    async fn verify(&self, _: &str) -> VerificationOutcome {
        VerificationOutcome::Pending {
            reason: "Multi-perspective verification wired in Phase 8 (VERIFY-01..04)".into(),
        }
    }
}
```

**`VerificationOutcome` location decision (RESEARCH §7 Option a):** define in `crates/kay-provider-openrouter/src/event.rs` alongside the new `AgentEvent::TaskComplete` variant to break the cycle (kay-tools → kay-provider-openrouter); re-export from `kay-tools::verifier`. See `crates/kay-provider-openrouter/src/event.rs:11-13` for existing import style.

---

### `crates/kay-tools/src/markers/mod.rs` + `markers/shells.rs` (utility, security primitive)

**Analog:** **NONE** — this is genuinely novel per RESEARCH §4 "KIRA marker protocol is load-bearing for the TB 2.0 score." Planner uses RESEARCH §Code Examples (lines 455-535) verbatim as the starting point.

**Module structure (planner decision, CONTEXT.md Claude's Discretion):**
- `markers/mod.rs` — `MarkerContext` struct, `ScanResult` enum, `scan_line` fn (constant-time compare), tests
- `markers/shells.rs` — `wrap_unix_sh`, `wrap_windows_ps`, `wrap_windows_cmd` (per-OS wrap templates)

**Key primitives from RESEARCH:**
```rust
// markers/shells.rs
pub struct MarkerContext {
    pub nonce_hex: String,
    pub seq: u64,
    pub line_prefix: String,
}
impl MarkerContext {
    pub fn new(counter: &AtomicU64) -> Self {
        let mut nonce_bytes = [0u8; 16];
        OsRng.try_fill_bytes(&mut nonce_bytes).expect("OsRng never fails");
        // ^^^ NB: .expect() violates crate-root deny; use .map_err + unreachable!()
        //     or gate with #[allow(clippy::expect_used)] + justification comment
        let nonce_hex = hex::encode(nonce_bytes);
        let seq = counter.fetch_add(1, Ordering::Relaxed);
        let line_prefix = format!("__CMDEND_{nonce_hex}_{seq}__");
        Self { nonce_hex, seq, line_prefix }
    }
}
```

```rust
// markers/mod.rs — scan with subtle::ConstantTimeEq (per D-03)
use subtle::ConstantTimeEq;
pub fn scan_line(line: &str, m: &MarkerContext) -> ScanResult {
    if !line.starts_with("__CMDEND_") {
        return ScanResult::NotMarker;
    }
    let expected = m.line_prefix.as_bytes();
    let actual = &line.as_bytes()[..expected.len().min(line.len())];
    if actual.ct_eq(expected).unwrap_u8() == 0 {
        return ScanResult::ForgedMarker;  // SHELL-05
    }
    let tail = &line[expected.len()..];
    if let Some(num_str) = tail.strip_prefix("EXITCODE=") {
        if let Ok(n) = num_str.trim_end().parse::<i32>() {
            return ScanResult::Marker { exit_code: n };
        }
    }
    ScanResult::ForgedMarker
}
```

**Anti-pattern enforcement** (RESEARCH §Anti-Patterns): no `unwrap()` / `expect()` in scan or kill paths — crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` already covers this. For the legitimately-infallible `OsRng.try_fill_bytes` call, wrap it in a `.map_err(|_| unreachable!(...))` or use `rand::Rng::fill_bytes` (non-fallible).

---

### `crates/kay-tools/src/builtins/execute_commands.rs` (adapter, streaming)

**Analog (partial):** `crates/forge_services/src/tool_services/shell.rs` lines 20-65 — for struct-shape only. **Behavior bypasses `ToolExecutor::execute`** per RESEARCH §14 parity audit.

**ForgeShell struct pattern** (shell.rs:20-65):
```rust
pub struct ForgeShell<I> {
    env: Environment,
    infra: Arc<I>,
}

impl<I: EnvironmentInfra> ForgeShell<I> {
    pub fn new(infra: Arc<I>) -> Self {
        let env = infra.get_environment();
        Self { env, infra }
    }
    fn validate_command(command: &str) -> anyhow::Result<()> {
        if command.trim().is_empty() {
            bail!("Command string is empty or contains only whitespace");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<I: CommandInfra + EnvironmentInfra> ShellService for ForgeShell<I> {
    async fn execute(
        &self,
        command: String,
        cwd: PathBuf,
        keep_ansi: bool,
        silent: bool,
        env_vars: Option<Vec<String>>,
        description: Option<String>,
    ) -> anyhow::Result<ShellOutput> {
        Self::validate_command(&command)?;
        // collect-then-return — NOT streaming
        let mut output = self.infra.execute_command(command, cwd, silent, env_vars).await?;
        // ...
    }
}
```

**Kay ExecuteCommandsTool (per RESEARCH §4, §5, §6 — composes):**
- Struct holds: `Arc<Services>`, `Arc<dyn Sandbox>`, `Arc<AtomicU64>` (seq counter), `schema: Value`, `name: ToolName`, `description: String`, `mpsc::Sender<AgentEvent>` (for streaming).
- `invoke` pipeline: validate args (reject `__CMDEND_` substring per §4 threat model) → `sandbox.check_shell` → pick PTY vs tokio::process (heuristic §5) → wrap with marker (§4) → spawn → two `BufReader::lines()` tasks (or PTY spawn_blocking per §5) → per-line `scan_line` → emit `AgentEvent::ToolOutput` → on marker: emit `Closed{marker_detected:true}` → on EOF before marker: emit `Closed{marker_detected:false}` (SHELL-05 recovery) → timeout cascade on expiration (§6).
- Use RESEARCH §Code-Examples timeout cascade verbatim (lines 537-571).

**Critical divergence:** must normalize cwd manually before spawn because the marker path doesn't go through `ToolExecutor` (RESEARCH Pitfall 7). Copy the 6-line `normalize_path` logic from `forge_app::ToolExecutor` helpers (requires read at plan-write time).

---

### `crates/kay-tools/src/builtins/fs_read.rs` (adapter, delegator — CRUD)

**Analog:** `crates/forge_services/src/tool_services/fs_read.rs` lines 94-134

**Adapter struct pattern** (fs_read.rs:94-104):
```rust
pub struct ForgeFsRead<F> {
    infra: Arc<F>,
}
impl<F> ForgeFsRead<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self { infra }
    }
}

#[async_trait::async_trait]
impl<F: FileInfoInfra + EnvironmentInfra<Config = forge_config::ForgeConfig> + InfraFsReadService>
    FsReadService for ForgeFsRead<F>
{
    async fn read(
        &self,
        path: String,
        start_line: Option<u64>,
        end_line: Option<u64>,
    ) -> anyhow::Result<ReadOutput> {
        let path = Path::new(&path);
        assert_absolute_path(path)?;
        // ... delegation to infra.read(...)
    }
}
```

**Kay FsReadTool (per RESEARCH Pattern 1):**
```rust
pub struct FsReadTool<S> {
    services: Arc<S>,
    sandbox: Arc<dyn Sandbox>,
    name: ToolName,
    description: String,
    schema: Value,   // hardened once at construction
}

#[async_trait::async_trait]
impl<S: /* Services bounds */> Tool for FsReadTool<S> {
    fn name(&self) -> &ToolName { &self.name }
    fn description(&self) -> &str { &self.description }
    fn input_schema(&self) -> &Value { &self.schema }  // or &Schema per planner Q

    async fn invoke(&self, args: Value, ctx: &ToolCallContext) -> Result<ToolOutput, ToolError> {
        let input: FSRead = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs {
                tool: self.name.clone(),
                reason: e.to_string(),
            })?;
        self.sandbox.check_fs_read(Path::new(&input.file_path)).await
            .map_err(|d| ToolError::SandboxDenied {
                tool: self.name.clone(),
                reason: d.reason,
            })?;
        let executor = forge_app::ToolExecutor::new(self.services.clone());
        executor.execute(ToolCatalog::Read(input), ctx).await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name.clone(),
                source: e,
            })
    }
}
```

**Input struct reference** (forge_domain/src/tools/catalog.rs:193-212):
```rust
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, ToolDescription, PartialEq)]
#[tool_description_file = "crates/forge_domain/src/tools/descriptions/fs_read.md"]
pub struct FSRead {
    #[serde(alias = "path")]
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<i32>,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i32>,
}
```

Schema generated via `schemars::schema_for!(FSRead)` once at construction; `harden_tool_schema` applied immediately.

---

### `crates/kay-tools/src/builtins/fs_write.rs` / `fs_search.rs` / `net_fetch.rs` (adapters, delegators)

**Analogs:**
- `fs_write.rs` → `crates/forge_services/src/tool_services/fs_write.rs` (ForgeFsWrite struct pattern)
- `fs_search.rs` → `crates/forge_services/src/tool_services/fs_search.rs`
- `net_fetch.rs` → `crates/forge_services/src/tool_services/fetch.rs`

**Use identical adapter skeleton to fs_read.rs above.** Only changes per file:
- Input struct: `FSWrite` (`catalog.rs:214-230`), `FSSearch` (`catalog.rs:232-?`), `NetFetch` (`catalog.rs:622-630`)
- Sandbox check: `check_fs_write`, `check_fs_search`, `check_net`
- ToolCatalog variant: `Write`, `FsSearch`, `Fetch`

**Parity guarantee** (RESEARCH §14): no custom logic — the `ToolExecutor::execute` delegation preserves ForgeCode's behavior byte-for-byte, including the `check_fs_write` read-before-overwrite enforcement inside ToolExecutor.

---

### `crates/kay-tools/src/builtins/image_read.rs` (adapter, CRUD with quota)

**Analog:** `crates/forge_services/src/tool_services/image_read.rs:11-97`

**Core delegation** (image_read.rs:49-96):
```rust
#[async_trait::async_trait]
impl<F: FileInfoInfra + EnvironmentInfra<Config = forge_config::ForgeConfig>
     + forge_app::FileReaderInfra> ImageReadService for ForgeImageRead<F>
{
    async fn read_image(&self, path: String) -> anyhow::Result<Image> {
        let path = Path::new(&path);
        assert_absolute_path(path)?;
        let max_image_size_bytes = self.infra.get_config()?.max_image_size_bytes;
        crate::tool_services::fs_read::assert_file_size(&*self.infra, path, max_image_size_bytes).await?;
        // ... decode by extension, build Image
        let image = Image::new_bytes(content, format.mime_type());
        Ok(image)
    }
}
```

**Kay ImageReadTool:** wraps `ForgeImageRead` + layers `ImageQuota::try_consume()` check before delegation (per D-07). Returns `ToolError::ImageCapExceeded { scope, limit }` on cap hit.

---

### `crates/kay-tools/src/builtins/task_complete.rs` (tool, novel)

**Analog:** **NONE** — no ForgeCode parity equivalent (RESEARCH §14). Structure follows the generic Tool-trait adapter pattern.

**Skeleton (per D-06):**
```rust
pub struct TaskCompleteTool {
    verifier: Arc<dyn TaskVerifier>,
    sender: mpsc::Sender<AgentEvent>,
    name: ToolName,
    description: String,
    schema: Value,
}

#[async_trait::async_trait]
impl Tool for TaskCompleteTool {
    // name/description/input_schema per pattern
    async fn invoke(&self, args: Value, _ctx: &ToolCallContext) -> Result<ToolOutput, ToolError> {
        let input: TaskCompleteInput = serde_json::from_value(args).map_err(...)?;
        let outcome = self.verifier.verify(&input.summary).await;
        let verified = matches!(outcome, VerificationOutcome::Pass { .. });
        // emit AgentEvent::TaskComplete { call_id, verified, outcome: outcome.clone() }
        // via self.sender
        Ok(ToolOutput::text(format!("task_complete: {:?}", outcome)))
    }
}
```

---

### `crates/kay-tools/src/quota.rs` (utility, atomics)

**Analog:** no direct in-tree analog. Copy RESEARCH §10 verbatim (lines 1018-1052).

---

### `crates/kay-tools/src/default_set.rs` (factory)

**Analog:** `crates/forge_app/src/tool_registry.rs:36-43` (services-in, registry-out pattern)

```rust
impl<S: Services + ...> ToolRegistry<S> {
    pub fn new(services: Arc<S>) -> Self {
        Self {
            services: services.clone(),
            tool_executor: ToolExecutor::new(services.clone()),
            // ... executor wiring
        }
    }
}
```

**Kay factory signature (per D-11, D-12):**
```rust
pub fn default_tool_set<S: /* bounds */>(
    services: Arc<S>,
    sandbox: Arc<dyn Sandbox>,
    verifier: Arc<dyn TaskVerifier>,
    event_sender: mpsc::Sender<AgentEvent>,
    quota: Arc<ImageQuota>,
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(FsReadTool::new(services.clone(), sandbox.clone())));
    registry.register(Arc::new(FsWriteTool::new(services.clone(), sandbox.clone())));
    registry.register(Arc::new(FsSearchTool::new(services.clone(), sandbox.clone())));
    registry.register(Arc::new(NetFetchTool::new(services.clone(), sandbox.clone())));
    registry.register(Arc::new(ExecuteCommandsTool::new(...)));
    registry.register(Arc::new(ImageReadTool::new(services.clone(), quota)));
    registry.register(Arc::new(TaskCompleteTool::new(verifier, event_sender)));
    registry
}
```

---

### `crates/kay-provider-openrouter/src/event.rs` (MODIFIED, additive variants)

**Analog:** the file itself (`crates/kay-provider-openrouter/src/event.rs:1-72`)

**Existing pattern** — the enum is already `#[non_exhaustive]` (line 15) with crate-doc comment at line 1-9 explicitly listing Phase 3 as a planned extension point:

```rust
//! `AgentEvent` is the delta-granular frame type emitted by `Provider::chat`.
//! ... For Phase 2 it already carries the annotation so Phase 3
//! can add `ToolOutput`, Phase 4 `SandboxViolation`, Phase 5 `TurnEnd`, Phase 8
//! `Verification` without breaking callers.

#[non_exhaustive]
#[derive(Debug)]
pub enum AgentEvent {
    TextDelta { content: String },
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, arguments_delta: String },
    ToolCallComplete { id: String, name: String, arguments: Value },
    ToolCallMalformed { id: String, raw: String, error: String },
    Usage { prompt_tokens: u64, completion_tokens: u64, cost_usd: f64 },
    Retry { attempt: u32, delay_ms: u64, reason: RetryReason },
    Error { error: ProviderError },
}
```

**Additive changes per D-08:**
```rust
// Append to the enum:
    ToolOutput {
        call_id: String,
        chunk: ToolOutputChunk,
    },
    TaskComplete {
        call_id: String,
        verified: bool,
        outcome: VerificationOutcome,
    },

// Add below the AgentEvent enum:
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ToolOutputChunk {
    Stdout(String),
    Stderr(String),
    Closed { exit_code: Option<i32>, marker_detected: bool },
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum VerificationOutcome {
    Pass    { rationale: String },
    Fail    { reasons: Vec<String> },
    Pending { reason: String },
}
```

**Clone note** (RESEARCH §7): `AgentEvent` doesn't derive Clone; `ToolOutputChunk` and `VerificationOutcome` do (no non-Clone fields).

---

### Test files — `tests/*.rs` (integration, role-match)

**Primary analog for all integration tests:** `crates/kay-provider-openrouter/tests/tool_call_reassembly.rs:1-80`

**Header pattern** (lines 1-38):
```rust
//! Integration test: per-tool_call.id reassembly (PROV-01, AC-04).
//! ...

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider, ToolSchema,
};
use serde_json::json;

use crate::common::mock_server::MockServer;

#[tokio::test]
async fn fragmented_tool_call_reassembles_under_single_id() {
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("tool_call_fragmented");
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;
    // ... build provider + chat request + assert events
}
```

**Test discipline pattern:**
- `#![allow(clippy::unwrap_used, clippy::expect_used)]` at test-file level — required because crate-root `#![deny(...)]` applies. Test code is explicitly exempted.
- `#[tokio::test]` macros for async tests
- `mod common;` for shared test harness (mock_server.rs pattern from `tests/common/mod.rs`)
- `pretty_assertions::assert_eq!` for readable diff output
- proptest for `schema_hardening_property.rs` — see `forge_app::utils` tests for in-tree proptest patterns (schema → hardened → assert `required` is sorted + contains all property keys)

**Per-file test focus:**

| Test file | Focus | Covers | Pattern from |
|-----------|-------|--------|--------------|
| `registry_integration.rs` | all 7 tools round-trip schema + emit ToolDefinition | TOOL-01/05/06 | tool_call_reassembly.rs (full integration) |
| `marker_streaming.rs` | spawn `sh -c` sub-shell; assert Stdout frames arrive BEFORE Closed | SHELL-01, SHELL-03 | streaming_happy_path.rs |
| `marker_race.rs` | inject forged `__CMDEND_...` in command output; assert stream continues | SHELL-05 | tool_call_malformed.rs |
| `timeout_cascade.rs` | sleep 300s command; assert SIGTERM at grace, SIGKILL at 2s, reap | SHELL-04 | retry_429_503.rs (timeout assertion style) |
| `pty_integration.rs` | `#[cfg(unix)]`-gated; spawn a tty-requiring command | SHELL-02 | marker_streaming.rs |
| `image_quota.rs` | 3rd `image_read` in a turn returns `ImageCapExceeded` | TOOL-04 | cost_cap_turn_boundary.rs |
| `schema_hardening_property.rs` | proptest: every registered tool's schema has sorted `required` | TOOL-05 | forge_app/src/utils.rs tests (mod tests at line 777+) |

**No new `tests/common/` directory needed** unless planner opts to add a shared test helper; for Phase 3, most tests don't need HTTP mocking — they're local subprocess / pure-function tests.

---

### `Cargo.toml` (workspace root, MODIFIED)

**Analog:** itself — append `"crates/kay-tools"` to the `members` list (currently lines 6-38) and add new workspace-dep entries to `[workspace.dependencies]` (currently lines 52-188).

**Additions per RESEARCH §11:**
```toml
# Append to members list (line ~38):
    "crates/kay-tools",

# Append under [workspace.dependencies] (near line 188):
portable-pty = "0.8"
subtle       = "2"
# Optionally:
# nix = { version = "0.29", features = ["signal"] }
```

**Modification** to existing `windows-sys` entry (line 187): add `"Win32_System_Threading"` to feature list, or declare that feature subset per-crate in kay-tools Cargo.toml.

---

## Shared Patterns

### Authentication / Policy Check (Sandbox DI)

**Source:** `crates/kay-tools/src/sandbox.rs` (Phase 3 new file; `NoOpSandbox` impl)

**Apply to:** ALL builtin tool structs — every tool constructor takes `sandbox: Arc<dyn Sandbox>` and every `invoke` calls the appropriate `check_*` before delegation.

```rust
self.sandbox.check_fs_read(Path::new(&input.file_path)).await
    .map_err(|d| ToolError::SandboxDenied {
        tool: self.name.clone(),
        reason: d.reason,
    })?;
```

This mirrors the `ToolRegistry::check_tool_permission` pattern in `forge_app/src/tool_registry.rs:64-91` — centralized policy check before execution.

### Error Handling

**Source:** `crates/kay-tools/src/error.rs` (Phase 3 new file)
**Apply to:** Every `Tool::invoke` impl — map each failure mode to a specific `ToolError` variant (never bare `anyhow::Error` except wrapped in `ExecutionFailed { source }`).

**Pattern applied everywhere:**
```rust
// Arg deserialization
let input: T = serde_json::from_value(args)
    .map_err(|e| ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() })?;

// Sandbox check
self.sandbox.check_*(...).await
    .map_err(|d| ToolError::SandboxDenied { tool: self.name.clone(), reason: d.reason })?;

// Delegate to forge_app
forge_app::ToolExecutor::new(svc).execute(ToolCatalog::X(input), ctx).await
    .map_err(|e| ToolError::ExecutionFailed { tool: self.name.clone(), source: e })?;
```

### Schema Hardening

**Source:** `crates/kay-tools/src/schema.rs` (Phase 3 new file wrapping `forge_app::utils::enforce_strict_schema`)
**Apply to:** Every Tool struct's construction — `schemars::schema_for!(InputType)` → `.to_value()` → `harden_tool_schema(&mut value, &hints)` → cache the hardened Value.

**Call site pattern (applied in every `TOOL::new()` constructor):**
```rust
pub fn new(services: Arc<S>, sandbox: Arc<dyn Sandbox>) -> Self {
    let mut schema = serde_json::to_value(schemars::schema_for!(FSRead))
        .expect("schema_for! always succeeds");
    harden_tool_schema(&mut schema, &TruncationHints {
        output_truncation_note: Some("For large files, use start_line/end_line ranges.".into()),
    });
    Self {
        services,
        sandbox,
        name: ToolName::new("fs_read"),
        description: FSRead::default().description(),
        schema,
    }
}
```

### Async trait pattern

**Canonical example:** `crates/kay-provider-openrouter/src/provider.rs:62-75` (Provider trait) and `crates/forge_services/src/tool_services/shell.rs:40-65` (ShellService impl).

**Kay-tools convention:**
- `#[async_trait::async_trait]` macro at both trait definition site AND every impl block.
- `Send + Sync + 'static` bounds on the trait (required for `Arc<dyn Tool>`).
- No generic parameters on the trait itself (Tool) — keeps it object-safe. Services generics go on the STRUCT (e.g., `FsReadTool<S>`) not the trait.

### Test organization

**Canonical example:** `crates/kay-provider-openrouter/tests/` layout:
```
tests/
├── common/
│   ├── mod.rs
│   └── mock_server.rs       # shared test harness
├── fixtures/                 # JSON cassettes
├── tool_call_reassembly.rs
├── streaming_happy_path.rs
├── ...
```

**Kay-tools layout per RESEARCH §1:**
```
tests/
├── registry_integration.rs
├── marker_streaming.rs
├── marker_race.rs
├── timeout_cascade.rs
├── pty_integration.rs
├── image_quota.rs
└── schema_hardening_property.rs
```

(No `common/` shared module needed unless PTY tests want a shared sub-process spawner helper.)

### Workspace dep addition

**Pattern from Phase 2.5 plans:** top-level `Cargo.toml` `[workspace.dependencies]` table holds a version (e.g., `portable-pty = "0.8"`); per-crate `Cargo.toml` consumes via `{ workspace = true }`. See `Cargo.toml:80-90` (async-trait, futures, etc.) for existing entries.

**Platform-conditional dep pattern (new for Phase 3):** per-crate Cargo.toml uses `[target.'cfg(unix)'.dependencies]` / `[target.'cfg(windows)'.dependencies]` — not yet used anywhere in the workspace; see `forge_services` for target-cfg'd code (`#[cfg(unix)]` blocks exist but no target-conditional Cargo.toml entry yet).

## No Analog Found

Files where the closest analog covers only struct shape, not behavior. Planner MUST use RESEARCH §Code Examples verbatim as the starting point rather than copying from a forge_* file:

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/kay-tools/src/markers/mod.rs` | security primitive | transform | No marker-protocol implementation exists anywhere in the workspace; this is KIRA-derived novel code |
| `crates/kay-tools/src/markers/shells.rs` | shell wrap template | transform | No shell-wrap-with-nonce-tail code exists in forge_services or elsewhere |
| `crates/kay-tools/src/builtins/execute_commands.rs` | streaming shell | streaming | `forge_services::ForgeShell::execute` is collect-then-return; Kay's marker protocol legitimately replaces it (RESEARCH §14 parity audit flags this explicitly) |
| `crates/kay-tools/src/builtins/task_complete.rs` | verifier-gated tool | request-response + event | No parity equivalent in `ToolCatalog` |

**For these files:** copy the sandbox/trait-object scaffolding from the shared patterns above; use RESEARCH §Code Examples (lines 412-571) as the behavior blueprint.

## Metadata

**Analog search scope:**
- `crates/kay-provider-openrouter/` — full (lib.rs, provider.rs, event.rs, error.rs, tests/)
- `crates/forge_app/` — utils.rs (schema), tool_executor.rs, tool_registry.rs, infra.rs
- `crates/forge_domain/` — tools/catalog.rs (input structs), tools/definition/, tools/call/context.rs
- `crates/forge_services/src/tool_services/` — fs_read.rs, shell.rs, image_read.rs (adapter patterns)
- `crates/kay-core/NOTICE` — attribution template
- `Cargo.toml` (workspace), `deny.toml` (license gate)

**Files scanned:** ~35 source files + 4 test files + 2 configs
**Pattern extraction date:** 2026-04-20

---

## PATTERN MAPPING COMPLETE
