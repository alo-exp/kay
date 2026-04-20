# Attributions

This file lists third-party source code incorporated into Kay beyond what
`Cargo.lock` can express (because cargo's dependency graph doesn't cover
vendored source).

## ForgeCode (primary upstream)

- **Origin:** https://github.com/antinomyhq/forgecode
- **License:** Apache-2.0 (see LICENSE and NOTICE)
- **Upstream commit:** `022ecd994eaec30b519b13348c64ef314f825e21`
- **Import date:** 2026-04-19
- **Location in Kay:** each upstream `forge_*` crate is imported as its own
  workspace sub-crate at `crates/forge_<name>/` (23 sub-crates in total:
  `forge_api`, `forge_app`, `forge_ci`, `forge_config`, `forge_display`,
  `forge_domain`, `forge_embed`, `forge_fs`, `forge_infra`,
  `forge_json_repair`, `forge_main`, `forge_markdown_stream`, `forge_repo`,
  `forge_select`, `forge_services`, `forge_snaps`, `forge_spinner`,
  `forge_stream`, `forge_template`, `forge_test_kit`, `forge_tool_macros`,
  `forge_tracker`, `forge_walker`). `crates/kay-core/` is a thin
  aggregator re-exporter that surfaces the top-of-DAG sub-crates
  (`forge_api`, `forge_config`, `forge_domain`, `forge_json_repair`,
  `forge_repo`, `forge_services`) for downstream Kay callers.
- **Deliberate divergences from upstream:**
  - `rust-toolchain.toml` pinned to `1.95` (ForgeCode pins `1.92`) — see 01-RESEARCH.md §Pitfall 3
  - Per-crate `authors` replaced with `["Kay Contributors <contributors@kay.dev>"]` per D-03
  - Any `.github/workflows/*.yml` from ForgeCode is NOT imported — Kay's CI workflow is separate
  - Phase 2.5 split the initial mono-crate layout into 23 workspace
    sub-crates. Source content is byte-identical to upstream at the pinned
    SHA; only `Cargo.toml` manifests and workspace wiring were authored
    by Kay. A small set of workspace-level dependency version bumps was
    required to make the split build against Rust 1.95 — see
    `.planning/phases/02.5-kay-core-sub-crate-split/02.5-03-SUMMARY.md`
    §Rule 3 Deviations for the authoritative list.
  - New files added by Kay: NOTICE (ForgeCode had none), CONTRIBUTING.md (DCO + clean-room), SECURITY.md, CODE_OF_CONDUCT.md (Contributor Covenant v2.1)
- **Purpose:** The unmodified ForgeCode harness is the parity baseline for Terminal-Bench 2.0 (EVAL-01). The `forgecode-parity-baseline` git tag anchors the import SHA.

## Kay's own copyright line policy

Kay is a contributor collective. The LICENSE appendix line reads
`Copyright 2026 Kay Contributors`. Individual contributions are tracked via
DCO `Signed-off-by` trailers in git history (see CONTRIBUTING.md).
