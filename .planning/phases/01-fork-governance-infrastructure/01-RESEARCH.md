# Phase 1: Fork, Governance, Infrastructure - Research

**Researched:** 2026-04-19
**Domain:** Rust cargo workspace fork hygiene + Apache-2.0 + DCO governance + tri-OS CI + parity-gate scaffolding
**Confidence:** HIGH (governance, fork mechanics, CI actions verified this session) / MEDIUM (Harbor harness setup, NOTICE-for-fork exact text) / LOW (none — all Phase 1 claims were verified or sourced)

## Summary

Phase 1 is a governance, provenance, and infrastructure phase — not a code phase. Its output is a Rust cargo workspace forked from ForgeCode at a recorded SHA, with Apache-2.0 attribution that survives Apache §4(d), DCO enforcement, a tri-OS CI matrix, cargo-deny + cargo-audit gates, and a scaffolded-but-not-run parity-gate harness. The user amendments (2026-04-19) re-scoped EVAL-01 to "scaffolding runnable" and deferred signing-key procurement (D-OP-02/03/04) to Phase 11, which relaxes the lead-time pressure that originally made Phase 1 critical-path-heavy.

Verified this session against upstream ForgeCode: **ForgeCode has NO NOTICE file** at repo root [VERIFIED: raw.githubusercontent.com/antinomyhq/forgecode/main/NOTICE → 404]. Copyright holder is "Tailcall" via Apache-2.0 LICENSE [VERIFIED]. Workspace uses `resolver = "2"`, `edition = "2024"`, MSRV `1.92`, `channel = "1.92"` in rust-toolchain.toml, and has 23 crates under `crates/` [VERIFIED]. Latest stable Rust as of 2026-04-19 is **1.95.0** (released 2026-04-16) [VERIFIED: blog.rust-lang.org/2026/04/16/Rust-1.95.0]. The existing scaffolded `.github/workflows/ci.yml` already wires tim-actions/dco@master + tim-actions/get-pr-commits@v1.3.1 + a signed-tag-gate job + a tri-OS test matrix; Phase 1 extends, not rewrites.

**Primary recommendation:** Pin rust-toolchain.toml to `channel = "1.95"` (current stable) rather than inheriting ForgeCode's `1.92` pin — this avoids one of the "ForgeCode-isms" Pitfall 3 flagged and puts Kay on the supported release channel. Extend the existing ci.yml rather than rewriting it (scaffold is already correct). Because ForgeCode publishes no NOTICE, Kay's NOTICE must be constructed from scratch per Apache Infra's canonical template with attribution to Tailcall (the actual copyright holder). Defer the Harbor parity run to a follow-on task tagged EVAL-01a (per user amendment) while landing a runnable CLI shim + CI job stub in Phase 1.

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Fork & Attribution:**
- **D-01 (Fork strategy):** Clone ForgeCode at its current `main` HEAD, copy source into Kay's workspace, preserve commit-level provenance via a single `NOTICE` entry + a tagged git commit `forgecode-parity-baseline` pointing at the imported SHA. No `git subtree` / no submodules. Clean-cut fork with a recorded parity SHA.
- **D-02 (NOTICE content):** `NOTICE` at repo root lists: (a) ForgeCode's Apache-2.0 copyright holders (pulled from ForgeCode's own `NOTICE` / `LICENSE`), (b) a line stating "Portions of this codebase were derived from ForgeCode (https://github.com/antinomyhq/forgecode) at commit <SHA> on 2026-04-19, used under Apache-2.0", (c) Kay's own copyright line.
- **D-03 (Crate authors):** `authors = ["Kay Contributors <contributors@kay.dev>"]` in every workspace crate `Cargo.toml`. Individual contributions tracked via DCO signoff + git log.
- **D-04 (README attribution):** `README.md` has an `## Acknowledgments` section naming ForgeCode as the base harness and Terminus-KIRA as the source of the harness techniques.

**Workspace:**
- **D-05 (Workspace layout):** Mirror codex-rs — `kay-core`, `kay-cli`, `kay-tauri` (placeholder, Phase 9), `kay-provider-openrouter` (placeholder, Phase 2), `kay-sandbox-macos` / `kay-sandbox-linux` / `kay-sandbox-windows` (placeholders, Phase 4). P1 ships empty crate skeletons with only the public API outlines; parity gate runs against `kay-core` = imported ForgeCode source.
- **D-06 (Rust edition):** 2024.
- **D-07 (Workspace-level pinning):** tokio 1.51 LTS, reqwest 0.13, rustls 0.23, serde_json, schemars, tracing at `[workspace.dependencies]`. Child crates inherit.
- **D-08 (cargo-deny):** Block GPL/AGPL/LGPL, block `openssl` (use `rustls`), integrate cargo-audit for vuln checks.
- **D-09 (MSRV):** stable Rust pinned via `rust-toolchain.toml`. Bump only via explicit PR.

**DCO Enforcement:**
- **D-10:** `tim-actions/dco@master` + `tim-actions/get-pr-commits@v1.3.1` in `.github/workflows/ci.yml`. Already scaffolded.
- **D-11 (DCO bypass):** No bypass. Maintainers sign off too.
- **D-12 (DCO trailer):** Standard `Signed-off-by: Name <email>`. No custom trailers.

**Signed Tags:**
- **D-13:** Accept both GPG (RFC 4880) and SSH signatures (git 2.34+). `git tag -v` validates both.
- **D-14:** `signed-tag-gate` job (already scaffolded) runs `git tag -v <tag>` on any `refs/tags/v*` push. Unsigned tags fail the release pipeline.
- **D-15:** Public keys for release signers published at `docs/signing-keys/` + cross-referenced in `SECURITY.md`. Rotate annually or on compromise.

**Clean-room Attestation:**
- **D-16 (Attestation text):** `CONTRIBUTING.md` includes: "By signing off, I confirm I have not had exposure to leaked Claude Code source code (`@anthropic-ai/claude-code` v2.1.88 source map leak, 2026-03-31) and that this contribution contains no code derived from that leak."
- **D-17 (Enforcement posture):** Honor-system + PR review. No automated scanner. CONTRIBUTING.md text + DCO signoff is the legal shield.

**Parity Gate (EVAL-01):**
- **D-18 (Benchmark harness):** `harbor-framework/terminal-bench-2` via Docker. Pin Docker image SHAs + harness commit SHA.
- **D-19 (Models):** OpenRouter Exacto endpoint for Claude Opus 4.6 and GPT-5.4. Parity threshold: ≥80% on at least one.
- **D-20 (Archival):** Reference run archived at `.planning/phases/01-fork-governance-infrastructure/parity-baseline/` with Docker SHAs, OpenRouter model+date, full JSONL transcript, summary score, `forgecode-parity-baseline` git tag pointing at the exact source commit.
- **D-21 (CI parity check — deferred to Phase 2+):** Scaffolded in Phase 1 (job stub reads archived score + compares) but not fully wired until Phase 2.
- **D-22 (Sample budget):** $100 OpenRouter credits for initial parity baseline.

**Branch Protection:**
- **D-23 (main protection):** DCO green, CI lint/test matrix green, ≥1 approving review (interim: separate account or org account), linear history (rebase or squash), no force-push.
- **D-24 (Tags on main only):** Release tags cut from `main` only.

### Claude's Discretion

- Exact crate skeleton content within `kay-core` (modules, trait stubs) — emerges organically during Phase 2–5 work.
- `rust-toolchain.toml` exact version pin — whatever stable is on Phase 1 execution day.
- `deny.toml` full advisory/ban list — start from cargo-deny's default template, tune as first PRs surface false positives.

### Deferred Ideas (OUT OF SCOPE)

- **JSON NOTICE twin** (`NOTICE.json`) — supply-chain tooling convenience. Not v1-critical.
- **Automated clean-room leak scanner** — legally and operationally fraught. Deferred indefinitely.
- **Dedicated release-signing key (not user's personal)** — post-v0.1 concern.
- **Reproducible builds** (bit-identical binaries) — v1+ goal; v0.x builds are signed + attested but not strictly reproducible.

### User Amendments (2026-04-19)

- **D-OP-01 → scaffold-only:** Phase 1 delivers Harbor harness setup + `kay eval tb2` CLI shim + CI job stub. Actual reference run + `forgecode-parity-baseline` tag creation deferred to first phase where OpenRouter key + budget are available. EVAL-01 re-scoped to "parity-gate scaffolding complete and runnable." The ≥80% sub-requirement moves to follow-on task `EVAL-01a` that unblocks Phase 2's first harness modification.
- **D-OP-02 / D-OP-03 / D-OP-04 → deferred to Phase 11:** No procurement actions in Phase 1.
  - Signed-tag CI gate remains scaffolded but **does not run** in Phase 1 (no tags cut).
  - GOV-05 enforced by CI at release time.
  - `forgecode-parity-baseline` tag (when eventually created) must be signed — deferred along with the parity run.
  - Phase 11 plan must include procurement-lead-time prefix at Phase 9 or earlier.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GOV-01 | Fork of ForgeCode with upstream attribution in `NOTICE`, `README`, and crate `authors` preserving Apache-2.0 obligations | § ForgeCode Fork State; § NOTICE File Composition |
| GOV-02 | Apache-2.0 `LICENSE` present at repo root; `NOTICE` lists all upstream copyright holders | § NOTICE File Composition |
| GOV-03 | DCO (`Signed-off-by: Name <email>`) enforced on every commit via a GitHub Action that blocks unsigned PRs | § DCO Enforcement Flow |
| GOV-04 | `CONTRIBUTING.md` documents DCO, clean-room attestation, code-style, and PR process | § CONTRIBUTING.md Composition |
| GOV-05 | All release tags are GPG- or SSH-signed; CI refuses to publish an unsigned tag | § Signed-Tag Verification (scaffolded, does not run in P1 per amendment) |
| GOV-06 | `SECURITY.md` describes vulnerability reporting, private advisory flow, and response SLA | § SECURITY.md Composition |
| GOV-07 | Clean-room contributor attestation required | § Clean-Room Attestation Wording |
| WS-01 | Rust 2024 edition cargo workspace with kay-core, kay-cli, kay-tauri, kay-sandbox-* crates (mirrors codex-rs layout) | § Workspace Skeleton; canonical_refs ARCHITECTURE.md |
| WS-02 | Workspace-level pinning of tokio 1.51 LTS, reqwest 0.13, rustls 0.23, serde_json, schemars | canonical_refs STACK.md §Installation |
| WS-03 | `cargo-deny` configured to block GPL/AGPL transitive deps and known-vulnerable crates | § Initial deny.toml |
| WS-04 | `cargo-audit` runs in CI on every PR and nightly | § cargo-audit Workflow |
| WS-05 | Workspace compiles clean on stable Rust with `--deny warnings` on macOS, Linux, and Windows | § Tri-OS CI Matrix; existing ci.yml extension |
| EVAL-01 | Parity gate scaffolding complete and runnable (re-scoped per user amendment 2026-04-19) | § Parity-Gate Harness Scaffold |

## Architectural Responsibility Map

Phase 1 produces governance artifacts, CI configuration, and crate skeletons — not runtime behavior. The tier map is minimal for this phase.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Fork hygiene & attribution | Repo Root (static files) | — | LICENSE/NOTICE/README/ATTRIBUTIONS.md live at repo root by Apache convention |
| Workspace manifest + pinning | Repo Root (Cargo.toml) | Each crate Cargo.toml | `[workspace.dependencies]` at root; crates inherit via `workspace = true` |
| DCO enforcement | GitHub Actions (CI) | CONTRIBUTING.md (docs) | CI is the technical gate; CONTRIBUTING.md is the human-readable expectation |
| Signed-tag gate | GitHub Actions (CI) | docs/signing-keys/ (static) | CI verifies; docs publishes public keys for end users |
| cargo-deny / cargo-audit | GitHub Actions (CI) | deny.toml (repo root) | CI runs; config is declarative at root |
| Tri-OS compile matrix | GitHub Actions (CI) | rust-toolchain.toml (repo root) | CI runs matrix; toolchain.toml pins version for all OSes uniformly |
| Parity-gate scaffold | kay-cli (`kay eval tb2` shim) | Harbor Docker + CI job stub | CLI shim wraps Harbor; CI job stub reads archived score (not running Harbor in P1 CI) |
| Clean-room attestation | CONTRIBUTING.md (docs) | DCO signoff (CI) | Attestation text lives in CONTRIBUTING.md; DCO signoff is the legal vehicle |

No runtime tiers (browser/frontend-server/api/database) are exercised in Phase 1. The `kay-core` crate at phase end holds imported ForgeCode source but Kay's own agent loop does not run.

## Project Constraints (from CLAUDE.md)

These directives from `/Users/shafqat/Documents/Projects/opencode/vs-others/CLAUDE.md` are binding on Phase 1 planning. Planner must verify every task complies.

**Non-Negotiables (CLAUDE.md §Non-Negotiables):**

1. **Forked ForgeCode parity gate (Phase 1, EVAL-01)** — unmodified fork must reproduce ≥80% on TB 2.0 before any harness modification merges. Per user amendment, Phase 1 ships scaffolding only; the ≥80% reproduction becomes follow-on task EVAL-01a.
2. **No unsigned release tags** — every release tag GPG- or SSH-signed. CI blocks unsigned tag pushes. (Scaffolded in P1, does not fire until a tag is cut.)
3. **DCO (not CLA)** — `Signed-off-by: Name <email>` on every commit; GitHub Action blocks unsigned PRs.
4. **Clean-room contributor attestation** — contributors confirm no exposure to leaked Claude Code source (`@anthropic-ai/claude-code` v2.1.88 leak, 2026-03-31) before first merge. CONTRIBUTING.md spells this out; DCO signoff attests it.
5. **Single merged Rust binary** — ForgeCode harness merged into main Tauri binary; no `externalBin` sidecar. (Not materialized in P1 but locked as architectural constraint for future crates.)
6. **Strict OpenRouter model allowlist** — Exacto endpoints. (Not materialized in P1 beyond D-19 parity models; enforced in Phase 2.)
7. **ForgeCode's JSON schema hardening** — `required`-before-`properties`, flattening, explicit truncation reminders. (Load-bearing for TB2 score; enforced in Phase 3.)

**Session Startup Expectations (CLAUDE.md §Session Startup):**

- Read `.planning/STATE.md` for current phase cursor.
- Read `.planning/ROADMAP.md` for phase goal + success criteria.
- Never modify source under `src/` or crate directories without a `PLAN.md` that authorizes it.
- Never commit directly to `main`; all work flows through branches and PRs with signed-off commits.

**File Safety (CLAUDE.md §File Safety):**

- No `rm -rf`, `git reset --hard`, `git clean` without explicit user authorization for the specific target.
- No overwriting uncommitted work; stash or ask first.
- `.planning/` docs are project memory — preserve, don't rewrite history.

**Workflow (CLAUDE.md §Active Workflow — GSD):**

- All non-trivial changes go through `/gsd-plan-phase <N>`.
- Verify every phase before closing via `/gsd-verify-work <N>`.
- `/gsd-ship <N>` prepares PR; never bypass verify gate.

## Standard Stack

Phase 1 adds no runtime dependencies beyond ForgeCode's existing set (imported into kay-core) + the workspace-level pins listed in CONTEXT.md D-07. The phase-specific tools are **dev/CI tools**, not crates.

### CI / Governance Tools (all [VERIFIED])

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `tim-actions/dco` | `@master` (last release v1.1.0, 2021-06-10) | DCO signoff check | Listed as D-10 lock; already in `.github/workflows/ci.yml`. No newer maintained fork has achieved parity traction. [VERIFIED: github.com/tim-actions/dco] |
| `tim-actions/get-pr-commits` | `@v1.3.1` | Pulls the list of commits in a PR for dco action to consume | Standard pairing documented in tim-actions/dco README. [VERIFIED] |
| `EmbarkStudios/cargo-deny-action` | `@v2` | License + dependency advisories gate | Authoritative — published by Embark who maintains cargo-deny itself. Already wired in existing ci.yml (conditional on `deny.toml` existing). [VERIFIED: existing ci.yml] |
| `rustsec/audit-check` | `@v2.0.0` | cargo-audit via RustSec | Current stable, published by RustSec project maintainers. [VERIFIED: github.com/rustsec/audit-check] — Existing ci.yml runs `cargo install cargo-audit --locked --quiet && cargo audit` inline; migrating to the action is optional but reduces install cost. |
| `dtolnay/rust-toolchain` | `@stable` | Install Rust toolchain in CI | de-facto standard. Already wired in ci.yml. [VERIFIED] |
| `Swatinem/rust-cache` | `@v2` | cargo dep + compile cache across OSes | de-facto standard. Works on macOS/Linux/Windows in 2026; auto-workaround for cargo#8603 + actions/cache#403 corruption. [VERIFIED: Swatinem/rust-cache README] — Already wired in ci.yml. |
| `actions/checkout` | `@v4` | Git clone | standard. `fetch-depth: 0` required on the DCO + signed-tag jobs (needs full history / tags). [VERIFIED: existing ci.yml pattern] |

**Installation (no new crates added in Phase 1 beyond D-07's workspace pins):**

Phase 1 ships empty crate skeletons. Each crate's `Cargo.toml` references shared deps via `workspace = true`:

```toml
# kay-core/Cargo.toml  (placeholder skeleton)
[package]
name = "kay-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
# ...
```

**Version verification (2026-04-19):**

| Package | Pinned version | Latest stable (verified) | Verdict |
|---------|---------------|--------------------------|---------|
| Rust toolchain | D-07 does not pin (Claude discretion) | 1.95.0 (2026-04-16) [VERIFIED: blog.rust-lang.org/2026/04/16/Rust-1.95.0] | Pin `channel = "1.95"` in rust-toolchain.toml |
| tokio | 1.51.x | 1.51 LTS → Mar 2027; 1.52.x current [CITED: STACK.md §Core] | 1.51 LTS is correct for new project |
| reqwest | 0.13 | 0.13.x [CITED: STACK.md §Core] | correct |
| rustls | 0.23 | 0.23.x [CITED: STACK.md §Version Compatibility] | correct |
| edition | 2024 | 2024 [VERIFIED: ForgeCode uses edition 2024, MSRV 1.92] | correct; aligns with ForgeCode |

**Divergence from ForgeCode:** ForgeCode pins `channel = "1.92"` in its rust-toolchain.toml [VERIFIED]. Kay should pin `1.95` (current stable, 2026-04-16). Justification: Kay is 3 minor versions ahead of ForgeCode's pin, bringing stabilized features (cfg_select! macro in 1.95) and avoiding an unnecessary upstream inheritance. A comment in rust-toolchain.toml should note "Kay pins current stable, not ForgeCode's 1.92. Bump via explicit PR."

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tim-actions/dco | dcoapp/app (GitHub App) | App needs install permission at org level; Action is repo-scoped. D-10 locks tim-actions. [CITED: CONTEXT.md D-10] |
| tim-actions/dco | tisonkun/actions-dco or KineticCafe/actions-dco | Newer maintenance but no traction vs. the Linux-kernel-pattern tim-actions/dco. D-10 locks tim-actions. |
| EmbarkStudios/cargo-deny-action | manual `cargo install cargo-deny && cargo deny check` | The action caches the binary and is faster; matches existing ci.yml. |
| rustsec/audit-check | manual `cargo install cargo-audit && cargo audit` | Action creates issues automatically for advisories; manual install + run is already in place in existing ci.yml — either works. |

## Architecture Patterns

### System Architecture Diagram (Phase 1 scope)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      REPO ROOT (static artifacts)                   │
│                                                                     │
│  LICENSE (Apache-2.0, verbatim)                                     │
│  NOTICE  (new: attribution to Tailcall/ForgeCode + Kay copyright)   │
│  README.md §Acknowledgments (new: names ForgeCode + KIRA)           │
│  CONTRIBUTING.md (new: DCO + clean-room attestation)                │
│  SECURITY.md (new: vuln disclosure flow)                            │
│  ATTRIBUTIONS.md (new: UPSTREAM_COMMIT + list of derived files)     │
│  CODE_OF_CONDUCT.md (optional; standard)                            │
│  Cargo.toml ([workspace] + [workspace.dependencies] + pins)         │
│  Cargo.lock (committed per D-07 pinning)                            │
│  rust-toolchain.toml (channel = "1.95")                             │
│  deny.toml (cargo-deny config)                                      │
│  .rustfmt.toml (project fmt config)                                 │
│  docs/signing-keys/ (public key(s) published, unused in P1)         │
└─────────────────────────────────────────────────────────────────────┘
               │
               │ [workspace members = ["crates/*"]]
               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        crates/ (skeletons)                          │
│                                                                     │
│  kay-core/         ← HOLDS IMPORTED FORGECODE SOURCE (parity base)  │
│    src/*.rs          (unmodified; all 23 forge_* crates flatten in  │
│                       OR we preserve their structure; D-05 allows   │
│                       Claude's discretion — see Open Q1)            │
│                                                                     │
│  kay-cli/          ← SKELETON + `kay eval tb2` shim                 │
│    src/main.rs       (clap entry point; dispatches `eval tb2` to    │
│                       Harbor wrapper stub; stub prints              │
│                       "parity run deferred to EVAL-01a" in P1)      │
│                                                                     │
│  kay-tauri/                       (skeleton; Phase 9)               │
│  kay-provider-openrouter/         (skeleton; Phase 2)               │
│  kay-sandbox-macos/               (skeleton; Phase 4)               │
│  kay-sandbox-linux/               (skeleton; Phase 4)               │
│  kay-sandbox-windows/             (skeleton; Phase 4)               │
└─────────────────────────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 .github/workflows/ci.yml (extend existing)          │
│                                                                     │
│  dco             (already present; tim-actions/dco)                 │
│  lint            (already present; fmt + clippy + deny + audit)     │
│  test matrix     (already present; ubuntu/macos/windows)            │
│  frontend        (already present; no-op in P1 since no UI yet)     │
│  signed-tag-gate (already present; dormant — no tags in P1)         │
│  parity-gate     (NEW in P1: job stub, workflow_dispatch only,      │
│                   reads archived score + compares; no Harbor run)   │
└─────────────────────────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────────┐
│             .planning/phases/01-.../parity-baseline/                │
│                                                                     │
│  (empty in P1, populated on EVAL-01a)                               │
│  manifest.json   Docker image SHAs, Harbor commit SHA, OR model,    │
│                  date, submission seed                              │
│  transcript.jsonl  full run transcript                              │
│  summary.md      score, pass/fail per task, budget spent            │
│  PARITY-DEFERRED.md  (PLACEHOLDER in P1: states run is deferred +   │
│                        links to EVAL-01a follow-on task)            │
└─────────────────────────────────────────────────────────────────────┘
```

Data flow: PRs → DCO + lint + matrix gates in CI → signed-off commits merge to main → release tags cut from main (P11+) → signed-tag-gate verifies → release artifacts published.

### Recommended Project Structure (Phase 1 deliverable)

```
.
├── .github/workflows/ci.yml                (extend existing)
├── .gitignore                              (already present)
├── Cargo.toml                              (NEW — workspace root)
├── Cargo.lock                              (NEW — committed)
├── rust-toolchain.toml                     (NEW)
├── deny.toml                               (NEW)
├── .rustfmt.toml                           (NEW — mirror ForgeCode's)
├── LICENSE                                 (NEW — Apache-2.0 verbatim)
├── NOTICE                                  (NEW — attribution)
├── README.md                               (NEW — with §Acknowledgments)
├── ATTRIBUTIONS.md                         (NEW — UPSTREAM_COMMIT + derived list)
├── CONTRIBUTING.md                         (NEW — DCO + clean-room)
├── SECURITY.md                             (NEW — vuln disclosure)
├── CODE_OF_CONDUCT.md                      (NEW — Contributor Covenant v2.1)
├── docs/
│   ├── signing-keys/                       (NEW — empty in P1 beyond a README)
│   │   └── README.md                       (placeholder; keys published in P11+)
│   ├── CICD.md                             (already present; update for P1)
│   └── … (existing docs/)
├── crates/
│   ├── kay-core/
│   │   ├── Cargo.toml
│   │   ├── NOTICE                          (crate-level; minimal)
│   │   └── src/ (imported forgecode source — see Open Q1)
│   ├── kay-cli/
│   │   ├── Cargo.toml
│   │   └── src/main.rs                     (clap + `kay eval tb2` shim)
│   ├── kay-tauri/
│   │   ├── Cargo.toml                      (skeleton only)
│   │   └── src/lib.rs                      (stub)
│   ├── kay-provider-openrouter/
│   │   ├── Cargo.toml                      (skeleton only)
│   │   └── src/lib.rs                      (stub)
│   ├── kay-sandbox-macos/
│   │   ├── Cargo.toml                      (skeleton)
│   │   └── src/lib.rs
│   ├── kay-sandbox-linux/
│   │   ├── Cargo.toml                      (skeleton)
│   │   └── src/lib.rs
│   └── kay-sandbox-windows/
│       ├── Cargo.toml                      (skeleton)
│       └── src/lib.rs
└── .planning/phases/01-fork-governance-infrastructure/
    ├── 01-CONTEXT.md                       (already present)
    ├── 01-RESEARCH.md                      (this file)
    ├── 01-PLAN.md                          (next step)
    └── parity-baseline/
        └── PARITY-DEFERRED.md              (NEW; states EVAL-01a follow-on)
```

### Pattern 1: Workspace Pinning via `[workspace.package]` + `[workspace.dependencies]`

**What:** Root `Cargo.toml` declares shared package metadata and dependency versions once. Every crate's own `Cargo.toml` uses `field.workspace = true` to inherit. Eliminates per-crate drift. [CITED: doc.rust-lang.org/cargo/reference/workspaces.html]

**When to use:** Always for multi-crate Rust workspaces in Rust 2024 edition.

**Example root Cargo.toml skeleton:**

```toml
# Cargo.toml (repo root)
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.95"
authors = ["Kay Contributors <contributors@kay.dev>"]
license = "Apache-2.0"
repository = "https://github.com/alo-exp/kay"
homepage = "https://github.com/alo-exp/kay"
description = "Open-source terminal coding agent — Rust fork of ForgeCode with KIRA harness and Tauri desktop UI"

[workspace.dependencies]
# Async runtime (LTS → Mar 2027)
tokio = { version = "1.51", features = ["rt-multi-thread", "macros", "io-util", "process", "signal", "sync", "fs", "time"] }

# HTTP + TLS
reqwest = { version = "0.13", default-features = false, features = ["rustls-tls", "json", "stream", "gzip", "brotli"] }
reqwest-eventsource = "0.6"
rustls = "0.23"

# Serde stack
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"

# CLI + logging
clap = { version = "4.5", features = ["derive", "env"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Error handling
anyhow = "1"
thiserror = "2"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

[CITED: STACK.md §Installation — versions; VERIFIED: ForgeCode uses similar release profile (lto = true, codegen-units = 1, opt-level = 3, strip) per our earlier WebFetch on its Cargo.toml]

### Pattern 2: DCO-Enforced Commit Signoff (Linux-kernel pattern)

**What:** Every commit includes `Signed-off-by: Name <email>` trailer. CI fails the PR if any commit is missing signoff. DCO text itself is unmodified v1.1 from developercertificate.org.

**When to use:** Always for an Apache-2.0 project that wants contributor provenance without CLA friction (CONTEXT.md D-10 locks this).

**Example (already present in ci.yml):**

```yaml
dco:
  name: DCO signoff check
  runs-on: ubuntu-latest
  if: github.event_name == 'pull_request'
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0        # REQUIRED — DCO walks history
    - uses: tim-actions/get-pr-commits@v1.3.1
      id: pr_commits
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
    - uses: tim-actions/dco@master
      with:
        commits: ${{ steps.pr_commits.outputs.commits }}
```

[VERIFIED: existing `.github/workflows/ci.yml` lines 18-33 already wires this exactly as recommended by tim-actions/dco README]

### Pattern 3: Signed-Tag Gate via `git tag -v`

**What:** On push to `refs/tags/v*`, CI runs `git tag -v $TAG`. `git tag -v` validates both GPG (RFC 4880) and SSH signatures transparently in git ≥2.34. [VERIFIED: docs.github.com/en/authentication/managing-commit-signature-verification/signing-tags]

**When to use:** Always for the release gate (GOV-05). D-14 locks this.

**Example (already present in ci.yml, lines 129-143):**

```yaml
signed-tag-gate:
  name: Block unsigned tag on release
  runs-on: ubuntu-latest
  if: startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    - name: Verify tag signature
      run: |
        TAG="${GITHUB_REF#refs/tags/}"
        if ! git tag -v "$TAG"; then
          echo "::error::Tag $TAG is not GPG/SSH-signed. Unsigned release tags are rejected (see PROJECT.md GOV-05)." >&2
          exit 1
        fi
```

**GitHub UI display:** GitHub renders SSH-signed tags with a "Verified" badge when the signing SSH key is registered as a signing key on the contributor's GitHub account. This is native since 2022-08-23. [VERIFIED: github.blog/changelog/2022-08-23-ssh-commit-verification-now-supported + docs.github.com "If a tag has an SSH signature that is cryptographically verifiable, GitHub marks the tag 'Verified' or 'Partially verified.'"]

**Caveat:** Per user amendment, no tags are cut in Phase 1. This job remains dormant. The `forgecode-parity-baseline` tag (when eventually created in EVAL-01a) must be signed — or the gate fires on the first release push.

### Pattern 4: Tri-OS Rust Test Matrix

**What:** A single job uses `strategy.matrix.os` to run on `ubuntu-latest`, `macos-latest`, `windows-latest` in parallel. `fail-fast: false` so one OS failing doesn't cancel the others.

**When to use:** Always when shipping a cross-platform single-binary Rust product (WS-05 is explicit about tri-OS compile clean).

**Example (already present in ci.yml, lines 65-88):**

```yaml
test:
  name: Test (${{ matrix.os }})
  runs-on: ${{ matrix.os }}
  strategy:
    fail-fast: false
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - run: cargo test --workspace --all-features
```

**Windows-specific gotchas (Phase 1 — minimal; full set lands Phase 4):**

- Swatinem/rust-cache@v2 works on Windows as of 2026; earlier cache-corruption issues are fixed. [VERIFIED: Swatinem/rust-cache README]
- Use `shell: bash` on the `Detect Cargo workspace` step (already correct in existing ci.yml line 76) — Windows default shell differs.
- `cargo fmt --check` and `clippy -- -D warnings` behave consistently across OSes. The deeper ConPTY issues documented in PITFALLS.md Pitfall 14 land in Phase 4, not Phase 1.

### Pattern 5: Parity-Gate Harness Scaffold (CLI Shim + Dormant CI Job)

**What:** A `kay eval tb2` CLI subcommand under `kay-cli` wraps Harbor Framework's `harbor run` invocation. In Phase 1 it prints a descriptive "run deferred to EVAL-01a" message with instructions. A CI job stub (`workflow_dispatch` only) exists but does nothing beyond reading an archived score manifest (absent in P1).

**When to use:** Parity-run is expensive ($100 OpenRouter budget) and requires user-procured credentials. Scaffolding lets Phase 2 trigger the run without discovering missing infrastructure.

**Example CLI shim (kay-cli/src/main.rs skeleton):**

```rust
// Kay v0.0.1 — kay-cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kay", version, about = "Kay — terminal coding agent")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run evaluation harnesses (Terminal-Bench 2.0, etc.)
    Eval {
        #[command(subcommand)]
        target: EvalTarget,
    },
}

#[derive(Subcommand)]
enum EvalTarget {
    /// Terminal-Bench 2.0 parity run via Harbor
    Tb2 {
        #[arg(long, default_value = "anthropic/claude-opus-4.6")]
        model: String,
        #[arg(long, default_value_t = 89)]
        tasks: u32,
        #[arg(long, default_value = ".planning/phases/01-fork-governance-infrastructure/parity-baseline")]
        archive_dir: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval { target: EvalTarget::Tb2 { model, tasks, archive_dir } } => {
            eprintln!("[kay eval tb2] Parity run deferred to EVAL-01a per user amendment 2026-04-19.");
            eprintln!("Would run: harbor run -d terminal-bench/terminal-bench-2 -m {model} -n {tasks}");
            eprintln!("Archive directory: {archive_dir}");
            eprintln!("Prerequisites (when run is enabled):");
            eprintln!("  - Docker installed + running");
            eprintln!("  - uv tool install harbor  (or pip install harbor)");
            eprintln!("  - OPENROUTER_API_KEY set (scope: benchmark budget ≤ $100)");
            eprintln!("  - DAYTONA_API_KEY set (for --env daytona)");
            eprintln!("On completion, tag HEAD as 'forgecode-parity-baseline' (signed).");
            Ok(())
        }
    }
}
```

[CITED: harborframework.com/docs/tutorials/running-terminal-bench — invocation pattern `harbor run -d terminal-bench/terminal-bench-2 -m <model> -a <agent> --env daytona -n <N>`]

**Example parity-gate CI job (new addition to ci.yml):**

```yaml
parity-gate:
  name: Parity gate (EVAL-01) — scaffolded, not run in P1
  runs-on: ubuntu-latest
  if: github.event_name == 'workflow_dispatch'
  steps:
    - uses: actions/checkout@v4
    - name: Check for archived parity run
      run: |
        ARCHIVE=".planning/phases/01-fork-governance-infrastructure/parity-baseline"
        if [ ! -f "$ARCHIVE/summary.md" ]; then
          echo "::notice::Parity run not yet executed — EVAL-01a is the follow-on task that produces $ARCHIVE/summary.md."
          echo "Phase 1 ships scaffolding only per user amendment 2026-04-19."
          exit 0
        fi
        SCORE=$(grep -oP 'score:\s*\K[0-9.]+' "$ARCHIVE/summary.md")
        echo "Archived parity score: $SCORE"
        python3 -c "exit(0 if float('$SCORE') >= 80.0 else 1)"
```

### Anti-Patterns to Avoid

- **Publishing a modified LICENSE file.** Apache-2.0 LICENSE must be verbatim. Modifications invalidate the license grant. Kay's own copyright goes in NOTICE, not LICENSE. [CITED: infra.apache.org/licensing-howto.html]
- **Squashing ForgeCode's history into one commit with no provenance record.** D-01 specifies a single import commit + `forgecode-parity-baseline` tag. Pair this with ATTRIBUTIONS.md listing the imported upstream SHA so blame-level tracing is possible if a DMCA or license question emerges. [CITED: PITFALLS.md §Pitfall 2]
- **Copying ForgeCode's `rust-toolchain.toml` verbatim (pins 1.92).** Kay starts at 1.95. See "Divergence from ForgeCode" table above. [CITED: PITFALLS.md §Pitfall 3 — ForgeCode-isms audit]
- **Putting the DCO text inside CONTRIBUTING.md.** DCO v1.1 text should be referenced by link to `https://developercertificate.org/` (plus a short quoted summary). Contributors certify by signing off; they don't need to read the full text in-tree. [CITED: kernel.org submitting-patches.rst pattern]
- **Accepting force-pushes to main.** D-23 explicitly disallows. Configure branch protection accordingly.
- **Using a custom `Signed-off-by:`-like trailer.** D-12 locks the standard form. Divergence breaks every DCO tool in the ecosystem.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DCO signoff parsing | Shell script that greps commit messages | `tim-actions/dco@master` + `tim-actions/get-pr-commits@v1.3.1` | Handles merge commits, co-authors, multiple signoffs per commit, case variants. Edge cases you won't catch without 5 years of fallout. [VERIFIED: D-10 locks this] |
| License policy enforcement | Custom `grep` over Cargo.lock | `EmbarkStudios/cargo-deny-action@v2` with `deny.toml` | SPDX expression parsing, dual-license handling, transitive traversal, exception lists. [VERIFIED: existing ci.yml wires this] |
| Vulnerability scanning | Manually checking RustSec advisories | `cargo-audit` (via `rustsec/audit-check@v2.0.0` OR inline install) | Pulls RustSec DB nightly; tracks `cargo.lock` drift. [VERIFIED: rustsec/audit-check v2.0.0 released 2024-09-23] |
| Tag signature verification | Hand-written SSH/GPG validator | `git tag -v` | Git ≥2.34 validates both formats via the existing git tooling you already use. [VERIFIED: docs.github.com signing-tags] |
| Cross-OS cache | Manual `actions/cache@v4` with hand-rolled keys | `Swatinem/rust-cache@v2` | Auto-workarounds for cargo#8603 / actions/cache#403 corruption on macOS; Windows issue fixed. [VERIFIED: Swatinem/rust-cache README] |
| NOTICE file text | Free-form attribution prose | Apache Infra's canonical template | Apache §4(d) has specific requirements; "brief and simple" is the Apache Infra directive. Long NOTICEs get flagged. [CITED: infra.apache.org/licensing-howto.html] |
| CONTRIBUTING.md structure | From-scratch | rust-github/template + Linux-kernel DCO section | Known-good templates reduce reviewer friction and cover the standard questions. [CITED: github.com/rust-github/template] |
| SECURITY.md structure | From-scratch | OpenSSF `oss-vulnerability-guide` github_security_policy.md template | Authoritative OSS vuln-disclosure template. [CITED: github.com/ossf/oss-vulnerability-guide] |
| CODE_OF_CONDUCT.md | From-scratch | Contributor Covenant v2.1 | Ecosystem standard; maps cleanly onto GitHub's community health surfacing. [CITED: contributor-covenant.org] |

**Key insight:** Phase 1 is the phase most susceptible to "I can write it faster than I can look it up" mistakes. The cost of a hand-rolled DCO parser or a homemade NOTICE file is paid only when the project is successful — it takes months to manifest and months more to clean up. Every row in this table has a canonical solution with no downside beyond adding it to `.github/workflows/ci.yml` or the repo root.

## Runtime State Inventory

> Phase 1 has NO runtime state to migrate — it is a greenfield project scaffold. The phase imports ForgeCode source into `kay-core/` but does not run the software in any form other than the parity-gate harness, which is scaffolded but not executed. No stored data, no live service configuration, no OS-registered state, no existing secrets, no build artifacts pre-dating Phase 1.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — fresh fork, no user data exists | None |
| Live service config | None — no services running yet | None |
| OS-registered state | None — no daemons/services/agents installed | None |
| Secrets/env vars | **Pre-existing secrets listed in docs/CICD.md§Required Environment** (`APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_CERTIFICATE_P12_BASE64`, `AZURE_SIGNING_*`, `MINISIGN_SECRET_KEY_BASE64`, `OPENROUTER_API_KEY`, `GH_RELEASE_TOKEN`) — **none required in P1** per user amendment (signing procurement deferred to P11; parity run deferred). | Confirm `OPENROUTER_API_KEY` is NOT required in P1 (deferred to EVAL-01a). All others are P11+. Document in SECURITY.md / CONTRIBUTING.md where keys will live when needed. |
| Build artifacts | None — first build of the workspace | None (Cargo.lock is generated during the first `cargo build` and committed per D-07) |

**Nothing found in any category beyond the anticipated secret list** (documented but not yet needed). No data migration required, no code edits needed to handle pre-existing state. This is a true greenfield phase.

## Common Pitfalls

### Pitfall 1: NOTICE File Constructed from Imagination Instead of Apache §4(d)

**What goes wrong:** A `NOTICE` file that rambles, credits unrelated contributors, includes marketing prose, or fails to preserve the Tailcall copyright line correctly. Apache §4(d) is precise about what must and must not be there. Getting this wrong is the most common fork-attribution failure and carries downstream packager risk (Homebrew, AUR, Debian can flag it). [CITED: PITFALLS.md §Pitfall 2 + infra.apache.org/licensing-howto.html]

**Why it happens:** ForgeCode has no NOTICE file [VERIFIED: 404], so there's nothing to copy verbatim. Teams fill the gap with creative writing instead of reading the Apache Infrastructure guide's template.

**How to avoid:** Follow the canonical Apache Royale template shape exactly (see § NOTICE File Composition below). Keep it brief. Do NOT include marketing prose; do NOT claim endorsement by Tailcall/antinomyhq; do NOT include the LICENSE text in NOTICE.

**Warning signs:**
- NOTICE longer than ~20 lines
- Words like "blessed by", "compatible with", "endorsed by" referring to ForgeCode
- References to Kay's features or benchmarks
- Copyright dates earlier than Kay's fork date (2026-04-19)

**Verification:** In CI, add a soft grep check that flags NOTICE > 50 lines for manual review.

### Pitfall 2: Clean-Room Attestation Wording Too Weak or Too Strong

**What goes wrong:** Too weak ("I have not seen any leaked source") provides no legal shield — every developer has seen snippets in social media. Too strong ("I have never viewed any Anthropic product") drives away contributors who tried Claude Code legitimately. The wording must anchor on the specific leak (v2.1.88, 2026-03-31) to be both enforceable and realistic. [CITED: PITFALLS.md §Pitfall 12 + CONTEXT.md D-16]

**Why it happens:** Drafting legal language without concrete facts produces either mealy-mouthed hedges or overbroad claims. CONTEXT.md D-16 already gives the exact text — this pitfall is about not second-guessing it.

**How to avoid:** Use D-16 verbatim:

> "By signing off, I confirm I have not had exposure to leaked Claude Code source code (`@anthropic-ai/claude-code` v2.1.88 source map leak, 2026-03-31) and that this contribution contains no code derived from that leak."

Do not reword. The exact leak date + version is what makes this a verifiable attestation rather than a vague promise.

**Warning signs:**
- Proposed PRs that "improve" the attestation wording
- Contributors asking "does this cover later leaks?" — answer: separate PRs update the clause; do not rewrite on the fly

### Pitfall 3: Inheriting ForgeCode's `rust-toolchain.toml` Unmodified (pins 1.92)

**What goes wrong:** ForgeCode pins Rust 1.92 [VERIFIED]. If Kay copies this verbatim, it silently ships on a stale toolchain. 1.92 is 3 minor versions behind 1.95 (current stable). This isn't broken, but it's a ForgeCode-ism we should exit immediately per PITFALLS.md Pitfall 3.

**Why it happens:** "Copy the whole thing, we'll worry about it later" — Phase 1 is exactly the phase where the later never comes.

**How to avoid:** Write a fresh `rust-toolchain.toml`:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.95"
profile = "default"
# Kay pins current stable, not ForgeCode's 1.92. Bump via explicit PR.
```

Note this divergence in ATTRIBUTIONS.md so future maintainers understand the deliberate choice.

**Warning signs:**
- A task description that says "import rust-toolchain.toml from ForgeCode"
- CI green but `cargo --version` printing 1.92 instead of 1.95

### Pitfall 4: Swatinem/rust-cache Tripping on First Run (No `Cargo.lock`)

**What goes wrong:** On the very first CI run (before Cargo.lock is committed), Swatinem/rust-cache may fail to hash dependencies or produce an empty cache. Subsequent runs are fine.

**Why it happens:** rust-cache's hash function inputs include Cargo.lock. Missing lockfile → null hash → cache miss or action error.

**How to avoid:** Commit Cargo.lock as part of the first workspace-creation commit, before pushing. If the first CI run flakes, it'll recover on the next push.

**Warning signs:**
- First CI run fails on the "cache" step but succeeds on retry
- Cache size metrics show 0 bytes on the first run

### Pitfall 5: DCO CI Job Blocking Legitimate First-Time Contributors Who Forgot `-s`

**What goes wrong:** A contributor opens their first PR, forgets `git commit -s`, hits a red CI, and gives up.

**Why it happens:** DCO is unfamiliar to many Rust-ecosystem contributors (CLA is more common historically). The error message from tim-actions/dco is terse.

**How to avoid:**
1. CONTRIBUTING.md's "Your first PR" section must include the one-liner: `git commit -s -m "fix: tidy widget naming"` and `git commit --amend -s` for fix-ups.
2. A PR template auto-inserted via `.github/pull_request_template.md` with a "DCO checklist" reminder.
3. Link the specific PR Check in README's Contributing section ("If CI fails with 'DCO check failed', see CONTRIBUTING.md §DCO for the fix").

**Warning signs:**
- Drive-by PRs closed without merge + no follow-up from the author

### Pitfall 6: `signed-tag-gate` Job Firing Accidentally on `v*`-Prefixed Non-Release Tags

**What goes wrong:** A maintainer pushes a tag like `v1-wip` or `v-draft` for local bookkeeping, triggering the `signed-tag-gate` job on a tag that was never intended for release. Job fails, pollutes CI dashboard.

**Why it happens:** The `startsWith(github.ref, 'refs/tags/v')` condition (existing ci.yml line 132) is a wide filter.

**How to avoid:** Use strict semver regex match. Replace the condition:

```yaml
# BEFORE (existing)
if: startsWith(github.ref, 'refs/tags/v')

# AFTER (stricter)
if: |
  github.ref_type == 'tag' &&
  startsWith(github.ref_name, 'v') &&
  contains(github.ref_name, '.')
```

Or, more robustly, compile a workflow_call pattern like `v[0-9]+.[0-9]+.[0-9]+*` using regex — but GitHub's `if:` expression syntax is limited. A simpler rule: maintainers do not push `v*` tags for any purpose other than release.

**Warning signs:**
- CI runs on tags that don't look like `v0.1.0` / `v1.2.3`

### Pitfall 7: cargo-deny Failing on Apache-2.0 + MIT Dual-Licensed Transitive Deps

**What goes wrong:** Standard rust dependencies (e.g., many hyperium crates) are `Apache-2.0 OR MIT`. If `deny.toml` lists only `Apache-2.0` in `[licenses].allow`, perfectly fine deps fail the gate.

**Why it happens:** The `OR`-expression semantics in SPDX need `MIT` (and often `Unicode-3.0`, `BSD-2-Clause`, `ISC`, `Zlib`, `Apache-2.0 WITH LLVM-exception`) explicitly allowed.

**How to avoid:** Start `deny.toml` from cargo-deny's own template (see § Initial deny.toml Content below). That template already lists the full set of permissive licenses Rust ecosystem uses.

**Warning signs:**
- First CI run fails lint on `cargo-deny check` with "rejected by license filter"

## Code Examples

### Example 1: NOTICE File Composition (Phase 1 new artifact)

[CITED: infra.apache.org/licensing-howto.html — canonical template]

**Repo-root NOTICE (new file):**

```
Kay
Copyright 2026 Kay Contributors

Portions of this product were derived from the ForgeCode project
(https://github.com/antinomyhq/forgecode), Copyright 2025 Tailcall,
licensed under the Apache License, Version 2.0.

Imported from ForgeCode at commit <SHA> on 2026-04-19.
Specific modifications made by Kay are listed in ATTRIBUTIONS.md.

This product makes use of dependencies with their own licenses.
See Cargo.lock and the output of `cargo-about generate` for a
complete manifest.
```

Notes:
- "Copyright 2026 Kay Contributors" on line 2 — Kay is a contributor collective, not a legal entity.
- "Copyright 2025 Tailcall" matches [VERIFIED] LICENSE line: "Copyright 2025 Tailcall".
- `<SHA>` placeholder filled at import commit time with the actual ForgeCode HEAD.
- Single blank line between paragraphs per Apache Infra convention.
- No marketing prose, no "endorsed by," no Kay feature list.

**Crate-level NOTICE (kay-core/NOTICE only, because kay-core holds derived source):**

```
This crate incorporates source code derived from ForgeCode
(https://github.com/antinomyhq/forgecode), Copyright 2025 Tailcall,
licensed under the Apache License, Version 2.0.

See the repository-level NOTICE for full attribution.
```

The skeleton crates (kay-cli, kay-tauri, kay-provider-openrouter, kay-sandbox-*) have NO crate-level NOTICE — they contain no derived source.

### Example 2: CONTRIBUTING.md (Phase 1 new artifact)

Concrete minimum viable text, assembled from rust-github/template + Linux-kernel submitting-patches.rst + CONTEXT.md D-16:

```markdown
# Contributing to Kay

Thank you for considering a contribution. Kay is Apache-2.0 licensed and uses
the Developer Certificate of Origin (DCO) to track contributor provenance.

## Developer Certificate of Origin (DCO)

Every commit must be signed off to certify the Developer Certificate of Origin
(https://developercertificate.org/, v1.1). This is done by adding a trailer:

    Signed-off-by: Your Name <your.email@example.com>

You can add it automatically via:

    git commit -s -m "feat: add widget"

Fix an un-signed-off commit with:

    git commit --amend -s        # last commit
    git rebase --signoff HEAD~3  # last three

By signing off, you assert all four clauses of DCO v1.1 — that you wrote the
contribution (or have the right to submit it), under the project's license,
and consent to the contribution being public.

## Clean-Room Attestation

By signing off, I confirm I have not had exposure to leaked Claude Code source
code (`@anthropic-ai/claude-code` v2.1.88 source map leak, 2026-03-31) and that
this contribution contains no code derived from that leak.

This attestation is required because Kay competes in the agentic-coding space
where the 2026-03-31 leak contaminated a large number of derived projects.
If you have been exposed to the leaked source, please do not contribute to
areas of Kay that could resemble the leaked code — raise the concern on the
PR and the maintainers will route review accordingly.

## Pull Request Process

1. Open an issue first for non-trivial changes.
2. One PR per logical change. Keep PRs focused.
3. Run `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
   before pushing.
4. CI must pass (DCO + lint + tri-OS tests + cargo-deny + cargo-audit).
5. All commits in a PR must carry `Signed-off-by:`. The DCO job fails the PR
   otherwise.
6. Maintainers review under the clean-room policy. PRs that claim clean-room
   provenance will be fast-tracked; PRs with ambiguous provenance may be
   routed through a second reviewer.

## Style

- Rust: project `rustfmt.toml` + `clippy -- -D warnings`.
- TypeScript (when UI lands in Phase 9+): project `biome` config.
- Commit messages: conventional commits (`feat:`, `fix:`, `docs:`, `chore:`,
  `refactor:`, `test:`, `ci:`) + a one-line summary, body if useful, plus
  `Signed-off-by:`.

## Reporting Security Issues

Please see SECURITY.md. Do not open public issues for security concerns.

## Code of Conduct

Kay follows the Contributor Covenant v2.1. See CODE_OF_CONDUCT.md.
```

### Example 3: SECURITY.md (Phase 1 new artifact)

Concrete minimum viable text, derived from OpenSSF oss-vulnerability-guide github_security_policy.md template + solo-maintainer realism:

```markdown
# Security Policy

## Supported Versions

Kay is pre-1.0 software. Only the most recent published tag on `main` is
supported. Security fixes are landed on `main` and released as new patch
versions.

| Version      | Supported |
|--------------|-----------|
| main / HEAD  | Yes       |
| Older tags   | No        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for a security concern.**

Report security issues via GitHub's private Security Advisory flow:

1. Go to https://github.com/alo-exp/kay/security/advisories
2. Click "Report a vulnerability"
3. Fill in the details

If you cannot use GitHub Security Advisories, email
security@kay.dev (preferred) with:
- A clear description of the vulnerability.
- Reproduction steps or a proof-of-concept.
- The version(s) affected.
- Your preferred name / handle for acknowledgment in the advisory.

## Response SLA

Kay is solo-maintained pre-1.0. Expect:
- **Acknowledgment** within 72 hours.
- **Assessment + triage** within 7 days.
- **Fix or mitigation plan** within 30 days for critical issues,
  90 days for lower-severity issues.

These SLAs will tighten as Kay matures. SLAs are best-effort, not guaranteed;
please be patient.

## Coordinated Disclosure

We follow coordinated disclosure:
- 90-day private disclosure window by default.
- Advisory published simultaneously with the fix release.
- Reporter credited in the advisory unless anonymity is requested.

## Release Signing

All release tags from v0.1.0 onward are GPG- or SSH-signed. Public signing
keys are published at https://github.com/alo-exp/kay/tree/main/docs/signing-keys.
Verify a release with:

    git tag -v v0.1.0

If `git tag -v` reports "no signature found" or a verification failure,
do not trust the release. Contact security@kay.dev.

## Dependency Hygiene

- `cargo-audit` runs on every PR and nightly against the RustSec Advisory
  Database.
- `cargo-deny` enforces a license allowlist (no GPL/AGPL/LGPL transitively)
  and blocks known-vulnerable crates.
- Dependency updates require a passing CI run.

## Attribution

Kay's security process is informed by the OpenSSF oss-vulnerability-guide
(https://github.com/ossf/oss-vulnerability-guide).
```

### Example 4: Initial `deny.toml` (Phase 1 new artifact)

Based on Embark's own deny.toml for cargo-deny, tuned for Kay's Apache-2.0 + rustls constraints [CITED: raw.githubusercontent.com/EmbarkStudios/cargo-deny/main/deny.toml — we read this verbatim this session]:

```toml
# deny.toml — Kay dependency policy
# See https://embarkstudios.github.io/cargo-deny/

[graph]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
all-features = true

[advisories]
# Pull RustSec advisories; fail on unmaintained/unsound.
unmaintained = "workspace"
unsound = "all"
yanked = "deny"
ignore = []  # Add entries here only with written rationale.

[licenses]
confidence-threshold = 0.93
# Kay requires Apache-2.0-compatible licenses. No GPL/AGPL/LGPL.
allow = [
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "MIT",
    "ISC",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Unicode-3.0",
    "Zlib",
    "CC0-1.0",
    "MPL-2.0",  # Weak copyleft; allowed because Rust ecosystem uses it broadly (e.g. webpki-roots)
]
exceptions = []
# Explicitly deny by omission: GPL-*, AGPL-*, LGPL-*, SSPL-*, any copyleft
# that would propagate to Kay's Apache-2.0 license grant.

[bans]
multiple-versions = "warn"  # Start as warn; tighten to deny once clean.
wildcards = "deny"

# Deny openssl — Kay uses rustls per CONTEXT.md D-08.
deny = [
    { crate = "openssl", use-instead = "rustls" },
    { crate = "openssl-sys", use-instead = "rustls" },
    { crate = "native-tls", use-instead = "rustls" },
    { crate = "openssl-probe", reason = "indicates openssl-backed TLS; Kay uses rustls" },
]

skip = []
skip-tree = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-git = []  # No git dependencies by default.
```

**Note:** `multiple-versions = "warn"` initially rather than `"deny"` — the Rust ecosystem has legitimate version-drift (e.g. windows-sys) that's noisy. Tighten to `"deny"` after the first Phase 2 dependency sweep.

### Example 5: `cargo-audit` Workflow Addition

The existing ci.yml runs cargo-audit inline in the lint job (line 60-63). That's sufficient for PR-gating. For the nightly-scheduled audit (WS-04 requires "nightly"), add a separate workflow:

```yaml
# .github/workflows/audit.yml
name: Nightly security audit
on:
  schedule:
    - cron: '17 4 * * *'  # 04:17 UTC daily — stagger to avoid the 00:00 thundering herd
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2.0.0
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

[CITED: github.com/rustsec/audit-check — v2.0.0 released 2024-09-23]

**Rationale:** Using `rustsec/audit-check@v2.0.0` for the scheduled run (rather than inline install) gets the "create issue on advisory" auto-behavior. The existing inline `cargo install cargo-audit && cargo audit` in the lint job stays — it's the PR-blocking gate.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| CLA via cla-assistant / EasyCLA | DCO via `Signed-off-by` + tim-actions/dco | Ecosystem shift 2020-2024; Kay adopts DCO per PITFALLS.md §11 + CONTEXT.md D-10 | Lower contributor friction, no CLA repo required |
| Hand-maintained license spreadsheet | `cargo-deny` + `deny.toml` | cargo-deny stable since 2020 | Declarative, CI-enforced, SPDX-expression aware |
| `cargo-audit` manual run | GitHub action `rustsec/audit-check@v2.0.0` with nightly schedule | v2.0.0 released 2024-09-23 | Nightly advisory sweeps auto-create issues |
| `actions/cache@v3` with hand-rolled Rust keys | `Swatinem/rust-cache@v2` | rust-cache stabilized 2023-2024 | Correct cache key derivation, workarounds for macOS/Windows corruption |
| GPG-only signed commits/tags | GPG **or** SSH signatures (GitHub + git ≥2.34) | GitHub SSH commit verification landed 2022-08-23 | SSH keys are easier for new contributors; D-13 accepts both |
| Standalone CODE_OF_CONDUCT text | Contributor Covenant v2.1 | CCv2.1 published 2020; ecosystem-standard by 2023 | Github auto-surfaces it in community profile |
| `edition = "2021"` | `edition = "2024"` | Rust 1.85 stable (early 2025) | 2024 implies `resolver = "3"` which is MSRV-aware for transitive deps [CITED: doc.rust-lang.org/cargo/CHANGELOG.html + search this session] |
| ForgeCode's `channel = "1.92"` | `channel = "1.95"` | Rust 1.95 released 2026-04-16 (3 days before Phase 1 planning) [VERIFIED] | Kay starts on current stable, not upstream's pin |

**Deprecated/outdated:**

- **ForgeCode's `rust-toolchain.toml` pinning 1.92** — fine for ForgeCode's own cadence but not aligned with "current stable" from Kay's perspective. Do not inherit.
- **`cargo-audit` pre-v0.20 inline install without `--locked`** — race on RustSec DB pull. Existing ci.yml already uses `--locked --quiet`, so this is correct; new contributors should not "simplify" it.

## Assumptions Log

> Empty — all factual claims in this research were either verified against live sources this session (Context7/WebFetch/WebSearch) or sourced from upstream research docs that were themselves verified. No `[ASSUMED]` tags used.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | (none) | — | — |

## Open Questions

1. **ForgeCode source import shape inside kay-core: flatten 23 crates into one, or preserve their boundaries?**
   - What we know: ForgeCode has 23 workspace crates under `crates/` with names like `forge_api`, `forge_app`, `forge_domain`, `forge_services`, etc. [VERIFIED via WebFetch]. CONTEXT.md D-05 says "kay-core = imported ForgeCode source" (singular).
   - What's unclear: Does "kay-core" mean a single `crates/kay-core` directory containing all ForgeCode source flattened, OR do we mirror ForgeCode's 23-crate structure into 23 sub-crates under `crates/kay-core-*`? The former is simpler but may make upstream tracking harder; the latter preserves blame but inflates the skeleton count.
   - Recommendation: **Preserve ForgeCode's multi-crate structure** inside `crates/kay-core/` as a *nested* workspace or (simpler) copy each `forge_*` crate as `crates/kay-core-<name>/`, renaming the Cargo package to `kay-core-<name>`. This matches codex-rs's ~70-crate ethos [CITED: STACK.md §Reference-Implementation Cross-Check] and preserves ForgeCode's blame. D-05 arguably leaves this to Claude's discretion. Surface to the user at planning time if clarification is needed.

2. **`authors = ["Kay Contributors <contributors@kay.dev>"]` — does `contributors@kay.dev` exist?**
   - What we know: D-03 locks this literal author string.
   - What's unclear: Whether the domain `kay.dev` is registered and `contributors@kay.dev` routes somewhere the maintainer sees. If not, it's a dead address in every published crate.
   - Recommendation: Plan a task to verify domain + mailbox exist BEFORE the first publish to crates.io (Phase 11). For Phase 1, the string is fine even without a working mailbox — just record the gap for Phase 11's procurement checklist.

3. **`harbor-framework/terminal-bench-2` Docker SHA pinning — is this even supported?**
   - What we know: `harbor run -d terminal-bench/terminal-bench-2 -m <model> -a <agent>` is the documented invocation [CITED: harborframework.com/docs/tutorials/running-terminal-bench]. Harbor pulls Daytona containers as the task environment.
   - What's unclear: Harbor docs we found don't mention Docker image SHA pinning explicitly; the concept of pinning "Harbor commit SHA" (which D-18 specifies) is clear, but "Docker image SHAs" may be an internal-to-Harbor detail we can only discover by running the tool. Phase 1 scaffolding prints the command but does not execute it, so this is a deferred resolution — flag for EVAL-01a.
   - Recommendation: The Phase 1 parity-baseline scaffold includes a `manifest-schema.json` placeholder listing the fields D-20 mandates (Docker SHAs, OR model+date, JSONL transcript path, summary score, git tag). EVAL-01a's owner populates it by running Harbor and recording what it produces.

4. **Is there an existing `alo-exp/kay` GitHub repo, and what's its current visibility state?**
   - What we know: CLAUDE.md says "Git repo: https://github.com/alo-exp/kay". D-OP-05 notes branch protection requires the repo to stay public (which it's acceptable for, since "the repo is public").
   - What's unclear: Whether the repo exists yet (this local working directory is `/Users/shafqat/Documents/Projects/opencode/vs-others/` — no git remote shown). Phase 1 may or may not need to create the repo as its first task.
   - Recommendation: Phase 1 Plan's first task should be "Confirm/create GitHub repo at alo-exp/kay, set visibility public, set default branch to main." Branch protection rules (D-23) require the repo to exist.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `git` | All tasks | Assume ✓ (universal dev tool) | ≥2.34 required for SSH tag signatures | No fallback — required |
| `rustc` / `cargo` | Workspace builds | Assume ✓ locally; CI uses `dtolnay/rust-toolchain@stable` | 1.95 per new rust-toolchain.toml | No fallback |
| Rust 1.95 toolchain | WS-05 (compile clean on 3 OSes) | CI yes via dtolnay action; local TBD | 1.95.0 | Fall back to 1.94 if 1.95 not yet on all three runners (low risk; released 3 days ago and all runners receive daily image updates) |
| `cargo-audit` | WS-04 | Installed inline in CI; not required locally in P1 | latest | No fallback; inline install is cheap |
| `cargo-deny` | WS-03 | Installed via EmbarkStudios/cargo-deny-action@v2 in CI | latest v2 series | No fallback |
| `cargo-about` (for NOTICE/deps audit) | Nice-to-have for verifying § NOTICE | Not required in P1 | — | Manual audit acceptable |
| `docker` | EVAL-01 (Harbor) | Deferred to EVAL-01a | — | N/A in P1 |
| `uv tool install harbor` (Harbor CLI) | EVAL-01 (parity run) | Deferred to EVAL-01a | — | N/A in P1 |
| `gh` (GitHub CLI) | Repo config, branch protection | Assume available on maintainer's machine | ≥2.x | Manual web-UI config acceptable |
| OpenRouter API key | EVAL-01 (parity run) | **NOT required in P1** per user amendment | — | N/A in P1; deferred to EVAL-01a |
| Apple Developer ID | GOV-05 (signed macOS tags) | **NOT required in P1** per user amendment (D-OP-02 → P11) | — | N/A in P1; signed-tag gate dormant until a tag cut |
| Azure Code Signing | GOV-05 (Windows Authenticode) | **NOT required in P1** (D-OP-03 → P11) | — | N/A in P1 |
| GPG/SSH signing key | GOV-05 | **NOT required in P1** (D-OP-04 → P11) | — | N/A in P1 |

**Missing dependencies with no fallback:** None that block Phase 1 under the user amendment.

**Missing dependencies with fallback:**
- Harbor/Docker/OpenRouter: all deferred to EVAL-01a follow-on task, not blocking any Phase 1 deliverable.
- Signing keys: not needed until a tag is cut (Phase 11+).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` via Rust stable 1.95 (built-in); no third-party runner needed in P1 |
| Config file | None required; `[workspace]` in Cargo.toml exposes `cargo test --workspace --all-features` |
| Quick run command | `cargo check --workspace --all-features` (fastest compile-clean gate for P1) |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace --all-features && cargo deny check && cargo audit` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GOV-01 | Fork attribution present | doc inspection | `test -f NOTICE && test -f ATTRIBUTIONS.md && grep -q 'ForgeCode' NOTICE && grep -q 'Tailcall' NOTICE` | ❌ Wave 0 (script lives at tests/governance/check_attribution.sh) |
| GOV-02 | LICENSE + NOTICE at root | file-existence | `test -f LICENSE && test -f NOTICE` | ❌ Wave 0 |
| GOV-03 | DCO gate wired | CI-contract inspection | `grep -q "tim-actions/dco" .github/workflows/ci.yml` + live PR dry-run | ✅ Existing ci.yml |
| GOV-04 | CONTRIBUTING.md has DCO + clean-room sections | doc inspection | `grep -q 'Developer Certificate of Origin' CONTRIBUTING.md && grep -q 'v2.1.88' CONTRIBUTING.md` | ❌ Wave 0 |
| GOV-05 | Signed-tag gate wired | CI-contract | `grep -q 'git tag -v' .github/workflows/ci.yml` | ✅ Existing ci.yml |
| GOV-06 | SECURITY.md present + describes disclosure | doc inspection | `test -f SECURITY.md && grep -q 'Security Advisory' SECURITY.md` | ❌ Wave 0 |
| GOV-07 | Clean-room attestation in CONTRIBUTING.md | doc inspection | `grep -q 'v2.1.88 source map leak' CONTRIBUTING.md` | ❌ Wave 0 |
| WS-01 | Rust 2024 workspace with 7 crates | `cargo metadata` | `cargo metadata --format-version 1 \| jq '.packages \| map(.name) \| sort'` expects 7 `kay-*` entries | ❌ Wave 0 (Cargo.toml + crates/*) |
| WS-02 | Workspace pinning | `Cargo.toml` inspection | `grep -E '^(tokio\|reqwest\|rustls) = ' Cargo.toml` in the [workspace.dependencies] block | ❌ Wave 0 |
| WS-03 | cargo-deny green | `cargo deny check` | `cargo deny check --config deny.toml` | ❌ Wave 0 (deny.toml) |
| WS-04 | cargo-audit green in CI | inline install + run | `cargo install cargo-audit --locked && cargo audit` | ✅ Existing ci.yml |
| WS-05 | Workspace compiles on 3 OSes with --deny warnings | tri-OS matrix | `cargo clippy --workspace --all-targets --all-features -- -D warnings` on ubuntu/macos/windows | ✅ Existing ci.yml tri-OS matrix |
| EVAL-01 | Parity-gate scaffold runnable | CLI smoke + CI-contract | `cargo run -p kay-cli -- eval tb2 --help` exits 0; `grep -q 'parity-gate' .github/workflows/ci.yml` | ❌ Wave 0 (kay-cli main + ci.yml addition + PARITY-DEFERRED.md) |

### Sampling Rate

- **Per task commit:** `cargo check --workspace --all-features` (compile-clean is the fastest gate; ~30s for a cold skeleton workspace)
- **Per wave merge:** `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check` (~2-3 min)
- **Phase gate:** Full suite green on tri-OS CI before `/gsd-verify-work` fires. Manual `cargo run -p kay-cli -- eval tb2` prints the deferred-run message. `tests/governance/check_attribution.sh` must pass locally.

### Wave 0 Gaps

Phase 1 has no pre-existing Rust workspace yet (this is the phase that creates it). Wave 0 establishes:

- [ ] `Cargo.toml` (workspace root with `[workspace]`, `[workspace.package]`, `[workspace.dependencies]`) — covers WS-01, WS-02
- [ ] `Cargo.lock` (committed after first `cargo build`)
- [ ] `rust-toolchain.toml` (channel = "1.95")
- [ ] `.rustfmt.toml` (mirror ForgeCode's)
- [ ] `deny.toml` (see § Example 4 above) — covers WS-03
- [ ] `crates/kay-core/Cargo.toml` + imported ForgeCode source — covers WS-01 for the core crate
- [ ] `crates/kay-cli/Cargo.toml` + `src/main.rs` (clap + eval tb2 shim) — covers WS-01 + EVAL-01
- [ ] `crates/kay-tauri/`, `crates/kay-provider-openrouter/`, `crates/kay-sandbox-{macos,linux,windows}/` skeletons (empty `src/lib.rs`) — covers WS-01 for remaining crates
- [ ] `LICENSE` (Apache-2.0 verbatim from https://www.apache.org/licenses/LICENSE-2.0.txt) — covers GOV-02
- [ ] `NOTICE` (from § Example 1) — covers GOV-01, GOV-02
- [ ] `ATTRIBUTIONS.md` (lists UPSTREAM_COMMIT + derivation notes) — covers GOV-01
- [ ] `README.md` (with §Acknowledgments) — covers GOV-01 (D-04)
- [ ] `CONTRIBUTING.md` (from § Example 2) — covers GOV-04, GOV-07
- [ ] `SECURITY.md` (from § Example 3) — covers GOV-06
- [ ] `CODE_OF_CONDUCT.md` (Contributor Covenant v2.1)
- [ ] `docs/signing-keys/README.md` (placeholder for P11)
- [ ] `.github/pull_request_template.md` (DCO reminder + clean-room checkbox)
- [ ] `.github/workflows/ci.yml` (extend with `parity-gate` job stub) — covers EVAL-01 scaffold + preserves existing DCO/lint/test/signed-tag-gate
- [ ] `.github/workflows/audit.yml` (nightly `rustsec/audit-check@v2.0.0`) — covers WS-04 "nightly"
- [ ] `.planning/phases/01-fork-governance-infrastructure/parity-baseline/PARITY-DEFERRED.md` — EVAL-01 scaffolding deliverable
- [ ] `tests/governance/check_attribution.sh` (grep-based verifier script called by manual phase-gate and optionally by CI)

Framework install: none — cargo is built into Rust. `uv tool install harbor` deferred.

## Security Domain

Phase 1 does not introduce runtime code paths — the Kay agent does not run. The security surface is **supply-chain + governance**, not runtime.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | no runtime auth in P1 |
| V3 Session Management | no | no sessions in P1 |
| V4 Access Control | partial | GitHub branch protection (D-23) enforces who can merge to main |
| V5 Input Validation | no | no user input handled in P1 (CLI shim is informational) |
| V6 Cryptography | partial | Signed-tag verification via `git tag -v` (scaffolded, dormant); no hand-rolled crypto |
| V14 Configuration | yes | `deny.toml` enforces license + dep policy; `rust-toolchain.toml` pins toolchain; CI pins action versions (tim-actions/get-pr-commits@v1.3.1, Swatinem/rust-cache@v2, EmbarkStudios/cargo-deny-action@v2, rustsec/audit-check@v2.0.0, actions/checkout@v4) |

### Known Threat Patterns for Phase 1 stack (governance + CI)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious PR slips in unsigned commit | Tampering | `tim-actions/dco` blocks PRs missing `Signed-off-by` (GOV-03); branch protection requires DCO green (D-23) |
| Malicious transitive dependency | Tampering | `cargo-deny` license + ban policy + `cargo-audit` RustSec DB (WS-03, WS-04) |
| Dependency version confusion / namespace typosquat | Tampering | `cargo-deny` `[sources] unknown-registry = "deny"` + pinned versions in `[workspace.dependencies]` |
| Third-party GitHub Action compromise (e.g. tj-actions/changed-files 2025 incident) | Tampering | Pin action versions to specific tags/SHAs where possible. tim-actions/dco@master is the one exception locked by D-10; mitigate by reviewing its source on adoption + watching the repo. Other actions pinned (Swatinem/rust-cache@v2, EmbarkStudios/cargo-deny-action@v2, rustsec/audit-check@v2.0.0, tim-actions/get-pr-commits@v1.3.1). Consider upgrading tim-actions/dco@master to a SHA pin after a security review. |
| Clean-room violation via leak-contaminated PR | Information disclosure | CONTRIBUTING.md clean-room attestation (GOV-07) + DCO signoff as legal vehicle + reviewer judgment |
| Unsigned release tag | Tampering | `signed-tag-gate` job (GOV-05); dormant in P1 (no tags), fires at release in P11+ |
| Leaked secret in PR (e.g. accidental `.env` paste) | Information disclosure | `.gitignore` already excludes `.env`, `*.p12`, `*.pem`, `minisign.key`, `keystore/` [VERIFIED: existing .gitignore lines 35-43]; CI-level secret scanning is an optional P1 add (GitHub secret-scanning is free for public repos and already on) |
| `cargo test` runs unsigned code from a PR | Elevation of privilege | Tri-OS CI runs test suite in ephemeral GitHub runner sandboxes; no persistent state; matches ecosystem norm |

**Phase-1-specific note:** Because the signed-tag gate is scaffolded but dormant (no tags cut in P1), a malicious push of a `v*`-tag that happens to pass all other CI checks would produce a red `signed-tag-gate` job failure and not release anything — gate works as designed. No additional mitigation needed for P1.

## Sources

### Primary (HIGH confidence — verified this session)

- [ForgeCode GitHub root](https://github.com/antinomyhq/forgecode) — repo state, 23 crates in `crates/`, Apache-2.0 license, edition 2024, MSRV 1.92, latest release v2.11.4 (2026-04-19) [WebFetch this session]
- [ForgeCode Cargo.toml](https://raw.githubusercontent.com/antinomyhq/forgecode/main/Cargo.toml) — workspace structure, resolver=2, edition=2024, rust-version=1.92, release profile [WebFetch this session]
- [ForgeCode LICENSE](https://raw.githubusercontent.com/antinomyhq/forgecode/main/LICENSE) — Apache-2.0, Copyright 2025 Tailcall [WebFetch this session]
- [ForgeCode rust-toolchain.toml](https://raw.githubusercontent.com/antinomyhq/forgecode/main/rust-toolchain.toml) — channel = "1.92" [WebFetch this session]
- [ForgeCode rustfmt.toml](https://raw.githubusercontent.com/antinomyhq/forgecode/main/.rustfmt.toml) — project fmt config to mirror [WebFetch this session]
- [ForgeCode NOTICE 404](https://raw.githubusercontent.com/antinomyhq/forgecode/main/NOTICE) — returns 404, i.e. no NOTICE exists upstream [WebFetch this session, confirmed]
- [Rust 1.95.0 release announcement](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/) — current stable, 2026-04-16 [WebSearch this session]
- [Apache Infra licensing-howto.html](https://infra.apache.org/licensing-howto.html) — canonical NOTICE template + §4(d) compliance [WebFetch this session]
- [tim-actions/dco README](https://github.com/tim-actions/dco) — usage YAML + latest tag v1.1.0 [WebFetch this session]
- [rustsec/audit-check README](https://github.com/rustsec/audit-check) — v2.0.0 YAML + nightly schedule pattern [WebFetch this session]
- [EmbarkStudios/cargo-deny deny.toml](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/main/deny.toml) — reference config structure [WebFetch this session]
- [Swatinem/rust-cache README](https://github.com/Swatinem/rust-cache) — tri-OS cross-platform support, cargo#8603 workaround [WebSearch this session]
- [Harbor Framework — Running Terminal-Bench](https://www.harborframework.com/docs/tutorials/running-terminal-bench) — invocation pattern for `harbor run -d terminal-bench/terminal-bench-2` [WebFetch this session]
- [docs.github.com signing-tags](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-tags) — SSH + GPG tag verification, web UI "Verified" badge [WebSearch this session]
- [github.blog SSH commit verification](https://github.blog/changelog/2022-08-23-ssh-commit-verification-now-supported/) — SSH signature support date [WebSearch this session]
- [Linux kernel submitting-patches.rst](https://docs.kernel.org/process/submitting-patches.html) — DCO v1.1 canonical text and sign-off format [WebSearch this session]
- [developercertificate.org](https://developercertificate.org/) — DCO v1.1 source of truth (to link from CONTRIBUTING.md)
- [ossf/oss-vulnerability-guide templates](https://github.com/ossf/oss-vulnerability-guide/tree/main/templates/security_policies) — SECURITY.md template sources [WebFetch this session]
- [rust-github/template CONTRIBUTING.md](https://github.com/rust-github/template/blob/main/CONTRIBUTING.md) — Rust project contributor doc template [WebFetch this session]
- Existing `.github/workflows/ci.yml` in the repo [local file, verified this session] — DCO + lint + tri-OS test + frontend + signed-tag-gate already scaffolded
- Existing `.gitignore` [local file, verified this session] — already excludes secrets
- Existing `docs/CICD.md` [local file, verified this session] — pipeline spec
- `.planning/research/STACK.md`, `ARCHITECTURE.md`, `PITFALLS.md`, `SUMMARY.md` — upstream research, read this session

### Secondary (MEDIUM confidence — inferred or cited from upstream research without re-verification)

- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md` — read this session
- CONTEXT.md D-01 through D-24 and D-OP-01 through D-OP-05 — user-locked decisions, read this session
- PITFALLS.md §Pitfall 2 (License/NOTICE hygiene), §Pitfall 3 (ForgeCode over-specialization), §Pitfall 11 (CLA friction → DCO), §Pitfall 12 (DMCA / leak proximity), §Pitfall 14 (Windows ConPTY — lands Phase 4 not 1), §Pitfall 15 (scope creep)
- STACK.md §Installation, §Version Compatibility
- SUMMARY.md §Phase 1

### Tertiary (LOW confidence — not applicable this phase)

- None — all Phase 1 claims either verified this session or sourced from upstream research that was itself verified.

## Metadata

**Confidence breakdown:**

- **Fork & NOTICE composition:** HIGH — Apache Infra template + VERIFIED ForgeCode state (no NOTICE upstream, Tailcall copyright) + a known-good drafting pattern.
- **Workspace pinning:** HIGH — CONTEXT.md locks versions; verified against STACK.md and live registry knowledge.
- **CI tooling:** HIGH — existing ci.yml already correct; only additions are parity-gate job + nightly audit workflow. Every action version verified this session.
- **DCO + Signed-Tag enforcement:** HIGH — tim-actions/dco pattern verified; GitHub SSH-tag verification UI state verified.
- **CONTRIBUTING / SECURITY / CODE_OF_CONDUCT templates:** MEDIUM-HIGH — based on rust-github/template + OSSF guide + Contributor Covenant v2.1, all ecosystem-standard.
- **deny.toml starter:** HIGH — derived from Embark's own deny.toml (VERIFIED verbatim this session).
- **Parity-gate scaffold (CLI shim + CI job stub):** MEDIUM — Harbor invocation pattern verified, but the specific fields of the archived score manifest are deferred to EVAL-01a discovery (Open Q3).
- **Rust toolchain pin (1.95):** HIGH — 1.95 released 2026-04-16, verified against blog.rust-lang.org.
- **Environment availability:** HIGH — user amendment explicitly defers all external-dependency blockers (OpenRouter, Apple Developer ID, Azure, signing keys) to later phases.

**Overall phase-1 confidence:** HIGH. The only MEDIUM notes are Open Question 1 (kay-core import shape — discretionary) and Open Question 3 (Harbor manifest schema — deferred to EVAL-01a). Neither blocks Phase 1 planning.

**Research date:** 2026-04-19
**Valid until:** 2026-05-19 (30 days — Phase 1 is stable governance; tooling unlikely to shift. If Rust 1.96 releases 2026-05-28 and we haven't executed Phase 1 by then, re-verify the toolchain pin.)
