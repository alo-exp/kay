---
phase: 07-context-engine
audited: 2026-04-22
asvs_level: 1
block_on: critical
auditor: Claude (gsd-security-auditor)
status: SECURED
threats_total: 7
threats_closed: 7
threats_open: 0
---

# Phase 7 — Security Audit Report

**Phase:** 7 — Context Engine (kay-context)
**Threats Closed:** 7/7
**ASVS Level:** 1
**Block-on:** critical
**Result:** SECURED

---

## Threat Verification

| Threat ID | Category   | Disposition   | Evidence |
|-----------|------------|---------------|----------|
| TM-01 | Tampering  | mitigate      | Paths enter the system only via `TreeSitterIndexer::index_file` (called from forge_walker/notify events). No user-supplied path parameter in any public store API. `store.rs:50` (`check_and_set_index_state`), `watcher.rs:61` — paths are filesystem-sourced, never user-supplied. |
| TM-02 | Tampering  | mitigate      | All SQL in `store.rs` uses `rusqlite::params![]` positional bindings exclusively. No string interpolation into any SQL statement. Evidence: `store.rs:152` (`INSERT INTO symbols`), `store.rs:168` (`DELETE FROM symbols`), `store.rs:173` (`DELETE FROM index_state`), `store.rs:190` (`SELECT content_hash`), `store.rs:213` (`INSERT OR REPLACE INTO index_state`), `store.rs:267` (`INSERT OR REPLACE INTO symbols_vec`), `store.rs:344` (`symbols_fts MATCH ?1`). The FTS5 `MATCH` clause receives the query string as a bound parameter (`?1`), not as string-interpolated SQL. |
| TM-03 | Injection  | mitigate      | `_ctx_packet` is explicitly unused in Phase 7 (`loop.rs:446-450`, `#[allow(unused)]` attribute). The `ContextPacket` returned by `NoOpContextEngine::retrieve` is the default value and is never injected into any outbound request or prompt. Phase 8+ carry-forward is documented in `engine.rs:3-6` with a hardening comment requiring `[USER_DATA: session_title]` delimiters. |
| TM-04 | Tampering  | mitigate      | `sqlite-vec` is pinned at exactly `=0.1.10-alpha.3` in the workspace `Cargo.toml:204`. The `=` prefix in the version requirement prevents Cargo from resolving any other version, including patch-level alpha updates with ABI changes. |
| TM-05 | Elevation  | mitigate      | `watcher.rs:61`: `debouncer.watcher().watch(root, notify::RecursiveMode::Recursive)` — the watch scope is anchored to `root` (the caller-supplied project directory). `RecursiveMode::Recursive` expands only within that root. No additional watch paths are registered. Callback is invoked only for events whose paths pass both `is_source_file()` and `!should_ignore()` filters. |
| TM-06 | Elevation  | accept        | `FakeEmbedder` in `embedder.rs:20-35` contains no hardcoded credentials, secrets, API keys, or sensitive data. It holds a single `pub dimensions: usize` field and returns `vec![0.0f32; self.dimensions]` — a deterministic zero-vector with no runtime secret exposure. The always-compiled decision is documented in the inline comment at `embedder.rs:19` ("Always compiled so integration tests (tests/) can import it without feature flags") and reviewed under IN-01 in `07-REVIEW.md:162-170`. The review explicitly labels the decision intentional and low-risk. |
| TM-07 | Denial     | mitigate      | `watcher.rs:7`: `const DEBOUNCE_MS: u64 = 500`. `watcher.rs:37`: `Duration::from_millis(DEBOUNCE_MS)` passed to `new_debouncer`. The 500 ms debounce window coalesces all filesystem events within each half-second burst into a single `on_invalidate()` invocation. The callback itself is O(1) — it carries no DB write; it signals invalidation only. This bounds burst-induced DB thrashing regardless of IDE autosave rate. |

---

## Accepted Risks Log

### TM-06 — FakeEmbedder always compiled

**Risk:** `FakeEmbedder` is compiled into all build profiles (no `#[cfg(test)]` or `testing` feature gate), exposing a test-only type in the production public API surface.

**Why accepted:** The type contains no secrets or credentials. It emits only zero-vectors (`0.0f32`). Production code that imports `kay-context` will not instantiate it unless explicitly constructing `FakeEmbedder { dimensions: N }`. The binary cost is trivial. The decision was reviewed by the code reviewer (IN-01 in `07-REVIEW.md`) and found to be intentional for integration test ergonomics.

**Residual exposure:** Minor API surface inflation (rustdoc pollution, autocomplete noise). No runtime security consequence.

**Owner:** Phase 7 executor
**Review reference:** `.planning/phases/07-context-engine/07-REVIEW.md` §IN-01

---

## Unregistered Flags

None. No `## Threat Flags` section was present in a SUMMARY.md for Phase 7 requiring cross-reference.

---

## Notes

- ASVS Level 1 verification scope: basic parameterized query hygiene (TM-02), no user-controlled path injection (TM-01/TM-05), no hardcoded secrets (TM-06). All Level 1 checks pass.
- TM-03 deferred risk is structurally sound: `_ctx_packet` is unused at the call site (`loop.rs:447-452`) with an explicit `#[allow(unused)]` that will become a compile error if the variable is removed and the deferred injection is wired without the hardening comment being resolved.
- The `sqlite-vec` alpha pin (TM-04) is a workspace-level lock, not a crate-level override, which is the correct position — a crate-level `[patch]` could be silently overridden by workspace resolution, whereas the workspace `[workspace.dependencies]` entry is the authoritative resolution anchor.
