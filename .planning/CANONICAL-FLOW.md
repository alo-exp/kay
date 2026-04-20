# Kay — Canonical Composable Flow

> **Status:** Load-bearing contract. `/silver` is the sole orchestrator for this R&D-heavy Rust project.
> All phase work routes through this flow. Short-circuits (silver:fast) are reserved for ≤3-file typos.
> **Created:** 2026-04-20 after the user directive mandating 100% TDD + full test-pyramid rigor.

---

## Invariants (non-negotiable)

1. **Single orchestrator.** `/silver` composes the flow and retains end-to-end ownership, even when execution hands off to GSD / Superpowers / silver-bullet skills.
2. **TDD is mandatory.** Every behavioural commit follows the Superpowers `test-driven-development` loop: red → green → refactor. No tests ⇒ no commit. No exceptions for "scaffold" tasks that introduce logic.
3. **Full test pyramid on every phase.** `unit + integration + smoke + live E2E` coverage is designed upfront via the `testing-strategy` skill, not retrofitted.
4. **Quality gates before planning AND before shipping.** 9-dimension `silver-quality-gates` runs in *design-time* mode after brainstorming and in *adversarial* mode before `gsd-ship`.
5. **GSD is the execution backbone.** Planning, execution, verification flow through GSD skills. `/silver` wraps but never replaces `/gsd-plan-phase`, `/gsd-execute-phase`, `/gsd-verify-work`.
6. **Parity gate persistence.** Every phase re-runs the ForgeCode TB 2.0 parity check (EVAL-01) before merge. Regression > 2pp ⇒ automatic block.
7. **Never skip a flow node.** Skips require an explicit "artifact already exists" detection (silver-feature FLOW 0 context scan). Missing artifact ≠ "simple enough to skip."

---

## The Canonical Pipeline (14 Flows)

```
┌─ /silver (ROUTER + ORCHESTRATOR) ──────────────────────────────────────────┐
│                                                                            │
│  FLOW 0  BOOTSTRAP       Load preferences, detect repo state               │
│  FLOW 1  ORIENT          Read PROJECT.md, ROADMAP.md, STATE.md, CLAUDE.md  │
│  FLOW 2  INTEL           MultAI research for novel architecture decisions  │
│  FLOW 3  BRAINSTORM      (a) product-brainstorming (Engineering plugin)    │
│                          (b) Superpowers brainstorming skill               │
│                          (c) gsd-discuss-phase for gray areas              │
│  FLOW 4  SPECIFY         silver:spec or /gsd-add-phase requirement capture │
│  FLOW 5  PLAN            /gsd-plan-phase → RESEARCH.md + PLAN.md + VALID.  │
│  FLOW 6  TEST-STRATEGY   /testing-strategy skill designs pyramid:          │
│                            • unit (cargo test --lib)                       │
│                            • integration (cargo test --test *)             │
│                            • property (proptest)                           │
│                            • smoke (cargo run minimal flows)               │
│                            • live E2E (Terminal-Bench 2.0, Tauri UI E2E)   │
│  FLOW 7  DESIGN-CONTRACT (UI phases only — skip for harness phases)        │
│  FLOW 8  QUALITY-GATES   silver-quality-gates DESIGN-TIME mode (9 dims)    │
│                          Hard block on any ❌ before execution begins.     │
│  FLOW 9  TDD-EXECUTE     /gsd-execute-phase wrapping Superpowers TDD skill │
│                          per task: RED commit → GREEN commit → REFACTOR    │
│  FLOW 10 REVIEW          Superpowers requesting-code-review +              │
│                          /gsd-code-review (static) + /gsd-code-review-fix  │
│  FLOW 11 VERIFY          /gsd-verify-work conversational UAT +             │
│                          live E2E smoke run (TB 2.0 parity + UI click-path)│
│  FLOW 12 SECURE          silver-bullet:security + /gsd-secure-phase        │
│                          + cargo-deny + cargo-audit + threat-model walk    │
│  FLOW 13 QUALITY-GATES-2 silver-quality-gates ADVERSARIAL mode (pre-ship)  │
│  FLOW 14 SHIP            /gsd-ship → signed tag (Non-Negotiable #2) + DCO  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Delegation Map (who does what)

| Stage | Primary skill | Fallback | Owned artifact |
|-------|---------------|----------|----------------|
| Context scan | `silver-feature` FLOW 0 | — | (in-memory) |
| Product brainstorm | `product-management:product-brainstorming`¹ | Superpowers `brainstorming` | `.planning/phases/NN-<slug>/NN-BRAINSTORM.md` |
| Design brainstorm | Superpowers `brainstorming` | — | appended to same file |
| Gray-area resolution | `gsd-discuss-phase` | — | `NN-CONTEXT.md` + `NN-DISCUSSION-LOG.md` |
| Research | `gsd-phase-researcher` (spawned by `/gsd-plan-phase`) | MultAI `multai:research` | `NN-RESEARCH.md` |
| Pattern analog map | `gsd-pattern-mapper` | — | `NN-PATTERNS.md` |
| Planning | `gsd-planner` | — | `NN-*-PLAN.md` |
| Plan verification | `gsd-plan-checker` (up to 3 revision passes) | — | (inline verdict) |
| Test strategy | Sidekick `testing-strategy` | — | `NN-TEST-STRATEGY.md` |
| Design-time quality | `silver-bullet:silver-quality-gates` (design-time) | — | `NN-QUALITY-GATES.md` |
| TDD execution | Superpowers `test-driven-development` wrapped in `gsd-executor` | — | per-commit RED→GREEN→REFACTOR |
| Code review | Superpowers `requesting-code-review` + `gsd-code-reviewer` | — | `NN-REVIEW.md` |
| UAT / verify | `gsd-verify-work` | — | `NN-UAT.md` + `NN-VERIFICATION.md` |
| Security | `silver-bullet:security` + `gsd-security-auditor` | — | `NN-SECURITY.md` |
| Pre-ship quality | `silver-bullet:silver-quality-gates` (adversarial) | — | append to `NN-QUALITY-GATES.md` |
| Ship | `gsd-ship` | — | signed tag + PR |

¹ `product-management` plugin is not currently installed on this host. `silver-feature` FLOW 3 handles graceful fallback: it invokes `product-brainstorming` via the Skill tool; on failure it proceeds with Superpowers `brainstorming` only. When `product-management` plugin is later installed, the flow lights up automatically with no orchestration change.

---

## Test Pyramid Policy (FLOW 6 output, enforced in FLOW 9)

Every phase's `NN-TEST-STRATEGY.md` must include all five tiers, with concrete cargo commands and macOS-compatible tooling:

| Tier | Scope | Tooling (macOS) | Where tests live | Frequency |
|------|-------|-----------------|------------------|-----------|
| **Unit** | Single function / module | `cargo test --lib` + `pretty_assertions` + `tokio::test` | `crates/<crate>/src/**/tests` (inline `#[cfg(test)]`) | Every commit |
| **Integration** | Cross-module, real filesystem, real process spawn | `cargo test --test <name>` (optionally `cargo-nextest` when installed) | `crates/<crate>/tests/*.rs` | Every commit that crosses a module seam |
| **Property** | Invariant search (fuzzing-lite) | `proptest` + optional `cargo-fuzz` (nightly) for adversarial markers | `crates/<crate>/tests/*_property.rs` | Every phase that adds a parser/protocol |
| **Smoke** | Shortest end-to-end happy path | `cargo run -p kay-cli -- <subcmd>` exit-0 check | `scripts/smoke/*.sh` + `just smoke` (or `make smoke`) | Every phase, wave-end |
| **Live E2E** | Realistic agent run | Terminal-Bench 2.0 harness run (Phase 1 baseline infra) + Tauri UI `cargo tauri dev` click-path (Phase 9/9.5 onward) | `evals/tb2/` + `frontend/tests/e2e/` | Every phase close, pre-ship, and nightly CI |

**Gate:** FLOW 9 (TDD-EXECUTE) cannot start a task until its unit + integration harness exists (RED tests first). Smoke + E2E run at wave-end (FLOW 11).

---

## TDD Loop (FLOW 9 inner loop — per task)

For every task in a `NN-<plan>-PLAN.md`:

```
┌─ Task N ────────────────────────────────────────────────────────┐
│ 1. READ      read_first artifacts + existing analog (PATTERNS)  │
│ 2. RED       write failing unit test(s) — commit "test:" msg    │
│              cargo test --lib <test_name>  (must FAIL)          │
│ 3. RED-INT   write failing integration test(s) if seam-crossing │
│              cargo test --test <integ>     (must FAIL)          │
│ 4. GREEN     minimal implementation to pass tests — commit      │
│              cargo test -p <crate> --all-targets  (must PASS)   │
│ 5. REFACTOR  clean up, clippy -D warnings, dedupe — commit      │
│              cargo clippy --workspace --all-targets -- -D warn  │
│ 6. VERIFY    run wave-level sampling per VALIDATION.md          │
│ 7. CHECKPOINT if blocked: write checkpoint file, pause          │
└─────────────────────────────────────────────────────────────────┘
```

Each of steps 2–5 produces a distinct atomic commit with conventional-commit prefix (`test:`, `feat:`, `refactor:`). This makes the TDD trail reviewable and revertable.

**Deviation handling:** If a GREEN step needs more than +20% estimated LoC, pause → spawn `gsd-debugger` for diagnosis → update plan, do not silently grow.

---

## Quality-Gate Budget (FLOW 8 + FLOW 13)

Run the 9-dimension `silver-quality-gates` skill twice per phase:

- **Pre-plan (FLOW 8, design-time mode):** after RESEARCH + PLAN + TEST-STRATEGY, before FLOW 9. Fails block planning iteration.
- **Pre-ship (FLOW 13, adversarial mode):** after VERIFY + SECURE, before `/gsd-ship`. Fails block the release.

Dimensions: Modularity · Reusability · Scalability · Security · Reliability · Usability · Testability · Extensibility · AI/LLM-Safety.

Phase 3-specific focus (R&D Rust harness): **Reliability** (marker protocol under prompt injection), **Security** (PTY escape, sandbox seam discipline), **Testability** (deterministic nonce seam), **Extensibility** (Phase 4 sandbox + Phase 8 verifier swap points).

---

## Hand-off Protocol (when /silver delegates)

When `/silver` hands work to a sub-skill, it records:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 SILVER BULLET ► DELEGATING TO {skill}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Flow:          FLOW N · {flow-name}
Expected out:  {artifact path}
Return gate:   {what /silver checks before the next flow}
Fallback:      {on skill failure}
```

After the sub-skill returns, `/silver` runs its return gate (file existence, frontmatter, required-section grep) and either advances or iterates on the failure path. The orchestrator is never "done" until FLOW 14 completes.

---

## Cross-Reference

- Phase-level state: `.planning/STATE.md`
- Roadmap: `.planning/ROADMAP.md`
- Requirements: `.planning/REQUIREMENTS.md`
- Project definition: `.planning/PROJECT.md`
- Non-negotiables: `CLAUDE.md` §Non-Negotiables
- GSD tooling: `~/.claude/get-shit-done/` workflows
- Silver-Bullet orchestration: `~/.claude/plugins/cache/alo-labs/silver-bullet/0.23.4/`
- Superpowers TDD skill: `~/.claude/plugins/cache/superpowers-marketplace/superpowers/5.0.5/skills/test-driven-development/`
- Sidekick testing-strategy: `~/.claude/plugins/marketplaces/sidekick/.forge/skills/testing-strategy/`

---

## Change Control

This document is the canonical flow contract. Changes require:
1. ADR-style discussion in a new `.planning/adr/NNNN-flow-change.md`
2. User approval (document lives at repo root of `.planning/`)
3. PR labeled `flow-change` — does not merge via silver:fast

Last updated: 2026-04-20
