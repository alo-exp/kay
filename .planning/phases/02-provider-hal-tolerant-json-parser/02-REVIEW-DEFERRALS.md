---
phase: 02-provider-hal-tolerant-json-parser
deferred_at: 2026-04-20
review_path: .planning/phases/02-provider-hal-tolerant-json-parser/02-REVIEW.md
fix_report_path: (returned directly to parent agent; no REVIEW-FIX.md file per workflow)
deferred_findings:
  - IN-03
  - IN-04
---

# Phase 2: Code Review Deferrals

Two **Informational** findings from `02-REVIEW.md` are deliberately not
fixed in this review-fix pass. Both are minor DX/hygiene items with non-
trivial surface implications that deserve explicit planning rather than
drive-by patches.

## IN-03 — `MockServer::load_sse_cassette` panic contract

**File:** `crates/kay-provider-openrouter/tests/common/mock_server.rs:89-90`

**Reviewer recommendation:** "Move the helper into `#[cfg(test)]` or
document the panic contract."

**Why deferred:**

1. The `common/` module in integration tests is **intentionally** a
   regular module, not `#[cfg(test)]`-gated — it's compiled once per
   integration binary by `mod common;` declarations. Moving it under
   `#[cfg(test)]` would silently exclude it from every `tests/*.rs`
   binary (integration tests are themselves the `cfg(test)` scope, but
   the helper module participates in their normal compilation). That
   would break the mock helper for every downstream test.

2. The reviewer's own finding rates this as **Low priority** ("current
   posture is conventional"), noting the crate-wide lint forbids
   `.unwrap()` / `.expect()` but NOT `panic!()`, so the current code
   is clippy-clean. `panic!` inside a test helper is idiomatic — the
   crash IS the failure and the path includes the cassette filename
   for debugging.

3. Adding a doc comment noting "call this from a #[tokio::test];
   fixture load failure aborts the test" is defensible but the
   module's single user is the test file 12 lines away, where the
   call is obviously a cassette load. Noise-to-signal is unfavorable.

**Remediation path:** Optional future hygiene pass. If adopted, prefer
updating the doc comment (keep the panic) rather than moving to
`cfg(test)`.

## IN-04 — `ApiKey::from(String)` accepts empty-after-trim

**File:** `crates/kay-provider-openrouter/src/auth.rs:33-37`

**Reviewer recommendation:** Add a `TryFrom<String>` impl that rejects
empty-after-trim, have `OpenRouterProviderBuilder::api_key` delegate
through it.

**Why deferred:**

1. **Security impact is zero.** Empty key → 401 → `ProviderError::Auth
   { reason: AuthErrorKind::Invalid }` per the plan 02-10 classifier.
   The reviewer explicitly classifies this as "minor DX roughness",
   not a security issue.

2. The current `From<String>` impl carries a documented rationale
   (auth.rs:29-32): "Empty input is allowed here — the upstream HTTP
   call will surface a 401, mapped to `Auth::Invalid`." Changing the
   contract requires either:

   a. Breaking the `From` trait (API-compat cost; `From<String>` is
      public — Phase 3+ callers may depend on infallibility), or
   b. Adding a parallel `TryFrom<String>` AND routing
      `OpenRouterProviderBuilder::api_key(impl Into<String>)` through
      the fallible path — which itself becomes fallible, propagating
      a `Result` through the builder chain.

   Either option widens the public API surface right before Phase 3
   starts. The plan for the builder pattern across future phases
   (tool registry builder, agent-loop builder, TUI config builder)
   has not been set — picking a `TryFrom` pattern here without that
   alignment risks inconsistency.

3. `resolve_api_key` (the primary code path per D-08 precedence)
   **already** rejects empty-after-trim at resolution time. Only the
   test-oriented `.api_key("test-key")` bypass can smuggle empty
   strings today, and tests don't pass empty strings.

**Remediation path:** Bundle with Phase 3's first builder-shape
decision. If Phase 3 adopts `TryFrom` as the canonical builder pattern,
retrofit `ApiKey` accordingly. If it sticks with infallible `From` +
build-time validation (the current pattern), document the choice in
`.planning/PATTERNS.md` and close IN-04 as wont-fix.

## Summary

| Finding | Severity | Status | Remediation Phase |
|---------|----------|--------|-------------------|
| IN-03 | Informational | Deferred | Optional hygiene pass |
| IN-04 | Informational | Deferred | Phase 3 builder-pattern alignment |

All **Critical, High, Medium, and Low** findings were fixed in this
review-fix pass and committed atomically with DCO signoff. Informational
findings IN-01 (unused deps) and IN-02 (stale dead-code allow) were
also fixed since they were trivial and low-risk. Only IN-03 and IN-04
are deferred.

---

*Deferred: 2026-04-20*
*Fixer: Claude (gsd-code-fixer)*
