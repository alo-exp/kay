---
phase: 2
slug: provider-hal-tolerant-json-parser
status: populated
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-20
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Body (Per-Task Verification Map) populated by the planner in step 8 of the plan-phase workflow — one row per task across all 10 plans.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stable 1.95, 2024 edition) + `cargo clippy` + `cargo fmt --check` + `proptest` (for tolerant parser) + `mockito` (for OpenRouter SSE fixtures) |
| **Config file** | `Cargo.toml` (workspace) + `crates/kay-provider-openrouter/Cargo.toml` (dev-dependencies added in plan 02-01 and 02-06) |
| **Quick run command** | `cargo test -p kay-provider-openrouter --all-features` (plan-local) |
| **Full suite command** | `cargo test --workspace --all-features` (ONCE plan 02-05 lands — the CI cleanup plan removes the `--exclude kay-core` escape clause) |
| **Estimated runtime** | ~30-90 seconds local; ~3-6 minutes in CI matrix (Ubuntu + macOS + Windows) |

See `RESEARCH.md §Validation Architecture` for the full 10-dimension breakdown.

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p <crate-touched>` + `cargo clippy -p <crate-touched> -- -D warnings`
- **After every plan wave:** Run `cargo test -p kay-core -p kay-provider-openrouter --all-features`
- **Before `/gsd-verify-work`:** Full `cargo test --workspace --all-features` green on at least one OS locally (cross-OS verified in CI on PR)
- **Max feedback latency:** < 60 seconds for task-level verification; < 5 minutes for wave-level

**Special sampling for D-01 rename + path-rewrite plans (02-02 through 02-05):** after EACH per-subtree commit, run `cargo check -p kay-core 2>&1 | grep -cE "(E0432|E0433|E0583)"` to confirm the error count is monotonically non-increasing. Plans 02-03 and 02-05 write `/tmp/02-0N-residual-errors.txt` after completion so the subsequent plan's baseline is known.

---

## Per-Task Verification Map

> Populated by the planner in step 8. One row per task across all 10 plans.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01.T1 | 02-01 | 1 | PROV-01, PROV-04, PROV-05, PROV-07 | — | Dev deps resolve | build | `cargo check -p kay-provider-openrouter --tests` | YES | ⬜ pending |
| 02-01.T2 | 02-01 | 1 | PROV-05 | TM-06 | 6 SSE cassettes + MockServer compile | build | `cargo check -p kay-provider-openrouter --tests && ls crates/kay-provider-openrouter/tests/fixtures/sse/*.jsonl \| wc -l` = 6 | YES | ⬜ pending |
| 02-02.T1 | 02-02 | 2 | PROV-01 prereq | T-02-02-01 | 23 renames, zero content delta | build | `ls crates/kay-core/src/forge_*/mod.rs \| wc -l` = 23 AND `cargo check -p kay-core 2>&1 \| grep -c E0583` = 0 | NO (creates 23 files) | ⬜ pending |
| 02-03.T1 | 02-03 | 3 | PROV-01 prereq | T-02-03-01 | 12 leaf subtrees rewritten | build | 12 subtrees with 0 residual `use crate::X` (non-forge) AND 12 signed-off commits | NO | ⬜ pending |
| 02-03.T2 | 02-03 | 3 | PROV-01 prereq | T-02-03-01 | forge_domain rewritten | build | 0 residual `use crate::X` (non-forge) in forge_domain/ | NO | ⬜ pending |
| 02-03.T3 | 02-03 | 3 | PROV-01 prereq | T-02-03-01 | forge_fs/snaps/spinner/tracker rewritten | build | 0 residual `use crate::X` (non-forge) in each; 4 signed-off commits | NO | ⬜ pending |
| 02-04.T1 | 02-04 | 4 | PROV-01 prereq | T-02-04-01 | forge_app rewritten (103 files) | build | 0 residual `use crate::X` (non-forge) in forge_app/; single commit limited to forge_app/ | NO | ⬜ pending |
| 02-05.T1 | 02-05 | 5 | PROV-01 prereq | T-02-05-01 | Upper 5 subtrees rewritten; kay-core compiles clean | build | `cargo check -p kay-core 2>&1 \| grep -cE "(E0432\|E0433\|E0583)"` = 0 AND `cargo check --workspace` exits 0 | NO | ⬜ pending |
| 02-05.T2 | 02-05 | 5 | PROV-01 prereq | T-02-05-01 | `--exclude kay-core` removed everywhere | build | 0 `exclude kay-core` occurrences in ci.yml/CONTRIBUTING/CICD.md/pr_template; STATE.md blocker removed | NO | ⬜ pending |
| 02-06.T1 | 02-06 | 6 | PROV-02 | — | Cargo deps resolve + dep tree has backon/async-trait | build | `cargo fetch -p kay-provider-openrouter` exits 0 AND `cargo tree -p kay-provider-openrouter \| grep -c backon` >= 1 | YES | ⬜ pending |
| 02-06.T2 | 02-06 | 6 | PROV-01, PROV-02, PROV-08 | T-02-06-01, T-02-06-03 | Public type contract, #[non_exhaustive], deny unwrap | unit + build | `cargo test -p kay-provider-openrouter --lib` exits 0 with >= 2 tests; `cargo clippy -p kay-provider-openrouter --lib -- -D warnings` exits 0 | NO (creates 3 src files) | ⬜ pending |
| 02-07.T1 | 02-07 | 7 | PROV-04 | TM-04, TM-08 | Allowlist gate with normalization + charset validation | unit + integration | `cargo test -p kay-provider-openrouter --lib allowlist::unit` = 8 passed AND `cargo test -p kay-provider-openrouter --test allowlist_gate` = 6 passed | NO | ⬜ pending |
| 02-07.T2 | 02-07 | 7 | PROV-03 | TM-01 | Env-wins auth resolution + ApiKey Debug redacted | unit + integration | `cargo test -p kay-provider-openrouter --lib auth::unit` = 6 passed AND `cargo test -p kay-provider-openrouter --test auth_env_vs_config` = 4 passed | NO | ⬜ pending |
| 02-08.T1 | 02-08 | 8 | PROV-02 | TM-01 | UpstreamClient fallible ctor + header builder | unit | `cargo test -p kay-provider-openrouter --lib client::unit` = 3 passed; no .unwrap()/.expect() in non-test code | NO | ⬜ pending |
| 02-08.T2 | 02-08 | 8 | PROV-01, PROV-02, PROV-05 | TM-04, TM-08 | Translator + OpenRouterProvider impl (strict parse) | unit + build | `cargo test -p kay-provider-openrouter --lib translator::unit` >= 3 passed; `cargo clippy -p kay-provider-openrouter --lib -- -D warnings` exits 0 | NO | ⬜ pending |
| 02-08.T3 | 02-08 | 8 | PROV-01, PROV-05 | T-02-08-02 | End-to-end SSE streaming + reassembly | integration | `cargo test -p kay-provider-openrouter --test streaming_happy_path` = 2 passed AND `cargo test --test tool_call_reassembly` = 1 passed | NO | ⬜ pending |
| 02-09.T1 | 02-09 | 9 | PROV-05 | TM-06, TM-07 | Two-pass parser (strict + forge_json_repair fallback) | unit | `cargo test -p kay-provider-openrouter --lib tool_parser::unit` = 6 passed | NO | ⬜ pending |
| 02-09.T2 | 02-09 | 9 | PROV-05 | TM-06 | Translator uses parse_tool_arguments + 1MB cap | integration non-regression | `cargo test -p kay-provider-openrouter --test streaming_happy_path` still = 2 passed AND `--test tool_call_reassembly` still = 1 passed (non-regression) | NO | ⬜ pending |
| 02-09.T3 | 02-09 | 9 | PROV-05 | TM-05, TM-07 | Never-panic invariant + malformed cassette | property + integration | `cargo test -p kay-provider-openrouter --lib tool_parser::unit` >= 8 passed (incl. 2 proptest) AND `--test tool_call_malformed` = 1 passed | NO | ⬜ pending |
| 02-10.T1 | 02-10 | 10 | PROV-06, PROV-07, PROV-08 | TM-03, TM-06 | Retry primitives + CostCap struct | unit | `cargo test -p kay-provider-openrouter --lib retry::unit` >= 10 passed AND `cost_cap::unit` = 7 passed | NO | ⬜ pending |
| 02-10.T2 | 02-10 | 10 | PROV-06, PROV-07, PROV-08 | TM-03, TM-06, TM-08 | Retry + cost_cap wired into provider + translator | integration non-regression | Existing integration tests from plans 02-07/08/09 all still exit 0 | NO | ⬜ pending |
| 02-10.T3 | 02-10 | 10 | PROV-06, PROV-07, PROV-08 | TM-01, TM-03, TM-06, TM-08 | Retry + cost cap + error taxonomy integration tests | integration | `cargo test -p kay-provider-openrouter --test retry_429_503` >= 2 passed AND `--test cost_cap_turn_boundary` = 2 passed AND `--test error_taxonomy` = 5 passed | NO | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Total expected test count at end of Phase 2 (sum of automated verifies above):**
- Unit: 8 allowlist + 6 auth + 3 client + ~3 translator-inline + 6+2 tool_parser + 10+ retry + 7 cost_cap = **~45+**
- Integration: 6 allowlist_gate + 4 auth + 2 streaming_happy_path + 1 tool_call_reassembly + 1 tool_call_malformed + 2 retry_429_503 + 2 cost_cap_turn_boundary + 5 error_taxonomy = **~23**
- Total: **~68+ tests**

---

## Wave 0 Requirements (populated by plan 02-01)

- [x] `crates/kay-provider-openrouter/tests/common/mod.rs` created with `pub mod mock_server;` (plan 02-01 Task 1)
- [x] `crates/kay-provider-openrouter/tests/common/mock_server.rs` created with `mock_openrouter_chat_stream`, `mock_rate_limit`, `mock_server_error_503`, `load_sse_cassette` (plan 02-01 Task 2)
- [x] `crates/kay-provider-openrouter/tests/fixtures/sse/` with 6 cassettes: happy_path, tool_call_fragmented, tool_call_malformed, rate_limit_429, server_error_503, usage_without_cost (plan 02-01 Task 2)
- [x] `crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json` with D-07 launch allowlist (plan 02-01 Task 2)
- [x] `[dev-dependencies]` added to `crates/kay-provider-openrouter/Cargo.toml`: mockito 1.7, proptest 1.11, pretty_assertions 1, tokio with test features (plan 02-01 Task 1)
- [x] `cargo test --workspace --exclude kay-core` continues to pass (baseline — plans 02-02..02-05 drive kay-core to green; at plan 02-05 we can drop `--exclude`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real-network smoke test | PROV-02 | Requires OpenRouter API key + ~$0.05 budget; not in CI | `OPENROUTER_API_KEY=sk-or-... cargo test -p kay-provider-openrouter --test real_network -- --ignored` (plan 02-10 may add this as an `#[ignore]` test; otherwise manual) |
| `:exacto` suffix routing behavior | PROV-04 | OpenRouter-internal; captured trace substitutes for CI | Maintainer runs real-network smoke test against `anthropic/claude-sonnet-4.6:exacto` vs `anthropic/claude-sonnet-4.6` and archives response diffs as fixtures |
| Cost-cap enforcement on live traffic | PROV-06 | Needs real cost data; ~$0.25 budget | Maintainer runs `--max-usd 0.10` test that should trip after ~5 real turns; confirms `ProviderError::CostCapExceeded` fires at turn boundary |

*All other Phase 2 behaviors have automated verification via mockito fixtures + property tests + unit tests.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify (22 task rows, all populated above)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (every task has automated grep/cargo check)
- [x] Wave 0 covers all MISSING references (plan 02-01 creates MockServer + 6 cassettes; no Wave 0 gaps remain at execute time)
- [x] No watch-mode flags in automated commands (all `cargo test`, `cargo check`, `cargo clippy` are one-shot)
- [x] Feedback latency < 60s per task, < 5 min per wave
- [x] `nyquist_compliant: true` set in frontmatter
- [ ] Full suite `cargo test --workspace --all-features` (no `--exclude`) passes locally — validated by plan 02-05's success criterion + `/gsd-verify-work 2` gate

**Approval:** pending (planner-side ✅; executor signoff deferred to end of plan 02-10)
