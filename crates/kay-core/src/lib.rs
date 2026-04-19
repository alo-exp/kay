//! kay-core — imported ForgeCode source tree
//!
//! This crate is the parity baseline for EVAL-01. Source was imported
//! from ForgeCode (https://github.com/antinomyhq/forgecode) at commit
//! 022ecd994eaec30b519b13348c64ef314f825e21 on 2026-04-19. See repo-root
//! NOTICE, ATTRIBUTIONS.md, and the `forgecode-parity-baseline` git tag
//! for provenance.
//!
//! Each `forge_*` module below is a subtree of an upstream crate's
//! `src/` directory, renamed but otherwise unmodified. Cross-module
//! references between these subtrees may not compile cleanly — that
//! is expected per plan 01-03; the parity baseline is the *unmodified*
//! imported source, and cross-module-reference shape is a downstream
//! harness adjustment task.

#![allow(dead_code)]
#![allow(clippy::all)]

// Module declarations — one per copied forge_* subtree.
pub mod forge_api;
pub mod forge_app;
pub mod forge_ci;
pub mod forge_config;
pub mod forge_display;
pub mod forge_domain;
pub mod forge_embed;
pub mod forge_fs;
pub mod forge_infra;
pub mod forge_json_repair;
pub mod forge_main;
pub mod forge_markdown_stream;
pub mod forge_repo;
pub mod forge_select;
pub mod forge_services;
pub mod forge_snaps;
pub mod forge_spinner;
pub mod forge_stream;
pub mod forge_template;
pub mod forge_test_kit;
pub mod forge_tool_macros;
pub mod forge_tracker;
pub mod forge_walker;
