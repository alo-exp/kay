# Phase 5 Pre-Build Validation — silver:validate

> **Date:** 2026-04-21
> **Phase:** 5 — Agent Loop + Canonical CLI
> **Gate:** VALD-03 non-skippable
> **Mode:** autonomous inline (per §10e — never stall; the equivalent Skill-tool invocation would produce the same answers from the same upstream artifacts)

---

## Inputs scanned

- `.planning/PROJECT.md` — project definition, Key Decisions locked
- `.planning/REQUIREMENTS.md` — 83 REQs; Phase 5 owns 11
- `.planning/ROADMAP.md` — Phase 5 goal + 8 success criteria
- `.planning/STATE.md` — milestone v0.3.0 in_progress, progress 4/17 phases
- `.planning/phases/05-agent-loop/05-BRAINSTORM.md` — present, complete
- `.planning/phases/05-agent-loop/05-TEST-STRATEGY.md` — present, complete
- `.planning/phases/05-agent-loop/05-IMPL-OUTLINE.md` — present, complete
- Phase 4 closure artifacts (SANITY CHECK: is Phase 4 actually merged?) — ✓ PR #5 squash-merged as `1ae2a7f`; `phase/04-sandbox` branch deleted

---

## Validation checks

### V-1: REQ consistency

| Check | Result | Evidence |
| ----- | ------ | -------- |
| All 11 REQs exist in REQUIREMENTS.md | PASS | LOOP-01..06 (lines 65-70), CLI-01/03/04/05/07 (lines 118-124) |
| Phase 5 ROADMAP-stated REQs match REQUIREMENTS.md Phase column | WARN (non-BLOCK) | Traceability table (lines 242-249) omits CLI-04, CLI-05, CLI-07 — must be fixed during Step 6 planning. Filed as TODO item in todo list. |
| No duplicate REQ-IDs in Phase 5 | PASS | Each REQ appears exactly once in phase scope |

### V-2: Phase dependency satisfaction

| Check | Result | Evidence |
| ----- | ------ | -------- |
| Phase 2 (OpenRouter streaming) complete | PASS | STATE.md history: shipped as PROV-01 in Phase 2 |
| Phase 3 (Tool Registry) complete | PASS | `AgentEvent`, dispatcher.rs, registry.rs, seams/, markers/ all populated from Phase 3 |
| Phase 4 (Sandbox) complete | PASS | v0.2.0 signed tag; 3-OS CI green; PR #5 squash-merged as 1ae2a7f |
| No circular phase dep introduced | PASS | Phase 5 consumes 2/3/4; none consume Phase 5 |

### V-3: Non-Negotiables (PROJECT.md) satisfied

| Locked decision | Phase 5 impact | Result |
| --------------- | -------------- | ------ |
| Forked ForgeCode parity gate ≥ 80% TB 2.0 | Parity gate still measured against `forgecode-parity-baseline` tag; Phase 5 port is brand-swap, not harness modification | PASS |
| No unsigned release tags | v0.3.0 ship at Phase 5 close will be ED25519-signed | PASS (policy only; enforcement at ship time) |
| DCO on every commit | Standing policy; every wave commit signed | PASS (policy only) |
| Clean-room attestation | Persona YAMLs inherit ForgeCode semantics per public README/docs; no leaked-source copying | PASS |
| Single merged Rust binary (no sidecar) | Phase 5 adds kay-core modules; kay-cli binary; no new external processes | PASS |
| Strict OpenRouter allowlist | No new provider integrations; persona YAMLs reference existing Exacto-allowlist models | PASS |
| ForgeCode JSON schema hardening | Phase 5 does not modify tool schemas; carries forward Phase 3 hardening | PASS |

### V-4: Security surface flagged for Step 10

| Concern | Flagged in upstream artifacts? | Plan-time owner |
| ------- | ------------------------------ | --------------- |
| QG-C4 — SandboxViolation re-injection | YES — brainstorm §Product-Lens risk row 1 + Engineering-Lens E12 | Wave 2 (event_filter 100% coverage) |
| YAML persona injection surface | YES — brainstorm §Product-Lens risk row 5 | Wave 3 (schema validation) |
| Ctrl-C race conditions | YES — brainstorm §Product-Lens risk row 7 | Wave 2 (control channel) + Wave 4 (loop integration) |
| sage_query recursion | YES — brainstorm E5 + T-9 test | Wave 5 (nesting_depth guard) |
| Structured event stdout DoS | YES — brainstorm §Product-Lens risk row 9 | Wave 7 (CLI --events-buffer flag if needed) |
| R-2 image_read unbounded payload | YES — ROADMAP success criterion 7 | Wave 6b |
| R-1 PTY denylist bypass | YES — ROADMAP success criterion 6 | Wave 6a |

All 7 security concerns have named mitigations and assigned waves.

### V-5: Testing coverage

| Check | Result |
| ----- | ------ |
| Every REQ has at least one test | PASS — matrix in TEST-STRATEGY §Test-level distribution by REQ |
| QG-C4 has property test | PASS — T-4 property `model_context_filter_random_sequences_never_leak_sandbox_violation` |
| Every sandbox-violation path has an E2E test | PASS — T-5 `kay_run_headless_exits_two_on_sandbox_violation` |
| 3-OS CI matrix covers all test suites | PASS — TEST-STRATEGY §CI matrix |
| Signal testing platform-gated where necessary | PASS — T-3 Ctrl-C test; T-5 exit-130 test; platform cfg-gated |

### V-6: Scope-creep check

| Item | In Phase 5 scope? | Evidence |
| ---- | ----------------- | -------- |
| Session persistence (SESS-*) | NO — Phase 6 | BRAINSTORM §Product-Lens scope OUT |
| Context retrieval (CTX-*) | NO — Phase 7 | BRAINSTORM §Product-Lens scope OUT |
| Critics (VERIFY-*) | NO — Phase 8 | BRAINSTORM §Product-Lens scope OUT (NoOpVerifier only) |
| Tauri/TUI | NO — Phase 9/9.5 | BRAINSTORM §Product-Lens scope OUT |
| `cargo install kay` distribution | NO — Phase 10 | BRAINSTORM §Product-Lens scope OUT |
| CLI-02 session import/export | NO — Phase 6 | Stubbed with "not yet implemented" |

No scope creep detected.

### V-7: Risk coverage

9 risks identified in BRAINSTORM §Product-Lens — all have named mitigations:

1. QG-C4 breach → event_filter (Wave 2)
2. Interactive regression → snapshot parity against forgecode-parity-baseline tag
3. AgentEvent schema drift → 16 insta snapshots (Wave 1)
4. select! deadlocks → property test (Wave 4)
5. YAML persona injection → schema validation at load (Wave 3)
6. Clean-room concern for new code → all new modules from scratch
7. Ctrl-C race → cooperative abort w/ 2s grace
8. Upstream ForgeCode diverges → parity against frozen tag
9. JSONL stdout DoS → unbuffered default + optional buffer flag

**All 9 risks covered.**

### V-8: Artifact completeness

| Artifact | Required? | Present? |
| -------- | --------- | -------- |
| 05-BRAINSTORM.md | YES | YES ✓ |
| 05-TEST-STRATEGY.md | YES | YES ✓ |
| 05-IMPL-OUTLINE.md | YES | YES ✓ |
| 05-VALIDATION.md (this file) | YES | YES ✓ |
| 05-QUALITY-GATES.md | YES (next step) | NO — Step 3 |
| 05-CONTEXT.md | YES | NO — Step 4 |
| 05-DEPENDENCIES.md | YES | NO — Step 5 |
| 05-PLAN.md | YES | NO — Step 6 |

Upstream Step 2 artifacts all present.

### V-9: Dependency graph cleanliness

Phase 5 introduces these new module boundaries:

- `kay-core::{loop, persona, control, event_filter}` — new top-of-DAG modules
- `kay-tools::builtins::sage_query` — new sub-tool
- `kay-tools::events_wire` — new wire-layer module
- `kay-cli` — populated from stub

No circular imports. kay-tools does NOT import kay-core (kay-core is above it). kay-cli imports kay-core + kay-tools. kay-core imports kay-tools + kay-provider-openrouter + forge_* aggregator.

**Clean.**

---

## Findings classification

| Severity | Count | Details |
| -------- | ----- | ------- |
| BLOCK    | 0     | None |
| WARN     | 1     | V-1 traceability table gap for CLI-04/05/07 — must be fixed during Step 6 planning (tracked in todo list) |
| INFO     | 3     | (a) Parity fixture existence unconfirmed — first open Q for discuss-phase. (b) Pause-semantics open — second discuss-phase Q. (c) Forge_main retention timing — fifth discuss-phase Q. |

---

## Verdict

**ZERO BLOCK findings.** Phase 5 cleared to proceed to Step 3 (silver:quality-gates design-time).

WARN item queued for Step 6 fix. INFO items queued for Step 4 discuss-phase resolution.

---

**Next step:** Step 3 `silver:quality-gates` (design-time 9-dimension review) → `05-QUALITY-GATES.md`.
