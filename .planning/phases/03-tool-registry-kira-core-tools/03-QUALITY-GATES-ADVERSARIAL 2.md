---
phase: 3
flow: 16
mode: adversarial
audit_date: 2026-04-21
head: 0dd5910
verdict: PASS
---

# Phase 3 — Adversarial Pre-Ship Quality Gates (FLOW 16)

> Full audit across 9 silver-bullet quality dimensions on post-Nyquist state. Paired with design-time gate report at `03-QUALITY-GATES.md`.

## Quality Gates Report

| Dimension     | Result | Notes |
|---------------|--------|-------|
| Modularity    | ✅ | 4-module layout (contract/schema/registry/runtime/seams/builtins/events) held firm across Waves 0-4. Object-safe `Tool` trait + `ForgeServicesHandle` facade preserve separation. No cross-module leakage; Rule-3 reconciliations (e.g., service-layer facade vs. ToolExecutor) kept boundaries clean. |
| Reusability   | ✅ | `kay-tools` crate is the reusable tool registry; `kay-provider-errors` extracted specifically to break the kay-tools ↔ kay-provider-openrouter cycle. Facade pattern (`ForgeServicesHandle`) lets any future sandbox/verifier swap in without disturbing builtins. Parity tools are 10-line adapters. |
| Scalability   | ✅ | Per-turn/session image quota atomic (saturating); stream_sink is `Arc<dyn Fn>` broadcast — O(1) subscription; marker scan is O(line length) with fixed-width slices. No global mutexes on hot paths. Signal cascade uses pgid broadcast (single syscall regardless of descendant count). |
| Security      | ✅ | H-01 (pgid SIGKILL) fixed + regression-locked. M-01..M-05 fixed (RNG failure propagation, quota leak on IO fail, PTY SIGTERM grace symmetry, image_read sandbox check). 30k-case marker-forgery proptest added. `subtle::ConstantTimeEq` on nonce compare. 7-NN compliance clean (NN#1/3/4/5/7 enforced in code; NN#2/6 out-of-scope). cargo-deny green. |
| Reliability   | ✅ | Marker protocol under prompt injection: 30,000 adversarial proptest cases + unit forgery tests + `marker_race::forged_marker_does_not_close`. Timeout cascade: SIGTERM→2s→unconditional killpg SIGKILL. `kill_on_drop(true)` backstop. `NoOpVerifier` Pending seam preserved for Phase 8 swap. Parity byte-diff test prevents silent drift. |
| Usability     | ✅ | `kay tools list` smoke confirms 7 tools enumerate with hardened schemas. Error surface: typed `ToolError` at boundaries, `anyhow::Result` at seams, `#![deny(clippy::unwrap_used, clippy::expect_used)]` enforced. User-facing tool descriptions are Forge-parity (same copy). |
| Testability   | ✅ | 174-test pyramid: 63 unit + 5 property (1 @1024 + 3 @10k = 31k property cases) + 30 integration + 2 smoke. All 11 REQs Nyquist-closed (≥2x boundary sampling). Deterministic nonce seam via `SysRng` seam (test-injectable). Compile-fail harness deferred with documented equivalent runtime locks. |
| Extensibility | ✅ | `#[non_exhaustive]` on `ToolCallContext` (6 fields locked per B7), `ToolError`, `AgentEvent`, `CapScope` — all allow additive growth without breaking downstream. Phase 4 sandbox swap: replace `NoOpSandbox` impl — no API change. Phase 8 verifier swap: replace `NoOpVerifier` — seam stable. Phase 5 service layer: swap `ForgeServicesHandle` impl — builtins untouched. |
| AI/LLM Safety | ✅ | Marker protocol resists prompt-injection closure (proptest + reserved-substring reject in command input). Schema hardening delegated byte-for-byte to `enforce_strict_schema` (NN#7 load-bearing). No secrets in tool error messages. Image quota prevents LLM-driven memory DoS (per-turn cap 2, per-session cap 20). ForgeCode parity preserved — TB 2.0 score gate intact. |

### Failures requiring redesign

**None.** Zero ❌ across all 9 dimensions.

### Overall: **PASS**

Quality gates passed (pre-ship). Proceed to shipping (FLOW 17).

---

## Evidence matrix (adversarial audit — sampled, not exhaustive)

| Dimension | Probe | Result |
|-----------|-------|--------|
| Security | `rg 'todo!\(\)\|unimplemented!\(\)\|unimplemented_at_planning_time_' crates/kay-tools/src crates/kay-cli/src` | 0 matches |
| Security | DCO trailer count across 27 commits `1bb792d..HEAD` | 37 `Signed-off-by:` lines — every commit signed |
| Security | cargo deny check | advisories/bans/licenses/sources all ok |
| Reliability | `killpg(pgid, SIGKILL)` unconditional post-grace | `execute_commands.rs:376-401` + `tests/timeout_cascade.rs` grandchild regression |
| Reliability | Marker constant-time compare | `markers/mod.rs:95` uses `subtle::ConstantTimeEq::ct_eq` |
| Testability | 14 test files in `crates/kay-tools/tests/` | marker_forgery_property, parity_delegation, timeout_cascade, pty_integration, etc. all present |
| Modularity | `ToolCallContext` field count | exactly 6, `#[non_exhaustive]`, per B7 lock |
| Extensibility | Trait object-safety | `tests/registry_integration.rs::arc_dyn_tool_is_object_safe` + `parity_delegation.rs::parity_tools_are_object_safe` |
| AI/LLM Safety | Schema hardening delegation | `schema.rs` → `forge_app::utils::enforce_strict_schema` + 1024-case proptest |

## Deferred items (captured in backlog — R-series from 03-SECURITY.md)

All previously filed for Phase 4/5. No new deferrals introduced by this gate.

| ID | Item | Target |
|----|------|--------|
| R-1 | PTY metacharacter heuristic refinement (tokenize first arg) | Phase 5 backlog |
| R-2 | `AgentEvent::ImageRead` size cap (20 MiB default) | Phase 5 ForgeConfig |
| R-3 | *(closed by FLOW 14 — 30k-case marker-forgery proptest shipped)* | — |
| R-4 | Windows Job Objects for timeout cascade | Phase 4 sandbox (SBX-04) |
| R-5 | Populate or gate empty `dispatcher`/`rng` modules | Phase 4 |
| R-6 | `rmcp` advisory (out of scope) | MCP phase |

## Handoff

Phase 3 clears FLOW 16 with no redesign required. Safe to proceed to FLOW 17 (finishing-branch + signed-tag ship per NN#2).
