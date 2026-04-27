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

// Phase 5 native modules (not re-exports — new code authored in kay-core).
// Coverage gate: `event_filter` is enforced at 100%-line + 100%-branch by
// the `coverage-event-filter` CI job (QG-C4).
pub mod event_filter;

// LOOP-06 control channel — Pause/Resume/Abort pipe into the Wave 4
// `tokio::select!` agent loop. See `control.rs` module doc for the full
// rationale; tests live in `tests/control.rs`.
pub mod control;

// LOOP-03 persona loader — YAML serde struct + post-parse validators
// for `forge` / `sage` / `muse` agent profiles. See `persona.rs` module
// doc; tests live in `tests/persona.rs`.
pub mod persona;

// LOOP-01 / LOOP-05 / LOOP-06 agent loop — `run_turn` drives one turn
// via a biased `tokio::select!` over control / input / tool / model
// channels. Module named `r#loop` because `loop` is a Rust keyword;
// the `loop.rs` file path and `r#loop` path segment are both per
// PLAN.md Wave 4 T4.2. See `loop.rs` module doc; tests live in
// `tests/loop.rs`, `tests/loop_dispatcher_integration.rs`, and
// `tests/loop_property.rs`.
pub mod r#loop;

// Phase 13: Enhanced planning system with requirements tracking,
// threat modeling, rollback plans, and quality gates.
pub mod planner;
