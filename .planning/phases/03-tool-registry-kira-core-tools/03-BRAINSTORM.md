---
phase: 3
slug: tool-registry-kira-core-tools
created: 2026-04-20
status: draft
composed-by: /silver:feature FLOW 3 (product-brainstorm + superpowers:brainstorming)
---

# Phase 3 — Brainstorm

> Dual-lens brainstorm per `/silver:feature` FLOW 3. §Product-Lens captures PM thinking
> (problem, users, metrics, scope). §Engineering-Lens (FLOW 3b, added after this section
> lands) captures engineering architecture + risk. Both inform the plan revision in FLOW 9.

---

## §Product-Lens  *(FLOW 3a — product-brainstorming)*

### 1. Frame

**What are we exploring?** Phase 3 delivers the tool-execution surface that the Phase 5 agent loop drives and the Phase 9 Tauri UI renders. Seven tools: KIRA trio (`execute_commands`, `task_complete`, `image_read`) + parity minimum (`fs_read`, `fs_write`, `fs_search`, `net_fetch`).

**Why now?** Phase 2 closed the provider HAL + JSON repair. Without a tool surface, the agent loop in Phase 5 has nothing to call. Every downstream phase (5, 6, 8, 9, 12) depends on this contract being stable and well-tested.

**What do we already know?**
- Terminus-KIRA's writeup attributes ≥6 pp of TB 2.0 lift specifically to the marker protocol + harness discipline.
- ForgeCode's `enforce_strict_schema` is load-bearing for tool-calling reliability (required-first, `additionalProperties: false`, flattened `allOf`).
- Prompt-injected `__CMDEND__` markers have been demonstrated to corrupt naïve harnesses (Terminus-KIRA §Pitfall 4).
- The NoOpVerifier pattern is how OpenCode handled its Phase 3/Phase 7 decoupling in their pre-Gemini refactor.

**Constraints:**
- Parity with ForgeCode at the tool-delegation layer (TB 2.0 baseline must not regress > 2pp).
- Clean-room attestation: no TypeScript-derived structure from the v2.1.88 leak.
- Apache-2.0 license — no GPL-linked shells, no openssl.
- 100% TDD + full test pyramid (user directive 2026-04-20).

**Great outcome definition:** All 5 ROADMAP success criteria green + zero plan-checker BLOCKERS + smoke-proof that `kay-cli exec -- echo hi` streams `ToolOutput` frames + PTY fallback actually works on macOS + `task_complete` returns `Pending` end-to-end.

---

### 2. Personas + Jobs-to-be-Done

| Persona | Context | JTBD | Expected outcome |
|---------|---------|------|------------------|
| **Kay CLI power user** (devs running `kay` in terminal for TB-like tasks) | Long-running shell commands (builds, test runs, interactive TUIs like `htop`) | "When a command is long-running or interactive, I want to see output stream as it happens so I can abort or intervene" | Live stdout frames, PTY fallback for TUIs, timeout safety |
| **Kay desktop-app user** (Phase 9 Tauri UI) | Reviewing agent tool-call timeline | "When the agent runs a command, I want to see the command, its output, and completion status in a rich UI" | Structured `AgentEvent::ToolOutput` / `TaskComplete` frames that the UI can render |
| **TB 2.0 evaluator** (benchmark harness) | Running the published eval on a held-out task set | "When evaluating the harness, I want reproducible completion-detection semantics so scores are deterministic" | Marker-protocol based detection, unforgeable under prompt-injection |
| **Kay contributor** (writing a new tool) | Adding a tool in Phase 5+ | "When I add a new tool, I want a clear trait contract and delegation pattern so I don't regress parity" | Object-safe `Tool` trait + documented delegation to `forge_app::ToolExecutor::execute` |
| **Security auditor** (Phase 10 handoff) | Reviewing for CVE / supply-chain risk | "When I audit this phase, I want to see the sandbox seam, timeout cascade, and marker protocol formally tested" | `NoOpSandbox` DI seam, SIGTERM→SIGKILL cascade, `subtle::ConstantTimeEq` on nonce |

**Anti-persona (explicitly not served):** users expecting MCP tool registration at runtime; users wanting custom tool plugins; users wanting Gemini-style tool schemas. These are deferred (D-10, §Deferred Ideas).

---

### 3. Problem Decomposition (first principles)

The Phase 3 contract has four separable problems that are often conflated:

1. **Tool dispatch** — how does the agent loop find a tool by name and invoke it?
2. **Schema emission** — how does the provider serialize tool contracts to the wire?
3. **Command execution** — how does `execute_commands` actually run a shell command reliably?
4. **Output streaming** — how does streamed output reach the UI / TUI / CLI?

Plan-checker BLOCKERS #1, #6, #7 are all *Problem 1* confusion (trait signature + object-safety + context threading). BLOCKER #2 is a *Problem 2/4 seam* issue (VerificationOutcome placement). BLOCKER #3 is a *Problem 3* cross-plan dep-version issue. BLOCKERS #4, #5, #8 are planning-hygiene regressions.

**Decomposition principle:** keep these four concerns in four modules that communicate via value types, not across-module traits. This reduces the cross-plan compile coupling that led to BLOCKER #2.

---

### 4. HMW (How Might We) prompts

- HMW detect `execute_commands` completion when the command-emitted stdout cannot be trusted?
- HMW give `task_complete` a real "no silent success" guarantee even though Phase 8 hasn't landed?
- HMW keep ForgeCode parity for 6 tools while introducing an object-safe trait the agent loop can dispatch from?
- HMW enforce image-budget caps without starving legitimate multi-image workflows?
- HMW make the PTY path as transparent to the agent as the non-PTY path?
- HMW let Phase 4 swap in a real sandbox without touching tool impls?
- HMW guarantee that a tool's emitted schema cannot regress below ForgeCode's hardening baseline?

---

### 5. Ideation — solution sweep *(divergent — quantity over quality)*

For the riskiest problem (marker protocol completion detection), six distinct approaches:

1. **Static sentinel** `__CMDEND__` — simplest, trivially forgeable. **Rejected (D-03B).**
2. **Per-session nonce** `__CMDEND_<128-bit>__EXITCODE=N` — unforgeable by prompt injection, parseable. **Chosen (D-03A).**
3. **Named-pipe / unix-socket signal** — OOB, Windows portability trap. **Rejected (D-03D).**
4. **Exit-code via tempfile** — needs sandbox write grant, still not injection-safe. **Rejected (D-03C).**
5. **`wait()` on child** — works but loses the "stream output before exit" property (SHELL-03 fails).
6. **PTY-only** — Terminus-style, 10-50× slower startup for headless. **Rejected (D-04B).**

For `task_complete` verifier:

1. **`NoOpVerifier` returning `Pending`** — honors SC #5, Phase 8 swap point. **Chosen (D-06A).**
2. **Always-success stub** — silently defeats SC #5. **Rejected.**
3. **Defer the tool entirely** — blocks LOOP-05 signal in Phase 5. **Rejected.**
4. **LLM-as-judge minimal** — scope creep; Phase 8 work. **Rejected.**

For image caps:

1. **Per-turn 2 / per-session 20** (upper SC #4 bound) — max bandwidth, tunable. **Chosen (D-07A).**
2. **Per-turn 1 / per-session 10** (lower SC #4 bound) — starves multi-image workflows. **Rejected.**
3. **Cost-aware dynamic cap** — couples tools to cost logic; over-engineered for v1. **Rejected.**

**All three riskiest design choices already have locked decisions** (CONTEXT.md D-03, D-06, D-07). The brainstorm confirms these; doesn't reopen them.

---

### 6. Assumption sweep *(what if we're wrong?)*

| # | Assumption | Confidence | Blast radius if wrong | Cheapest test |
|---|------------|-----------:|----------------------|---------------|
| A1 | 128-bit nonce is enough entropy to resist prompt-injection | **High** | Premature stream close → false task success | Property test: 10k forgery attempts via `proptest` (scheduled: `markers::scan_line_forged`) |
| A2 | `subtle::ConstantTimeEq` defends against timing side-channel | Medium | Theoretical; unlikely in LLM loop | Benchmark: `criterion` microbench comparing `==` vs `ct_eq` timing variance |
| A3 | `portable-pty = "0.8"` works on macOS arm64 + x64 + Linux + Windows | High (MIT/Apache license verified) | PTY fallback fails → interactive tools hang | Integration test: `pty_integration.rs` with `cat` on each OS in CI |
| A4 | SIGTERM → 2s grace → SIGKILL is sufficient for `cargo test` runaway | Medium | Zombie children; child-of-child survives | Integration test: `timeout_cascade.rs` with `bash -c 'sleep 100 & wait'` |
| A5 | `NoOpVerifier` returning Pending won't bore the agent into infinite retry | Medium | Agent loop stall | Phase 5 integration smoke (not this phase); doc the contract clearly |
| A6 | `forge_app::ToolExecutor` remains API-stable for parity delegation | **High** (Phase 2.5 pinned deps) | Parity regressions | Integration test: `parity_delegation.rs` byte-diff |
| A7 | Image base64 data-URI returns fit within provider context limits | Medium | Context overflow; OpenRouter error 413 | Integration test: `image_quota.rs` with 512KB PNG × 2 |
| A8 | The 7-tool set is enough for realistic TB 2.0 tasks | Medium | Phase 5 needs more tools; delays eval | TB 2.0 local dry-run after Phase 5 closes; tool-use histogram analysis |

The four **Medium-confidence** assumptions (A2, A4, A5, A7) should get explicit tests in FLOW 5a test-strategy. A1, A3, A6 already have tests scoped in VALIDATION.md.

---

### 7. Inversion / reverse brainstorm *(how to make Phase 3 fail)*

- Use a static marker → prompt injection closes stream early. ✅ Prevented by D-03.
- Skip `subtle::ConstantTimeEq` → timing side channel. ✅ Prevented by 03-04-02 test.
- Have tools import each other's error types → compile cycle + cross-plan churn. ⚠️ BLOCKER #2 example; must be fixed in revision pass.
- Hardcode the sandbox → Phase 4 refactor invalidates Phase 3 tests. ✅ Prevented by D-12 (Sandbox DI).
- Ship `image_read` without quota → starvation. ✅ Prevented by D-07.
- Allow `Arc<dyn Tool>` to close over `!Send` types → agent loop breaks. ⚠️ Must verify in revision pass (object-safety audit BLOCKER #6).
- Ship `unimplemented!()` placeholders → TDD red tests never land. ⚠️ BLOCKER #4, #5; revision pass must replace every placeholder with real implementation + RED test.
- Leave the marker-nonce seam untested → regressions invisible. ✅ Prevented by `markers::scan_line_*` + property test.

---

### 8. Scope boundary test

**In scope** (Phase 3):
- Object-safe `Tool` trait in `kay-tools`.
- Immutable `ToolRegistry` built by `default_tool_set(...)`.
- 7 tools with delegation to `forge_app::ToolExecutor` (except `execute_commands`).
- `harden_tool_schema` wrapper reusing `forge_app::utils::enforce_strict_schema`.
- Marker protocol + PTY fallback + timeout cascade in `execute_commands`.
- `AgentEvent::ToolOutput` + `AgentEvent::TaskComplete` additive variants.
- `ToolError` enum.
- `NoOpVerifier` + `TaskVerifier` trait seam.
- `NoOpSandbox` + `Sandbox` trait seam.
- Image caps enforcement in `image_read`.

**Out of scope (deferred):**
- MCP tool registration.
- Runtime tool plugin API.
- `fs_patch`, `plan_create`, `todo_*`, `skill_fetch`, `sem_search` tools (Phase 5+).
- Gemini/Anthropic tool schemas (v2).
- Cost-aware tool throttling (v2).
- Tool-output persistence to session store (Phase 6).
- Multi-perspective verifier (Phase 8).

**Scope creep red flags to guard:**
- "While we're at it, let's add `fs_patch`" — NO, deferred.
- "Let's make the registry mutable so tests can stub" — NO, use DI.
- "Why not run the sandbox here since it's a few lines" — NO, Phase 4.

---

### 9. Success metrics mapping

| ROADMAP SC | Measurement | Test |
|------------|-------------|------|
| #1 `Tool` trait is object-safe | `cargo check -p kay-tools` compiles `Arc<dyn Tool>` usage + compile_fail test for violations | `registry::` unit tests; trybuild compile_fail test for object-safety regressions |
| #2 `ToolRegistry` exposes 7 hardened tools | Count + schema dump match reference | `tool_definitions_emit` + `schema_hardening_property` |
| #3 Hardened schemas stream on wire | OpenRouter request logs show `required` sorted, `additionalProperties: false` | `events::phase3_additions` + network-log assertion |
| #4 `execute_commands` streams with marker parse | `ToolOutput` frames arrive before process exit; marker parse extracts exit code | `streaming_latency` + `markers::scan_line_marker_match` + `execute_commands_e2e` |
| #5 `task_complete` returns Pending by default | NoOpVerifier end-to-end | `task_complete_pending` |

**Secondary / operational metrics:**
- TB 2.0 parity delta ≤ 2pp regression (measured at Phase 3 close via dry-run harness).
- clippy `-D warnings` clean.
- `cargo deny check` clean.

---

### 10. Key product-lens findings (feeding into Engineering-Lens)

1. **Problem decomposition (§3) is the root cause of plan-checker BLOCKERS.** The revision pass should restructure plan dependencies so each plan owns one problem.
2. **Risk-weighted test coverage (§6) shapes FLOW 5a test-strategy.** Medium-confidence assumptions (A2, A4, A5, A7) need explicit tests beyond what VALIDATION.md already scopes.
3. **Scope boundaries (§8) are tight and MUST be re-enforced** at FLOW 9 plan-revision. Do not let blocker-fixes become scope creep.
4. **Success metrics (§9) are already the right ones** — plan revision must ensure each of SC #1–#5 has a named automated test that will actually fail if the behavior regresses.
5. **Real-PTY behavior (A3)** must be smoke-tested on macOS (host env per user directive). Add `just smoke-pty` to FLOW 5a test-strategy.

---

### 11. Deferred / set-aside (capture list)

- MCP → ROADMAP add-on phase post v1
- Gemini/Anthropic schemas → v2
- Cost-aware throttling → v2
- Multi-perspective verifier → Phase 8
- Tool-output persistence → Phase 6
- Runtime plugin API → v2
- fs_patch / todo_* / etc. → Phase 5+

---

## §Engineering-Lens  *(FLOW 3b — superpowers:brainstorming)*

> Autonomous-mode brainstorm (bypass-permissions / standing directive). Project context
> already exhaustively explored — CONTEXT.md D-01..D-12, RESEARCH.md §1–§15, PATTERNS.md
> 31-file analog map, 5 PLAN.md files, VALIDATION.md test map. Engineering proposals
> below are best-judgment picks with explicit rationale; each is a locked autonomous
> decision for FLOW 9 plan-revision to consume.

### E1. Four-module architecture *(resolves B1, B2, B7)*

Under the four-problem decomposition from §Product-Lens §3, restructure `kay-tools` so each plan owns exactly one problem and they communicate via value types, not via cross-module trait imports:

```
crates/kay-tools/src/
  ├── lib.rs                  (re-exports only)
  ├── contract/               ⬅  "the Tool trait + value types"     (Problem 1 = dispatch)
  │   ├── mod.rs
  │   ├── tool.rs             — pub trait Tool (object-safe)
  │   ├── context.rs          — ToolCallContext (frozen shape)
  │   ├── output.rs           — ToolOutput, ToolOutputChunk
  │   └── error.rs            — ToolError enum
  ├── schema/                 ⬅  "schema emission"                   (Problem 2)
  │   ├── mod.rs
  │   ├── hardening.rs        — harden_tool_schema wrapper
  │   └── definitions.rs      — ToolDefinition, tool_definitions()
  ├── registry/               ⬅  "dispatch table"                    (Problem 1 — fast path)
  │   ├── mod.rs
  │   ├── registry.rs         — ToolRegistry (Arc<dyn Tool> by name)
  │   └── default_set.rs      — default_tool_set(ctx) builder
  ├── runtime/                ⬅  "command execution + streaming"     (Problems 3 + 4)
  │   ├── mod.rs
  │   ├── markers.rs          — Marker protocol (nonce + scan_line)
  │   ├── pty.rs              — portable-pty fallback
  │   ├── timeout.rs          — SIGTERM → 2s → SIGKILL cascade
  │   └── stream.rs           — ToolOutput frame emitter
  ├── seams/                  ⬅  "DI points for later phases"
  │   ├── mod.rs
  │   ├── sandbox.rs          — trait Sandbox + NoOpSandbox  (Phase 4 swap)
  │   └── verifier.rs         — trait TaskVerifier + NoOpVerifier + VerificationOutcome  (Phase 8 swap)
  ├── builtins/               ⬅  "the 7 tools"
  │   ├── mod.rs
  │   ├── execute_commands.rs — diverges (uses runtime::)
  │   ├── task_complete.rs    — uses seams::verifier
  │   ├── image_read.rs       — real impl (no placeholders)
  │   ├── fs_read.rs          — delegates to forge_app::ToolExecutor
  │   ├── fs_write.rs         — delegates
  │   ├── fs_search.rs        — delegates
  │   └── net_fetch.rs        — delegates
  └── events.rs               — AgentEvent::ToolOutput + ::TaskComplete additions
```

**Dependency arrows** (compile-time, enforced by module visibility — no cycles):

```
builtins → {contract, schema, runtime, seams}
schema   → contract
registry → {contract, schema, builtins}
runtime  → contract
seams    → contract
events   → contract
```

**Resolves B2:** `VerificationOutcome` lives in `seams::verifier` in Plan 03-01 (scaffold all modules). Plan 03-02 imports from there — no hidden cross-plan dep.

**Resolves B7:** `ToolCallContext` is frozen in 03-01 with its FINAL shape (ctx holds `Arc<dyn Sandbox>`, `Arc<dyn TaskVerifier>`, `session_id`, `image_quota`, `timeout_secs`). No field accumulation across plans. All fields whose consumers arrive later are `Option<_>` or pre-initialized with the No-Op DI instances.

---

### E2. `Tool` trait signature — final & object-safe *(resolves B1, B6)*

```rust
// crates/kay-tools/src/contract/tool.rs
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Canonical name as used on the wire (e.g. "execute_commands").
    fn name(&self) -> &'static str;

    /// Human-readable description (used in schema + UI).
    fn description(&self) -> &'static str;

    /// Emits the JSON-schema for this tool's input, ALREADY HARDENED.
    /// Returns owned `serde_json::Value` (NOT `&schemars::Schema`) so the
    /// trait is dyn-compatible — `schemars::Schema` has invalidated object-safety
    /// trait bounds historically (generic `JsonSchema` derive proliferation).
    fn input_schema(&self) -> Value;

    /// Runs the tool. Streams `ToolOutputChunk`s via `ctx.emit(...)`.
    /// Returns the final `ToolOutput` value (which is also emitted as the
    /// terminal `ToolOutputChunk::Final{...}`).
    async fn invoke(&self, ctx: &ToolCallContext, input: Value)
        -> Result<ToolOutput, ToolError>;
}
```

**B1 resolution:** canonical return type is `serde_json::Value`. Plans 03-01 and 03-02 were wrong (proposed `&schemars::Schema`). Engineering rationale: `schemars::Schema` is not object-safe-friendly across versions (0.8 vs 1.0 break), downstream needs JSON anyway (provider serializes it), and we save a `Schema::to_json()` call per turn. Emit happens once at startup (into `ToolDefinition` cache), so the "owned Value" cost is irrelevant.

**B6 resolution:** `default_tool_set(...)` takes `Arc<dyn Sandbox>` and `Arc<dyn TaskVerifier>` — NOT `Arc<dyn forge_app::services::Services>`. For the 4 parity tools that delegate, the delegation target is a concrete `forge_app::ToolExecutor` obtained via a factory closure `Arc<dyn Fn() -> ToolExecutor + Send + Sync>` — this keeps `ToolExecutor`'s generics internal and `default_tool_set` object-safe.

```rust
pub fn default_tool_set(
    sandbox: Arc<dyn Sandbox>,
    verifier: Arc<dyn TaskVerifier>,
    executor_factory: Arc<dyn Fn() -> forge_app::ToolExecutor + Send + Sync>,
    image_quota: ImageQuota,
    timeout: Duration,
) -> ToolRegistry { ... }
```

---

### E3. Object-safety proof harness *(resolves B6)*

Add a `trybuild` compile_fail test in Plan 03-01 Wave 0 that DOES NOT compile if the `Tool` trait becomes object-unsafe:

```rust
// crates/kay-tools/tests/object_safety.rs
#[test]
fn tool_trait_is_object_safe() {
    // If this compiles, the trait is object-safe.
    fn _assert_dyn_compatible<T: Tool + ?Sized>() {}
    fn _assert_usable_as_trait_object() {
        let _: Arc<dyn Tool> = Arc::new(MyFakeTool);
    }
}
```

Plus a `trybuild` failure fixture at `tests/ui/tool_not_object_safe.rs` that WOULD fail to compile if someone added a generic method to the trait — this is the regression sentinel.

---

### E4. Dependency version unification *(resolves B3)*

Single workspace-scoped deps in the root `Cargo.toml`:

| Crate | Version | Rationale |
|-------|--------:|-----------|
| `portable-pty` | `0.8` | KIRA parity; license verified MIT |
| `subtle` | `2.5` | Constant-time eq; widely adopted |
| `nix` | **`0.29`** | Lowest-common-denominator. 0.30 pulled in extra features Kay doesn't need. Must align Plan 03-01 to 03-04. |
| `windows-sys` | `0.59` | `TerminateProcess` stopgap pre-Phase 4 Job Objects |
| `hex` | `0.4` | Marker nonce hex encoding |
| `rand` | `0.8` | Already in workspace; nonce OsRng source |
| `async-trait` | `0.1` | Tool trait |
| `proptest` | **`1.x`** | Property tests for marker scanner (add now, used in FLOW 5a) |

Plan 03-01 must set ALL of these in workspace deps section. Subsequent plans reference via `*.workspace = true`.

---

### E5. Zero-placeholder TDD discipline *(resolves B4, B5)*

**Policy (locked):** No production file may contain `unimplemented!()`, `todo!()`, or any `_at_planning_time_*` sentinel at the end of any wave. Plans must state:

> Before implementation in any task: write the failing RED test with a REAL assertion against the expected real-world behavior (not "assert fn exists"). Then write the minimal GREEN code that satisfies that assertion. Placeholders are a BLOCKER.

**Resolution for B4:** The `parity_delegation.rs` and `execute_commands_e2e.rs` integration tests in Plan 03-05 currently call a helper `unimplemented_at_planning_time_use_forge_app_test_helper()`. Replace with:

- Real helper `kay_tools_test_kit::forge_executor_fixture()` in `crates/kay-tools/tests/common/mod.rs` (shared test harness)
- This fixture constructs a real `forge_app::ToolExecutor` using `forge_test_kit` (already a workspace dep since Phase 2.5 sub-crate split)
- Fixture is tested itself — its own RED test asserts it spawns a working executor

**Resolution for B5:** `image_read.rs invoke()` gets a real impl NOW:

```rust
// crates/kay-tools/src/builtins/image_read.rs
pub struct ImageRead { quota: Arc<ImageQuotaTracker> }

#[async_trait]
impl Tool for ImageRead {
    fn name(&self) -> &'static str { "image_read" }
    fn description(&self) -> &'static str { include_str!("image_read.description.md") }
    fn input_schema(&self) -> Value { /* owned schema */ }

    async fn invoke(&self, ctx: &ToolCallContext, input: Value)
        -> Result<ToolOutput, ToolError>
    {
        let args: ImageReadArgs = serde_json::from_value(input)
            .map_err(ToolError::InvalidArgs)?;
        self.quota.try_consume(ctx.session_id, args.count)
            .map_err(ToolError::QuotaExceeded)?;
        let bytes = tokio::fs::read(&args.path).await
            .map_err(|e| ToolError::Io(args.path.clone(), e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let mime = mime_guess::from_path(&args.path).first_or_octet_stream();
        Ok(ToolOutput::DataUri {
            mime: mime.to_string(),
            data_b64: b64,
        })
    }
}
```

TDD order for 03-05-02 (image_read):
1. RED: `image_quota::per_turn_cap_enforced` — real assertion on 3rd read rejected.
2. RED: `image_quota::per_session_cap_enforced` — 21st read rejected.
3. RED: `image_read::missing_file_returns_io_error` — real assertion on error variant.
4. RED: `image_read::base64_roundtrip` — writes known PNG bytes, reads back, base64-decodes, byte-equals.
5. GREEN: the impl above, no placeholders.

---

### E6. Frontmatter hygiene *(resolves B8)*

Plan 03-01 (scaffold) currently claims all 11 REQ-IDs in frontmatter. Fix: scaffold plan claims ZERO satisfied requirements. It creates the file tree that *enables* later plans to satisfy requirements. Frontmatter:

```yaml
requirements_satisfied: []        # scaffold only — no behavior
requirements_enabled:             # prerequisite for
  - TOOL-01
  - TOOL-06
  - SHELL-01
  - ...
```

Add `requirements_enabled` field to the plan frontmatter schema (breaking-change to the planner template — capture in Deferred Improvements of WORKFLOW.md; update planner template post-Phase 3).

Each subsequent plan sets `requirements_satisfied` to the exact REQ-IDs its tasks close. Sum across plans 03-02..03-05 MUST equal the full set {TOOL-01..06, SHELL-01..05}. Plan-checker will verify.

---

### E7. Test-pyramid seam for FLOW 5a  *(feeds /testing-strategy)*

Concrete macOS-native test-pyramid seam the engineering lens recommends `/testing-strategy` formalize:

| Tier | Scope | Count target | Files | Tooling |
|------|-------|--------------|-------|---------|
| **Unit (lib)** | Module-internal | ~45 tests | `src/**/*.rs` with `#[cfg(test)]` | `cargo test -p kay-tools --lib` + `pretty_assertions` + `tokio::test` |
| **Integration** | Crate-external, seam-crossing | ~18 tests | `tests/*.rs` | `cargo test -p kay-tools --tests` |
| **Property** | Invariants | 3 suites | `tests/schema_hardening_property.rs`, `tests/marker_forgery_property.rs`, `tests/quota_arithmetic_property.rs` | `proptest = "1"` |
| **trybuild compile-fail** | API regression fences | 2 fixtures | `tests/ui/*.rs` + `tests/object_safety.rs` | `trybuild = "1"` |
| **Smoke** | CLI happy path | 1 script | `scripts/smoke/phase3.sh` — spawns `kay-cli exec -- echo hi`, asserts `ToolOutput::Chunk{"hi\n"}` + `ToolOutput::Final{exit: 0}` arrive in order | bash + `jq` |
| **Live E2E** | Realistic PTY run | 1 macOS test | `scripts/smoke/phase3-pty.sh` — spawns `kay-cli exec --tty -- sh -c 'tty && exit 0'`, asserts `/dev/ttys*` output | bash on macOS |

**Live E2E on macOS specifically** — user directive: "test automation tools in this host macOS environment." The smoke harness leverages:
- `cargo run --bin kay-cli` (no notarization required for local smoke)
- `expect(1)` already available on macOS for PTY driving
- No TB 2.0 here (that's Phase 12) — just mechanical streaming + PTY proof

**Wave 0 addition:** scaffold `scripts/smoke/phase3.sh` and `crates/kay-tools/tests/common/mod.rs` (shared test harness with `forge_executor_fixture`).

---

### E8. Plan-revision targets *(concrete diffs FLOW 9 must produce)*

| Plan | Change required | BLOCKERS resolved |
|------|-----------------|-------------------|
| 03-01 | Scaffold ALL module tree (add `seams/`, `contract/`, `schema/`, `registry/`, `runtime/`, `builtins/`, `events.rs`); freeze `ToolCallContext` fields; frontmatter → `requirements_satisfied: []` + `requirements_enabled: [...]`; unify nix=0.29; add trybuild object-safety harness; add `tests/common/mod.rs` | B1, B2, B3, B6, B7, B8 |
| 03-02 | Depend only on 03-01; import `VerificationOutcome` from `kay_tools::seams::verifier` (scaffold owns it); fix `Tool::input_schema` → `Value`; add object-safety assertion test | B1, B2, B6 |
| 03-03 | Add SHELL-03 to `requirements_satisfied`; extend tests to cover streaming ordering (chunk-before-final) | — (W5 warning) |
| 03-04 | Unify nix=0.29; timeout cascade test split into 2 tasks (per W4 warning recommendation); note sandbox-tier limitation in task description | B3 + W4 warning |
| 03-05 | Replace `unimplemented_at_planning_time_use_forge_app_test_helper()` with `forge_executor_fixture` from common kit; write real `image_read::invoke` per E5; replace kay-cli wiring placeholder with concrete `kay_cli::boot::install_tool_registry()` | B4, B5 + W3 warning |

---

### E9. Design-review gate *(the spec review loop this skill normally runs)*

Per Superpowers `brainstorming` skill step 7, the spec normally gets reviewed by `spec-document-reviewer`. In the /silver flow, this reviewer role is served by `silver:silver-quality-gates` (FLOW 6, next) + `gsd-plan-checker` (FLOW 9 final loop). Both run before any code lands. No separate `spec-document-reviewer` dispatch — it would duplicate silver-quality-gates.

---

### E10. Autonomous decisions logged

Append to `.planning/WORKFLOW.md` Autonomous Decisions table:
- 2026-04-21 · Engineering-lens four-module layout (E1) · resolves B2/B7 by making cross-plan deps visible at module level
- 2026-04-21 · `Tool::input_schema() -> serde_json::Value` (E2/B1) · trait object-safety + provider serialization path
- 2026-04-21 · `default_tool_set` takes factory closure not `Arc<dyn Services>` (E2/B6) · preserves object-safety
- 2026-04-21 · Workspace-pin `nix = "0.29"` (E4/B3) · lowest-common-denominator parity
- 2026-04-21 · Zero-placeholder policy (E5/B4/B5) · TDD integrity — user directive
- 2026-04-21 · Scaffold plan declares `requirements_enabled` not `requirements_satisfied` (E6/B8) · frontmatter truthiness
- 2026-04-21 · FLOW 5a test pyramid seeded from E7 · 45 unit + 18 integration + 3 property + 2 trybuild + 2 macOS smoke
- 2026-04-21 · Spec-review gate via silver-quality-gates + gsd-plan-checker (E9) · avoid duplicate review

---

### E11. What's NOT in scope for revision *(hold the line)*

Per §Product-Lens §8 scope fence, plan revision MUST NOT introduce:
- New tools beyond the 7-tool set
- MCP support
- Runtime mutability
- Gemini/Anthropic schemas
- Cost-aware throttling
- Tool-output persistence (Phase 6 concern)
- Real TaskVerifier logic (Phase 8)
- Real Sandbox logic (Phase 4)

If any B1–B8 resolution creeps into the above, treat it as a new BLOCKER and STOP.

---

## Cross-Reference

- Decisions locked: `03-CONTEXT.md` D-01..D-12
- Research: `03-RESEARCH.md` §1–§15
- Validation: `03-VALIDATION.md`
- Patterns map: `03-PATTERNS.md`
- 5 plans with 8 BLOCKERS: `03-01-PLAN.md` … `03-05-PLAN.md`
- Canonical flow: `.planning/CANONICAL-FLOW.md`
- Workflow manifest: `.planning/WORKFLOW.md`
