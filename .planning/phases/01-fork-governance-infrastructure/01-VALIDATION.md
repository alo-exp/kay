---
phase: 1
slug: fork-governance-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-19
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

Phase 1 is governance + scaffolding + CI. There is no production code to unit-test; validation here means "the scaffold compiles, CI gates trigger correctly, and all governance files pass their own assertions." This strategy treats scaffolding assertions as first-class tests.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) + shell-based CI assertion scripts + GitHub Actions dry-run via `act` (optional, local only) |
| **Config file** | `Cargo.toml` workspace, `.github/workflows/ci.yml` |
| **Quick run command** | `cargo check --workspace --all-features --offline` (≤ 30s after first warm cache) |
| **Full suite command** | `cargo test --workspace --all-features && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check && cargo deny check && cargo audit` |
| **Estimated runtime** | ~90–180s on a warm cache (macOS M-series); longer first-run and on tri-OS matrix |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --workspace --all-features` (quick) — must exit 0.
- **After every plan wave:** Run the full suite above (lint + test + deny + audit) locally before marking the wave complete.
- **Before `/gsd-verify-work`:** Full suite green locally AND CI matrix (macOS, ubuntu, windows) green on the PR for Phase 1.
- **Max feedback latency:** 180s (full suite on a warm cache). Quick-check latency < 10s.

---

## Per-Task Verification Map

The Phase 1 plan has not yet been produced by `gsd-planner`; the planner is expected to emit tasks with IDs matching `01-NN-MM`. This map is seeded with the verification strategy per REQ-ID so the planner can reference it when emitting `<automated>` blocks. Task IDs will be filled in by the planner's output; the Requirement / Secure Behavior / Test Type / Command columns are fixed.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| *(planner-assigned)* | 01 | 1 | GOV-01 | — | Fork attribution present in NOTICE, README, every crate's `authors` field | file-existence + content-grep | `test -f NOTICE && grep -q 'ForgeCode' NOTICE && grep -q '## Acknowledgments' README.md` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 01 | 1 | GOV-02 | — | `LICENSE` is Apache-2.0 at repo root; `NOTICE` lists ForgeCode copyright holders | content-grep | `grep -q 'Apache License' LICENSE && grep -q 'antinomyhq' NOTICE` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 02 | 1 | GOV-03 | — | DCO GitHub Action active in ci.yml; absent signoff blocks the PR | YAML assertion + integration test | `grep -q 'tim-actions/dco' .github/workflows/ci.yml && grep -q 'dco:' .github/workflows/ci.yml` | ✅ | ⬜ pending |
| *(planner-assigned)* | 02 | 1 | GOV-04 | — | `CONTRIBUTING.md` exists; documents DCO + clean-room attestation + PR process | content-grep | `test -f CONTRIBUTING.md && grep -q 'Developer Certificate of Origin' CONTRIBUTING.md && grep -q 'clean-room' CONTRIBUTING.md` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 02 | 2 | GOV-05 | — | Signed-tag gate job in ci.yml runs `git tag -v` and fails on unsigned tags for `refs/tags/v*` | YAML assertion | `grep -q 'git tag -v' .github/workflows/ci.yml && grep -q 'signed-tag-gate' .github/workflows/ci.yml` | ✅ | ⬜ pending |
| *(planner-assigned)* | 02 | 2 | GOV-06 | — | `SECURITY.md` exists with vulnerability-reporting flow + response SLA language | content-grep | `test -f SECURITY.md && grep -qE '(report|disclos)' SECURITY.md && grep -q 'Security Advisory' SECURITY.md` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 02 | 2 | GOV-07 | — | Clean-room attestation clause present in CONTRIBUTING.md citing the specific leak date + version | content-grep | `grep -q '2026-03-31' CONTRIBUTING.md && grep -q 'v2.1.88' CONTRIBUTING.md` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 03 | 1 | WS-01 | — | Rust 2024 cargo workspace with the 7 named crates present and recognized by `cargo metadata` | cargo metadata + grep | `cargo metadata --format-version 1 \| jq '.workspace_members \| length' \| grep -qE '^[7-9]$'` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 03 | 1 | WS-02 | — | Workspace-level `[workspace.dependencies]` pins tokio=1.51, reqwest=0.13, rustls=0.23 | TOML grep | `grep -A1 'tokio' Cargo.toml \| grep -q '1.51' && grep -A1 'reqwest' Cargo.toml \| grep -q '0.13'` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 04 | 1 | WS-03 | — | `deny.toml` blocks GPL/AGPL/LGPL and blocks `openssl`; `cargo-deny check` exits 0 | cargo-deny invocation | `cargo deny check 2>&1 \| tee /dev/stderr \| grep -q 'error\[' ; [ $? -ne 0 ]` | ❌ W0 | ⬜ pending |
| *(planner-assigned)* | 04 | 1 | WS-04 | — | `cargo-audit` runs in CI (PR + nightly); advisory DB auto-updated | YAML assertion + local dry run | `grep -q 'cargo audit' .github/workflows/ci.yml && cargo install cargo-audit --locked --quiet && cargo audit` | ✅ | ⬜ pending |
| *(planner-assigned)* | 04 | 2 | WS-05 | — | Workspace compiles clean on stable Rust with `--deny warnings` on macOS, Linux, Windows (CI matrix job green) | CI green on tri-OS | *(GitHub Actions check: `Test (ubuntu-latest)`, `Test (macos-latest)`, `Test (windows-latest)` all pass)* | ✅ | ⬜ pending |
| *(planner-assigned)* | 05 | 2 | EVAL-01 (scaffold-only per user amendment) | — | Harbor harness + `kay eval tb2` CLI shim + CI job stub exist and are runnable; the actual ≥80% reproduction run is deferred | file-existence + `kay eval tb2 --dry-run` exit 0 | `test -f kay-cli/src/eval.rs && cargo run -p kay-cli -- eval tb2 --dry-run` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Phase 1 has no production code; "Wave 0" here is the set of foundational files whose **absence** causes all verification commands above to fail. Planner should emit these as early tasks in Wave 1 so later waves can assert on them.

- [ ] `Cargo.toml` (workspace root) — required by every `cargo` invocation
- [ ] `rust-toolchain.toml` — pins the stable Rust channel for reproducibility across tri-OS
- [ ] `LICENSE` (Apache-2.0 text at repo root)
- [ ] `NOTICE` (attribution per Apache-2.0 §4(d))
- [ ] `README.md` (has `## Acknowledgments` section)
- [ ] `CONTRIBUTING.md` (DCO + clean-room attestation)
- [ ] `SECURITY.md` (vulnerability reporting)
- [ ] `deny.toml` (cargo-deny config with license blocks)
- [ ] `.github/workflows/ci.yml` (already exists from silver-init — extend, do not rewrite; add parity-gate job stub)
- [ ] `kay-core/` directory with imported ForgeCode source (single import commit, tagged `forgecode-parity-baseline`)
- [ ] `kay-cli/`, `kay-tauri/`, `kay-provider-openrouter/`, `kay-sandbox-macos/`, `kay-sandbox-linux/`, `kay-sandbox-windows/` — skeleton crates with `Cargo.toml` + `src/lib.rs` stubs only
- [ ] `kay-cli/src/eval.rs` — the `kay eval tb2 --dry-run` scaffold for EVAL-01 deferred-run

*Existing infrastructure covers:* `.github/workflows/ci.yml` (already has DCO + signed-tag gates), `.gitignore`, `docs/CICD.md`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ForgeCode clone SHA is current at import time | GOV-01 | Automation can't verify "latest at import" post hoc; this is captured in the import commit's `NOTICE` line and the `forgecode-parity-baseline` tag message | When executor imports: run `git ls-remote https://github.com/antinomyhq/forgecode.git main` the same day as the copy; record SHA in commit body and NOTICE. |
| Tag `forgecode-parity-baseline` is signed once signing key is procured | GOV-05 (deferred per D-OP-04) | Tag signing requires user GPG/SSH key, deferred to Phase 11 by user choice | Phase 11 task will re-tag or super-sede this tag with a signed equivalent. Phase 1 tag is unsigned by design and flagged in VERIFICATION.md. |
| CONTRIBUTING.md legal text review | GOV-04, GOV-07 | A lawyer should review the clean-room attestation clause before Kay accepts external contributions. For v0.x solo dev, self-review is acceptable | Phase 1 ships the attestation with a "self-reviewed; legal review encouraged before v1" note in SECURITY.md. |
| Parity baseline ≥ 80% on TB 2.0 | EVAL-01a (deferred per user amendment) | Requires OpenRouter API key + ~$100 budget, deferred to when key is provided | Split into follow-on task EVAL-01a; unblocks first Phase 2 harness merge. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify commands (columns above) or Wave 0 dependencies noted
- [ ] Sampling continuity: quick-check (`cargo check`) runs after every task commit — no 3 consecutive tasks without automated verify (enforced by sampling-rate policy above)
- [ ] Wave 0 covers all MISSING references (Cargo.toml, LICENSE, NOTICE, etc. all enumerated above)
- [ ] No watch-mode flags (all commands are one-shot)
- [ ] Feedback latency < 180s (full suite); < 10s (quick check)
- [ ] `nyquist_compliant: true` set in frontmatter (flipped when planner emits tasks and the map above has concrete task IDs)

**Approval:** pending — awaits planner task emission + `gsd-plan-checker` validation
