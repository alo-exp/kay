---
phase: 2
slug: provider-hal-tolerant-json-parser
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-20
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Body (Per-Task Verification Map) populated by the planner in step 8 of the plan-phase workflow — the planner owns one row per task it creates.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stable 1.95, 2024 edition) + `cargo clippy` + `cargo fmt --check` + `proptest` (for tolerant parser) + `mockito` (for OpenRouter SSE fixtures) |
| **Config file** | `Cargo.toml` (workspace) + `crates/*/Cargo.toml` (per-crate dev-dependencies added by planner) |
| **Quick run command** | `cargo test -p kay-provider-openrouter --all-features` (plan-local) |
| **Full suite command** | `cargo test --workspace --all-features` (ONCE D-01 path rewrites complete — Phase 2 explicitly removes the `--exclude kay-core` escape clause) |
| **Estimated runtime** | ~30-90 seconds local; ~3-6 minutes in CI matrix (Ubuntu + macOS + Windows) |

See `RESEARCH.md §Validation Architecture` for the full 10-dimension breakdown (unit, integration, property, contract, streaming, error taxonomy, retry, cost cap, allowlist, security).

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p <crate-touched>` + `cargo clippy -p <crate-touched> -- -D warnings` (quick command scoped to the crate the task modified)
- **After every plan wave:** Run `cargo test -p kay-core -p kay-provider-openrouter --all-features`
- **Before `/gsd-verify-work`:** Full suite `cargo test --workspace --all-features` green on at least one OS locally (cross-OS verified in CI on PR)
- **Max feedback latency:** < 60 seconds for task-level verification; < 5 minutes for wave-level

**Special sampling for D-01 rename + path-rewrite plans (01-01 through ~01-24):** After EACH per-subtree commit, run `cargo check -p kay-core` to confirm the E0583 count is monotonically non-increasing and no regression in `--exclude kay-core` test paths.

---

## Per-Task Verification Map

> Populated by the planner in step 8. One row per task the planner emits.
> Keep this table in sync with `<automated>` blocks inside each `NN-MM-PLAN.md`.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| *(planner fills rows per task)* | | | | | | | | | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/kay-provider-openrouter/tests/` directory created with shared fixture module
- [ ] `crates/kay-provider-openrouter/tests/fixtures/sse/` with at least 5 real OpenRouter SSE trace cassettes for happy path + malformed tool_calls + 429 Retry-After + 503 server error + Usage-without-cost
- [ ] `dev-dependencies` added to `crates/kay-provider-openrouter/Cargo.toml`: `mockito = "1.5"`, `proptest = "1.5"`, `tokio = { version = "1.51", features = ["rt-multi-thread", "macros", "test-util"] }`, `insta = { version = "1.40", features = ["yaml"] }`
- [ ] `cargo test --workspace --exclude kay-core` continues to pass (baseline — Phase 2 must not regress existing tests while rename+rewrite happens)

*Planner refines this list based on the actual tasks chosen.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real-network smoke test | PROV-02 | Requires OpenRouter API key + ~$0.05 budget; not in CI | `OPENROUTER_API_KEY=sk-or-... cargo test -p kay-provider-openrouter --test real_network -- --ignored` |
| `:exacto` suffix routing behavior | PROV-04 | OpenRouter internal; captured trace substitutes for CI | Maintainer runs real-network smoke test against `anthropic/claude-sonnet-4.6:exacto` and `anthropic/claude-sonnet-4.6`, diffs the responses, archives both as fixtures |
| Cost-cap enforcement on live traffic | PROV-06 | Needs real cost data; ~$0.25 budget | Maintainer runs `--max-usd 0.10` test that should trip after ~5 turns against real OpenRouter; confirms `ProviderError::CostCapExceeded` fires at turn boundary |

*All other phase behaviors have automated verification via mockito fixtures + property tests.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (especially SSE fixture cassettes)
- [ ] No watch-mode flags in automated commands
- [ ] Feedback latency < 60s per task, < 5 min per wave
- [ ] `nyquist_compliant: true` set in frontmatter
- [ ] Full suite `cargo test --workspace --all-features` (no `--exclude`) passes locally before `/gsd-verify-work`

**Approval:** pending
