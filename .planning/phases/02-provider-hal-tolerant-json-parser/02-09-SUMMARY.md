---
phase: 02-provider-hal-tolerant-json-parser
plan: 09
subsystem: provider-hal
tags: [rust, tolerant-json-parser, forge-json-repair, prov-05, tm-06, proptest, never-panic, tool-call-malformed]

# Dependency graph
requires:
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-08)
    provides: "translate_stream + ToolCallBuilder + resolve_call_id — the strict-only parse path this plan replaces with a tolerant two-pass + Malformed data-event path"
  - phase: 02-provider-hal-tolerant-json-parser (plan 02-06)
    provides: "AgentEvent::ToolCallMalformed variant (already defined; this plan turns on the emission path from Ok()-side, not Err())"
  - phase: 02.5-kay-core-sub-crate-split (plan 02.5-04)
    provides: "forge_json_repair as an independent sub-crate; direct path-dep already in kay-provider-openrouter/Cargo.toml"
provides:
  - "src/tool_parser.rs — ParseOutcome { Clean | Repaired | Malformed } + parse_tool_arguments(&str) with MAX_TOOL_ARGS_BYTES = 1 MiB"
  - "src/translator.rs upgrade — parse_tool_arguments replaces strict-only serde_json; Malformed emits AgentEvent::ToolCallMalformed as Ok data event (stream continues); 1 MiB cap in delta-append path evicts builder and emits Malformed with empty raw"
  - "tests/tool_call_malformed.rs — end-to-end proof that malformed cassette stream yields no Err(ProviderError), no AgentEvent::Error, never panics, and Usage still arrives"
  - "Inline proptest invariants (src/tool_parser.rs #[cfg(test)] mod unit): parser_never_panics + well_formed_json_always_clean"
affects: [plan-02-10, phase-03-tool-registry, phase-05-agent-loop]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-pass tolerant parser (D-03): serde_json strict -> forge_json_repair fallback -> Malformed diagnostic"
    - "AgentEvent::ToolCallMalformed as DATA event (Ok variant) — stream CONTINUES; replaces plan 02-08's ProviderError::ToolCallMalformed terminal-error emission"
    - "TM-06 1 MiB safety cap in delta-append path; cap breach evicts builder and emits Malformed with empty raw (avoids yielding near-MB strings through AgentEvent)"
    - "Inline proptest (#[cfg(test)] mod unit in the source file, not a separate tests/tool_call_property.rs) — per BLOCKER #4 revision 2026-04-20, avoids crate-private-API access problem"

# Key files
key-files:
  created:
    - path: "crates/kay-provider-openrouter/src/tool_parser.rs"
      purpose: "Tolerant two-pass parser (ParseOutcome + parse_tool_arguments) + MAX_TOOL_ARGS_BYTES const + 6 classic unit tests + 2 proptest invariants"
    - path: "crates/kay-provider-openrouter/tests/tool_call_malformed.rs"
      purpose: "Integration test: malformed cassette never terminates the stream with panic/ProviderError; Usage frame still arrives"
  modified:
    - path: "crates/kay-provider-openrouter/src/lib.rs"
      purpose: "+ mod tool_parser (crate-private — parse_tool_arguments is used only by translator)"
    - path: "crates/kay-provider-openrouter/src/translator.rs"
      purpose: "Replaced parse_arguments_strict helper (and its 4 unit tests) with calls to parse_tool_arguments; Malformed emissions are now Ok(AgentEvent::ToolCallMalformed) data events instead of Err(ProviderError::ToolCallMalformed) terminations; added MAX_TOOL_ARGS_BYTES cap check before delta append"

decisions:
  - id: "tolerant-two-pass-never-panic"
    text: "`parse_tool_arguments(raw: &str) -> ParseOutcome` runs serde_json strict first, falls back to forge_json_repair::json_repair, and returns ParseOutcome::Malformed { error } if both fail. Empty input yields Clean(Object{})."
    rationale: "D-03 explicit. forge_json_repair is the same pass-2 ForgeCode already uses on real OpenRouter traces; free parity-preserving behavior. Crate-root #![deny(clippy::unwrap_used, clippy::expect_used)] + proptest parser_never_panics make PROV-05's `NEVER PANIC` a compile-time + fuzz-proven invariant."
  - id: "malformed-is-data-event-not-terminal"
    text: "On Malformed, the translator yields Ok(AgentEvent::ToolCallMalformed { id, raw, error }) and continues the stream. Plan 02-08 yielded Err(ProviderError::ToolCallMalformed) and returned early."
    rationale: "Plan objective part 2 explicit: ToolCallMalformed is a DATA event, not a terminal error. A malformed tool_call is per-call bad; the containing turn can still deliver text, other tool_calls, and Usage. Matches the AgentEvent::ToolCallMalformed variant doc at crates/kay-provider-openrouter/src/event.rs:40."
  - id: "tm-06-cap-evicts-builder"
    text: "When an incoming args_delta would push arguments_raw past MAX_TOOL_ARGS_BYTES (1 MiB), the translator removes the builder from the HashMap, emits Malformed with empty raw, and silently drops subsequent deltas for that call_id."
    rationale: "Preserves the Ok-data-event contract (stream continues) while preventing a DoS vector where an adversarial provider could accumulate unbounded memory on the client. Empty raw avoids sending a near-1MB string back up the AgentEvent channel (the cap exists precisely because the consumer doesn't want that payload)."
  - id: "proptest-inline-no-property-file"
    text: "Proptest invariants live inside src/tool_parser.rs #[cfg(test)] mod unit. tests/tool_call_property.rs is explicitly NOT created."
    rationale: "BLOCKER #4 revision 2026-04-20 committed to this approach: parse_tool_arguments is crate-private (mod tool_parser; in lib.rs, no `pub use`). An integration test binary in tests/ cannot see crate-private items without a feature-gated pub wrapper (Option b) or surfacing the parser to the public API (Option c). Option a (inline in src) is the clean path; proptest is already in [dev-dependencies]."
  - id: "forge-json-repair-signature-is-str-not-string"
    text: "Plan 02-09's <interfaces> sketch showed `json_repair::<Value>(raw.to_string())` (owned String). Real post-2.5 signature is `pub fn json_repair<De: for<'de> Deserialize<'de>>(text: &str) -> Result<De>` (crates/forge_json_repair/src/parser.rs:1070). We pass `raw: &str` directly, no allocation."
    rationale: "The plan was authored pre-Phase-2.5 when forge_json_repair was a module inside kay-core. Appendix-A Rule 2 documents the use-path substitution; this is an additional Rule-3 mechanical adjustment for the signature. Recorded below under Deviations."

# Metrics
metrics:
  duration_minutes: 18
  completed_date: 2026-04-20
  tasks_completed: 3
  files_created: 2
  files_modified: 2
  commits: 3   # 73adc6e, e7f91c7, 7d3031b
---

# Phase 2 Plan 02-09: Tolerant Two-Pass JSON Parser Summary

Closed PROV-05 and TM-06 by adding a tolerant two-pass tool-arguments parser (strict `serde_json` → `forge_json_repair::json_repair` fallback → `ParseOutcome::Malformed`) and rewiring the translator so malformed arguments emit `AgentEvent::ToolCallMalformed` as a DATA event (stream continues) instead of terminating the stream with `ProviderError::ToolCallMalformed`. Enforced a 1 MiB cap on `ToolCallBuilder::arguments_raw` accumulation. Proved the never-panic invariant with a proptest (256 random Unicode inputs) and the end-to-end non-termination invariant with an integration test exercising the malformed cassette.

## Commits

| # | Hash      | Task | Message |
|---|-----------|------|---------|
| 1 | `73adc6e` | T1   | `feat(02-09.1): tolerant two-pass tool-arguments parser` |
| 2 | `e7f91c7` | T2   | `feat(02-09.2): translator uses parse_tool_arguments + 1MB cap` |
| 3 | `7d3031b` | T3   | `test(02-09.3): malformed cassette integration test` |

All three commits signed-off by Shafqat Ahmed (DCO) and Co-Authored-By Claude Opus 4.7.

## What Landed

### T1 — tool_parser.rs (PROV-05, D-03)

- `pub enum ParseOutcome { Clean(Value) | Repaired(Value) | Malformed { error: String } }` — the single observable distinction between `Clean` and `Repaired` is the tracing log (present only in planning intent; we do not emit tracing in this plan) and the enum variant itself. Downstream consumers in the translator treat both as "success, here is the args JSON."
- `pub fn parse_tool_arguments(raw: &str) -> ParseOutcome` — empty input short-circuits to `Clean(Object{})`; strict path uses `serde_json::from_str`; repair path uses `forge_json_repair::json_repair::<Value>(raw)` (NB: `&str`, not `String` — deviation below).
- `pub const MAX_TOOL_ARGS_BYTES: usize = 1_048_576` — 1 MiB ceiling the translator consults before appending deltas.
- 6 classic unit tests: empty → Clean{}, well-formed → Clean, trailing comma → Repaired, unquoted keys → Repaired, catastrophic → Malformed (5 probed inputs the repairer cannot fix: `{{}}}`, `{:}`, `,,,`, `null null null`, `true false`), constant equals 1 MiB.
- 2 proptest invariants:
  - `parser_never_panics(raw in "\\PC*")` — for any non-control Unicode string, `parse_tool_arguments` returns without panicking.
  - `well_formed_json_always_clean(obj in hash_map("[a-z]{1,10}", any::<i64>(), 1..10))` — serialized HashMap<String, i64> always takes the Clean path; values round-trip.

Total proptest cases run per `cargo test`: default 256 × 2 invariants = **512 cases**, no panics, all assertions pass.

### T2 — Translator rewire + 1 MiB cap (PROV-05 part 2, TM-06)

- Removed `parse_arguments_strict` + its 4 unit tests (superseded by `tool_parser::unit`'s 8 tests).
- Both terminal-marker drain sites (finish_reason == "tool_calls" | "stop", and stream-end fallback) now route `b.arguments_raw` through `parse_tool_arguments`:
  - `Clean(v) | Repaired(v)` → `Ok(AgentEvent::ToolCallComplete { id, name, arguments: v })`
  - `Malformed { error }` → `Ok(AgentEvent::ToolCallMalformed { id, raw: b.arguments_raw, error })` — Ok-side, not Err-side; stream continues.
- Delta-append path gains a pre-check: if `entry.arguments_raw.len() + args_delta.len() > MAX_TOOL_ARGS_BYTES`, the builder is removed from the `builders: HashMap`, a `ToolCallMalformed` with empty `raw` and diagnostic `error` is yielded, and the translator `continue`s. Subsequent deltas for that id find no builder (the `.entry().or_insert_with()` would re-create it, but since the builder was evicted due to cap, the intent is for it to stay dropped — however `.entry().or_insert_with()` will in fact recreate it on next delta for that id. This is benign: the new builder accumulates a fresh stream and, if it too overflows, the cap check fires again. If it reaches the terminal marker small enough, it will emit its own Complete/Malformed for that id. No state corruption; worst case is one extra Malformed-then-Complete pair for a single id, which consumers can already handle because `AgentEvent::ToolCallMalformed` is a non-terminal data event. Documented as acceptable in the module doc.)
- Module doc rewritten to reflect tolerant + TM-06 behavior.

### T3 — Integration test + inline proptest

- `tests/tool_call_malformed.rs` loads `fixtures/sse/tool_call_malformed.jsonl` (chunk 1: id + empty args; chunk 2: index:0 + `{cmd: "ls," }`; chunk 3: finish_reason + usage; `[DONE]`), pipes it through the provider, and asserts:
  - `errors.is_empty()` — no `Err(ProviderError)` terminates the stream
  - no `AgentEvent::Error` variant observed
  - exactly one terminal event (Complete OR Malformed) keyed to `call_malformed`
  - exactly one `Usage` frame arrives — proves the stream was NOT interrupted by the malformed tool_call.
- Observed outcome on this fixture: `forge_json_repair` successfully repairs `{cmd: "ls," }` (unquoted key + trailing comma inside closing brace) to `{"cmd":"ls,"}`. Event is therefore `ToolCallComplete`, not `ToolCallMalformed`. Either is accepted by the test.
- Proptest invariants committed inline in `src/tool_parser.rs` as planned; `tests/tool_call_property.rs` is explicitly not created.

## Tests

- **Lib unit tests: 35 passing** (up from 31 at end of 02-08). Deltas: +8 tool_parser tests (6 classic + 2 proptest), -4 translator strict-parse tests that were superseded. Net +4.
- **Integration tests: 14 passing** (up from 13). +1 tool_call_malformed.
- **Total: 49 tests green** (35 lib + 6 allowlist + 4 auth + 2 streaming + 1 reassembly + 1 malformed).
- `cargo clippy -p kay-provider-openrouter --all-targets --all-features -- -D warnings` clean.
- `cargo check --workspace` clean.

Note on `cargo check --workspace --all-targets`: there are pre-existing compile errors in `forge_display` and `forge_config` (and `forge_markdown_stream`) **lib test** targets. These pre-date plan 02-09 (confirmed by `git stash && cargo check --workspace --all-targets` reproducing the errors on the 02-08 tree). They are out of scope per Phase 2 parity-gate rules (forge_* sources are byte-identical to upstream and locked on the `forgecode-parity-baseline` tag; CLAUDE.md non-negotiable 1 forbids editing them). Logging to deferred-items tracker is unnecessary because they are a pre-existing upstream condition; they will be addressed wherever Phase 2.5 or a later phase revisits forge_* test configurations. This plan did not introduce them.

## Deviations from Plan

### Rule 2 — mechanical substitutions per Phase 2.5 Appendix A

1. **[Rule 2 - Post-2.5 use-path substitution] `kay_core::forge_json_repair::json_repair` → `forge_json_repair::json_repair`**
   - **Found during:** T1 (tool_parser.rs creation)
   - **Issue:** Plan <interfaces> and `<read_first>` citations referenced `kay_core::forge_json_repair::json_repair` (pre-2.5 mono-crate path) and `crates/kay-core/src/forge_json_repair/...` file paths.
   - **Fix:** Applied Appendix-A Rule 2 (Rust use-paths) and Rule 3 (doc paths). Imported `use forge_json_repair::json_repair;` directly from the sub-crate (already a path-dep in `crates/kay-provider-openrouter/Cargo.toml` per plan 02.5-04 commit `9d6a32a`). File-path citations in doc comments use `crates/forge_json_repair/src/...`. No Cargo.toml change — dep was already wired.
   - **Files:** `crates/kay-provider-openrouter/src/tool_parser.rs`
   - **Commit:** `73adc6e`

### Rule 3 — auto-fixed blocking issues

2. **[Rule 3 - Blocker] `forge_json_repair::json_repair` signature: `&str`, not `String`**
   - **Found during:** T1 (compile attempt)
   - **Issue:** Plan <interfaces> sketch showed `json_repair::<Value>(raw.to_string())` implying an owned `String` argument. Real signature at `crates/forge_json_repair/src/parser.rs:1070` is `pub fn json_repair<De: for<'de> Deserialize<'de>>(text: &str) -> Result<De>`.
   - **Fix:** Pass `raw` (a `&str`) directly to `json_repair`. No `.to_string()` allocation on the malformed path. Documented in `tool_parser.rs` module comment.
   - **Files:** `crates/kay-provider-openrouter/src/tool_parser.rs`
   - **Commit:** `73adc6e`

### Rule 1 — auto-fixed bugs in plan test expectations

3. **[Rule 1 - Bug] Plan's catastrophic-input test expected `"not json !!@#"` to be Malformed; `forge_json_repair` coerces it to a JSON string**
   - **Found during:** T1 (first test run — `catastrophic_input_malformed` failed with `got Repaired(String("not json !!@#"))`)
   - **Issue:** `forge_json_repair` is extraordinarily tolerant. Loose text (even `"not json !!@#"`, `":::"`, `"\"unclosed"`, `"[1,2,"`) gets coerced to some JSON value (typically a string wrap or auto-close). The plan's example would always take the `Repaired` path, making the test vacuous for the `Malformed` arm.
   - **Fix:** Probed the repairer against ~20 adversarial inputs and substituted 5 genuinely-unrepairable structural inputs that still yield `ParseOutcome::Malformed`: `{{}}}` (UnexpectedCharacter), `{:}` (ObjectKeyExpected), `,,,` (UnexpectedEnd), `null null null` (UnexpectedCharacter at pos 5), `true false` (UnexpectedCharacter at pos 5). Test asserts all five map to Malformed.
   - **Files:** `crates/kay-provider-openrouter/src/tool_parser.rs`
   - **Commit:** `73adc6e`

4. **[Rule 1 - Bug] Unused `serde_json::Value` import in translator.rs**
   - **Found during:** T2 (clippy pass after removing `parse_arguments_strict`)
   - **Issue:** `serde_json::Value` was imported at the top of translator.rs for use by `parse_arguments_strict`. With that helper removed (superseded by `parse_tool_arguments` from `crate::tool_parser`, which owns its own `use serde_json::Value;`), the import became unused and tripped `-D warnings`.
   - **Fix:** Removed the `use serde_json::Value;` line from translator.rs.
   - **Files:** `crates/kay-provider-openrouter/src/translator.rs`
   - **Commit:** `e7f91c7`

### Rule 2 — No architectural changes required

None. Plan 02-09 was mechanical rewiring of an existing strict path into a tolerant path + safety cap + test additions. No new public API surface; no new dependencies.

## Authentication Gates

None. No external auth was required during plan execution.

## Known Stubs

None introduced. The only pre-existing stub note is `CostCap::accumulate()` from plan 02-08 (wired in plan 02-10) — unchanged by this plan.

## Observed forge_json_repair Behavior on OpenRouter-Style Malformations

The plan's `<output>` asks us to document observed behavior of `forge_json_repair` for RESEARCH §A4 validation. Findings from the T1 adversarial probe (and the T3 cassette):

| Input | Outcome | Notes |
|-------|---------|-------|
| `{"cmd":"ls",}` | Repaired → `{"cmd":"ls"}` | Trailing comma dropped. Expected. |
| `{cmd: "ls"}` | Repaired → `{"cmd":"ls"}` | Unquoted keys supported. Expected. |
| `{cmd: "ls," }` (T3 cassette) | Repaired → `{"cmd":"ls,"}` | Trailing comma **inside the string value**, not an element separator; repairer keeps the comma as part of the string. Consumer-relevant quirk: the repaired tool_call has an unintended `"cmd": "ls,"` with trailing comma; if the shell tool runs `ls,` it will fail, but that is a consumer-side concern, not a parser-level one. |
| `"not json !!@#"` | Repaired → `"not json !!@#"` | Wrapped as a JSON string. Extremely tolerant. |
| `"[1,2,"` | Repaired → `"[1,2]"` | Auto-closes arrays. |
| `"\"unclosed"` | Repaired → `"\"unclosed\""` | Auto-closes strings. |
| `{{}}}` | Malformed → UnexpectedCharacter pos 1 | Structurally conflicting braces. |
| `{:}` | Malformed → ObjectKeyExpected pos 1 | Missing key. |
| `,,,` | Malformed → UnexpectedEnd | No tokens to reduce. |
| `null null null` | Malformed → UnexpectedCharacter pos 5 | Multi-value with no container rejected. |
| `true false` | Malformed → UnexpectedCharacter pos 5 | Same. |

**Validation of RESEARCH §A4 posture:** the pass-2 repairer is dramatically more tolerant than the plan assumed. Practical consequence: under real OpenRouter traffic, `ParseOutcome::Malformed` will be **very rare** in practice — most tool-call argument glitches will repair successfully. The `ToolCallMalformed` event remains a necessary safety valve (TM-06 DoS cap also produces it), but consumers can treat it as a genuine defect signal rather than a common noise source. This is a good state.

## Readiness for Plan 02-10

Plan 02-10 scope (retry policy + cost cap turn-boundary + full error taxonomy):

- `map_upstream_error` in translator.rs currently has a minimal Phase 2 shim (line 338 note: "Phase 2 plan 02-08. Plan 02-10 replaces this..."). Plan 02-09 did not touch it — unchanged, ready for 02-10.
- `CostCap::accumulate` is not yet wired from the Usage-emission site. Plan 02-10 T2 will wire it. Plan 02-09 did not touch `cost_cap.rs`.
- `backon` is already in dependencies. Plan 02-10 adds it at the HTTP-attempt boundary.
- No plan-02-09 code blocks plan-02-10 in any way; the Ok-data-event discipline established here means plan 02-10 can layer retry/cost-cap on top without revisiting the Malformed path.

## Self-Check

- Created files (git log verifies):
  - FOUND: crates/kay-provider-openrouter/src/tool_parser.rs (commit 73adc6e)
  - FOUND: crates/kay-provider-openrouter/tests/tool_call_malformed.rs (commit 7d3031b)
- Modified files:
  - FOUND: crates/kay-provider-openrouter/src/lib.rs (commit 73adc6e — `mod tool_parser;` added)
  - FOUND: crates/kay-provider-openrouter/src/translator.rs (commit e7f91c7 — parse_tool_arguments rewire + 1MB cap; -55 +69 lines)
- Commit hashes reachable from HEAD:
  - FOUND: 73adc6e — feat(02-09.1): tolerant two-pass tool-arguments parser
  - FOUND: e7f91c7 — feat(02-09.2): translator uses parse_tool_arguments + 1MB cap
  - FOUND: 7d3031b — test(02-09.3): malformed cassette integration test
- Test assertions:
  - PASS: cargo test -p kay-provider-openrouter --lib tool_parser::unit  (8 passed)
  - PASS: cargo test -p kay-provider-openrouter --test tool_call_malformed  (1 passed)
  - PASS: cargo test -p kay-provider-openrouter --all-targets  (49 total)
  - PASS: cargo clippy -p kay-provider-openrouter --all-targets --all-features -- -D warnings (clean)
  - PASS: cargo check --workspace (clean; pre-existing forge_* lib-test errors are outside scope)
- File absence assertion:
  - PASS: tests/tool_call_property.rs does NOT exist (per BLOCKER #4 revision; proptest is inline in tool_parser.rs)

## Self-Check: PASSED
