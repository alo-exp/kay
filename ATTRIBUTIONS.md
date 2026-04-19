# Attributions

This file lists third-party source code incorporated into Kay beyond what
`Cargo.lock` can express (because cargo's dependency graph doesn't cover
vendored source).

## ForgeCode (primary upstream)

- **Origin:** https://github.com/antinomyhq/forgecode
- **License:** Apache-2.0 (see LICENSE and NOTICE)
- **Upstream commit:** `022ecd994eaec30b519b13348c64ef314f825e21`
- **Import date:** 2026-04-19
- **Location in Kay:** `crates/kay-core/src/**` and any `crates/kay-core-*` if multi-crate structure is preserved
- **Deliberate divergences from upstream:**
  - `rust-toolchain.toml` pinned to `1.95` (ForgeCode pins `1.92`) — see 01-RESEARCH.md §Pitfall 3
  - Per-crate `authors` replaced with `["Kay Contributors <contributors@kay.dev>"]` per D-03
  - Any `.github/workflows/*.yml` from ForgeCode is NOT imported — Kay's CI workflow is separate
  - New files added by Kay: NOTICE (ForgeCode had none), CONTRIBUTING.md (DCO + clean-room), SECURITY.md, CODE_OF_CONDUCT.md (Contributor Covenant v2.1)
- **Purpose:** The unmodified ForgeCode harness is the parity baseline for Terminal-Bench 2.0 (EVAL-01). The `forgecode-parity-baseline` git tag anchors the import SHA.

## Kay's own copyright line policy

Kay is a contributor collective. The LICENSE appendix line reads
`Copyright 2026 Kay Contributors`. Individual contributions are tracked via
DCO `Signed-off-by` trailers in git history (see CONTRIBUTING.md).
