---
phase: 02-provider-hal-tolerant-json-parser
review_date: 2026-04-20
depth: standard
files_scoped: 21
files_scoped_list:
  - crates/kay-provider-openrouter/Cargo.toml
  - crates/kay-provider-openrouter/src/lib.rs
  - crates/kay-provider-openrouter/src/allowlist.rs
  - crates/kay-provider-openrouter/src/auth.rs
  - crates/kay-provider-openrouter/src/client.rs
  - crates/kay-provider-openrouter/src/cost_cap.rs
  - crates/kay-provider-openrouter/src/error.rs
  - crates/kay-provider-openrouter/src/event.rs
  - crates/kay-provider-openrouter/src/openrouter_provider.rs
  - crates/kay-provider-openrouter/src/provider.rs
  - crates/kay-provider-openrouter/src/retry.rs
  - crates/kay-provider-openrouter/src/tool_parser.rs
  - crates/kay-provider-openrouter/src/translator.rs
  - crates/kay-provider-openrouter/tests/allowlist_gate.rs
  - crates/kay-provider-openrouter/tests/auth_env_vs_config.rs
  - crates/kay-provider-openrouter/tests/cost_cap_turn_boundary.rs
  - crates/kay-provider-openrouter/tests/error_taxonomy.rs
  - crates/kay-provider-openrouter/tests/retry_429_503.rs
  - crates/kay-provider-openrouter/tests/streaming_happy_path.rs
  - crates/kay-provider-openrouter/tests/tool_call_malformed.rs
  - crates/kay-provider-openrouter/tests/tool_call_reassembly.rs
  - crates/kay-provider-openrouter/tests/common/mock_server.rs
findings:
  critical: 0
  high: 0
  medium: 2
  low: 4
  informational: 4
  total: 10
status: issues_found
verdict: issues_found
---

# Phase 2: Code Review Report

**Reviewed:** 2026-04-20
**Depth:** standard
**Files Scoped:** 21 (12 src + 8 integration tests + 1 shared mock_server + Cargo.toml)
**Verdict:** **issues_found** (no critical/high — 2 medium, 4 low, 4 informational)

## Executive Summary

Phase 2's `kay-provider-openrouter` crate is in strong shape. The never-panic invariant is structurally enforced (crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]`), every public enum carries `#[non_exhaustive]`, the `ApiKey` Debug redaction is airtight, the control-char / CRLF smuggling defense on model IDs is correctly placed pre-allowlist, the two-pass tolerant parser is proptest-fuzzed for never-panic, the 1 MiB tool-args cap is wired with empty-raw eviction to avoid back-channeling large payloads through `AgentEvent`, Mutex poison recovery on `CostCap` is graceful and clippy-clean, and the `reqwest_eventsource::retry::Never` policy plus `backon`-only retry orchestration correctly neutralizes the 3×3=9 amplification risk (TM-09). No security-relevant bugs, no API-key leakage paths, no panic escape routes. The issues found are localized: one stale doc comment that contradicts reality and will mislead future readers (Medium); one latent data-loss path in `open_and_probe`'s first-Message defensive branch that silently drops the first SSE message (Medium, defensive-only branch so never exercised today but would surprise under provider variance); a small index-mapping edge case in `resolve_call_id`; four unused workspace deps that should be pruned; and a handful of minor stylistic / doc improvements. Nothing blocks closeout.

## Critical Issues

None.

## High Issues

None.

## Medium Issues

### MD-01: Stale doc comment claims `serde_json/preserve_order` is enabled when the whole `OrderedObject` scaffolding exists precisely because it isn't

**File:** `crates/kay-provider-openrouter/src/openrouter_provider.rs:26-30`
**Category:** quality (documentation / factual drift)

**Issue:**
The module-level doc comment reads:

```rust
//! **NN-7 ordering is enforced locally** —
//! serde_json is compiled with `preserve_order`, and `build_request_body`
//! inserts `required` BEFORE `properties` for every tool's parameters
//! schema. The `nn7` test asserts this on every PR.
```

But the Cargo.toml `indexmap` comment (line 24-29) and the internal doc comments on `reorder_tool_parameters` (lines 345-352) and `OrderedObject` (lines 444-457) all correctly explain the OPPOSITE: `serde_json/preserve_order` is deliberately OFF at the workspace level (so it does not leak into `forge_app` and trip a `clippy::large_enum_variant` threshold that would violate parity). NN-7 ordering is instead enforced by a custom `Serialize` impl + `IndexMap`-backed `OrderedObject`. The line 28 statement is factually wrong and contradicts the body of the module.

**Evidence:**
```rust
// Line 26-30 (WRONG):
//! NN-7 ordering is enforced locally —
//! serde_json is compiled with `preserve_order`, and `build_request_body`
//! inserts `required` BEFORE `properties` for every tool's parameters
//! schema. The `nn7` test asserts this on every PR.

// Line 345-352 (CORRECT):
/// NN-7 enforcement uses `indexmap::IndexMap` + a custom `Serialize` impl
/// (`OrderedObject`) that iterates keys in insertion order. This keeps
/// `serde_json/preserve_order` OFF at the workspace level — enabling it
/// would flip upstream `forge_app::dto::openai::error::Error` across the
/// `clippy::large_enum_variant` threshold (IndexMap is larger than BTreeMap),
/// and we MUST NOT patch upstream forge_code code per parity.
```

**Why it matters:** A future maintainer reading the file top-down will be told `preserve_order` is on, search the workspace, discover it isn't, and spend time assuming the comment or the code is wrong. This is exactly the kind of load-bearing parity constraint (non-negotiable 7) that must not be miscommunicated — tripping it later via "fix the comment by enabling the feature" would silently violate parity.

**Recommended fix:** Rewrite lines 26-30 to match reality:

```rust
//! **NN-7 ordering is enforced locally** via a custom `Serialize` impl on
//! `OrderedObject` (IndexMap-backed). `serde_json/preserve_order` stays OFF
//! at the workspace level (enabling it would flip upstream
//! `forge_app::dto::openai::error::Error` past `clippy::large_enum_variant`
//! and break parity). `build_request_body` inserts `required` BEFORE
//! `properties` for every tool's parameters schema. The `nn7` test asserts
//! this on every PR.
```

---

### MD-02: `open_and_probe` silently drops the first SSE Message when Open is skipped

**File:** `crates/kay-provider-openrouter/src/openrouter_provider.rs:308-315`
**Category:** bug (latent data loss on defensive branch)

**Issue:**
The `open_and_probe` probe pulls one event off the EventSource and, when that first event is `Event::Message(_)` rather than `Event::Open`, accepts it as "equivalent to Open" and hands the EventSource to the translator. But the matched message's payload is discarded via the `_` binding — the translator's first `.next().await` will see the SECOND message, not the first. The inline comment says "letting the translator see that message next" which is incorrect; the event has already been consumed off the stream.

**Evidence:**
```rust
Some(Ok(Event::Message(_))) => {
    // OpenRouter + reqwest_eventsource normally delivers Open before
    // any Message. Tolerate the rare ordering variation by treating
    // first-Message as equivalent to Open (and letting the translator
    // see that message next). This branch is defensive — no known
    // OpenRouter path produces it.
    Ok(es)
}
```

Because the binding is `Event::Message(_)`, the `MessageEvent` with its `data` payload is dropped. If OpenRouter ever sends the first chunk (e.g., a text delta or tool_call start) before Open — or if the upstream proxy reorders — that first delta is lost.

**Why it matters:** The comment correctly notes "no known OpenRouter path produces it," so exposure today is nil. But the branch exists specifically for defense in depth, and in its current form it silently corrupts the stream instead of defending it. The plan for this branch to ever fire without corrupting results is exactly the wrong disposition: either we don't need the branch (remove it, let the `None` / error branches handle unexpected orderings), or we need to capture and re-yield the message.

**Recommended fix (preferred — make the branch actually correct):**
Capture the Message and prepend it to the stream via `stream::once` + `chain`, or change `open_and_probe` to return `(EventSource, Option<Event>)`:

```rust
Some(Ok(Event::Message(msg))) => {
    // Prepend the consumed message so the translator still sees it.
    let captured = futures::stream::iter(vec![Ok(Event::Message(msg))]);
    // Caller reconstructs by chaining captured.chain(es). Requires
    // refactoring open_and_probe's return type.
    ...
}
```

Or, if we want to keep the return type simple, change the match to return an error for the Message-first case (it is not actually observed in the wild, and returning a typed error is strictly better than silent data loss):

```rust
Some(Ok(Event::Message(_))) => Err(ProviderError::Stream(
    "unexpected Message before Open; refusing to proceed with lossy state".into(),
)),
```

**Secondary fix:** If keeping the current behavior, update the comment to reflect reality: the first message IS consumed; this branch chooses "silent loss of one event" over "refuse to start" as the tolerance strategy. That's a documented tradeoff, not a drive-by branch.

## Low Issues

### LO-01: `resolve_call_id` uses `index_to_id.len() as u32` as the synthesized index, which can collide with existing entries

**File:** `crates/kay-provider-openrouter/src/translator.rs:153`
**Category:** bug (corner-case correctness)

**Issue:**
When a tool-call delta arrives with `id` present but `index` absent (Anthropic-via-OpenRouter edge case noted in verification §Latent bugs), the code synthesizes an index via `index_to_id.len() as u32`. If the existing `index_to_id` map has a hole — e.g., already contains index 0 and index 2 (`len()==2`) — the synthesized slot for the third entry is 2, which collides with the existing mapping and overwrites it. A subsequent `index`-only delta that targeted the original index-2 tool call will now resolve to the wrong id.

**Evidence:**
```rust
if let Some(id) = &tc.id {
    let idx = tc.index.unwrap_or(index_to_id.len() as u32);
    index_to_id.insert(idx, id.clone());
    return Some(id.clone());
}
```

If indices `{0, 2}` are already registered and an id-only chunk arrives:
- `index_to_id.len() == 2`
- `idx = 2`
- `insert(2, new_id)` — clobbers the previous index-2 mapping.

**Why it's low, not medium:** Current OpenRouter traces do not produce sparse index sequences; the fragmented-tool-call cassette in tests always uses contiguous indices starting at 0. And the re-keying on id (lookups happen by call_id, not by index) means downstream events are usually routed correctly — the damage is limited to subsequent `index`-only lookups for the displaced id. Nonetheless, this is subtle and worth tightening.

**Recommended fix:** Use a free-slot scan rather than `len()`:

```rust
if let Some(id) = &tc.id {
    let idx = tc.index.unwrap_or_else(|| {
        (0u32..).find(|i| !index_to_id.contains_key(i)).unwrap_or(0)
    });
    index_to_id.insert(idx, id.clone());
    return Some(id.clone());
}
```

Note: the `unwrap_or(0)` only fires when indices `0..=u32::MAX` are all occupied, which is a theoretical impossibility; but the crate-wide lint forbids `.unwrap()` in production — refactor to use a loop + `Option::map`:

```rust
let idx = match tc.index {
    Some(i) => i,
    None => {
        let mut i = 0u32;
        while index_to_id.contains_key(&i) {
            i = i.saturating_add(1);
        }
        i
    }
};
```

---

### LO-02: `AgentEvent::Retry.attempt` semantics are ambiguous — is it "attempt that failed" or "attempt we're about to retry"?

**File:** `crates/kay-provider-openrouter/src/event.rs:55-61` (definition); `crates/kay-provider-openrouter/src/openrouter_provider.rs:262-266` (emission site)
**Category:** quality (API clarity)

**Issue:**
`AgentEvent::Retry { attempt, delay_ms, reason }` has no doc explaining whether `attempt` is:
- the 1-based index of the attempt that JUST failed, or
- the 1-based index of the upcoming retry attempt.

The implementation increments `attempt` at the top of the loop BEFORE firing, so an error on the first call produces `attempt == 1` in the Retry frame — meaning "attempt 1 failed, now waiting before attempt 2". The unit test asserts `*attempt == 1` for that case, so the contract is internally consistent, but a consumer reading just the enum variant could easily read it as "this is the attempt number that will be retried." Given that retry counters in UIs typically say "retry attempt 2 of 3," the ambiguity is likely to cause UI off-by-one bugs in Phase 9 (TUI) and Phase 10 (Tauri).

**Recommended fix:** Add a doc comment disambiguating:

```rust
/// A retryable upstream error is being retried after `delay_ms`. Emitted
/// BEFORE the backoff sleep so UIs can show progress.
///
/// `attempt` is the 1-based index of the attempt that JUST FAILED (so
/// `attempt == 1` means the first try errored and the wrapper is about to
/// sleep `delay_ms` before the second try). This matches `backon`'s
/// `max_times` semantics.
Retry {
    attempt: u32,
    delay_ms: u64,
    reason: RetryReason,
},
```

---

### LO-03: `delay.as_millis() as u64` silently truncates

**File:** `crates/kay-provider-openrouter/src/openrouter_provider.rs:264`
**Category:** quality (potential silent truncation)

**Issue:**
`delay.as_millis()` returns `u128`. The cast `as u64` silently truncates if the value exceeds `u64::MAX` milliseconds. In practice, `default_backoff` caps delay at 8 seconds and Retry-After integer-seconds parse through `u64`, so overflow is impossible today. But the truncation is unchecked — if somebody later raises `max_delay` or changes the Retry-After parser, the lossy cast hides the problem.

**Evidence:**
```rust
pre_events.push(AgentEvent::Retry {
    attempt,
    delay_ms: delay.as_millis() as u64,
    reason,
});
```

**Recommended fix:**
```rust
delay_ms: u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
```

Or, since `Duration::as_millis` on a `Duration` built from `u64` seconds never exceeds `u64`, add an assertion / `#[allow]` with rationale. The clippy lint `cast_possible_truncation` would catch this if raised; current config may permit it silently.

---

### LO-04: `tool_call_malformed` integration test does not assert the cassette actually exercises the Malformed branch

**File:** `crates/kay-provider-openrouter/tests/tool_call_malformed.rs:41` (test body)
**Category:** testability (test-intent drift risk)

**Issue:**
The test accepts either `ToolCallComplete` (repaired) OR `ToolCallMalformed` as valid outcomes — the cassette happens to be `{cmd: "ls," }` which `forge_json_repair` handles successfully today, so in practice the test always hits the Repaired path and never the Malformed path. That means the Malformed emission code (translator.rs:296-307) has no integration-level test coverage; if a future refactor breaks the malformed branch (e.g., emits `Err(ProviderError)` again, accidentally reverting the plan 02-09 upgrade), this test would still pass because the Repaired path alone satisfies the assertions.

**Evidence:** `translator.rs:296-307` is the critical never-panic-continue-stream invariant; the test accepts its non-execution.

**Recommended fix:** Add a second cassette `tool_call_catastrophic.jsonl` whose arguments are one of the known-Malformed-yielding inputs (`{{}}}`, `null null null`, etc.) and a second test that asserts exactly `ToolCallMalformed` + stream continuation. The `tool_parser::unit::catastrophic_input_malformed` test documents exactly which inputs reliably reach the Malformed branch.

## Informational

### IN-01: Four workspace dependencies are declared but unused

**File:** `crates/kay-provider-openrouter/Cargo.toml:12-15, 23, 31, 41`
**Category:** quality (dependency hygiene)

**Issue:** grep-verified that these dependencies are not referenced from any `src/` file:

- `forge_domain` (line 12)
- `forge_config` (line 13)
- `forge_services` (line 14)
- `forge_repo` (line 15)
- `anyhow` (line 23)
- `tracing` (line 31)
- `tokio-stream` (line 41)

`forge_json_repair` IS used (tool_parser.rs:26), so not every `forge_*` is unused. The others appear to be scaffolding from Phase 2.5 sub-crate split and never got pruned.

**Why it matters:** Unused deps slow builds, enlarge lockfile surface, and carry drift risk — `cargo udeps` would flag these. Not a bug; just hygiene.

**Recommended fix:** Remove them, or if retained intentionally for Phase 3, add a comment: `# Placeholder for Phase 3 wire-up; pin here to avoid lockfile churn when that lands.`

---

### IN-02: `#![allow(dead_code)]` at crate root is broader than needed

**File:** `crates/kay-provider-openrouter/src/lib.rs:19`
**Category:** quality

**Issue:** The comment says "Allow unused until plans 02-07 through 02-10 wire everything up." Phase 2 is now closed (all 10 plans shipped). The crate-wide `#![allow(dead_code)]` is no longer serving its original purpose and will mask future real dead-code introductions.

**Recommended fix:** Remove `#![allow(dead_code)]` and let the compiler surface any remaining actually-dead code. Verified items currently annotated at call-site with `#[allow(dead_code)]` (e.g., `retry_with_emitter` wrapper on line 277) will still work; anything else that lights up warns legitimately dead and should be pruned.

---

### IN-03: `MockServer::load_sse_cassette` panics on I/O error, not in a `#[cfg(test)]` module

**File:** `crates/kay-provider-openrouter/tests/common/mock_server.rs:89-90`
**Category:** quality (test harness hygiene)

**Issue:**
```rust
let raw = std::fs::read_to_string(&path)
    .unwrap_or_else(|e| panic!("failed to read cassette {path}: {e}"));
```

The integration-test `common` module is not `#[cfg(test)]`-gated; it's a regular module compiled into each integration binary. `panic!` here is acceptable for test helpers (crashing the test IS the failure), but the crate-wide lint does NOT forbid `panic!` — only `unwrap` / `expect`. So this is clippy-clean. However, this is one of the few `panic!` sites in the crate and deserves a note: the helper's contract is "call this from a #[tokio::test]; fixture load failure aborts the test."

**Recommended fix:** Move the helper into `#[cfg(test)]` or document the panic contract. Low priority — current posture is conventional.

---

### IN-04: `ApiKey::from(String)` trims whitespace but `as_str()` returns `&str`; empty-after-trim is legal and reaches `UpstreamClient`

**File:** `crates/kay-provider-openrouter/src/auth.rs:33-37`
**Category:** quality (defensive hardening)

**Issue:**
```rust
impl From<String> for ApiKey {
    fn from(s: String) -> Self {
        Self(s.trim().to_string())
    }
}
```

If a caller constructs an `ApiKey::from("   ".to_string())`, the resulting key is the empty string. The doc comment correctly notes "Empty input is allowed here — the upstream HTTP call will surface a 401, mapped to `Auth::Invalid`." This is fine for production (401 round-trip catches it), but it wastes a request. The test path fetches keys via `resolve_api_key` which already rejects empty-after-trim at resolution. Builders that bypass `resolve_api_key` via `.api_key("test-key")` can smuggle an empty string that only surfaces on the wire.

**Why it's informational:** No security impact (empty key → 401 → typed error). Just a minor DX roughness.

**Recommended fix:** Return `Result<ApiKey, ProviderError>` from `From<String>` — but `From` cannot be fallible. Alternative: add a `TryFrom<String>` impl that rejects empty-after-trim, and have `OpenRouterProviderBuilder::api_key` delegate through it.

---

## Scope Exclusions (confirmed honored per CLAUDE.md non-negotiable 1)

- `crates/forge_*/src/` — NOT reviewed. These are byte-identical upstream ForgeCode code locked on tag `forgecode-parity-baseline`. Phase 2.5 sub-crate split moved files but did not modify them (verified in `.planning/phases/02.5-kay-core-sub-crate-split/` SUMMARY files). Any finding in these files would violate parity.
- `.planning/` artifacts — not source code; reviewed via a separate verification pass.

## Strengths Worth Preserving

- **Never-panic invariant is structurally enforced**, not aspirational: crate-root `#![deny(clippy::unwrap_used, clippy::expect_used)]` + `#[allow]` only on test modules + proptest `parser_never_panics` on the tolerant-parser path. Zero production-code `.unwrap()` / `.expect()` confirmed via grep.
- **`ApiKey` redaction defense-in-depth**: custom `Debug` returns `"ApiKey(<redacted>)"`; no `Display`; not re-exported via `pub use`; crate-private `as_str()` only; test proves formatter output. TM-01 airtight.
- **Allowlist charset gate runs BEFORE allowlist compare** and returns empty allowed list on charset failure — prevents CRLF smuggling AND refuses to leak the allowlist to a smuggler. TM-04 correctly ordered (rare to see this sequencing done right).
- **CostCap Mutex poison recovery via `unwrap_or_else(|e| e.into_inner())`** is the idiomatic Rust pattern and clippy-clean without `#[allow]`. The poison-recovery rationale is documented inline (cost_cap.rs:51-54).
- **Retry-After precedence over backon** is implemented as a per-attempt override (not a schedule reset), honoring D-09's "backon resumes on subsequent attempts" semantic.
- **`reqwest_eventsource::retry::Never` policy set in UpstreamClient** prevents the 3×3=9 retry amplification that would otherwise happen with both layers retrying — TM-09 structurally enforced.
- **1 MiB tool-args cap with empty-raw Malformed eviction** avoids yielding near-1MB strings back through `AgentEvent` when the cap trips — subtle but correct for TM-06.
- **NN-7 `required`-before-`properties` enforcement via custom `Serialize` + `OrderedObject`** keeps `serde_json/preserve_order` OFF at the workspace level, preserving upstream `forge_app` parity. Load-bearing for TB 2.0 score.
- **`open_and_probe` pattern** is a correct structural fix for `reqwest_eventsource` delivering HTTP-status errors inside the stream rather than on the POST return — documented as Rule-2 critical deviation in 02-10-SUMMARY.

## Final Verdict

**issues_found** — no critical or high findings; 2 medium (stale doc comment that contradicts the code; silent data loss on defensive Message-before-Open branch); 4 low (synthesized-index collision corner case, `attempt` field semantics ambiguity, silent u128→u64 truncation, test does not cover the Malformed branch); 4 informational (unused deps, stale `#![allow(dead_code)]`, test helper panic contract, empty-key smuggling via non-resolve path). None block Phase 2 closeout; recommend addressing MD-01 and MD-02 before Phase 3 plans land so the code base is accurate for new contributors.

---

*Reviewed: 2026-04-20*
*Reviewer: Claude (gsd-code-reviewer, Opus 4.7)*
*Depth: standard*
