# CI/CD

Pipeline overview for Kay. The authoritative workflows live in `.github/workflows/`.

## Current State (Phase 1 complete)

Two workflow files + one governance invariant checker are live:

- **`.github/workflows/ci.yml`** — 6 jobs defined; triggers vary per job (see below).
  Workflow-level triggers: `push` to `main` and `v*.*.*` tags, `pull_request` to `main`, and manual `workflow_dispatch`.
  Always-on (every PR + push to main + every workflow run): `lint`, `test`, `frontend`. The other three are conditional.
  - `dco` — DCO signoff check via `tim-actions/dco@master` + `tim-actions/get-pr-commits@v1.3.1`. **PR-only**: `if: github.event_name == 'pull_request'`
  - `lint` — `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` + `EmbarkStudios/cargo-deny-action@v2` (gated on `hashFiles('deny.toml') != ''`) + inline `cargo-audit`; ubuntu-latest. **Always on.**
  - `test` — `cargo test --workspace --all-features` on the `ubuntu-latest` / `macos-latest` / `windows-latest` matrix (`fail-fast: false`). **Always on.** (Phase 2.5 split kay-core into 23 `forge_*` sub-crates, removing the prior `--exclude kay-core` carve-out.)
  - `frontend` — detects `crates/kay-tauri/ui/package.json`; if present runs pnpm typecheck + lint + test + build. **Always on** (no-op until Phase 9 creates the frontend).
  - `signed-tag-gate` — Pitfall-6-hardened `if: github.ref_type == 'tag' && startsWith(github.ref_name, 'v') && contains(github.ref_name, '.') && !startsWith(github.ref_name, 'v0.0.')`; runs `git tag -v` and fails unsigned release-shape tags. **Tag-only, v0.0.x exempt** — see §Gates and `SECURITY.md §Release Signing`.
  - `parity-gate` — **`workflow_dispatch` only** (Phase 1 scaffold per D-OP-01 user amendment); checks for an archived parity-run summary at `.planning/phases/01-fork-governance-infrastructure/parity-baseline/summary.md` and exits 0 with an informational notice if absent (Phase 1 state); upgrades to real Harbor invocation + score-threshold check in Phase 2.
- **`.github/workflows/audit.yml`** — cargo-audit via `rustsec/audit-check@v2.0.0`. Runs nightly at 04:17 UTC, on PRs/pushes that touch `Cargo.toml` / `Cargo.lock` / `crates/**/Cargo.toml` / `deny.toml`, and via manual `workflow_dispatch`
- **`tests/governance/check_attribution.sh`** — 36 grep-based invariants verifying NOTICE, ATTRIBUTIONS, README, CONTRIBUTING, SECURITY, `.forgecode-upstream-sha`, and the `forgecode-parity-baseline` tag remain intact

## Future Pipeline Stages (Phase 11 onwards)

1. **Tauri bundle (PR preview)** — `cargo tauri build` on each OS for artifact preview; signing disabled on PRs.
2. **Tauri bundle + notarize (main)** — full signing: Apple Developer ID (macOS), Azure Code Signing (Windows), AppImage signing (Linux). Runs on every merge to `main`, not only releases.
3. **Canary suite (nightly)** — 4-hour Tauri memory canary + real-repo eval + held-out TB subset.
4. **Release (tag `v*`)** — build, sign, notarize, publish GitHub release, `cargo publish` the CLI, `cargo publish` kay-tui, `tauri-plugin-updater` manifest signed with minisign.

## Gates

- **Unsigned release tag rejection** — CI runs `git tag -v` on every `v*.*.*`-shaped tag EXCEPT the `v0.0.x` pre-stable series, and fails if no signature is present (enforces GOV-05). The `v0.0.x` carve-out is deliberate: those are internal / audit builds that ship before Phase 11 signing-key procurement. From `v0.1.0` onward, signing is mandatory. See `SECURITY.md §Release Signing`.
- **DCO check** — `tim-actions/dco@master` blocks PRs missing `Signed-off-by` on any commit (enforces GOV-03). **Active now.**
- **Parity gate** — the forked baseline must continue to reproduce ≥ 80% on TB 2.0 (enforces EVAL-01). Scaffolded in Phase 1 as `workflow_dispatch` only; wires into PR/merge checks at Phase 2's first harness modification.
- **Canary memory delta** — nightly pipeline fails main if the 4-hour Tauri canary exceeds the memory-growth budget (Phase 9 onward).
- **Governance invariants** — `tests/governance/check_attribution.sh` can be run locally or wired into CI to verify attribution / NOTICE / clean-room attestation strings remain intact. **Active now** (local script; CI integration is a follow-up task).

## Required Environment

- `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_CERTIFICATE_P12_BASE64` (macOS notarization)
- `AZURE_SIGNING_TENANT_ID`, `AZURE_SIGNING_CLIENT_ID`, `AZURE_SIGNING_CERT_NAME` (Windows Authenticode)
- `MINISIGN_SECRET_KEY_BASE64` (Tauri updater)
- `OPENROUTER_API_KEY` (eval runs only; scoped to a dedicated benchmark-only key)
- `GH_RELEASE_TOKEN` (release step only)

## Secret Hygiene

- All secrets scoped to environment protections on GitHub (`production` environment for release, `nightly` for canary).
- `cargo-deny`, `cargo-audit`, and a secret-scan step block PRs that leak webhooks or keys.
- No secret is printed to logs; signing tools run in detached mode.

## Manual Pipelines

- `kay eval tb2 --submit` — produces the TB 2.0 submission artifact; run by a maintainer locally with the benchmark-only OpenRouter key, not from CI.
