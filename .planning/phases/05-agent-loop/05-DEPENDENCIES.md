# Phase 5 Dependencies — Planner Input

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop + Canonical CLI
> **Mode:** autonomous (§10e) — inline execution
> **Skill:** `gsd-analyze-dependencies`
> **Purpose:** Map every dependency (cross-phase, intra-phase, crate, external, CI) that `gsd-plan-phase` must honor.

---

## 1. Cross-phase dependencies (upstream)

Phase 5 inherits from prior phases. Each dependency item lists the exact asset consumed and the contract it exposes.

### Phase 2 (Provider HAL + OpenRouter streaming) — PROV-01..PROV-05

| Asset consumed | Contract | Wave where consumed |
| -------------- | -------- | ------------------- |
| `forge_api::ForgeApi` via `kay-core::forge_api` re-export | Async streaming provider interface | Wave 4 (loop invokes model calls via ServicesHandle) |
| `kay-provider-openrouter` → error types | `ProviderError` surfaces through `kay-provider-errors` into `AgentEvent::Error` | Wave 1 (wire snapshot must preserve ProviderError-to-Error mapping) |
| Retry backoff + `AgentEvent::Retry` | Already emitted by Phase 2 layer; Phase 5 loop is passthrough | Wave 1 (insta snapshot) + Wave 4 (pass-through) |

**No modifications to Phase 2 crates allowed.**

### Phase 3 (Tool Registry + KIRA Core Tools) — TOOL-01..TOOL-08

| Asset consumed | Contract | Wave where consumed |
| -------------- | -------- | ------------------- |
| `kay_tools::Tool` trait | Object-safe, async invoke(args, ctx, call_id) | Wave 4 (loop), Wave 5 (sage_query impl) |
| `kay_tools::runtime::dispatcher::dispatch(registry, tool_name, args, ctx, call_id) -> Result<ToolOutput, ToolError>` | Already-built seam the loop wraps | Wave 4 |
| `kay_tools::events::AgentEvent` enum + `#[non_exhaustive]` | Extended in Wave 1 with `Paused` and `Aborted` variants (in-phase modification per DL-4) | Wave 1 |
| `kay_tools::events::ToolOutputChunk` | Unchanged; #[non_exhaustive] preserved | Wave 1 (snapshot only) |
| `kay_tools::ToolError` | Extended in Wave 6b with `ImageTooLarge { path, actual_size, cap }` variant | Wave 6b |
| `kay_tools::seams::rng::{OsRngSeam, DeterministicRng}` | Used for property-test determinism + production entropy | Waves 1, 4 (property tests) |
| `kay_tools::runtime::ToolCallContext` | Extended in Wave 5 with `nesting_depth: u8` field | Wave 5 |
| `kay_tools::builtins::execute_commands::should_use_pty` | Modified in Wave 6a to tokenize on `[\s;|&]` | Wave 6a |
| `kay_tools::builtins::image_read::ImageReadTool` | Modified in Wave 6b to enforce `max_image_bytes` cap | Wave 6b |

**Allowed modifications (all in Phase 5 scope):**
- `crates/kay-tools/src/events.rs` — add 2 variants to `AgentEvent` (+ wire mirror)
- `crates/kay-tools/src/events.rs` — add `ImageTooLarge` variant to `ToolError`
- `crates/kay-tools/src/runtime/dispatcher.rs` — NEW `ToolCallContext.nesting_depth` field
- `crates/kay-tools/src/builtins/execute_commands.rs` — PTY tokenizer fix (R-1)
- `crates/kay-tools/src/builtins/image_read.rs` — metadata-size cap (R-2)
- `crates/kay-tools/src/builtins/sage_query.rs` — NEW module (Wave 5)
- `crates/kay-tools/src/events_wire.rs` — NEW module (Wave 1)

**No modifications to other Phase 3 modules allowed** (registry, seams/sandbox, marker polling, other builtins).

### Phase 4 (Sandbox — All Three Platforms) — SAND-01..SAND-10

| Asset consumed | Contract | Wave where consumed |
| -------------- | -------- | ------------------- |
| `kay_tools::seams::sandbox::Sandbox` trait (Phase 3 established, Phase 4 populated) | `async fn check_{shell,fs_read,fs_write,fs_search,net}` | Wave 4 (loop's dispatcher path) |
| `kay_sandbox_policy::SandboxPolicy` serde struct | Loaded via persona YAML? NO — persona defines tool access, not sandbox policy. Policy still comes from config. | Wave 4 (unchanged) |
| `AgentEvent::SandboxViolation { call_id, tool_name, resource, policy_rule, os_error }` | Emitted by sandbox seam (Phase 4); INTERCEPTED in Wave 2 event_filter | Wave 2 (QG-C4) |

**No modifications to Phase 4 sandbox crates allowed** — frozen territory.

### Phase 2.5 (kay-core sub-crate split)

| Asset consumed | Contract |
| -------------- | -------- |
| `kay_core` as aggregator re-exporter of `forge_{api,config,domain,json_repair,repo,services}` | Existing 6 re-exports PRESERVED; Phase 5 adds 4 NEW modules alongside |

---

## 2. Intra-phase dependencies (wave DAG)

This refines the 7-wave DAG from IMPL-OUTLINE with explicit artifact-level dependencies.

### 2.1 Module-level DAG

```
                                  ┌────────────────────────────────────┐
                                  │ kay-tools::events::AgentEvent       │
                                  │  (existing 11 variants + new 2)     │
                                  └───────┬────────────────────────────┘
                                          │ depended-on-by
                          ┌───────────────┼───────────────┬──────────────┐
                          ▼               ▼               ▼              ▼
         kay-tools::events_wire    kay-core::event_filter    kay-core::control    kay-core::loop
          (Wave 1; 16 snapshots)    (Wave 2; 100% cov)        (Wave 2)             (Wave 4)
                                                                  │                    │
                                                                  ▼                    ▼
                                                             ControlMsg    run_turn(services, verifier,
                                                              mpsc           control_rx, cancel_token)
                                                                                       ▲
                                                          kay-core::persona ───────────┤
                                                          (Wave 3; YAML serde)          │
                                                                                       │
                                                          kay-tools::builtins ──────────┤
                                                          ::sage_query                  │
                                                          (Wave 5; depends on loop)     │
                                                                                       │
                                                          kay-cli (Wave 7) ────────────┘
                                                          (imports ALL above)
```

**Key property:** no cycles. `kay-tools` does NOT import `kay-core`. `kay-cli` imports both but neither imports `kay-cli`.

### 2.2 Wave-level DAG (refined from IMPL-OUTLINE)

```
Wave 1 (events_wire + 2 new AgentEvent variants + 16 insta snapshots)
  │
  ├──────────────────────────────┐       ┌──────────────────────────────┐
  ▼                              ▼       ▼                              │
Wave 2                         Wave 3                                Wave 6 (R-1, R-2, trybuild)
(event_filter                  (persona                              (INDEPENDENT — can run
 + control)                     loader)                               anytime after Wave 1)
  │                              │
  └──────────┬──────────────────┘
             ▼
          Wave 4 (agent loop skeleton)
             │
             ├───────────────┐
             ▼               ▼
          Wave 5          [Wave 7 waits for ALL]
          (sage_query)
             │
             ▼
          Wave 7 (kay-cli + forge_main port — integration wave)
```

**Parallelizable sets (planner can dispatch to parallel sub-agents in Step 7):**
- {Wave 2, Wave 3, Wave 6} after Wave 1 completes
- {Wave 4, Wave 5} after Waves 2+3 complete (Wave 5 depends on Wave 4 Step 4 — loop API stable)

**Serial points:** Wave 1 is the contract lock — every downstream wave imports `AgentEventWire`. Wave 7 is the integration — every downstream consumer built.

### 2.3 Test fixture dependencies

| Fixture | Created in wave | Consumed by |
| ------- | --------------- | ----------- |
| 16 insta `.snap` files for AgentEvent | Wave 1 | Regression lock from Wave 1 onward |
| `crates/kay-core/personas/{forge,sage,muse}.yaml` | Wave 3 | Wave 4 (loop loads), Wave 5 (sage_query regression asserts on sage's tool_filter), Wave 7 (kay-cli default persona resolution) |
| `tests/fixtures/forgecode-banner.txt` | Wave 7 (from `git checkout forgecode-parity-baseline -- crates/forge_main/src/banner.rs`) | Wave 7 parity diff test |
| `tests/fixtures/forgecode-prompt.txt` | Wave 7 (same origin) | Wave 7 parity diff test |
| 3 trybuild compile-fail fixtures under `crates/kay-tools/tests/compile_fail/` | Wave 6c | Wave 6c runner |
| `.planning/CONTRACT-AgentEvent.md` | Wave 1 | Human-readable reference; GUI (Phase 9) + TUI (Phase 9.5) consumers |

---

## 3. External crate dependencies

### 3.1 Already in workspace (NO add required)

Inherited from Phase 2/3/4 (confirmed present in `Cargo.toml` `[workspace.dependencies]`):

| Crate | Version | Usage in Phase 5 |
| ----- | ------- | ---------------- |
| `tokio` | 1.51 (features: rt-multi-thread, macros, io-util, process, signal, sync, fs, time) | Wave 2 mpsc, Wave 4 select!, Wave 7 runtime — ALL features already enabled |
| `tokio-stream` | 0.1 | Wave 4 (already included in kay-tools) |
| `tokio-util` | 0.7 (features: rt) | Wave 2 cancel_token (`CancellationToken`) |
| `futures` | 0.3 | Wave 4 select!, Wave 7 stream processing |
| `async-trait` | 0.1 | Wave 3 persona loader (Future-based), Wave 4 Verifier trait |
| `serde` | 1 (features: derive) | Wave 1 wire, Wave 3 persona |
| `serde_json` | 1 | Wave 1 JSONL emission, Wave 7 CLI event stream |
| `serde_yml` | 0.0.12 | Wave 3 persona YAML loader (NOTE: NOT `serde_yaml` — workspace uses `serde_yml`, the maintained fork) |
| `clap` | 4.5 (features: derive, env) | Wave 7 kay-cli subcommands |
| `thiserror` | 2 | Waves 1, 2, 3 error types |
| `anyhow` | 1 | Wave 4 loop error fallback |
| `tracing` | 0.1 | All waves (structured logging) |
| `tracing-subscriber` | 0.3 (features: env-filter, json) | Wave 7 (CLI initializer) |
| `rand` | 0.10.0 | Wave 4 (PRNG for property tests via DeterministicRng seam) |
| `proptest` | 1 (already in kay-tools dev-deps) | Wave 2 property test, Wave 4 property test |
| `trybuild` | 1.0 (already in kay-tools dev-deps) | Wave 6c compile-fail tier |
| `insta` | 1 (features: yaml, json) | Wave 1 snapshot tests |
| `pretty_assertions` | 1 | Wave 1-7 test asserts |
| `tempfile` | 3 | Wave 3 (external YAML load test), Wave 6a/6b (filesystem tests) |

### 3.2 NEW dev-deps to add in workspace

These MUST be added to `[workspace.dependencies]` in root `Cargo.toml` via a Wave 7 pre-task commit (before Wave 7 implementation tasks):

| Crate | Version | Reason | Consumer wave |
| ----- | ------- | ------ | ------------- |
| `assert_cmd` | `"2"` | End-to-end CLI process-spawning tests | Wave 7 T-5 |
| `predicates` | `"3"` | Assertions on stdout/stderr (companion to assert_cmd) | Wave 7 T-5 |

**Rationale:** Phase 5 is the first phase to build a CLI binary that runs end-to-end. Earlier phases tested via in-process calls; Wave 7's 9 T-5 cases spawn the `kay` binary as a subprocess and assert exit codes + JSONL output. `assert_cmd` + `predicates` is the standard Rust pairing for this.

**Add location:** `[workspace.dependencies]` section of root `Cargo.toml`, grouped under a new `# Phase 5 (kay-cli) additions` comment block.

### 3.3 NEW runtime deps for kay-cli (Cargo.toml modifications)

Wave 7 populates `crates/kay-cli/Cargo.toml`. Additions:

```toml
# Phase 5 additions to kay-cli/Cargo.toml [dependencies] block
kay-core         = { path = "../kay-core" }
# kay-tools already declared (current skeleton)
tokio            = { workspace = true }
tokio-stream     = { workspace = true }
futures          = { workspace = true }
serde            = { workspace = true }
serde_json       = { workspace = true }
serde_yml        = { workspace = true }
tracing          = { workspace = true }
tracing-subscriber = { workspace = true }
thiserror        = { workspace = true }

# Phase 5 additions to kay-cli/Cargo.toml [dev-dependencies]
assert_cmd       = { workspace = true }
predicates       = { workspace = true }
tempfile         = { workspace = true }
insta            = { workspace = true }
pretty_assertions = { workspace = true }
```

### 3.4 NEW runtime deps for kay-core (Cargo.toml modifications)

Wave 1 (for event_filter prep) + Wave 3 (persona) + Wave 4 (loop) populate kay-core. Additions:

```toml
# Phase 5 additions to kay-core/Cargo.toml [dependencies] block
kay-tools        = { path = "../kay-tools" }  # needed for AgentEvent match in event_filter
tokio            = { workspace = true }       # mpsc for control channel
tokio-util       = { workspace = true }       # CancellationToken
futures          = { workspace = true }       # select! machinery
async-trait      = { workspace = true }       # Verifier trait
serde            = { workspace = true }       # persona deserialization
serde_yml        = { workspace = true }       # persona YAML loader
tracing          = { workspace = true }       # structured logging
thiserror        = { workspace = true }       # persona + loop error types

# Phase 5 additions to kay-core/Cargo.toml [dev-dependencies]
pretty_assertions = { workspace = true }
tempfile         = { workspace = true }
proptest         = "1"
insta            = { workspace = true }
```

---

## 4. CI dependencies

### 4.1 Existing CI workflow (inherited from Phase 4)

File: `.github/workflows/ci.yml`

| Job | Purpose | OS matrix |
| --- | ------- | --------- |
| test | `cargo test --workspace` | macos-14, ubuntu-latest, windows-latest |
| fmt | `cargo fmt --check` | ubuntu-latest |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ubuntu-latest |
| dco | Inline bash per-commit loop (Phase 4 fix for tim-actions/dco ARG_MAX crash) | ubuntu-latest |

### 4.2 Phase 5 additions to CI

Wave 7 pre-task commit augments `.github/workflows/ci.yml` with:

| New job | Purpose | OS matrix |
| ------- | ------- | --------- |
| coverage-event-filter | Enforce QG-C4 coverage threshold (100% line + 100% branch on `kay-core::event_filter`) via `cargo-llvm-cov --fail-under-lines 100 --fail-under-branches 100` restricted to that crate+module | ubuntu-latest |

**SHIP BLOCK contract:** if the coverage job fails, the PR CANNOT merge. The job must run on every PR targeting `main` + `phase/*` branches.

### 4.3 Signal-test platform gating

Wave 4 + Wave 7 have Ctrl-C tests that behave differently per OS:

| Test | macOS | Linux | Windows |
| ---- | ----- | ----- | ------- |
| T-3 Ctrl-C control-channel cooperative abort | ✅ (SIGINT via nix) | ✅ (SIGINT via nix) | ⚠️ Gated `#[cfg(not(windows))]` — Windows uses `GenerateConsoleCtrlEvent` in a separate test |
| T-5 E2E exit-130 on Ctrl-C | ✅ (libc::kill + SIGINT) | ✅ | ⚠️ Gated `#[cfg(not(windows))]` — Windows path documented but not executed in CI (complex to spawn + CTRL_C safely) |

**Decision (locked):** Phase 5 CI does NOT execute Ctrl-C tests on Windows runners. Documented in TEST-STRATEGY §CI matrix. Windows SIGINT coverage is a Phase 9 backlog item (when Tauri shell adds a proper Windows signal handler).

---

## 5. Deferred dependencies (out-of-scope for Phase 5)

These are visible dependencies that do NOT gate Phase 5:

| Dependency | Why deferred | Target phase |
| ---------- | ------------ | ------------ |
| Real `Verifier` implementation (not NoOp) | VERIFY-01..VERIFY-05 require critic harness | Phase 8 |
| Session store / persistence | SESS-01..SESS-08 | Phase 6 |
| Context retrieval / RAG | CTX-01..CTX-05 | Phase 7 |
| Tauri GUI that consumes JSONL | GUI-* | Phase 9 |
| TUI that consumes JSONL | TUI-* | Phase 9.5 |
| `cargo install kay` | REL-* | Phase 10 |
| Internal `forge_main` rebrand | Deferred per DL-3 | Phase 10 |
| `--events-buffer <N>` flag | Deferred per DL-5 (measurement only in Phase 5) | Phase 6+ if DoS observed |

---

## 6. Dependency invariants the planner must enforce

| Invariant | Enforcement mechanism |
| --------- | --------------------- |
| No circular crate imports | Wave-level DAG check + `cargo check --workspace` on every wave exit |
| Phase 4 sandbox crates not modified | Pre-commit scan: `git diff --name-only $PHASE_BASE | grep -v '^crates/kay-sandbox'` must be empty, or `git diff crates/kay-sandbox-*` is empty |
| Phase 2 provider crates not modified | Same pattern on `kay-provider-*` and `forge_api`/`forge_services`/`forge_config` (except allow-listed kay-tools modifications in §1.2 table) |
| `kay-tools` does NOT import `kay-core` | `cargo metadata --format-version 1` check: `kay-tools` dependencies must exclude `kay-core` |
| AgentEvent variant count exactly 13 | Insta snapshot count = 16 (matches DL-4 math); `cargo expand` check on `kay-tools::events::AgentEvent` |
| event_filter coverage = 100% line + 100% branch | CI job `coverage-event-filter` (§4.2) |
| DCO on every commit | CI job `dco` (already in workflow from Phase 4) |
| trybuild fixtures exist for 3 object-safety constraints | CI test run includes `compile_fail.rs` runner |

---

## 7. Upstream git state at phase start (anchor)

- **Branch:** `phase/05-agent-loop` (created from origin/main @ `1ae2a7f`)
- **Base commit:** `1ae2a7f` (Phase 4 squash merge)
- **Parity baseline tag:** `forgecode-parity-baseline` @ `9985d77` (used for Wave 7 parity fixtures)
- **Latest release tag:** `v0.2.0` (Phase 4 ship; ED25519-signed)
- **Target release tag:** `v0.3.0` (Phase 5 ship; will be ED25519-signed at closure)

---

## 8. Summary for planner

**Inputs planner must consume:**
1. This `05-DEPENDENCIES.md` (dependency map)
2. `05-CONTEXT.md` (locked decisions DL-1..DL-7)
3. `05-IMPL-OUTLINE.md` (7-wave skeleton)
4. `05-TEST-STRATEGY.md` (11 test suites)
5. `05-QUALITY-GATES.md` (7 carry-forward enforcement constraints)
6. `05-BRAINSTORM.md` (E1-E12 engineering decisions; §Product-Lens personas/metrics)
7. `05-VALIDATION.md` (pre-build gate result: 0 BLOCK)

**Outputs planner produces:**
- `05-PLAN.md` with atomic tasks mapped to waves, each task specifying:
  - Exact file path(s) to create/modify
  - Exact test name(s) (RED → GREEN pairing)
  - Exact commit message (DCO-signed)
  - REQ-ID back-reference
  - Wave assignment (1-7)

**Critical planner constraints (from DL-* + invariants):**
- First commit of Wave 7 MUST fix REQUIREMENTS.md traceability table (CLI-04/05/07 rows) — DL-6
- First commit of Wave 1 MUST add tokio/serde/serde_yml/etc. deps to `kay-core/Cargo.toml` — §3.4
- First commit of Wave 7 MUST populate `kay-cli/Cargo.toml` with new deps + add `assert_cmd`/`predicates` to workspace — §3.2 + §3.3
- First commit of Wave 6c MUST pre-exist trybuild fixtures (trybuild already a dev-dep)
- Coverage CI job must land BEFORE Wave 2 GREEN merges (so coverage assertion can be exercised)

---

**Next step:** Step 6 `gsd-plan-phase` → `05-PLAN.md` (atomic task breakdown).
