---
phase: 3
slug: tool-registry-kira-core-tools
mode: design-time
auto_detected: "5 PLAN.md files exist; no VERIFICATION.md → design-time per silver-quality-gates Step 0 row 3"
gate: FLOW 6 (pre-plan-revision) of canonical silver-feature flow
reviewer: silver-quality-gates 0.23.4 (9-dimension planning checklist)
review_date: 2026-04-21
review_mode: autonomous
status: PASS-conditional
---

# Phase 3 — Quality Gates Report (Design-Time)

> 9-dimension silver-quality-gates assessment against the Phase 3 **design surface** before FLOW 9 Plan-Revision.
> Design-time mode Planning Checklist per `silver-bullet/0.23.4/skills/{dim}/SKILL.md`.

---

## Design Surface Under Review

| Artifact | Role | Status |
|----------|------|--------|
| `.planning/REQUIREMENTS.md` | 11 REQ-IDs (TOOL-01..06, SHELL-01..05) | frozen |
| `.planning/ROADMAP.md` Phase 3 SC#1..#5 | Acceptance criteria | frozen |
| `03-CONTEXT.md` D-01..D-12 | Locked decisions | frozen |
| `03-RESEARCH.md` | Technical grounding + threat model T-3-01..08 | frozen |
| `03-BRAINSTORM.md §Product-Lens` | 11 sections; A1..A8 assumptions | complete |
| `03-BRAINSTORM.md §Engineering-Lens` | E1..E11 — architecture + BLOCKERS' fixes | complete |
| `03-TEST-STRATEGY.md` | 72-test pyramid (45 unit + 2 trybuild + 18 integ + 3 prop + 2 smoke + 2 E2E) | complete |
| `03-VALIDATION.md` + `.planning/VALIDATION.md` | Sampling + finding counts (8 BLOCK engineered, 6 WARN, 14 INFO) | complete |
| `03-01..05-PLAN.md` | 5 plans — pre-revision state | BLOCKED pending FLOW 9 |

Gate question: *Does the designed architecture — once applied by FLOW 9 Plan-Revision — satisfy all 9 quality dimensions?*

---

## Summary Table

| Dimension | Result | Notes |
|-----------|--------|-------|
| Modularity | ✅ Pass | 4-module layout (contract/schema/registry/runtime/seams/builtins/events) per E1 — each file ≤300 LOC hard limit; dependencies flow one direction; scaffold-owns-seams resolves B2+B7 circularity. |
| Reusability | ✅ Pass | `enforce_strict_schema` reused unmodified (D-02); parity delegation byte-identical for 6 of 7 tools (A6); factory closure in `default_tool_set` reusable across CLI + Tauri (Phase 9) + tests. |
| Scalability | ✅ Pass | Streaming marker protocol O(1) per line — no buffered stdout accumulation; immutable registry (D-11) → lock-free concurrent read; per-session image quota caps memory at 20×512KB ceiling. |
| Security | ✅ Pass | 128-bit nonce + `subtle::ConstantTimeEq` resists timing side-channels (A1+A2); marker seeded by `OsRng` (not PRNG); PTY denylist + non-PTY default reduces escape surface; sandbox seam isolates filesystem writes (D-12, Phase 4 tightens). Threat model T-3-01..08 all mapped to mitigations. |
| Reliability | ✅ Pass | SIGTERM→2s→SIGKILL cascade w/ waitpid reap (D-05); NoOpVerifier returns Pending never false success (D-06); property test P-02 covers 10k forgery attempts; SHELL-03 streaming latency integration test gates blocking behavior. |
| Usability | ✅ Pass | 7-tool set matches OpenRouter Exacto tool-calling idioms; schema hardening (D-02) produces OpenAI-compatible `tool_definitions()`; errors classified in `ToolError` enum (D-09) with user-facing messages distinct from transport errors. |
| Testability | ✅ Pass | 72-test pyramid with per-REQ closure matrix; every REQ has ≥2 tests in ≥2 tiers; deterministic nonce seam for unit tests (seedable `ChaCha20Rng` feature-flagged); trybuild compile-fail covers object-safety; macOS CI matrix + stable Rust 1.85 (no nightly required). |
| Extensibility | ✅ Pass | Phase 4 sandbox swap point: `Sandbox` trait in seams/ replaceable at CLI bootstrap without touching tool impls (D-12); Phase 8 verifier swap point: `TaskVerifier` trait similarly seam-isolated (D-06); trait object-safety verified via trybuild fixture T-01 (E3) so future tools compose cleanly; factory closure pattern in `default_tool_set` accepts Phase 9 Tauri-service factories. |
| AI/LLM Safety | ✅ Pass | Prompt-injection resistance is Phase 3's load-bearing safety property: (a) marker protocol entropy ensures agent-output cannot forge completion (A1); (b) NoOpVerifier Pending prevents hallucinated task_complete (A5); (c) image quota caps prevent token-exhaustion DoS (A8 scope); (d) schema hardening prevents agent emitting untyped args that bypass validation. Aligns with CLAUDE.md Non-Negotiable #7. |

---

## Per-Dimension Detail

### 1. Modularity ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Every planned file has a one-sentence responsibility | ✅ | E1 module table — contract.rs (trait+error), schema.rs (emission+hardening), registry.rs (Arc<dyn Tool> registry), runtime.rs (dispatcher+context), seams/*.rs (verifier, sandbox, rng), builtins/*.rs (7 tool impls), events.rs (AgentEvent variants). |
| No planned file exceeds soft line limit | ✅ | Largest projected: `builtins/execute_commands.rs` ≤280 LOC (marker + PTY + timeout + streaming); under 300 hard limit. |
| File structure organized by feature/domain | ✅ | Feature-based (contract/schema/registry/runtime/seams/builtins/events), not layer-based. |
| Any single task touches ≤3-5 source files | ✅ | PLAN-revision matrix E8: each task averages 2-3 files + its tests. |
| Public interface defined before impl | ✅ | E2 locks `Tool` trait signature; seams own downstream types (`VerificationOutcome` in seams/verifier.rs). |
| Dependencies flow one direction | ✅ | E1 dep arrow: `builtins → runtime → seams → schema → contract` (resolves B2 circular dep). |
| Context test: any task <1500 LOC total | ✅ | Each task's RED/GREEN touches ≤5 files averaging <800 LOC total. |

### 2. Reusability ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Upstream primitives reused not reimplemented | ✅ | D-02: `forge_app::enforce_strict_schema` reused unmodified; parity delegation for 6 of 7 builtins calls `forge_app::ToolExecutor` byte-identically (A6 owner: executor). |
| Tool trait reusable across binaries | ✅ | Object-safe `Arc<dyn Tool>` works in CLI (Phase 3), Tauri app (Phase 9), headless harness (Phase 1 TB 2.0 run). |
| Error types reusable across tools | ✅ | D-09 `ToolError` enum covers all 7 builtins + custom user tools (post-v1 MCP). |
| Factory closure enables DI | ✅ | E2 `default_tool_set<F>(make_svc: F)` allows mock services in unit tests, real services in prod, Tauri-service factory in Phase 9. |

### 3. Scalability ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| No buffered accumulation on critical path | ✅ | SHELL-03: stdout streams as `AgentEvent::ToolOutput` per line, not collected; integration test I-05 measures <50ms latency. |
| Registry read path lock-free | ✅ | D-11 immutable registry — `HashMap<&'static str, Arc<dyn Tool>>` frozen at construction; no RwLock needed. |
| Bounded memory on image quota | ✅ | D-07 per-turn cap 2, per-session cap 20; at 512KB average → 10MB session ceiling, well under CLI working-set budget. |
| Concurrent tool dispatch safe | ✅ | `Arc<dyn Tool + Send + Sync>` trait bound; no interior mutability in tool state. |
| Signal handling scales to child trees | ✅ | D-05 `nix::sys::signal::killpg` targets process group, not single pid — reaps child-spawned processes on `SIGTERM`. |

### 4. Security ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Threat model complete | ✅ | T-3-01..08 in 03-RESEARCH.md §Threat Model covers schema bypass, marker forgery, PTY escape, timeout evasion, sandbox escape, verifier bypass, image quota bypass, parity drift. |
| Cryptographic primitives correct | ✅ | `rand::rngs::OsRng` (not weak PRNG), 128-bit nonce (not 32/64-bit), `subtle::ConstantTimeEq` (not `==`). Property test P-02 runs 10k forgery attempts. |
| No user input reaches shell unescaped | ✅ | `execute_commands` uses `std::process::Command` arg vector (no shell expansion); only explicit `tty: true` or denylist enters PTY path. |
| Sandbox seam isolated | ✅ | D-12 `NoOpSandbox` DI pattern; Phase 4 will substitute real seccomp/bpf without touching call sites. |
| Prompt-injection hardening | ✅ | Marker protocol: pattern is `__CMDEND_<nonce>_<seq>__EXITCODE=N` — nonce is per-invocation secret unknown to agent. Even if agent echoes `__CMDEND_...__` in stdout, constant-time compare against stored nonce rejects it. |
| Schema hardening | ✅ | D-02 `enforce_strict_schema` — required sorted, `additionalProperties: false`, `allOf` flattened. Property test P-01 enforces invariants with 1024 cases. |

### 5. Reliability ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Timeout cascade correct | ✅ | SIGTERM → 2s grace → SIGKILL (D-05); integration test I-07 with bash `trap` verifies ordering. |
| No blocking on shutdown | ✅ | `tokio::select!` in streaming loop with cancellation token; abort on parent-task drop. |
| NoOpVerifier returns Pending | ✅ | D-06; unit test U-21 asserts `Pending`, not `Success`. |
| Retries not silently added | ✅ | Plans do NOT introduce retry logic — Phase 5 agent loop owns retry policy. |
| Error propagation preserves cause | ✅ | D-09 `ToolError::Runtime { source }` keeps `anyhow::Error` chain. |

### 6. Usability ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Tool schemas OpenAI-compatible | ✅ | D-02 `tool_definitions()` emits JSONSchema draft-07 style per OpenRouter Exacto contract (CLAUDE.md Non-Negotiable #6). |
| Errors classified for user display | ✅ | D-09 enum variants distinguish `Input` (bad args — user actionable) from `Transport` (retry later) from `Runtime` (show backtrace) from `Cancelled` (timeout). |
| Tool names match ForgeCode convention | ✅ | `execute_commands`, `task_complete`, `image_read`, `fs_read`, `fs_write`, `fs_search`, `net_fetch` — snake_case matches upstream. |
| Documentation landmarks present | ✅ | Each builtin module has top-of-file `//!` doc explaining tool semantics; research §11 captures user-visible behavior. |

### 7. Testability ✅ (Phase 3 focus area)

| Checklist item | Verdict | Evidence |
|---|---|---|
| Every REQ has ≥2 tests in ≥2 tiers | ✅ | 03-TEST-STRATEGY.md §3 closure matrix — 11 REQs × ≥2 tiers. |
| Deterministic nonce seam for unit tests | ✅ | 03-TEST-STRATEGY.md §2.1 unit tier — feature-flagged `ChaCha20Rng::from_seed` replaces `OsRng` behind `#[cfg(test)]` trait-object seam. |
| Object-safety verified | ✅ | Trybuild T-01 `tool_not_object_safe.fail.rs` compile-fails if future refactor breaks dyn compatibility. |
| Property testing covers entropy claims | ✅ | P-02 marker forgery: 10k cases assert no forgery closes stream. |
| macOS CI matrix | ✅ | §5 CI invocation matrix: macOS-13 + macOS-14 + ubuntu-22.04 + Windows (best-effort); cargo-nextest primary, `cargo test` fallback. |
| No nightly required | ✅ | All tooling works on stable 1.85; cargo-llvm-cov is optional nightly (WARN VAL-014 decided: ship on stable without line-cov gate). |
| Runtime budgets documented | ✅ | §6 budgets: unit <15s, integration <3min, property <60s, smoke <60s, E2E <2min each. |
| FLOW 9 per-task acceptance_criteria mandated | ✅ | §7 plan-revision obligations require each task's `<acceptance_criteria>` block to list test IDs. |

### 8. Extensibility ✅

| Checklist item | Verdict | Evidence |
|---|---|---|
| Phase 4 sandbox swap point isolated | ✅ | D-12 `Sandbox` trait in `seams/sandbox.rs` — `NoOpSandbox` today, seccomp/bpf tomorrow, no tool call-site changes. |
| Phase 8 verifier swap point isolated | ✅ | D-06 `TaskVerifier` trait in `seams/verifier.rs` — `NoOpVerifier` today, learned-verifier tomorrow, `task_complete` call-site unchanged. |
| Object-safety preserves future extension | ✅ | E2 factory closure + trait object-safe signatures mean MCP (post-v1) can register dynamic tools at runtime via `ToolRegistry::extend` without breaking the immutable-default registry. |
| Event emission additive | ✅ | D-08 AgentEvent variants added, not modified — downstream consumers (Tauri UI Phase 9) receive new variants via `#[non_exhaustive]` enum without recompile break. |
| No hardcoded 7-tool set at dispatch layer | ✅ | Registry is opaque `HashMap` — adding fs_patch/plan_create/todo_* in Phase 5+ requires only a builtins/ module + registration call, no dispatcher changes. |

### 9. AI/LLM Safety ✅ (Phase 3 focus area)

| Checklist item | Verdict | Evidence |
|---|---|---|
| Prompt injection cannot forge tool completion | ✅ | Marker nonce is secret to harness, never crosses LLM boundary; P-02 10k-forgery property test gates this invariant. |
| Hallucinated task_complete blocked | ✅ | NoOpVerifier Pending (D-06) returns non-success; Phase 5 agent loop interprets Pending as "continue thinking", not "task done". |
| Tool-call args schema-validated | ✅ | D-02 schema hardening + `serde::Deserialize` on tool input structs → malformed agent args rejected before invoke. |
| Resource exhaustion bounded | ✅ | Image quotas (2/turn, 20/session); timeout cascade bounded at 2s grace + indefinite SIGKILL wait → no unbounded agent-side resource consumption. |
| Agent output not interpreted as harness control | ✅ | Streaming stdout → `AgentEvent::ToolOutput` frames are pure data — harness does NOT re-parse for meta-commands. Only marker line closes stream. |
| Secrets not leaked across tool boundary | ✅ | `Tool::invoke` takes `ToolCallContext` that is explicit about what services are exposed; no global singletons; env vars not auto-propagated to subprocesses in `execute_commands` (stdin/env sanitized per KIRA pattern). |

---

## Failures Requiring Redesign

**None.** All 9 dimensions pass the design-time planning checklist.

---

## Conditional Pass — 2 prerequisites for FLOW 9 Plan-Revision

The Pass verdict is conditional on **Plan-Revision (FLOW 9) faithfully applying the §Engineering-Lens E1–E11 fixes** to the 5 PLAN.md files. Specifically:

1. All 8 BLOCK findings from `.planning/VALIDATION.md` VAL-001..VAL-008 must clear — gsd-plan-checker must report zero BLOCKERS.
2. Every task's `<acceptance_criteria>` block must enumerate the concrete test IDs from `03-TEST-STRATEGY.md` §3 per §7 plan-revision obligations.

Both prerequisites are mechanical application of already-designed fixes. Gate: `/silver:validate` re-run at end of FLOW 9 must return 0 BLOCK findings before FLOW 10 (TDD-Execute) begins.

---

## Overall: **PASS (design-time)** — proceed to FLOW 9 Plan-Revision

The design surface is structurally sound. Architecture (E1), trait signatures (E2), dep pinning (E4), zero-placeholder policy (E5), frontmatter hygiene (E6), and test pyramid (E7) comprehensively resolve every risk surfaced by the plan-checker and the 8 product-lens assumptions.

---

## Backlog Capture (Step 5 mandatory scan)

The quality-gate pass noted these items for post-Phase-3 backlog:

| # | Item | Severity | Target |
|---|------|----------|--------|
| 1 | Add `cargo-llvm-cov` nightly CI lane for line-coverage >90% gate (currently relying on per-REQ closure matrix on stable — VAL-014) | advisory | post-Phase-5 CI hardening |
| 2 | Criterion timing benchmark for `subtle::ConstantTimeEq` in marker validation (A2 confidence raise) | advisory | post-Phase-3 perf suite |
| 3 | `cargo-fuzz` nightly fuzz lane for `markers::scan_line` (A1 defense-in-depth) | advisory | post-v1 security hardening |
| 4 | Full Windows PTY matrix (currently best-effort only — VAL-010) | advisory | post-Phase-9 cross-platform sweep |
| 5 | Load test: 1000 concurrent tool invocations through registry (scalability ceiling check) | advisory | post-Phase-5 stress harness |
| 6 | `cargo-mutants` mutation testing on `markers/` + `schema/` (testability depth) | advisory | post-Phase-3 test quality |
| 7 | Promote trybuild from dev-dep to first-class test tier in planner template schema | advisory | gsd-planner tooling enhancement |
| 8 | Add `requirements_enabled` field to gsd-planner frontmatter schema (distinct from `requirements:`) | advisory | gsd-planner template evolution |

These are logged in `.planning/WORKFLOW.md` Deferred Improvements and will be filed to the GSD backlog per CLAUDE.md §Backlog Filing convention at FLOW 17 (ship) — not filed now because they all depend on Phase 3 completion to have concrete file references.

---

## Cross-Reference

- Canonical flow: `.planning/CANONICAL-FLOW.md` §Quality-Gate Budget
- Design surface index: this file §Design Surface Under Review
- Engineering fixes: `03-BRAINSTORM.md §Engineering-Lens` E1–E11
- Test pyramid: `03-TEST-STRATEGY.md` §§1–7
- Validation findings: `.planning/VALIDATION.md`
- Next gate: `silver-quality-gates` adversarial mode (FLOW 16, pre-ship)
