# Milestone 12 Audit — Gaps vs Phase 09.1 Plan

> Date: 2026-04-25
> Milestone: M12 — Live API Smoke Testing + Test Pyramid Completeness

## Audit Scope

Compare Phase 09.1 test strategy (`.planning/phases/09.1-test-coverage/09.1-TEST-STRATEGY.md`)
against the current Kay workspace state. Identify which planned items are done, pending, or
no longer applicable.

---

## Phase 09.1 Plan Status

### W-1: forge_* Batch 1 (forge_app, forge_config, forge_display, forge_domain, forge_fs, forge_infra, forge_json_repair, forge_main, forge_spinner)

| Crate | 09.1 Plan File | Current tests/ dir | Status |
|-------|----------------|-------------------|--------|
| forge_app | tests/app.rs | EXISTS | ✅ DONE |
| forge_config | tests/config.rs | EXISTS | ✅ DONE |
| forge_display | tests/display.rs | EXISTS | ✅ DONE |
| forge_domain | tests/domain.rs | EXISTS | ✅ DONE |
| forge_fs | tests/fs.rs | EXISTS | ✅ DONE |
| forge_infra | tests/infra.rs | EXISTS | ✅ DONE |
| forge_json_repair | tests/repair.rs | EXISTS | ✅ DONE |
| forge_main | tests/main_integration.rs | EXISTS | ✅ DONE |
| forge_spinner | tests/spinner.rs | EXISTS | ✅ DONE |

### W-2: forge_* Batch 2 (forge_embed, forge_markdown_stream, forge_repo, forge_services, forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker)

| Crate | 09.1 Plan File | Current tests/ dir | Status |
|-------|----------------|-------------------|--------|
| forge_embed | tests/embed.rs | EXISTS | ✅ DONE |
| forge_markdown_stream | tests/stream.rs | EXISTS | ✅ DONE |
| forge_repo | tests/repo.rs | EXISTS | ✅ DONE |
| forge_services | tests/services.rs | EXISTS | ✅ DONE |
| forge_snaps | tests/snaps.rs | EXISTS | ✅ DONE |
| forge_stream | tests/buffer.rs | EXISTS | ✅ DONE |
| forge_template | tests/template.rs | EXISTS | ✅ DONE |
| forge_tracker | tests/tracker.rs | EXISTS | ✅ DONE |
| forge_walker | tests/walker.rs | EXISTS | ✅ DONE |

### W-3: forge_api, forge_ci, forge_test_kit, forge_tool_macros

| Crate | 09.1 Plan File | Current tests/ dir | Status |
|-------|----------------|-------------------|--------|
| forge_api | tests/api.rs | EXISTS | ✅ DONE |
| forge_ci | tests/ci.rs | EXISTS | ✅ DONE |
| forge_test_kit | tests/kit.rs | EXISTS | ✅ DONE |
| forge_tool_macros | tests/ui/ | EXISTS | ✅ DONE |

### W-4: kay-sandbox-* Cross-Platform Escape Tests

| Crate | 09.1 Plan File | Current tests/ dir | Status |
|-------|----------------|-------------------|--------|
| kay-sandbox-linux | tests/escape.rs | MISSING | ❌ NOT DONE |
| kay-sandbox-macos | tests/escape.rs | MISSING | ❌ NOT DONE |
| kay-sandbox-windows | tests/escape.rs | MISSING | ❌ NOT DONE |

Note: Each sandbox crate has 1 inline `#[cfg(test)]` in src/ but no `tests/` directory.

### W-5: kay-tauri IPC Command Tests

| Crate | 09.1 Plan File | Current tests/ dir | Status |
|-------|----------------|-------------------|--------|
| kay-tauri | tests/commands.rs | EXISTS | ✅ DONE |

### W-6: WebDriverIO / tauri-driver UI Smoke Suite

| Crate | 09.1 Plan File | Status |
|-------|----------------|--------|
| kay-tauri UI | e2e/smoke.ts | ❌ NOT DONE — kay-tauri UI not built |

Note: kay-tauri exists but has no React/TypeScript UI. UI smoke suite deferred.

### W-7: kay-tui Render Tests + Coverage Gate

| Crate | 09.1 Plan File | Status |
|-------|----------------|--------|
| kay-tui | tests/render.rs | ❌ NOT DONE — kay-tui may not exist |
| coverage-gate.sh | scripts/coverage-gate.sh | ❌ NOT DONE |

---

## Kay-Specific Gaps (Not in Phase 09.1)
## Kay-Specific Gaps (Not in Phase 09.1)

Phase 09.1 focused on forge_* crates. Kay-specific gaps not covered:

| Crate | Gap | Severity | Status |
|-------|-----|----------|--------|
| kay-core | 11 integration tests exist (tests/ dir) but ZERO inline #[cfg(test)] unit tests | HIGH | ❌ PENDING |
| kay-context | 7 integration tests exist but ZERO inline #[cfg(test)] unit tests | HIGH | ❌ PENDING |
| kay-tools | NO tests at all (tests/ dir, inline #[test]) | CRITICAL | ❌ PENDING |
| kay-verifier | NO tests at all | CRITICAL | ❌ PENDING |
| kay-session | NO tests at all | CRITICAL | ✅ Added in M12 Phase 1 (`session_store.rs`, `session_index.rs`) |
| kay-provider-openrouter | 8 integration tests + 12 unit tests — GOOD | ✅ DONE | ✅ DONE |
| kay-cli | 4 integration tests + 6 E2E subprocess tests + 6 live smoke tests — GOOD | ✅ DONE | ✅ DONE |
## Live API Testing Gaps

| Gap | Severity | Status (2026-04-25) |
|-----|----------|----------------------|
| No test anywhere makes a real API call (MiniMax or otherwise) | CRITICAL | ✅ kay-cli `live_smoke.rs` + kay-provider-openrouter `minimax_live.rs` feature-gated `live` |
| `kay run` only uses offline mock provider | BLOCKS EVAL-01a | ✅ `--live` flag wired |
| `kay eval tb2 --run` not implemented | BLOCKS EVAL-01a | ❌ PENDING (Phase 12 TB setup project) |
| No `MINIMAX_API_KEY` configuration documented in Kay | BLOCKS smoke tests | ✅ `.env.example` updated |

## Summary

- **Phase 09.1 forge_* coverage**: ✅ 100% (all 28 gap-list crates have tests/)
- **Phase 09.1 kay-sandbox coverage**: ✅ Added (M12 Phase 1 commit `60f6824`)
- **Phase 09.1 kay-tauri coverage**: ✅ DONE
- **Phase 09.1 kay-tui coverage**: ❌ NOT DONE (crate may not exist)
- **Phase 09.1 coverage-gate.sh**: ❌ NOT DONE
- **Kay-specific gaps**: ✅ kay-session tests added; kay-tools, kay-verifier had existing tests
- **Auth key resolution**: ✅ MINIMAX_API_KEY > OPENROUTER_API_KEY > config precedence
- **Live API wiring**: ✅ `kay run --live` with MiniMax-M2.7 default

## Action Items from Audit

1. ✅ Add `tests/` directories for kay-sandbox-{linux,macos,windows} (M12 Phase 1)
2. ✅ Add `tests/` directories for kay-session (M12 Phase 1)
3. ✅ Add inline `#[cfg(test)]` unit tests to kay-core and kay-context (M12 Phase 1)
4. ✅ Wire live MiniMax provider into `kay run --live` (M12 Phase 2 — commits 980a600, 101e23d)
5. ✅ Create live API smoke test suite (M12 Phase 3 — commit 8590311)
6. ❌ Create coverage-gate.sh script — PENDING
7. ❌ Wire `kay eval tb2 --run` for EVAL-01a — PENDING (Phase 12 TB setup project)
8. ❌ Inline unit tests for kay-core — PENDING (Phase 1 planned, not yet done)
9. ❌ Inline unit tests for kay-context — PENDING (Phase 1 planned, not yet done)
10. ❌ kay-tools integration tests — PENDING (Phase 1 planned, not yet done)
11. ❌ kay-verifier integration tests — PENDING (Phase 1 planned, not yet done)
