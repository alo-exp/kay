# Phase 5 Brainstorm — Agent Loop + Canonical CLI

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop (Event-Driven Core) + Canonical CLI rebrand
> **REQs closed:** LOOP-01, LOOP-02, LOOP-03, LOOP-04, LOOP-05, LOOP-06, CLI-01, CLI-03, CLI-04, CLI-05, CLI-07 (11 REQs)
> **Residuals closed:** R-1 (PTY tokenizer `[\s;|&]`), R-2 (image_read `max_image_bytes` cap)
> **Infra added:** trybuild compile-fail tier
> **Branch:** `phase/05-agent-loop` (from origin/main @ `1ae2a7f`)
> **Milestone:** v0.3.0 (Agent Loop + Canonical CLI)
> **Mode:** autonomous

---

## §Product-Lens — PM brainstorm

### Problem

ForgeCode is a top-10 harness (81.8% on TB 2.0) that is **locked** to its legacy `forge_main` interactive shell. We inherited 23 `forge_*` crates through the Phase 2.5 split, but the brand, the binary name, and the event contract are still "forge" — not "kay." More importantly, ForgeCode has **no public frozen event surface** and no pause/resume semantics, which blocks a desktop GUI (Tauri, Phase 9) and a TUI (ratatui, Phase 9.5) from ever being built on top of it.

Phase 5 resolves three overlapping problems simultaneously:

1. **Headless benchmark-capable CLI** — TB 2.0 submissions require a `kay run --prompt "…"` non-interactive invocation that returns a deterministic exit code (0 = success, 1 = task failure, 2 = sandbox violation, 3+ = crash). Right now `forge_main` boots an interactive REPL by default, which breaks CI harnesses.
2. **Frozen event API** — Without a stable `AgentEvent` surface marked `#[non_exhaustive]`, every frontend gets retroactively broken when we add a variant. We burned this lesson in Phase 2→3 (we had to add `ToolOutput`, `TaskComplete`, `ImageRead`, `SandboxViolation` after Phase 2 callers were already consuming the enum). Phase 5 DECLARES the contract done.
3. **Persona plumbing** — ForgeCode has three personas (`forge` = write, `sage` = research, `muse` = plan) hard-wired in code. Kay needs the same three served by **one code path + three YAML files**, so persona addition in future phases is config, not a code change.

### User value

| Persona            | Before Phase 5                                    | After Phase 5                                                                                                                                |
| ------------------ | ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| **TB 2.0 judge**   | Has to script around the interactive prompt       | Runs `kay run --prompt "solve task X" --headless --persona forge --events jsonl` and parses a clean JSONL event stream + deterministic exit  |
| **Frontend dev** (Phase 9 Tauri, 9.5 TUI) | Nothing to build against                         | Consumes `kay-cli --events jsonl` JSONL stream — the contract that GUI + TUI both bind to. No parallel agent-loop implementation needed.     |
| **OSS contributor**| Sees ForgeCode branding everywhere, "where's Kay?" | Runs `kay` and sees Kay's banner, help text, prompt, config — the inherited interactive affordances (completer, editor, stream-renderer, highlighter) are preserved behavior, but the brand is Kay. |
| **Power user**     | Cannot cancel a runaway turn without SIGKILL     | Presses Ctrl-C → abort flows cleanly through control channel; can pause (Ctrl-Z semantics), inspect, resume.                                   |

### Personas (users of this phase's output)

- **TB 2.0 harness operator** — Invokes `kay run --prompt …` in CI; cares about exit codes, JSONL stability, zero-interactive-prompt default, `--max-usd` budget enforcement, no stdout pollution from banners.
- **Tauri frontend builder (Phase 9)** — Imports `kay_core::AgentEvent` and `ipc::Channel<AgentEvent>`; needs the enum locked so `tauri-specta` v2 bindings don't drift between releases.
- **TUI builder (Phase 9.5)** — Spawns `kay-cli --events jsonl` as a child process and reads the JSONL stream; cares about schema stability across Kay versions (ideally one line = one event, utf-8, no interleaving).
- **Persona author (Phase 8+)** — Drops a `sage-legal.yaml` into `~/.kay/personas/` and gets a new research persona at `kay run --persona sage-legal` with zero code change. This is the extensibility story for the whole project.
- **Kay OSS developer (all phases)** — Runs `kay` interactively, expects: all the ForgeCode interactive features they saw on the parity baseline, but under Kay's branding.

### Success metrics (what makes this phase a win)

| Metric                                                         | Target                                                                                           |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `cargo install kay` yields a fully-working agent (CLI-06 prep) | Works over bare SSH with zero GUI/TUI deps                                                       |
| TB 2.0 parity gate (from Phase 1)                              | No regression vs `forgecode-parity-baseline` — ≥ 80% on TB 2.0                                   |
| Interactive regression check vs `forgecode-parity-baseline`    | 100% behavior parity on completer, editor, stream-renderer, highlighter, prompt rendering, banner swap only |
| Per-turn event stream latency (Phase 2 baseline)               | TextDelta → stdout JSONL ≤ 5ms (no buffering regression)                                         |
| Persona config load time                                       | < 50ms per persona YAML                                                                          |
| Ctrl-C abort cleanup                                           | < 500ms from signal → turn aborted + events flushed + exit 130                                   |
| `AgentEvent` enum variant count                                | 11 locked (Phase 2: 8 baseline + Phase 3: 3 additive + Phase 4: 1 SandboxViolation — NO new variants in Phase 5 body unless critical; additions only via the `#[non_exhaustive]` escape hatch for a future phase) |
| Structured JSONL schema tests                                  | Every `AgentEvent` variant has a JSONL serialize round-trip test locking the on-wire shape       |
| Codebase size at phase close                                   | `forge_main` → 0 usages in Kay entry points; replaced by `kay-cli` crate                         |

### Scope boundaries (what's IN, what's OUT)

**IN — Phase 5 delivers:**

- Event-driven turn loop (`tokio::select!` over input/model/tool/control channels) living in `kay-core::loop` (new module)
- `AgentEvent` hardening: `#[non_exhaustive]`, doc-comment frozen-contract marker, JSONL `Serialize` impl, wire-schema tests
- `persona.rs` — YAML loader + schema for `{system_prompt, tool_filter, model, name}` fields; bundled `forge.yaml`, `sage.yaml`, `muse.yaml`
- Control channel: `enum ControlMsg { Pause, Resume, Abort }`; tokio::mpsc; Ctrl-C → Abort; pause/resume are stateful (drains queued events on pause, holds until resume)
- `kay-cli` crate fully populated (it's a stub today with just `clap`, `anyhow`, `kay-tools` deps)
- `kay` binary with subcommands: `kay run --prompt ... --persona ... --headless --events <fmt> --max-usd <n>`; `kay` (no args) → interactive mode
- Forge_main → kay-cli migration: all interactive features (completer, editor, stream-renderer, syntax highlighter, banner, prompt) ported with brand strings swapped
- `task_complete` tool gates on `NoOpVerifier` (Phase 8 swap target) → `AgentEvent::TaskComplete { verified: false, outcome: Pending }`
- Sage as sub-tool: a `sage_query` tool invokable by `forge` and `muse` runs a bounded nested turn with sage's persona + read-only tool filter
- **Residual R-1 fix**: `execute_commands.rs` PTY-routing heuristic tokenizes command on `[\s;|&]` before matching engage-denylist; 6 regression tests
- **Residual R-2 fix**: `ForgeConfig.image_read.max_image_bytes` (default 20 MiB); `ImageReadTool::new` reads cap; rejects over-cap reads with structured `ToolError::ImageTooLarge` BEFORE allocating
- **Infra**: `trybuild` added to workspace dev-deps; `kay-tools/tests/compile_fail/` fixtures lock Tool/ServicesHandle object-safety + factory-closure signatures

**OUT — explicitly deferred:**

- Real verification critics (Phase 8)
- Session persistence across restart (Phase 6 — SESS-*)
- Context-engine retrieval (Phase 7 — CTX-*)
- Tauri GUI (Phase 9)
- TUI shell (Phase 9.5)
- Any new AI-provider integration
- Any new tool types (we have the Phase 3 tool set, plus `sage_query` as a nested-turn sub-tool inside the loop)
- Full standalone-distribution packaging (Phase 10 — TB-* / RELEASE-*)
- CLI-02 session import/export (Phase 6)
- CLI-06 `cargo install kay` distribution (Phase 10)

### Risks + mitigations

| Risk                                                                                     | Impact                                           | Mitigation                                                                                                                                                                                                                                                                                          |
| ---------------------------------------------------------------------------------------- | ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **QG-C4 breach** — `AgentEvent::SandboxViolation` re-injected into model message history | Prompt-injection / sandbox bypass                | Hard-enforced at turn loop: **event filter** splits the stream into two sinks — UI/stdout sink (gets everything) vs. model-context sink (filters SandboxViolation, ToolCallMalformed, internal-control events). Unit test: `loop_never_reinjects_sandbox_violation` with proptest. Secure-phase audit.            |
| Interactive-regression vs. ForgeCode parity baseline                                     | Users perceive Kay as "broken ForgeCode"         | Port `forge_main` modules file-by-file; snapshot-test the terminal output against `insta` fixtures captured from the `forgecode-parity-baseline` tag before swap; diff only on brand-string columns.                                                                                              |
| `AgentEvent` JSONL schema drift                                                          | Phase 9 Tauri + Phase 9.5 TUI break at release   | Serialize schema is locked by **snapshot tests** (insta) — one fixture per variant, diff-fail on any shape change. Schema document committed to `.planning/CONTRACT-AgentEvent.md`.                                                                                                                |
| `tokio::select!` channel-close deadlocks                                                  | Interactive hangs, TB 2.0 CI flakes              | Property-test the select branches: spawn 10k loops with random close-order of input/model/tool/control channels; assert no loop hangs > 1 s. Use `tokio::time::timeout` guard rails in the loop body itself.                                                                                        |
| YAML persona injection (malicious `tool_filter` grants an unsafe tool)                   | Sandbox bypass via mis-configured persona        | YAML loader validates `tool_filter` against the Phase 3 registry's tool name set at load time; unknown names → hard error. Persona loader is **never** given user-input paths by default — bundled personas live in-binary; external personas loaded from `~/.kay/personas/` require `--personas-dir` opt-in. |
| `forge_main` has TypeScript-like structure (clean-room concern from PROJECT.md)          | Clean-room attestation invalidated               | `forge_main` was already merged in Phase 0; any file whose structure looks TS-derived is flagged. All NEW code (`kay-cli` body, loop module, persona loader) written from scratch per ForgeCode semantics docs + our own design.                                                                  |
| Ctrl-C race — signal arrives during tool execution mid-write                             | Filesystem corruption, orphan child processes    | Abort = cooperative cancellation: set abort flag, wait for current tool to finish or hit its own abort check; orphan cleanup uses Phase 4 Job Object / process-group kill. Hard-kill only after 2s grace.                                                                                         |
| `forgecode-parity-baseline` comparison fails because ForgeCode updated upstream          | Phase 5 can't prove parity                       | Parity is frozen against the `forgecode-parity-baseline` git tag from Phase 1 (commit `022ecd994…`), not `main`. Tag is immutable. Any update to upstream ForgeCode is irrelevant to our parity gate.                                                                                              |
| Structured-event stream overwhelms stdout in a hot loop (10k events/s)                   | CI log files explode; stream consumers drop     | CLI-05 spec: default is unbuffered 1-event-per-line JSONL; `--events-buffer <n>` flag for batched emission if needed. Doc: each variant's wire cost in bytes at the committed schema.                                                                                                             |

### Key assumptions (to test during planning)

- `tokio::select!` with 4 channels (input, model_stream, tool_output, control) holds under high-fanout. Assumption from Phase 2 where we already use `tokio::select!` with 2 channels.
- `forge_main`'s interactive modules can be ported without their own agent-loop references — the assumption is that `forge_main` is a VIEW over `forge_services`, not a parallel engine. Verify during plan.
- The `forgecode-parity-baseline` tag captured interactive-mode snapshots — if it didn't, we need to go back to Phase 1 and add them retroactively. This is the biggest unknown going into planning.
- YAML persona files are a hard schema contract, not a template language. No Handlebars, no interpolation at load time. If we need templating later, it happens inside the persona-executing code path, not the loader.

### Secondary/adjacent ideas (parked — NOT in Phase 5)

- Persona inheritance (`extends: forge.yaml`) — parked for Phase 11 personality polish
- A `kay trace` subcommand that replays a JSONL event stream into interactive view — parked for Phase 6 (session export covers the input side)
- A `kay persona validate <path>` subcommand — parked for Phase 11

---

## §Engineering-Lens — Engineering brainstorm

### Current system state (inputs)

- `crates/kay-core/src/lib.rs` — today just a **re-exporter** of 6 forge_* crates (forge_api, forge_config, forge_domain, forge_json_repair, forge_repo, forge_services). **Phase 5 expands this: adds `loop`, `persona`, `control`, `event_filter` modules.**
- `crates/kay-cli/Cargo.toml` — stub with `clap`, `anyhow`, `kay-tools`. No src/. **Phase 5 populates this from scratch.**
- `crates/kay-tools/src/events.rs` — `AgentEvent` enum, 11 variants total (`#[non_exhaustive]` in place). **Phase 5 freezes:** adds doc comments labeling as frozen contract, adds `Serialize` derive (careful: `ProviderError` and `reqwest::Error` are not Serialize — need a wire-shape newtype for JSONL output), adds variant-per-test JSONL snapshot coverage.
- `crates/kay-tools/src/runtime/dispatcher.rs` — Phase 4 populated: registry lookup + `tool.invoke()`. **Phase 5 adds:** dispatcher is called FROM the loop; Phase 5 does not modify dispatcher itself.
- `crates/kay-tools/src/seams/{sandbox,rng,verifier}.rs` — Phase 3+4 populated. **Phase 5 uses:** verifier seam is where `NoOpVerifier` for LOOP-05 lives.
- `crates/forge_main/src/*` — 33 `.rs` files. **Phase 5 ports selectively:** cli.rs, ui.rs, prompt.rs, banner.rs, completer/, editor.rs, highlighter.rs, stream_renderer.rs, input.rs, state.rs. **Leaves behind:** forge_main module itself stays as a compat shim that re-exports from kay-cli for the transition tag, then retires in Phase 10.

### Architecture decision summary (E1-E12)

#### E1 — Where does the agent loop live?

**Decision:** `kay-core::loop::run_turn(…) -> impl Stream<AgentEvent>` in a new `crates/kay-core/src/loop.rs` (NOT in `forge_app` or `forge_services` — both are upstream crates in the parity fork).

**Why:** `kay-core` is the NEW top-of-DAG. `forge_app` has its own orchestration that we DON'T want to disturb (Phase 1 parity gate). We sit `kay-core::loop` *above* `forge_services` in the call graph and let it compose `forge_services` + `kay-tools` + `kay-provider-openrouter`.

**Alternatives ruled out:**

- A: Loop in `kay-tools` — rejected: `kay-tools` already imports `kay-provider-errors`; adding loop there creates a circular dep temptation.
- B: Loop in `forge_app` — rejected: parity gate.
- C: New `kay-loop` crate — rejected: split adds crate-graph noise for one module; if loop grows to >4 files we revisit.

#### E2 — Channel topology (LOOP-01)

**Decision:** Four `tokio::mpsc` channels driving one `tokio::select!`:

| Channel   | Type                           | Sender          | Receiver           |
| --------- | ------------------------------ | --------------- | ------------------ |
| `input`   | `mpsc::Receiver<UserMessage>`  | UI / CLI / test | loop               |
| `model`   | `mpsc::Receiver<ModelFrame>`   | provider stream | loop               |
| `tool`    | `mpsc::Receiver<ToolFrame>`    | tool dispatcher | loop               |
| `control` | `mpsc::Receiver<ControlMsg>`   | signal handler + UI | loop           |

**Why:** Matches LOOP-01 wording exactly ("event-driven loop via `tokio::select!` over input/model/tool/control channels"). Buffered with bound `32` each to backpressure model stream explosions.

**Alternatives ruled out:**

- `async_channel` vs `tokio::mpsc`: tokio wins because we're already tokio-based and `select!` ergonomics.
- Single unified `Event` enum channel: rejected — kills back-pressure per-source; one slow tool stalls model stream.

**Risk:** select!-bias (first branch gets priority). Mitigate via `biased;` label + explicit ordering: control > input > tool > model. Abort signals must outrun model chunks.

#### E3 — Control channel semantics (LOOP-06)

**Decision:** `enum ControlMsg { Pause, Resume, Abort }`:

- `Pause`: loop moves into `paused` state; continues draining channels into a `Vec` buffer until `Resume` or `Abort`; emits `AgentEvent::Paused` (**NEW variant** required — adds to the 11 count → 12; justify in discuss-phase that we accept this growth because pause is a Phase 5 deliverable and the variant is needed on-wire).
- `Resume`: replay buffered events to caller, resume normal select.
- `Abort`: (a) set `aborted: true`, (b) send cancel-token to in-flight tool call, (c) drain channels for max 2s, (d) emit `AgentEvent::Aborted { reason }` (**NEW variant**), (e) return from `run_turn`.
- Ctrl-C handler: installed in `kay-cli` main; SIGINT → `control_tx.send(ControlMsg::Abort).await`.

**Why two new variants:** Phase 9 GUI + Phase 9.5 TUI both need to display pause/resume/abort state — these are UI-facing lifecycle events. Adding them now (during the freeze phase) costs 0; adding them later forces all consumers to re-handle.

**Alternatives ruled out:**

- One `ControlResult { paused: bool, aborted: bool }` event: rejected — state semantics mixed with event semantics.
- Cancel via `tokio::spawn` + abort-handle: rejected — cooperative > pre-emptive for tool calls (FS operations mid-write are not safe to just abort).

**Open question → discuss-phase:** Does `Pause` flush model stream to a holding buffer (we'd accumulate cost-unaware model tokens) or actually `cancel` the provider stream? Lean: **cancel provider stream** on pause, re-send the accumulated conversation on resume. Defer answer to discuss-phase; it affects cost accounting.

#### E4 — Persona representation (LOOP-03)

**Decision:** YAML schema:

```yaml
name: "forge"                    # persona id
display_name: "Forge"            # UI string
model: "anthropic/claude-opus-4.7" # OpenRouter model id
system_prompt: |
  You are Forge, ForgeCode's…    # inherits from ForgeCode's catalog verbatim
tool_filter:                     # allowlist against registry
  - fs_read
  - fs_write
  - fs_search
  - execute_commands
  - image_read
  - net_fetch
  - task_complete
  - sage_query                   # sage-as-subtool; sage.yaml omits this entry
max_turns: 50
```

`crates/kay-core/src/persona.rs`: `struct Persona`, `fn load(name: &str) -> Result<Persona>`, `fn from_path(p: &Path) -> Result<Persona>`. `serde_yaml`. Bundled YAMLs in `crates/kay-core/personas/{forge,sage,muse}.yaml` via `include_str!`.

**Why:** Config-not-code gives us LOOP-03 without a code path fan-out per persona. Matches standing ForgeCode convention.

**Alternatives ruled out:**

- TOML: rejected — persona system_prompts are multi-line strings; YAML handles them cleanly.
- JSON: rejected — no comments, awkward for long prompts.
- Ron / KDL: rejected — niche, no reason to deviate.

#### E5 — Sage as sub-tool (LOOP-04)

**Decision:** A new `sage_query` tool (in `kay-tools/src/builtins/sage_query.rs`) implements `Tool`; on invocation, it spawns a **nested** `run_turn` with `persona = sage`, `tool_filter = [fs_read, fs_search, net_fetch]`, `input = args["query"]`, and bounded `max_turns = 5`. Collects the nested turn's `AgentEvent::TaskComplete` + content and returns it as `ToolOutput::text(summary)`.

**Why:** sage-as-tool lets forge/muse call sage as any other tool — uniform interface. Prevents "sage is its own subprocess" complexity.

**Risk:** infinite recursion if sage calls sage_query. Mitigate: pass `nesting_depth: u8` in `ToolCallContext`; `sage_query.invoke` rejects if `nesting_depth >= 2`.

#### E6 — AgentEvent wire format (CLI-05)

**Decision:** JSONL — one event per line, UTF-8, `\n`-separated. Schema:

```json
{"type": "TextDelta", "content": "…"}
{"type": "ToolCallStart", "id": "call_…", "name": "fs_read"}
{"type": "Usage", "prompt_tokens": 1234, "completion_tokens": 56, "cost_usd": 0.01}
{"type": "Paused"}
{"type": "Aborted", "reason": "user_ctrl_c"}
```

Implemented via a **wire-layer newtype** `AgentEventWire`, because the runtime `AgentEvent` holds `ProviderError` (not Serialize) and `reqwest::Error`. `AgentEventWire` is a mirror enum with every field serializable; `From<&AgentEvent> for AgentEventWire`.

**Why:** Preserves runtime type ergonomics (`ProviderError` for error handling) while giving JSONL a stable shape. Error variants map to `{"type": "Error", "message": "...", "kind": "rate_limit_exceeded" }` — bounded, lossy, deterministic.

**Snapshot-tested via insta:** one `.snap` per variant, committed to `crates/kay-cli/tests/snapshots/`. Any schema drift = test failure. Schema doc lives in `.planning/CONTRACT-AgentEvent.md`.

#### E7 — kay-cli shape

**Decision:** `clap` derives; subcommands:

```
kay                            # interactive (default)
kay run --prompt <s>           # headless single-turn
  --persona <forge|sage|muse>  # default: forge
  --events <human|jsonl>       # default: human (stderr for logs, stdout for content); jsonl = all events on stdout
  --headless                   # alias: non-interactive + no banner + no prompt + exit after TurnEnd
  --max-usd <f>                # budget cap
  --max-turns <u32>            # turn cap
kay session list               # Phase 6 — stub in Phase 5, returns "not yet implemented" (CLI-02 deferred)
kay tui                        # Phase 9.5 — stub
kay --version / --help         # clap defaults
```

**Exit codes (CLI-03):**

| Code | Meaning                                                 |
| ---- | ------------------------------------------------------- |
| 0    | TurnEnd with `task_complete` verified (Phase 5: Pending — loops until max_turns or task_complete) |
| 1    | TurnEnd without task_complete (max_turns reached) OR model error |
| 2    | SandboxViolation emitted (distinct exit signals sandbox bypass attempt) |
| 3    | Config error (bad YAML, missing persona, etc.)         |
| 130  | SIGINT (ctrl-c aborted)                                 |

#### E8 — Forge_main → kay-cli port strategy

**Decision:** Port file-by-file in order of dependency: (1) banner.rs (trivial — brand strings), (2) prompt.rs + completer/ (input UI), (3) editor.rs (reedline wrapper), (4) stream_renderer.rs + highlighter.rs (output UI), (5) state.rs + ui.rs (session state), (6) cli.rs → kay-cli::cli. Each port commits atomically with its own snapshot-test pass. `forge_main` retained as a thin re-export shim through Phase 5 ship; deletion deferred to Phase 10.

**Risk:** `forge_main` is 33 files; porting everything is Phase-10-scope. Phase 5 scope is **entry-point-level rebrand** — the subcommand surface + banner + prompt + help text. Internal modules (porcelain.rs, tools_display.rs, vscode.rs, zsh/*, oauth_callback.rs) are Kay-ready under ForgeCode-parity semantics and stay put until Phase 11.

**Scope line:** What IS rebranded in Phase 5 = the user-visible surface (binary name, banner, help text, prompt string, error messages, version). What IS NOT yet = internal module names, crate names of the forge_* sub-crates (they stay `forge_*` — parity requirement), package author fields.

#### E9 — R-1 fix shape (residual close-out)

**Decision:** In `crates/kay-tools/src/builtins/execute_commands.rs`, replace the current naive `if first_token_of_cmd.starts_with(...)` check with:

```rust
fn should_use_pty(cmd: &str) -> bool {
    // Tokenize on [\s;|&] — engage-denylist check is per-token
    let tokens: Vec<&str> = cmd.split(|c: char| c.is_whitespace() || c == ';' || c == '|' || c == '&').filter(|s| !s.is_empty()).collect();
    tokens.iter().any(|t| ENGAGE_DENYLIST.contains(t))
}
```

6 regression tests:

1. `ssh user@host echo owned` → PTY (bare)
2. `ssh;echo owned` → PTY (semicolon)
3. `false | ssh host` → PTY (pipe)
4. `ssh host && echo` → PTY (and)
5. `echo & ssh host` → PTY (background-then-interactive)
6. `ssh_hello_world` → piped (prefix match bug guard — `ssh_hello` is NOT `ssh`)

#### E10 — R-2 fix shape (residual close-out)

**Decision:** `ForgeConfig.image_read.max_image_bytes: u64` (default `20 * 1024 * 1024`). `ImageReadTool::new(cfg: &ForgeConfig)` stores the cap. `invoke` calls `std::fs::metadata(path).len()`; if `> cap`, returns `ToolError::ImageTooLarge { path, actual_bytes, cap_bytes }` BEFORE `std::fs::read`. Serializes cleanly to wire.

**Why BEFORE read:** Allocating 500MB just to reject it is the DoS vector R-2 flagged. Size check via metadata is O(1).

#### E11 — trybuild infra

**Decision:** `trybuild = "1"` to workspace dev-dependencies. `crates/kay-tools/tests/compile_fail/` gets three fixtures:

1. `tool_not_object_safe.rs` — attempts `Box<dyn Tool>` with a non-object-safe generic method → compile-fail
2. `services_handle_not_object_safe.rs` — same for `ServicesHandle`
3. `default_tool_set_sig_change.rs` — imports `default_tool_set` with the wrong factory-closure signature

Snapshot-matched stderr via trybuild's native mechanism. Not snapshot-tested via insta — trybuild has its own format.

#### E12 — Event filter for QG-C4

**Decision:** `kay-core::event_filter::ModelContextFilter`:

```rust
pub fn for_model_context(ev: &AgentEvent) -> bool {
    !matches!(ev,
        | AgentEvent::SandboxViolation { .. }
        | AgentEvent::ToolCallMalformed { .. }  // also prompt-injection-grade (model sees raw bytes if re-injected)
        | AgentEvent::Paused
        | AgentEvent::Aborted { .. }
        | AgentEvent::Retry { .. }              // model already saw the underlying cause
    )
}
```

Agent loop calls `event_filter::for_model_context(&ev)` before pushing any event into the message-history vec that becomes the next turn's provider input. UI/stdout sink receives ALL events unfiltered.

Property-test: 10k randomly generated `AgentEvent` sequences → assert for_model_context blocks all SandboxViolation events in all positions.

### Testing architecture pointer (full details in 05-TEST-STRATEGY.md)

- **Unit**: loop channel-close resilience; persona YAML deserialization; R-1 tokenizer; R-2 size check; event_filter.
- **Integration**: `kay run --prompt "echo hi" --headless` end-to-end on all 3 OSes with mock provider.
- **Contract (NEW)**: `AgentEvent` insta snapshot per variant locking wire JSONL shape.
- **Property**: tokio::select! with random channel-close orderings; event_filter with random variant sequences.
- **Compile-fail (NEW via trybuild)**: object-safety + factory-closure signature.
- **Parity (NEW)**: `forgecode-parity-baseline` tag snapshot replay for interactive mode.

### Cross-phase dependencies (what downstream breaks if we get this wrong)

| Downstream consumer           | What it needs from Phase 5                                             | Break mode if we get it wrong                      |
| ----------------------------- | ---------------------------------------------------------------------- | -------------------------------------------------- |
| Phase 6 (Session Store)        | AgentEvent JSONL serialization                                         | Transcript format churn; migration pain            |
| Phase 7 (Context Engine)       | Loop hook-point for context retrieval (pre-provider-call callback)     | Rewriting loop to add retrieval callback           |
| Phase 8 (Critics)              | `NoOpVerifier` seam + `task_complete` gate                             | Retrofitting verification into finalized loop API  |
| Phase 9 (Tauri)                | `#[non_exhaustive]` AgentEvent + `ipc::Channel<AgentEvent>` consumption | Breaking UI on AgentEvent variant add              |
| Phase 9.5 (TUI)                | JSONL stream stability via `kay-cli --events jsonl`                    | TUI rebuild if schema drifts                       |

### Engineering brainstorm — summary decisions

- New `kay-core::loop` module owns `run_turn`
- 4-channel `tokio::select!` + biased priority: control > input > tool > model
- `ControlMsg{Pause,Resume,Abort}`; abort cooperative with 2s grace
- YAML personas; schema lock'd at load; bundled via `include_str!`
- `sage_query` as sub-tool with `nesting_depth` guard
- Wire layer: `AgentEventWire` for JSONL; insta snapshots lock the schema
- QG-C4 enforced via `event_filter::for_model_context`
- R-1 fix: tokenized denylist check, 6 regression tests
- R-2 fix: metadata size check before allocation
- trybuild added; compile-fail fixtures for object-safety

### Open questions for discuss-phase (Step 4)

1. **Pause semantics**: cancel provider stream on pause (re-send full history on resume) vs. holding buffer (accumulated cost)? Lean: cancel + re-send.
2. **Do we grow AgentEvent in Phase 5?** Adding `Paused` + `Aborted` takes variant count 11 → 13. Accept with justification, or fold pause/abort into existing `Retry`-style variant? Lean: **yes, grow** — adding now is free; retrofitting later breaks consumers.
3. **Persona directory location**: `~/.kay/personas/` is fixed; external load via `--personas-dir <dir>` flag. Bundled (in-binary) always available. Confirm no `$KAY_PERSONAS_DIR` env var (discourage env-var configuration for security surface).
4. **forge_main retention**: keep as re-export shim through end of Phase 10, or retire immediately at Phase 5 ship? Lean: keep shim through Phase 10 so `forgecode-parity-baseline`-era CI scripts still work.
5. **Interactive parity scope**: is 100% parity against the `forgecode-parity-baseline` tag achievable in Phase 5, or should we scope to "banner + prompt + help text + exit codes swap" and defer full UI parity to Phase 11? Lean: scope to entry-point-surface swap in Phase 5; full module port in Phase 11.

### Stored-but-parked ideas (do NOT implement in Phase 5)

- JSONL compression (gzip stream for long sessions) — YAGNI
- WebSocket event stream (in addition to JSONL) — wait for Phase 9
- Persona versioning (`schema_version: 1` field in YAML) — add if we break schema later
- Cost accounting at control-msg granularity — Phase 10's business
- `--events ndjson` variant name (same format, clearer name) — bike-shed; defer

---

## Brainstorm sign-off

- [x] Problem framed
- [x] Users identified
- [x] Success metrics quantified
- [x] Scope boundaries locked (IN / OUT / parked)
- [x] Top 9 risks mitigated
- [x] E1-E12 engineering decisions captured
- [x] Open questions flagged for discuss-phase
- [x] Cross-phase ripple documented

**Next step:** `/testing-strategy` (Step 2) → `.planning/phases/05-agent-loop/05-TEST-STRATEGY.md`.
