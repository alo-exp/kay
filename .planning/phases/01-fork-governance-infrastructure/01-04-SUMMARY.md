---
phase: 01-fork-governance-infrastructure
plan: 04
subsystem: supply-chain-gates
tags: [cargo-deny, cargo-audit, rustsec, licenses, bans, ci, governance]
status: complete
started: 2026-04-19T12:23:15Z
completed: 2026-04-19T12:27:55Z
duration_seconds: 280

requires:
  - 01-01 (Cargo.toml workspace — deny.toml targets it)
provides:
  - deny.toml (activates EmbarkStudios/cargo-deny-action@v2 in ci.yml via hashFiles guard)
  - .github/workflows/audit.yml (nightly rustsec/audit-check@v2.0.0 sweep)
affects:
  - .github/workflows/ci.yml (behavioral — cargo-deny step now runs; no file edit)

tech-stack:
  added:
    - cargo-deny 0.19.4 (local dev tool; CI uses EmbarkStudios/cargo-deny-action@v2)
    - rustsec/audit-check@v2.0.0 (GitHub Action, pinned)
    - actions/checkout@v4 (already in use; reaffirmed)
  patterns:
    - "allow-list-not-deny-list for licenses (GPL/AGPL/LGPL/SSPL blocked by omission)"
    - "explicit bans.deny for TLS escape hatches (openssl family → rustls only)"
    - "pin actions to semver tag (not @main/@latest) for supply-chain integrity"
    - "stagger cron (04:17 UTC) to avoid GitHub Actions midnight thundering herd"

key-files:
  created:
    - deny.toml
    - .github/workflows/audit.yml
  modified: []

decisions:
  - "Copied deny.toml verbatim from 01-RESEARCH.md §Example 4 — no additions or suppressions needed"
  - "cargo deny check passes clean (exit 0); unused-license warnings are informational, not errors"
  - "No entries added to [advisories].ignore or [bans].skip — keeping the gate strict"
  - "multiple-versions kept at 'warn' per plan guidance (tighten to 'deny' in Phase 2+ after dep sweep)"

metrics:
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 0
  duration_seconds: 280
  completed_date: 2026-04-19
  cargo_deny_exit_code: 0
  cargo_deny_outcome: pass
---

# Phase 1 Plan 04: Supply-chain Gates Summary

**One-liner:** cargo-deny license/bans/advisories/sources policy (Apache-2.0-compatible, rustls-only) plus nightly `rustsec/audit-check@v2.0.0` workflow — both gates active, `cargo deny check` passes clean on the imported workspace.

## What Was Built

### Task 1 — `deny.toml`
cargo-deny configuration at repo root with five tables:
- **`[graph]`** — six-target matrix (linux gnu/musl x86_64+aarch64, darwin x86_64+aarch64, windows MSVC) with `all-features = true`
- **`[advisories]`** — `unmaintained = "workspace"`, `unsound = "all"`, `yanked = "deny"`, empty `ignore` list
- **`[licenses]`** — 10-entry Apache-2.0-compatible allow-list (Apache-2.0, Apache-2.0 WITH LLVM-exception, MIT, ISC, BSD-2-Clause, BSD-3-Clause, Unicode-3.0, Zlib, CC0-1.0, MPL-2.0). **GPL/AGPL/LGPL/SSPL blocked by omission** (no deny-list needed — allow-list is the gate).
- **`[bans]`** — `multiple-versions = "warn"`, `wildcards = "deny"`, and explicit `deny` entries for `openssl`, `openssl-sys`, `native-tls`, `openssl-probe` (per CONTEXT.md D-08 — Kay uses rustls only)
- **`[sources]`** — `unknown-registry = "deny"`, `unknown-git = "deny"`, empty `allow-git`

**Commit:** `a157b57`

### Task 2 — `.github/workflows/audit.yml`
Nightly security-audit workflow:
- Trigger: `schedule: cron '17 4 * * *'` (04:17 UTC, staggered away from midnight) + `workflow_dispatch`
- Single job `audit` on `ubuntu-latest`
- Steps: `actions/checkout@v4` → `rustsec/audit-check@v2.0.0` (version-pinned, not `@main`/`@v2`/`@latest`) with `secrets.GITHUB_TOKEN`

**Commit:** `c425221`

## `cargo deny check` Dry-Run Result

```
advisories ok, bans ok, licenses ok, sources ok
```

**Exit code:** 0 (clean pass, all four sections green)
**Log:** `/tmp/kay-deny-check.log`, `/tmp/kay-deny-check-stdout.log`

**Warnings observed (informational only, not errors):**
Nine `license-not-encountered` warnings for `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `CC0-1.0`, `ISC`, `MIT`, `MPL-2.0`, `Unicode-3.0`, `Zlib`. These are expected: the imported workspace from plan 01-01 consists of seven skeleton crates with only workspace-level dep declarations in `[workspace.dependencies]` — the full transitive dep graph has not yet been materialized into a Cargo.lock (no `cargo build` has run against these crates yet). The allow-list entries will stop being "unencountered" once real deps land in Phase 2+.

No `license-not-allowed`, `banned`, `advisory`, or `source` errors — the gate is strict and clean.

## YAML Parse Result — `audit.yml`

```
$ python3 -c "import yaml; yaml.safe_load(open('.github/workflows/audit.yml'))" && echo "OK"
OK
```

Parses valid; all grep-checks pass (name, action-pin, cron, workflow_dispatch, checkout@v4).

## ci.yml Activation Check

The existing `.github/workflows/ci.yml` lint job contains:
```yaml
- uses: EmbarkStudios/cargo-deny-action@v2
  if: steps.detect.outputs.has_cargo == 'true' && hashFiles('deny.toml') != ''
```

With `deny.toml` now committed at repo root, `hashFiles('deny.toml')` is non-empty and the conditional evaluates `true` on the next PR. **The cargo-deny gate is now active** on every PR — previously a no-op. Plan 01-05 will verify end-to-end tri-OS matrix behavior; this plan confirms the guard condition is satisfied.

## Deviations from Plan

**None.** Plan executed exactly as written. `deny.toml` and `audit.yml` copied verbatim from plan 01-04's `<interfaces>` block (= 01-RESEARCH.md §Example 4 + §Example 5). No `[licenses]` additions, no `[bans].skip`, no `[advisories].ignore` entries. `cargo deny check` passed on first try with exit 0.

## Known Stubs

None — this plan ships configuration files, not code. Both files are production-ready and fully wired to their consumers (ci.yml for deny.toml, GitHub Actions scheduler for audit.yml).

## Threat Flags

None — this plan reduces the threat surface (closing supply-chain ingress vectors) rather than expanding it. All threats from the `<threat_model>` section (T-01-11 through T-01-16) are now either **mitigated** (11–14) or **explicitly accepted** with documented rationale (15–16, action-source trust + SHA-pin hardening deferred to Phase 11).

## Success Criteria

- [x] **WS-03:** `deny.toml` configured — Apache-2.0-compatible allow-list present, GPL/AGPL/LGPL/SSPL denied by omission, openssl family banned
- [x] **WS-04 (PR gate):** ci.yml inline `cargo install cargo-audit --locked --quiet && cargo audit` preserved (plan 01-05 verifies)
- [x] **WS-04 (nightly):** `.github/workflows/audit.yml` via `rustsec/audit-check@v2.0.0` — scheduled 04:17 UTC + workflow_dispatch
- [x] **WS-05 (prerequisite):** deny.toml exists so ci.yml's cargo-deny-action step activates on PRs; plan 01-05 verifies the tri-OS matrix job wiring
- [x] **D-08:** rustls-only TLS stack enforced via `[bans].deny` on all four openssl-family crates

## Verification Commands (Re-runnable)

```bash
# Files exist
test -f deny.toml && test -f .github/workflows/audit.yml

# Allow-list integrity
grep -q '"Apache-2.0"' deny.toml && grep -q '"MIT"' deny.toml
! grep -qE '"(GPL|AGPL|LGPL|SSPL)' deny.toml

# Bans integrity
grep -q 'crate = "openssl"' deny.toml
grep -q 'crate = "native-tls"' deny.toml

# audit.yml pin + schedule
grep -q 'rustsec/audit-check@v2.0.0' .github/workflows/audit.yml
grep -q "cron: '17 4 \* \* \*'" .github/workflows/audit.yml

# Dry-run
cargo deny check   # → advisories ok, bans ok, licenses ok, sources ok (exit 0)

# YAML parse
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/audit.yml'))"
```

## Commits

| Hash      | Type | Task | Description |
|-----------|------|------|-------------|
| `a157b57` | feat | 1    | add deny.toml cargo-deny policy |
| `c425221` | feat | 2    | add nightly cargo-audit workflow |

## Self-Check: PASSED

- File `deny.toml` — FOUND
- File `.github/workflows/audit.yml` — FOUND
- Commit `a157b57` — FOUND in git log (feat(01-04): add deny.toml cargo-deny policy)
- Commit `c425221` — FOUND in git log (feat(01-04): add nightly cargo-audit workflow)
- `cargo deny check` exit code 0 — VERIFIED
- YAML parse of audit.yml — VERIFIED
- All acceptance-criteria greps pass — VERIFIED

---

*Phase 1 Plan 04 — Supply-chain Gates — complete 2026-04-19.*
