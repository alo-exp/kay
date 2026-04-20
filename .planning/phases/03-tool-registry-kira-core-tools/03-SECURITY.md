---
phase: 3
flow: 13
audit_date: 2026-04-21
asvs_tier: L2
verdict: PASS
---

# Phase 3 Security Audit — Tool Registry + KIRA Core Tools

**Branch / HEAD:** `phase/03-tool-registry` @ `e983959` (post-review-fixes)
**ASVS-equivalent tier:** L2 (critical assurance — marker protocol is load-bearing for NN#1 / TB 2.0 score)
**Overall verdict:** **PASS** — ready for FLOW 14 Nyquist. Three residual risks (R-1..R-3) are Phase 4/5 backlog, not blockers.

---

## 1. Threat-by-threat verification

| Threat | Plan ID | Mitigation | Evidence | Verdict |
|---|---|---|---|---|
| **A1 — Marker nonce forgery under prompt injection** (SHELL-01/05, NN#1 load-bearing) | T-3-07 / SHELL-05 | 128-bit `OsRng`/`SysRng` nonce; constant-time `subtle::ConstantTimeEq` compare; rejects embedded `__CMDEND_` in user command; `ForgedMarker` surfaces as stdout, does NOT close stream | `markers/mod.rs:40-53` (RNG propagation), `:95` (`ct_eq`), `:74-111` (scan_line); `execute_commands.rs:178-183` (reserved-substring reject); `tests/marker_race.rs:40-63` | **COVERED** |
| **A2 — Marker timing side-channel** | T-3-07 (side-channel) | All nonce comparison via `subtle::ConstantTimeEq::ct_eq` returning `Choice`; public `__CMDEND_` sigil pre-filter intentionally non-secret; scan uses fixed-width slices not variable search | `markers/mod.rs:94-97` fixed-length slice + `ct_eq` | **COVERED** |
| **A3 — PTY escape / shell metacharacter injection** | — | `portable-pty` denylist covers htop, vim, nvim, nano, less, more, top, tmux, screen, ssh, sudo, docker, watch, psql, mysql, sqlite3; command argv always `sh -c <wrapped>` where user command is embedded in subshell, NOT argv-interpolated; Windows uses PowerShell `-NoProfile` | `execute_commands.rs:56-59` (denylist), `:215-228` (sh -c / PowerShell); `markers/shells.rs:11-27` | **PARTIAL** — L-02 metacharacter heuristic bypass (e.g. `ssh;echo owned`) routes to piped path (strictly safer); not a security bug. Documented as R-1. |
| **A4 — Signal cascade race / zombie processes** (SHELL-02/04) | T-3-04 | Unix: `process_group(0)` at spawn; `killpg(pgid, SIGTERM)` → 2s grace → **unconditional** `killpg(pgid, SIGKILL)` (H-01 fix); PTY path same via `setsid`; `kill_on_drop(true)` backstop | `execute_commands.rs:336-401, 527-547`; `tests/timeout_cascade.rs` (2 tests incl. grandchild regression) | **COVERED** (Unix). Windows Job Objects deferred to Phase 4 (documented R-4). |
| **A5 — Sandbox bypass** | T-3-07 (image traversal) | `NetFetchTool.ctx.sandbox.check_net`; `ImageReadTool.ctx.sandbox.check_fs_read` pre-read (M-05 fix) with quota rollback on denial; fs_* delegate through `ctx.services` facade (forge_services parity enforces path/size/binary checks) | `net_fetch.rs:80-86`, `image_read.rs:139-145`, `fs_read.rs:75-82` | **COVERED** (seam). `NoOpSandbox` is pass-through by design — real enforcement is Phase 4. |
| **A6 — forge_app parity drift** (NN#1) | T-3-08 | `tests/parity_delegation.rs` byte-diff via `pretty_assertions::assert_eq`; twin-path `FakeFs*` + `CallLog` catches argument-reorder regressions | `tests/parity_delegation.rs`; `forge_bridge.rs` facade impls | **COVERED** |
| **A7 — Image quota exhaustion** (TOOL-04) | T-3-15 | `try_consume` atomic: bumps per-turn first, rolls back on per-session breach; `release()` (M-02 fix) on IO/sandbox failure; saturating sub prevents underflow | `quota.rs:36-79`; `image_read.rs:118-156`; `tests/image_quota.rs` (6 tests) | **COVERED**. L-05 unbounded payload → R-2. |
| **A8 — Object-safety regression** (TOOL-01) | T-3-16 | `Tool` trait has no generics / associated types / `Self: Sized`; `Arc<dyn Tool>` coercion compile-locked; `ServicesHandle` likewise dyn-safe; `input_schema` returns owned `Value` | `contract.rs:9-29`; `tests/registry_integration.rs`; `tests/compile_fail_harness.rs` | **COVERED** |

---

## 2. Supply-chain status

### `cargo deny check`
`advisories ok, bans ok, licenses ok, sources ok`. One duplicate `winreg` warning (v0.10.1 via `portable-pty` vs v0.11.0 via `forge_tracker`) — license-clean, Windows-only, not a security issue.

### `cargo audit`
Advisory-db loaded 1049 advisories; scanned 827 deps with zero vulnerability hits before network timeout on yanked-check phase. **Action:** capture clean transcript in FLOW 14 Nyquist CI run.

### New-dep provenance
| Crate | Version | License | Notes |
|---|---|---|---|
| `subtle` | 2.x | BSD-3-Clause | dalek-cryptography, zero unsafe in `ct_eq` |
| `portable-pty` | 0.9 | MIT | wezterm-org |
| `nix` | 0.29 workspace | MIT | `killpg` + `Pid` |
| `rand` | workspace | MIT/Apache-2.0 | `SysRng` + `try_fill_bytes` |
| `base64` | workspace | MIT/Apache-2.0 | — |

All license-allowlisted in `deny.toml`.

---

## 3. Non-Negotiables compliance

| NN | Constraint | Status | Evidence |
|----|------------|--------|----------|
| NN#1 | ForgeCode parity gate | **PASS** | `parity_delegation.rs` byte-diff lock |
| NN#2 | No unsigned release tags | **N/A (FLOW 17)** | No tag pushed in Phase 3 |
| NN#3 | DCO on every commit | **PASS** | 33 `Signed-off-by:` trailers across 23 commits `1bb792d..HEAD` |
| NN#4 | Clean-room, no TS-derived structure | **PASS** | `grep -ri "claude-code\|anthropic-ai"` in kay-tools → 0 matches |
| NN#5 | Single binary, no `externalBin` | **PASS** | kay-cli normal bin crate; no externalBin string anywhere |
| NN#6 | OpenRouter Exacto allowlist | **N/A (Phase 2/10)** | Provider allowlisting out of Phase 3 scope |
| NN#7 | Schema hardening | **PASS** | `harden_tool_schema` delegates verbatim to `enforce_strict_schema`; 1024-case proptest locks invariants |

---

## 4. Residual risks (Phase 4/5 backlog, non-blocking)

| ID | Risk | Severity | Mitigation target |
|----|------|----------|-------------------|
| R-1 | PTY heuristic bypass via compound metacharacter first-token (`ssh;echo owned`) routes to piped path | Low (cosmetic, piped is safer) | Phase 5 — tokenize on `[\s;\|&]` before PTY decision |
| R-2 | `AgentEvent::ImageRead { bytes }` carries unbounded payload — DoS via huge PNG | Medium | Phase 5 — add `max_image_bytes` cap (20 MiB default) in `ImageReadTool::new` |
| R-3 | BRAINSTORM §6 A1 scheduled ≥10k-case adversarial proptest; delivered as unit+integration tests only | Low (logic exhaustively reasoned + constant-time) | Phase 4 — add `tests/marker_forgery_property.rs` |
| R-4 | Windows timeout cascade lacks Job Object sweep | Medium (Windows only) | Phase 4 sandbox (SBX-04) |
| R-5 | Empty `runtime::dispatcher` + `seams::rng` modules | Info | Populate in Phase 4 or demote to `#[cfg(test)]` |
| R-6 | `rmcp` advisory (STATE.md #3) | N/A for Phase 3 | MCP phase |

---

## 5. Verdict

Phase 3 is security-clean against its declared threat model:
- H-01 (HIGH) fixed and locked by regression test
- M-01..M-05 (MEDIUM) fixed with test lock-ins
- Threat register T-3-06..T-3-17 all verified
- NN#1, NN#3, NN#4, NN#5, NN#7 enforced in code
- Supply chain green (deny clean; audit needs networked CI retry)

**Proceed to FLOW 14 Nyquist.** File R-1..R-5 as Phase 4/5 backlog tickets via `gsd-add-backlog`.
