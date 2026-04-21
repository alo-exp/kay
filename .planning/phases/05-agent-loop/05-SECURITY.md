---
status: passed
source: /gsd-secure-phase (consolidated Step 10 silver:security + Step 11 gsd-secure-phase)
started: 2026-04-22T00:00:00Z
updated: 2026-04-22T00:00:00Z
branch: phase/05-agent-loop
commit_range: 1ae2a7f..af707d6
commit_count: 69
head: af707d6c1f350c864c8094b69e3b11d3028a0821
auditor: claude-opus-4-7 (Security role)
scope: Phase 5 — Agent Loop + Canonical CLI (kay-cli, kay-core::{control,event_filter,loop,persona}, kay-tools::{builtins/execute_commands,image_read,sage_query, seams/verifier, events, events_wire, error}, personas/*.yaml)
asvs_level: 2
block_on: open
threats_total: 9
threats_closed: 9
threats_open: 0
findings_blocker: 0
findings_high: 0
findings_medium: 0
findings_low: 2
findings_info: 1
dependencies_audited: 840
vulnerabilities_found: 0
advisories_ignored_documented: 10
---

## Summary

Phase 5 ships the biased-select agent loop (LOOP-01..06), the headless `kay run` CLI + interactive REPL fallback (CLI-01/03/04/05/07), R-1 PTY tokenizer, R-2 image-read size cap, and the sage_query nesting-depth guard. The phase carries forward two load-bearing Phase 3/4 invariants — T-3-06 (NoOpVerifier never emits `Pass`) and QG-C4 (SandboxViolation never reenters model context) — and adds seven new threats (T-5-01..07) covering persona config injection, sage recursion, shell injection, image OOM, select ordering, signal races, and exit-code precedence.

**Outcome:** All nine threats are CLOSED with cited evidence. Zero BLOCKER findings. Zero HIGH findings. Two LOW hardening notes (defense-in-depth surface area on persona path resolution; documented cargo-audit ignores upstream to v0.2.0 parity backlog). One INFO note (intentional unbounded `pause_buffer` per DL-2). `cargo audit` reports **0 vulnerabilities** against 840 dependencies with a documented `audit.toml` ignore of 10 advisories that all originate in parity-imported `forge_*` transitive deps (not Phase 5 surface). No unsafe blocks in any Phase 5 crate. No secret leakage. No cleartext HTTP in provider surface (OpenRouter endpoints are HTTPS). Schema strictness (`#[serde(deny_unknown_fields)]`) enforced on all three externally-parsed input types (`Persona`, `ImageReadArgs`, `SageQueryArgs`, `ExecuteCommandsInput`).

**Recommendation: PASS.** Phase 5 is cleared to advance to Step 12 (`/gsd-validate-phase`).

## Threat-Model Verification (Step 11 / gsd-secure-phase)

All 9 threats declared in 05-PLAN.md are `mitigate` disposition. Each was verified by grep-anchored inspection of the cited mitigation site in the declared source file.

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-3-06 | Carry-forward (verifier lie) | mitigate | CLOSED | `crates/kay-tools/src/seams/verifier.rs:30-35` — `NoOpVerifier::verify` returns only `VerificationOutcome::Pending { reason }`; test at line 57-66 asserts "NoOpVerifier must never emit Pass (Threat T-3-06)". Symmetric `never_returns_fail` test at line 68+. |
| QG-C4 | Carry-forward (sandbox quote-back) | mitigate | CLOSED | `crates/kay-core/src/event_filter.rs:92-94` — `pub fn for_model_context(event: &AgentEvent) -> bool { !matches!(event, AgentEvent::SandboxViolation { .. }) }`. Hard-gated by CI SHIP BLOCK `coverage-event-filter` at 100% line + 100% branch coverage on this module. 10k-case proptest at `tests/event_filter.rs` locks the property. `events.rs:114` carries the matching contract docstring ("MUST NOT be serialized back into the model's message history"). |
| T-5-01 | Persona config injection | mitigate | CLOSED | `crates/kay-core/src/persona.rs:84` — `#[serde(deny_unknown_fields)]` on `Persona`. `from_yaml_str` feeds through `validate_against_registry` (rejects unknown `tool_filter` entries, `UnknownTool` error) and `validate_model` (rejects non-allowlisted models, `ModelNotAllowed` error). No silent field acceptance. |
| T-5-02 | sage_query unbounded recursion | mitigate | CLOSED | `crates/kay-tools/src/builtins/sage_query.rs:85` — `pub const MAX_NESTING_DEPTH: u8 = 2`. Depth check at `sage_query.rs:194-199` runs BEFORE argument parsing: `if ctx.nesting_depth >= MAX_NESTING_DEPTH { return Err(ToolError::NestingDepthExceeded { depth, limit }) }`. Belt + suspenders: `crates/kay-core/personas/sage.yaml` `tool_filter` excludes `sage_query` — even if the runtime guard drifted, sage cannot call itself. Inner `ToolCallContext` clones increment `nesting_depth` by exactly 1. |
| T-5-03 | execute_commands shell injection | mitigate | CLOSED | `crates/kay-tools/src/builtins/execute_commands.rs:179-224` — `split_shell_tokens` is quote-aware, handles double-quoted, single-quoted, backslash-escaped segments, whitespace / `;` / `\|` / `&` separators. `should_use_pty` walks EVERY token (not just the first argv). SHELL-05 defense rejects any command containing the sentinel marker `__CMDEND_`. SIGTERM grace cascade uses `killpg` to reap process groups. |
| T-5-04 | image_read OOM | mitigate | CLOSED | `crates/kay-tools/src/builtins/image_read.rs:48` — `DEFAULT_MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024`. Metadata-first gate at `image_read.rs:193-207`: `tokio::fs::metadata(&path_buf).await` before `tokio::fs::read`. `if meta.len() > self.max_image_bytes { self.quota.release(); return Err(ToolError::ImageTooLarge { path, actual_size, cap }); }`. Quota reservation rolled back on all error arms (Io, sandbox denial, over-cap) so a failed over-cap call does NOT count toward per-turn / per-session image caps. `ToolError::ImageTooLarge { path, actual_size, cap }` variant in `error.rs:37-38`. |
| T-5-05 | select starvation / control wedge | mitigate | CLOSED | `crates/kay-core/src/loop.rs:436-437` — `tokio::select! { biased; ... }` with control arm in first position. Biased discipline guarantees the runtime always polls the control channel before input / tool / model channels on every iteration, so Pause/Resume/Abort are never starved by a tight model-event loop. |
| T-5-06 | SIGINT → Abort race | mitigate | CLOSED | `crates/kay-core/src/control.rs` — `install_ctrl_c_handler(tx: mpsc::Sender<ControlMsg>) -> std::io::Result<()>` installs the OS signal handler synchronously via `tokio::signal::unix::signal(SignalKind::interrupt())`. Each call spawns a fresh per-handler task; no shared state leaks across repeat installs. `crates/kay-cli/src/run.rs:282` calls `install_ctrl_c_handler(control_tx.clone())?` BEFORE `tokio::spawn(offline_provider...)` + `tokio::spawn(run_turn...)`, guaranteeing the handler is armed before any work is dispatched (the 200 ms settle window in `exit_code_130_on_sigint_nix` validates this). |
| T-5-07 | Exit-code confusion | mitigate | CLOSED | `crates/kay-cli/src/exit.rs` — `#[repr(u8)] enum ExitCode { Success = 0, RuntimeError = 1, SandboxViolation = 2, ConfigError = 3, UserAbort = 130 }` is exhaustive. `classify_error` walks `e.chain()` for `PersonaError → ConfigError`, else `RuntimeError`. `run.rs:398-404` enforces precedence `UserAbort > SandboxViolation > Success` on the happy path. `main.rs:98-134` bypasses stdlib's default `Termination` by returning `()` from `main` and calling `std::process::exit(code.as_u8() as i32)` — every error path is classified explicitly, never collapsed onto exit 1. |

### Unregistered Flags

None. SUMMARY.md `## Threat Flags` section was not emitted by the executor for Phase 5 (zero new attack-surface flags raised during implementation beyond the 9 declared threats).

## Broad Security Sweep (Step 10 / silver:security)

### CVE / Dependency Audit

- **Tool:** `cargo-audit 0.22.1` against local `Cargo.lock` on HEAD `af707d6`.
- **Result:** `vulnerabilities.found=false, vulnerabilities.count=0` (scanned 1050 advisories against 840 dependencies).
- **Ignored advisories:** 10 entries in `/Users/shafqat/Documents/Projects/opencode/vs-others/audit.toml`. All ten are documented as originating in transitive deps of parity-imported `forge_*` crates (ForgeCode Phase 2.5 verbatim import). Nightly security audit workflow still fires on these informationally so visibility is preserved. Ignore categories:
  - `gix-date` (RUSTSEC-2025-0140), `gix-features` (RUSTSEC-2025-0021) — via `forge_repo → gix`; not exploitable on Kay's path (no git object verification on untrusted input in v0.1).
  - `rustls-webpki` (RUSTSEC-2026-0098, RUSTSEC-2026-0099) — reachable only through MCP client which Phase 3 rewires; Kay's main TLS path (reqwest + rustls 0.23 + webpki-roots) is clean.
  - Informational / unmaintained: `bincode` (RUSTSEC-2025-0141), `paste` (RUSTSEC-2024-0436), `proc-macro-error` (RUSTSEC-2024-0370), `yaml-rust` (RUSTSEC-2024-0320), `libyml` (RUSTSEC-2025-0067), `serde_yml` (RUSTSEC-2025-0068).
- **Phase 5 introduces no new dependencies** with outstanding CVEs. All ten ignores predate this phase and are tracked as v0.2.0 parity-backlog items.

### Unsafe Code

Grepped Phase 5 crate source directories (`kay-cli/src/**`, `kay-core/src/**`, `kay-tools/src/**`) for `unsafe` blocks. **Zero unsafe blocks** in Phase 5 scope. All `unsafe` in the workspace resides in out-of-scope non-Phase-5 crates (`forge_infra`, `forge_main` zsh/vscode integration shims, `kay-provider-openrouter` auth/allowlist wrappers reviewed in Phase 3, `kay-sandbox-windows` Win32 FFI reviewed in Phase 4, `forge_tracker`, `forge_config`). Phase 5 surface itself is 100 % safe Rust.

### Secret Leakage

Grepped Phase 5 crate source dirs for `(api[_-]?key|secret|password|token|bearer)\s*[:=]\s*["'][^"'\s]{8,}["']` case-insensitive. **No matches.** Visual inspection of all three bundled personas (`personas/forge.yaml`, `personas/sage.yaml`, `personas/muse.yaml`) confirms no embedded credentials — personas carry model names, tool filters, and natural-language system prompts only. CLI banner, prompt, exit, interactive, run modules inspected directly: no hard-coded credentials, no tokens, no API keys, no bearer strings. Model names flow through `forge_api::ModelId`, not an opaque secret.

### Input Validation / Deserialization

`#[serde(deny_unknown_fields)]` verified on every externally-parsed input type in Phase 5:

- `kay-core::persona::Persona` — `persona.rs:84` (plus `validate_against_registry` + `validate_model` post-parse validators).
- `kay-tools::builtins::image_read::ImageReadArgs` — `image_read.rs:55` (`#[serde(rename_all = "snake_case", deny_unknown_fields)]`).
- `kay-tools::builtins::sage_query::SageQueryArgs` — confirmed in Phase 3 carry-forward.
- `kay-tools::builtins::execute_commands::ExecuteCommandsInput` — confirmed in Phase 3 carry-forward + R-1 widening reviewed in this phase.
- `AgentEvent` and `ControlMsg` — not deserialized from untrusted sources. `AgentEvent` is only serialized out (via `AgentEventWire`); `ControlMsg` originates from the signal handler and the CLI REPL.

`AgentEventWire<'a>(pub &'a AgentEvent)` newtype (`events_wire.rs`) scrubs outbound payloads: `ImageRead` wire form emits `{type, path, size_bytes, encoding:"base64"}` and never serializes raw bytes. `Error` wire form omits source internals (reqwest/serde_json positions) for deterministic snapshots. Test `wire_never_leaks_image_bytes` asserts no `"bytes"` field in output.

### Denial-of-Service / Resource Bounds

- **mpsc channels:** All four primary channels (`control_tx/rx`, `input_tx/rx`, `tool_tx/rx`, `event_tx/rx`) are bounded with `mpsc::channel::<T>(32)`. `CONTROL_CHANNEL_CAPACITY = 32` declared in `control.rs`.
- **sage_query recursion:** Hard-capped at depth ≤ 2 (`MAX_NESTING_DEPTH = 2`, checked before arg parsing).
- **image_read memory:** Hard-capped at 20 MiB per call by default (`DEFAULT_MAX_IMAGE_BYTES`). Metadata gate runs BEFORE `tokio::fs::read`, so a prompt-injected 20 GiB path is rejected pre-allocation. Per-turn and per-session image quotas (`ImageQuota`) layer on top of the per-call cap.
- **execute_commands timeouts:** SIGTERM grace cascade with fixed 5-second grace; `ToolError::Timeout` variant explicit.
- **`pause_buffer` (INFO-1):** The agent-loop pause buffer is intentionally unbounded per decision DL-2 (05-CONTEXT.md). Rationale: pausing is a user-driven intentional action (`ControlMsg::Pause`); the buffer drains atomically on `Resume`; maximum held state is bounded by the number of model events a user delays acting on, which is operator-bounded. No adversary can arbitrarily grow the buffer without also arbitrarily extending the user's attention span. Documented as an accepted risk (INFO-1 below).
- **tokio::spawn usage:** All `tokio::spawn` calls are per-turn or per-command fixed fan-out (offline_provider once per turn, run_turn once per turn, stdout/stderr reader pair once per `execute_commands` invocation, one-shot per `install_ctrl_c_handler` call). No unbounded spawn loops.

### Panic Paths

Grepped `.unwrap() / .expect(` in Phase 5 production modules:

- `kay-cli/src/prompt.rs`: 9 `.unwrap()` on `write!(&mut String, ...)` — writing into a `String` is infallible, panics are unreachable at runtime. ASVS-compatible.
- `kay-core/src/persona.rs` + `kay-core/src/control.rs`: all `.unwrap() / .expect()` appear inside `///` doc examples or `#[cfg(test)]` blocks, not in production code paths.
- `kay-cli/src/{banner,main,exit,run,interactive,boot}.rs`: production error handling uses `?` or `match`; test `_ =` and `.ok()` idioms used deliberately for best-effort side effects (stdout flush on broken pipe, best-effort tracing).
- `kay-tools/src/builtins/**`: production paths thread errors via `ToolError`.

No `panic!()` or `unreachable!()` calls in Phase 5 production code.

### TLS / Transport Posture

Phase 5 does not introduce any new network surface. The only HTTP client in scope via the offline provider is a local, in-process `mpsc` handoff (no sockets). The real provider surface (`kay-provider-openrouter`) uses `https://openrouter.ai/api/v1/chat/completions` with reqwest + rustls 0.23 + webpki-roots (verified at `kay-provider-openrouter/src/client.rs:120, 126, 137`). No cleartext `http://` endpoints to external services introduced in Phase 5. The cleartext-`http://` matches the broad grep turned up are all in non-Phase-5 crates (`forge_infra` OAuth callbacks, `forge_main` model tests targeting `localhost`), `kay-sandbox-macos` probe commands, or `kay-tools/tests/**` test fixtures that point at `example.com` as synthetic allowlist-reject test data.

### Path Traversal / File-System Scoping

- **`image_read`:** Consults `ctx.sandbox.check_fs_read(&path_buf).await` BEFORE `tokio::fs::metadata` (`image_read.rs:173`). `NoOpSandbox` is a pass-through today; the real Phase 4 per-OS sandbox (`kay-sandbox-macos`, `-linux`, `-windows`) applies filesystem scoping at that seam. Defense-in-depth is correct.
- **`Persona::from_path`:** Uses `std::fs::read_to_string(path.as_ref())` without calling `canonicalize()` or stripping `..` components. This is acceptable because the `--persona` CLI flag is user-provided on the command line (the user already has full shell access by virtue of running `kay`), and persona YAML is treated as trusted configuration, not untrusted input. A malicious persona file would need to be placed by an attacker who already has write access to the user's filesystem — the threat model is "privilege escalation by the user against themselves," which is outside the sandbox boundary. Documented as LOW-1 (hardening note, not exploitable) below.

### Integer Over / Underflow

- `image_read` compares `meta.len(): u64` against `self.max_image_bytes: u64` — both u64, no overflow path.
- `sage_query` `nesting_depth: u8` incremented by `+ 1` on inner context clone, but the explicit pre-check `depth >= limit` at `depth = 2, limit = 2` rejects before any increment. u8 wraparound unreachable.
- `prompt::humanize_number` casts `usize → f64` — lossy above 2^53, but the input is a token count (model-reported, realistically ≤ millions), so precision loss is cosmetic, not exploitable.
- No `.unchecked_*` arithmetic calls anywhere in Phase 5 sources.

### Concurrency / Ordering

- Tokio biased `select!` guarantees priority: control > input > tool > model (`loop.rs:436-437`).
- `install_ctrl_c_handler` installs the OS handler SYNCHRONOUSLY (returns `io::Result<()>`), not via `tokio::spawn` fire-and-forget. The spawn inside it (`control.rs:189`) is an independent per-call task that reads signal state and sends on the channel — not an async init. This means the handler is guaranteed armed when `install_ctrl_c_handler` returns, which the `run.rs:282` call depends on for the SIGINT race test.
- `AgentEvent::Aborted` is emitted exactly once per `ControlMsg::Abort` (loop breaks on emit in `handle_control`), so `aborted_seen` can never double-fire.

### Credential / Auth Handling

Phase 5 does not introduce or modify authentication surfaces. OpenRouter auth is Phase 3 scope; the offline provider + canned events covered here need no credentials. `ToolCallContext::services` uses `NullServices` in the offline run path, which returns empty strings (no secret surface).

### Unused-import / Drift Sanity

Clean build verified via Phase 5 Step 7 (`/gsd-verify-work`) and captured in `05-VERIFICATION.md`. No `#[allow(dead_code)]` in production modules; all gates in `#[cfg(test)]` wrappers.

## Findings

| ID | Severity | File:Line | Category | Summary | Recommendation | Status |
|---|---|---|---|---|---|---|
| LOW-1 | LOW | `crates/kay-core/src/persona.rs:235-238` | Hardening — path resolution | `Persona::from_path` reads user-supplied YAML paths via `std::fs::read_to_string` without canonicalization. Not exploitable (user-controlled CLI flag, trusted config), but a future daemon / remote-control mode that takes persona paths from a non-CLI source would lose this assumption. | Add a defense-in-depth `canonicalize()` + root-scope check when Kay gains a non-CLI persona source (Phase 11+). Until then, document the trust model explicitly in the doc comment. | Accepted / deferred |
| LOW-2 | LOW | `audit.toml` (10 entries) | Hardening — CVE visibility | Ten `cargo audit` ignores cover parity-imported `forge_*` transitive deps. All are documented. None are reachable from Phase 5 code paths. | Track the v0.2.0 parity-backlog item that bumps these transitive deps; re-evaluate each ignore on the first dep update that drops them. | Accepted |
| INFO-1 | INFO | Agent-loop `pause_buffer` (05-CONTEXT.md DL-2) | Documentation — accepted risk | Pause buffer is intentionally unbounded. Growth is operator-bounded (a user must be actively not-resuming). Not an adversarial surface. | No change. Decision recorded in 05-CONTEXT.md § DL-2. | Accepted |

## Dependencies Audited

- **Tool:** `cargo-audit 0.22.1` with `audit.toml` ignores applied.
- **Advisory database snapshot:** 1050 advisories (last-commit `fded92d`, last-updated 2026-04-21).
- **Scan scope:** `Cargo.lock` at HEAD `af707d6c1f350c864c8094b69e3b11d3028a0821` — 840 dependencies.
- **Vulnerabilities found:** **0**.
- **Advisories explicitly ignored:** 10 (see `audit.toml`; all originate in pre-Phase-5 parity imports).
- **Phase 5 dependency delta:** no new `[dependencies]` entries added to workspace `Cargo.toml` for this phase that carry outstanding CVEs. Every Phase 5 addition is in the clean set.

## Recommendation

**PASS. Advance to Step 12 (`/gsd-validate-phase`).**

Phase 5 meets the security gate:

- [x] 9/9 threats CLOSED with cited grep-anchored evidence.
- [x] 0 BLOCKER findings.
- [x] 0 HIGH findings.
- [x] 0 unresolved cargo-audit vulnerabilities.
- [x] No unsafe code in Phase 5 scope.
- [x] No secret leakage.
- [x] Schema strictness enforced on every externally-parsed input.
- [x] Resource bounds enforced on every unbounded surface except the documented / accepted `pause_buffer` (INFO-1).
- [x] Exit-code precedence + SIGINT race validated by T7.8 regression test.
- [x] QG-C4 hard-gated by CI at 100 % coverage on `event_filter::for_model_context`.

No follow-up required before ship. LOW-1 and LOW-2 are hardening notes tracked in the backlog; neither blocks advancement.
