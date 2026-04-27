# Phase 11: Cross-Platform Hardening + Release Pipeline

## Goal

A user downloads a signed, notarized, reproducibly-built Kay artifact for their OS from a GitHub release; `cargo install kay` yields the headless CLI from crates.io; the updater verifies signatures against a pre-pinned minisign public key.

---

## Requirements Covered

From `.planning/REQUIREMENTS.md`:

| REQ-ID | Requirement | Phase |
|--------|-------------|-------|
| REL-01 | Binary distribution matrix: macOS (arm64 + x64), Windows (x64), Linux (x64 + arm64) | 11 |
| REL-02 | macOS notarization via Apple Developer ID on every main merge | 11 |
| REL-03 | Windows Authenticode code signing via Azure Code Signing | 11 |
| REL-04 | Linux builds shipped as AppImage + tar.gz with SHA attestations | 11 |
| REL-05 | `cargo install kay` publishes the headless CLI to crates.io | 11 |
| REL-06 | Tauri bundler produces `.app`, `.msi`, `.AppImage` signed artifacts | 11 |
| REL-07 | `tauri-plugin-updater` uses minisign keypair committed to tauri.conf.json | 11 |
| CLI-06 | Standalone distribution — `cargo install kay` works over bare SSH | 11 |

---

## Task Breakdown

### Wave 1: CI Pipeline Foundation

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W1-T1 | Tauri config for 3 targets | `crates/kay-tauri/src-tauri/tauri.conf.json` | Enable macOS (.app), Windows (.msi), Linux (AppImage) targets | `tauri build --target aarch64-apple-darwin` succeeds |
| W1-T2 | GitHub Actions workflow for cross-platform builds | `.github/workflows/release.yml` | Matrix: macOS (arm64/x64), Windows (x64), Linux (x64/arm64) | Workflow file exists, passes YAML lint |
| W1-T3 | Rust toolchain matrix in CI | `.github/workflows/release.yml` | pinned stable toolchain for all platforms | CI spawns correct rustc version |

### Wave 2: Code Signing

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W2-T1 | macOS notarization setup | `.github/workflows/release.yml` + docs | Apple Developer ID env vars, `xcrun notarytool`, staples `.app` | Signed .app passes `spctl -a` gate |
| W2-T2 | Windows Authenticode signing | `.github/workflows/release.yml` + docs | Azure Code Signing or equivalent, signtool.exe | Signed .exe passes Windows SmartScreen |
| W2-T3 | Linux SHA attestations | `.github/workflows/release.yml` | SHA256 SUMS file signed with minisign | `sha256sum -c SUMS` passes |

### Wave 3: crates.io Distribution

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W3-T1 | kay-cli publishable to crates.io | `crates/kay-cli/Cargo.toml` + workspace config | version, description, license, repository fields | `cargo publish --dry-run` passes |
| W3-T2 | kay-tui publishable to crates.io | `crates/kay-tui/Cargo.toml` + workspace config | same fields | `cargo publish --dry-run` passes |
| W3-T3 | CI publish workflow | `.github/workflows/publish.yml` | Publish on tag push to crates.io | Crates.io receives the package |

### Wave 4: Tauri Updater

| # | Task | File Changed | Description | Verification |
|---|------|-------------|-------------|--------------|
| W4-T1 | Generate minisign keypair | `.github/` + `crates/kay-tauri/` | `minisign -G -s kay-release.key` | Public key committed, private in secrets |
| W4-T2 | Configure tauri-plugin-updater | `crates/kay-tauri/src-tauri/tauri.conf.json` | `plugins.updater.signing-keys` with public key | Updater config exists |
| W4-T3 | Windows CI PTY test suite | `.github/workflows/release.yml` | ConPTY flags, Ctrl+C, resize tests | Windows CI green |

---

## Verification Steps

1. **All CI workflows pass**: `cargo check --workspace --deny warnings` on all 3 OSes
2. **macOS notarization**: `xcrun notarytool verify Kay.app` returns clean
3. **Windows signing**: `signtool verify /pa sha1:<thumbprint> Kay.exe` returns valid
4. **Linux artifacts**: SHA256SUMS.sig validates SHA256SUMS
5. **crates.io publish**: `cargo install kay` downloads and builds successfully
6. **Tauri bundler**: `.app`, `.msi`, `.AppImage` all produced from `tauri build`

---

## Threat Model

### Supply Chain Risks

- **Apple Developer ID expiration**: Rotate before expiry; document in release runbook
- **minisign private key leakage**: Store in GitHub Actions secrets; never in repo
- **crates.io token theft**: Scoped to kay-cli/kay-tui only; rotate if compromised
- **Dependency tampering**: `cargo audit` runs on every PR (Phase 3/Phase 4 baseline)

### Build Reproducibility

- **Binary drift**: Pin rustc version in CI; document exact version in release notes
- **Lock file mismatch**: `Cargo.lock` committed; verified in CI gate

---

## Rollback

If a release pipeline issue is discovered post-merge:
1. Revert the workflow YAML changes (atomic rollback)
2. Remove any committed secrets from GitHub Actions env vars
3. Yanked crates.io release if already published: `cargo yank --version X.X.X`

---

## Dependencies

- **Depends on**: Phase 4 (sandbox for security baseline), Phase 9 (Tauri shell in place)
- **Parallelizable**: Release pipeline hardening can begin during Phase 10

---

## Exit Condition

PLAN.md exists, passes all quality gates (no ❌), and has been reviewed.

