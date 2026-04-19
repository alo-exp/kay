# CI/CD

Pipeline overview for Kay. The authoritative workflow lives in `.github/workflows/ci.yml`.

## Pipeline Stages

1. **Lint & audit** — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo-deny check`, `cargo-audit`.
2. **Unit + integration tests** — `cargo test --workspace` on the Linux / macOS / Windows matrix.
3. **Frontend build** — `pnpm install --frozen-lockfile && pnpm build` (when `kay-tauri/ui/` has content).
4. **Tauri bundle (PR preview)** — `cargo tauri build` on each OS for artifact preview; signing disabled on PRs.
5. **Tauri bundle + notarize (main)** — full signing: Apple Developer ID (macOS), Azure Code Signing (Windows), AppImage signing (Linux). Runs on every merge to `main`, not only releases.
6. **Canary suite (nightly)** — 4-hour Tauri memory canary + real-repo eval + held-out TB subset.
7. **Release (tag v\*)** — build, sign, notarize, publish GitHub release, `cargo publish` the CLI, `tauri-plugin-updater` manifest signed with minisign.

## Gates

- **Unsigned tag rejection** — CI refuses to publish a release for an unsigned git tag (enforces GOV-05).
- **DCO check** — GitHub Action `dco-bot` (or equivalent) blocks PRs missing `Signed-off-by` on any commit (enforces GOV-03).
- **Parity gate** — on every merge to `main`, the forked baseline must continue to reproduce ≥ 80% on TB 2.0 (enforces EVAL-01). Stored reference transcript compared with `harbor-diff`.
- **Canary memory delta** — nightly pipeline fails main if the 4-hour Tauri canary exceeds the memory-growth budget.

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
