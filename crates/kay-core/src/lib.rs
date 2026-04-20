//! kay-core — aggregator re-exporter for forge_* sub-crates
//!
//! Phase 2.5 split the original mono-crate into 23 `forge_*` workspace
//! sub-crates. This crate now re-exports the top-of-DAG layers so existing
//! `kay_core::forge_*` paths continue to resolve.
//!
//! Provenance: upstream ForgeCode at commit
//! `022ecd994eaec30b519b13348c64ef314f825e21` (2026-04-19). See repo-root
//! NOTICE, ATTRIBUTIONS.md, and the `forgecode-parity-baseline` git tag.

#![allow(dead_code)]
#![allow(clippy::all)]

pub extern crate forge_api;
pub extern crate forge_config;
pub extern crate forge_domain;
pub extern crate forge_json_repair;
pub extern crate forge_repo;
pub extern crate forge_services;
