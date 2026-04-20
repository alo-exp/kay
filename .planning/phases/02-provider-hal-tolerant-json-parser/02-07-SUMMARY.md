---
phase: 02-provider-hal-tolerant-json-parser
plan: 07
subsystem: provider-hal
tags: [rust, allowlist, api-key-auth, tm-01-redaction, tm-04-charset, tm-08-exacto-wire, rust-2024-unsafe-env, test-env-mutex]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-06)
    provides: "ProviderError::ModelNotAllowlisted + ProviderError::Auth + AuthErrorKind variants; crate-root #![deny(clippy::unwrap_used, clippy::expect_used)] lint"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-01)
    provides: "tests/fixtures/config/allowlist.json — the 3-model D-07 launch allowlist fixture loaded by Allowlist::from_path"
provides:
  - "Allowlist gate (src/allowlist.rs) — from_path/from_models/with_env_override/check/to_wire_model/models; canonicalize + validate_charset private helpers"
  - "ApiKey newtype (src/auth.rs) — custom Debug redacts to `ApiKey(<redacted>)`; pub(crate) as_str() accessor only"
  - "ConfigAuthSource + resolve_api_key() — D-08 env-wins-over-config precedence"
  - "TM-04 charset validation — \\r \\n \\t and non-ASCII rejected pre-allowlist-compare with empty allowed list (don't leak allowlist to smuggler)"
  - "TM-08 wire-format discipline — to_wire_model always appends :exacto; canonicalize always strips it; idempotent under repeated calls"
  - "Pitfall 7 normalization — ascii-lowercase + trim + trim_end_matches(:exacto)"
affects: [plan-02-08, plan-02-09, plan-02-10, phase-03-tool-registry, phase-05-agent-loop]

# Tech tracking
tech-stack:
  added: []   # No new runtime deps; serde + serde_json + thiserror already present from 02-06
  patterns:
    - "module-static Mutex for serializing env-mutating tests under Rust's parallel test harness"
    - "Rust 2024 `unsafe { std::env::set_var/remove_var }` wrapping in test-only scope"
    - "let-chain (`if let ... && let ...`) for config fallback in resolve_api_key (Rust 2024)"
    - "poisoned-mutex recovery via `.unwrap_or_else(|e| e.into_inner())` — preserves crate-root deny(unwrap_used)"
    - "crate-private newtype (ApiKey) with pub(crate) accessor — enforces single-boundary key material use"
    - "custom Debug impl (not derive) on credential-holding types — structurally guarantees no leak via `{:?}`"

key-files:
  created:
    - "crates/kay-provider-openrouter/src/allowlist.rs — Allowlist (from_path/from_models/with_env_override/check/to_wire_model/models) + ConfigFileShape (private) + 8 unit tests"
    - "crates/kay-provider-openrouter/src/auth.rs — ApiKey newtype + ConfigAuthSource + resolve_api_key() + 6 unit tests"
    - "crates/kay-provider-openrouter/tests/allowlist_gate.rs — 6 integration tests (launch-allowlist, reject-random, wire-exacto, CRLF-smuggling-reject, mixed-case-accept, exacto-input-accept)"
    - "crates/kay-provider-openrouter/tests/auth_env_vs_config.rs — 4 integration tests (missing-everywhere, env-wins-conflict, config-fallback, debug-no-leak-in-error)"
  modified:
    - "crates/kay-provider-openrouter/src/lib.rs — added `mod allowlist; mod auth;` + pub use re-exports (Allowlist, ConfigAuthSource, resolve_api_key); ApiKey intentionally NOT re-exported"

key-decisions:
  - "Rust 2024 unsafe-env mutation — wrapped all std::env::set_var/remove_var calls in `unsafe { }` blocks (Rule-3 auto-fix). Edition 2024 reclassified these as unsafe due to cross-thread data-race potential; this is a compile-time requirement, not a design choice."
  - "Module-static Mutex for test env serialization — cargo's default intra-binary test parallelism + process-global env = races. `static ENV_LOCK: Mutex<()> = Mutex::new(())` serializes env-mutating tests per binary. Preferred over serial_test dep (avoids adding a dev-dep for a 3-line solution). Poisoned-lock recovery via `unwrap_or_else(|e| e.into_inner())` keeps the crate-root `#![deny(clippy::unwrap_used)]` happy (Rule-3 auto-fix)."
  - "Clippy collapsible_if → let-chain — `if let Some(src) = config { if let Some(ref key) = src.api_key { ... } }` collapsed to `if let Some(src) = config && let Some(ref key) = src.api_key { ... }` per Rust 2024 let-chain stabilization (Rule-1 fix, required by `-D warnings`)."
  - "`#[allow(clippy::expect_used, clippy::unwrap_used)]` on the auth test module — the crate-root `#![deny(...)]` propagates into `#[cfg(test)]` modules, unlike the common assumption. Explicit module-level allow is the canonical workaround; applied only to test modules."
  - "TM-04 empty-allowed-list on charset rejection is intentional — validate_charset returns `allowed: Vec::new()` so a caller smuggling CRLF gets no allowlist hint back. Preserves the 'TM-04 response tells them nothing' posture vs. unknown-model rejection which returns the real allowlist for diagnostic UX."
  - "No 'schemars' bleed-through — allowlist file is deserialized via serde only; no schemars boundary expansion (matches 02-06's ToolSchema decision)."

patterns-established:
  - "Env-mutating test pattern for Rust 2024: (1) module-static `Mutex<()>` in the test module, (2) `let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner())` at test top, (3) wrap env calls in `unsafe { }`. Repeat per integration test binary (they are distinct processes? No — distinct binaries, same process context for that run, but separate invocations across `cargo test` have no shared state, so intra-binary serialization is what matters)."
  - "Credential redaction discipline: newtype + custom Debug impl + no Display impl + pub(crate) accessor only. Four barriers between a credential and stdout. TM-01 guaranteed structurally (a `println!(\"{key}\")` fails to compile; `println!(\"{key:?}\")` prints `ApiKey(<redacted>)`; an error carrying the type never leaks via `{:?}`)."
  - "TM-04 'smuggler gets empty hint' convention: allowlist-compare rejection returns the real allowed list for legitimate UX; charset rejection returns an empty list so an attacker probing for control-char handling learns nothing about allowlist contents."
  - "Post-2.5 crate-self-contained plan execution: plan 02-07 had zero forge_* or kay_core imports. Appendix A Rule 2 (use-path substitution) was applicable but not exercised — auth + allowlist both confine themselves to std + serde + crate::error. Documented as a 'no substitutions triggered' Appendix A deviation for traceability."

requirements-completed: [PROV-03, PROV-04]

# Metrics
duration: 6min
completed: 2026-04-20
---

# Phase 02 Plan 07: Allowlist Gate + API-Key Auth Summary

**Two sibling pre-HTTP gates shipped — `Allowlist::check` (PROV-04) rejects non-allowlisted models with `ProviderError::ModelNotAllowlisted` (including `\r\n\t` / non-ASCII smuggle attempts per TM-04), and `resolve_api_key` (PROV-03) resolves `OPENROUTER_API_KEY` env-wins-over-config credentials into an `ApiKey` newtype whose custom `Debug` redacts to `ApiKey(<redacted>)` always (TM-01). `to_wire_model` always appends `:exacto` (TM-08, Pitfall 8). All 26 tests green, clippy `-D warnings` clean.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-04-20T03:25:12Z
- **Completed:** 2026-04-20T03:31:11Z
- **Tasks:** 2
- **Files created:** 4 (allowlist.rs, auth.rs, allowlist_gate.rs, auth_env_vs_config.rs)
- **Files modified:** 1 (lib.rs — module decls + re-exports)

## Accomplishments

- **Allowlist gate (PROV-04, AC-07, D-07)**: `Allowlist::from_path` loads the D-07 launch-allowlist fixture (anthropic/claude-sonnet-4.6, anthropic/claude-opus-4.6, openai/gpt-5.4). `check` returns `Ok(())` for canonical-form matches or any mixed-case / `:exacto`-suffixed variant; returns `Err(ProviderError::ModelNotAllowlisted)` otherwise. `KAY_ALLOWED_MODELS` env override works; empty env string treated as unset.
- **Charset validation (TM-04)**: `\r`, `\n`, `\t`, and non-ASCII characters rejected BEFORE the allowlist compare, with `allowed: Vec::new()` in the error (smuggler doesn't learn the allowlist). CRLF-injection attack vector (`"anthropic/claude-sonnet-4.6\r\nX-Evil: 1"`) confirmed blocked by integration test.
- **Wire-format discipline (TM-08, Pitfall 8)**: `to_wire_model` always appends `:exacto`; `canonicalize` always strips it. Idempotent under re-invocation. User-facing canonical IDs never contain `:exacto`; wire IDs always do.
- **API key auth (PROV-03, D-08)**: `resolve_api_key` checks `OPENROUTER_API_KEY` env (wins if non-empty) then `ConfigAuthSource.api_key` then returns `ProviderError::Auth { reason: AuthErrorKind::Missing }`. Empty strings / whitespace-only values treated as unset.
- **TM-01 credential redaction**: `ApiKey` has a custom `Debug` (not derived) returning fixed `"ApiKey(<redacted>)"`; no `Display` impl (so `println!("{k}")` fails to compile at the type level). Only `pub(crate) as_str()` surfaces the raw bytes — and ApiKey itself is NOT re-exported through `lib.rs`, so even that accessor is invisible to downstream crates.
- **26 tests green** — 14 unit (8 allowlist + 6 auth) + 10 integration (6 allowlist_gate + 4 auth_env_vs_config). Plus the pre-existing 2 error-unit tests from 02-06 still pass (total crate lib = 16; integration = 10).
- **Clippy `-D warnings` clean** on all targets (including tests).
- **Workspace governance preserved**: `cargo check --workspace --all-features` exits 0; `tests/governance/check_attribution.sh` exits 0.

## Task Commits

Each task committed atomically with DCO sign-off (`git commit -s`):

1. **Task 1: Allowlist gate + normalization + charset validation + integration test** — `0b4a8c1` (feat). 8 unit tests + 6 integration tests green.
2. **Task 2: API key resolution + TM-01 redaction + integration test** — `f3586e8` (feat). 6 unit tests + 4 integration tests green.

Both commits include `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`. No `--no-verify` used. Hooks passed on each commit.

## Files Created/Modified

- `crates/kay-provider-openrouter/src/allowlist.rs` (233 lines) — Allowlist struct + 8 unit tests covering canonicalize (lowercase/trim/strip-exacto), check (accept canonical+mixed-case, reject unknown, reject CRLF+tabs, reject non-ASCII), to_wire_model (idempotent Exacto suffix), with_env_override (replace-base / empty-leaves-alone). Module-static `ENV_LOCK: Mutex<()>` serializes env-mutating tests.
- `crates/kay-provider-openrouter/src/auth.rs` (164 lines) — ApiKey (custom Debug redact) + ConfigAuthSource + resolve_api_key() + 6 unit tests (env-wins, config-fallback, empty-env-treated-unset, missing-everywhere→typed-error, empty-config-missing→Missing, debug_redacts). Uses Rust 2024 let-chain for config fallback.
- `crates/kay-provider-openrouter/tests/allowlist_gate.rs` (66 lines) — 6 integration tests against `tests/fixtures/config/allowlist.json`.
- `crates/kay-provider-openrouter/tests/auth_env_vs_config.rs` (79 lines) — 4 integration tests covering D-08 precedence + TM-01 no-leak-in-error.
- `crates/kay-provider-openrouter/src/lib.rs` — added `mod allowlist; mod auth;` + `pub use allowlist::Allowlist;` + `pub use auth::{ConfigAuthSource, resolve_api_key};` + comment explaining why `ApiKey` is intentionally NOT re-exported.

## Decisions Made

- **Rust 2024 `unsafe { std::env::* }` blocks for test env mutation** — edition 2024 reclassified `std::env::set_var` / `remove_var` as unsafe due to data-race potential across threads. Wrapping in `unsafe {}` blocks (test-only scope) is the canonical compile-time requirement, not a design choice. Plan 02-07's source code itself has zero env mutation.
- **Module-static Mutex for test env serialization** — `static ENV_LOCK: Mutex<()> = Mutex::new(())` at module scope. Each env-mutating test does `let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())`. Chosen over `serial_test` crate to avoid adding a dev-dep for a 3-line solution. Poisoned-lock recovery via `unwrap_or_else(|e| e.into_inner())` keeps the crate-root `#![deny(clippy::unwrap_used)]` happy.
- **`#[allow(clippy::expect_used, clippy::unwrap_used)]` on auth's unit test module** — the crate-root `#![deny(...)]` propagates through `#[cfg(test)]` modules contrary to a common assumption. Explicit module-level allow is the clean fix (no crate-wide softening). Applied only to the auth test module since the allowlist test module happens to use `unwrap_or_else` only.
- **Let-chain (`&&`) in `resolve_api_key`** — collapsed nested `if let Some(src) = config { if let Some(ref key) = src.api_key { ... } }` into `if let Some(src) = config && let Some(ref key) = src.api_key { ... }`. Rust 2024 stabilization. Clippy `collapsible_if` required a fix; the `&&`-chain form is the idiomatic choice over a `.map().filter()` Option chain here (preserves the trimmed-empty check at the innermost scope).
- **TM-04 empty-allowed-list on charset rejection** — validate_charset returns `allowed: Vec::new()` so a caller probing for control-char handling learns nothing about allowlist contents. Unknown-model rejection DOES return the real allowlist for legitimate diagnostic UX. Asymmetric by design.
- **ApiKey crate-private (no re-export)** — downstream crates (kay-cli, kay-tauri, kay-tui) never see the raw type. They only see `resolve_api_key() -> Result<ApiKey, ProviderError>` and use the returned value opaquely. Even `pub(crate) as_str()` is internal to kay-provider-openrouter; plan 02-08's OpenRouterProvider impl is the sole intended consumer.
- **No schemars boundary expansion** — allowlist JSON is plain serde; matches 02-06's ToolSchema `serde_json::Value` decision. Phase 3 TOOL-05 owns schemars introduction.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Rust 2024 unsafe `std::env::*` mutations**

- **Found during:** Task 1 first `cargo test` compilation
- **Issue:** Rust 2024 edition reclassified `std::env::set_var` and `std::env::remove_var` as `unsafe` functions due to cross-thread data-race potential. Plan text used them without `unsafe {}` wrappers; compile error E0133.
- **Fix:** Wrapped every `std::env::set_var` / `std::env::remove_var` call in `unsafe { }` blocks with SAFETY comments. Applied to both unit tests (allowlist + auth) and the auth integration test. Source code itself has zero env mutation.
- **Files modified:** `crates/kay-provider-openrouter/src/allowlist.rs`, `crates/kay-provider-openrouter/src/auth.rs`, `crates/kay-provider-openrouter/tests/auth_env_vs_config.rs`
- **Verification:** `cargo test -p kay-provider-openrouter` now compiles; all 26 tests pass.
- **Committed in:** `0b4a8c1` (Task 1) and `f3586e8` (Task 2)

**2. [Rule 1 - Bug] Test env contamination — cross-test races**

- **Found during:** Task 1 second `cargo test` run (after Rule-3 fix)
- **Issue:** `env_override_replaces_base_list` failed with `assertion failed: a.check("fake/model-a").is_ok()`. Root cause: cargo's test harness parallelizes intra-binary; two env-mutating tests race on process-global `KAY_ALLOWED_MODELS`. The empty-string test was removing the var between the set and the `with_env_override()` read in the other test.
- **Fix:** Added `static ENV_LOCK: Mutex<()> = Mutex::new(())` at module scope in both unit test modules. Each env-mutating test acquires the lock via `let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())`. Serializes env-mutating tests per binary. Poisoned-lock recovery ensures `#![deny(clippy::unwrap_used)]` compliance.
- **Files modified:** `crates/kay-provider-openrouter/src/allowlist.rs`, `crates/kay-provider-openrouter/src/auth.rs`, `crates/kay-provider-openrouter/tests/auth_env_vs_config.rs`
- **Verification:** Tests run repeatably. All 26 tests pass on every invocation.
- **Committed in:** `0b4a8c1` and `f3586e8`

**3. [Rule 1 - Bug] Clippy `collapsible_if` on nested `if let` in resolve_api_key**

- **Found during:** Task 2 `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings`
- **Issue:** Plan's literal code had `if let Some(src) = config { if let Some(ref key) = src.api_key { ... } }`. Clippy's `collapsible_if` lint fires under `-D warnings`, blocking the commit.
- **Fix:** Collapsed to a let-chain: `if let Some(src) = config && let Some(ref key) = src.api_key { let trimmed = key.trim(); if !trimmed.is_empty() { ... } }`. Rust 2024 stabilized let-chains, so this is an idiomatic shape.
- **Files modified:** `crates/kay-provider-openrouter/src/auth.rs`
- **Verification:** Clippy exits 0.
- **Committed in:** `f3586e8`

**4. [Rule 1 - Bug] `#![deny(clippy::expect_used)]` propagates through `#[cfg(test)]`**

- **Found during:** Task 2 `cargo clippy -p kay-provider-openrouter --all-targets`
- **Issue:** Crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` denies tests' `.expect("should resolve")` calls in the auth unit-test module. Contrary to the lib.rs comment "tests override with module-level attributes" — the crate-root deny still fires unless each test module explicitly allows.
- **Fix:** Added `#[allow(clippy::expect_used, clippy::unwrap_used)]` to the `mod unit` inside `auth.rs`. Allowlist's unit test module happens to use only `unwrap_or_else` so it didn't need the allow; applied only where needed.
- **Files modified:** `crates/kay-provider-openrouter/src/auth.rs`
- **Verification:** Clippy exits 0; tests still use `.expect(...)` for diagnostic messages.
- **Committed in:** `f3586e8`

**5. [Rule 2 - Post-2.5 Realignment, Appendix A Rule 2] No-op — no `kay_core::forge_*` imports introduced**

- **Found during:** Task 1/2 design review
- **Issue:** Plan 02-07's module surface is entirely self-contained within `kay-provider-openrouter` — uses only `std`, `serde`, and intra-crate `use crate::error::...`. Appendix A Rule 2 (`use kay_core::forge_X::Y` → `use forge_X::Y`) is applicable but not exercised.
- **Fix:** No substitutions needed. Plan 02-07 is a pure-internal HAL boundary extension; forge_* import substitutions fire in plan 02-08 when the OpenRouterProvider impl assembles.
- **Files modified:** none (preemptive documentation)
- **Verification:** `grep -rn "use kay_core" crates/kay-provider-openrouter/src/` → 0 matches. `grep -rn "use forge_" crates/kay-provider-openrouter/src/` → 0 matches.
- **Committed in:** N/A (documentation-only in this SUMMARY)

---

**Total deviations:** 5 auto-fixed (4 Rule-1/3 bug-fixes forced by Rust 2024 edition changes and clippy-under-warnings-deny; 1 Rule-2 post-2.5 realignment no-op). No Rule-4 architectural decisions required. No scope creep.

## Issues Encountered

- **Rust 2024 unsafe-env reclassification** surfaced at first compile. Known Rust change (RFC 3543 / Rust 1.80+); plan text predates the lockdown. Handled via mechanical `unsafe { }` wrapping with SAFETY comments.
- **Test parallelism race** surfaced at second test run. Mitigated with module-static `Mutex<()>` serialization. No flakes observed after fix across ~10 repeated runs.
- **Clippy's expect-used propagation through `cfg(test)`** — the comment in lib.rs ("tests override with module-level attributes") was true but required an explicit action per test module, not automatic. Added the allow where needed (auth test module).

## User Setup Required

None — plan 02-07 adds pure-code scaffolding. No runtime configuration, no credentials needed, no services to provision. `KAY_ALLOWED_MODELS` env override and `OPENROUTER_API_KEY` env var are plumbed but not yet consumed by any HTTP path (plan 02-08 wires them in).

## Threat Flags

No new threat flags. The four files added confine security-relevant surface to the threat register's existing TM-01 / TM-04 / TM-08 threats — all are mitigated with explicit tests:

- **TM-01** (API key leakage via Debug): `ApiKey`'s custom Debug impl returns the literal `"ApiKey(<redacted>)"`. Unit test `auth::unit::debug_redacts_key_material` asserts verbatim output. Integration test `debug_never_leaks_credential_in_error_display` asserts `ProviderError::Auth { reason: ... }` display does not contain `"sk-"`.
- **TM-04** (control-char / non-ASCII smuggling): `validate_charset` rejects `\r \n \t` and non-ASCII before allowlist compare. Unit tests `check_rejects_crlf_and_tabs` + `check_rejects_non_ascii` + integration test `crlf_smuggling_rejected_before_allowlist_compare` assert.
- **TM-08** (Exacto wire-suffix divergence): `to_wire_model` always appends `:exacto`; `canonicalize` always strips it. Tests `to_wire_model_always_appends_exacto` + `wire_model_always_has_exacto_suffix` + `exacto_suffix_input_accepted_and_canonicalized` assert.
- **T-02-07-04** (panic via unwrap on env): `resolve_api_key` pattern-matches `std::env::var(...).ok()` via `if let Ok(...)`; empty/whitespace routes to `Missing`. No `.unwrap()`. Crate-wide `#![deny(clippy::unwrap_used)]` enforces.

## Next Phase Readiness

- **Plan 02-08** (OpenRouterProvider streaming impl) can now compose:
  - `Allowlist::check` as a pre-flight gate (returns `ProviderError::ModelNotAllowlisted`)
  - `Allowlist::to_wire_model` to construct the on-wire model ID with `:exacto`
  - `resolve_api_key` to get an `ApiKey`, then `ApiKey::as_str()` inside the crate's HTTP module to build the Authorization header.
- **Plans 02-09 / 02-10** inherit both modules transparently — they own the tolerant JSON parser and retry/cost-cap layer above these two gates.
- **Contract invariants locked** for the full Phase 2:
  - No non-allowlisted HTTP request ever fires (TM-04 + PROV-04).
  - No API key ever renders through `{:?}` (TM-01).
  - Wire format always has `:exacto` suffix (TM-08).

## Self-Check: PASSED

Verification performed after SUMMARY draft:

**Files exist:**
- FOUND: crates/kay-provider-openrouter/src/allowlist.rs
- FOUND: crates/kay-provider-openrouter/src/auth.rs
- FOUND: crates/kay-provider-openrouter/tests/allowlist_gate.rs
- FOUND: crates/kay-provider-openrouter/tests/auth_env_vs_config.rs
- FOUND: crates/kay-provider-openrouter/src/lib.rs (modified — allowlist + auth modules declared)

**Commits exist (verified via `git log --oneline --all | grep`):**
- FOUND: 0b4a8c1 (Task 1 — allowlist gate + integration test)
- FOUND: f3586e8 (Task 2 — auth + integration test)

**Acceptance gates (verified during execution):**
- PASS: `cargo test -p kay-provider-openrouter --lib allowlist::unit` — 8 tests pass
- PASS: `cargo test -p kay-provider-openrouter --lib auth::unit` — 6 tests pass
- PASS: `cargo test -p kay-provider-openrouter --test allowlist_gate` — 6 tests pass
- PASS: `cargo test -p kay-provider-openrouter --test auth_env_vs_config` — 4 tests pass
- PASS: `cargo test -p kay-provider-openrouter` (full) — 16 lib + 6 allowlist_gate + 4 auth_env_vs_config = 26 tests pass
- PASS: `cargo clippy -p kay-provider-openrouter --all-targets -- -D warnings` (0 warnings)
- PASS: `cargo check --workspace --all-features` (0 exclusions, clean)
- PASS: `tests/governance/check_attribution.sh` (ALL INVARIANTS PASS)
- PASS: shape checks — `pub struct Allowlist`, `pub struct ApiKey`, `ApiKey(<redacted>)` literal, `pub use allowlist::Allowlist`, `pub use auth::{ConfigAuthSource, resolve_api_key}`, NO `pub use auth::*ApiKey`, NO `#[derive(Debug)]` on ApiKey, `pub(crate) fn as_str`

---
*Phase: 02-provider-hal-tolerant-json-parser*
*Plan: 07*
*Completed: 2026-04-20*
