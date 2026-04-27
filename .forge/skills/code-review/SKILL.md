---
id: code-review
title: Code Review
description: Structured code review checklist for Kay — correctness, performance, API consistency, and ForgeCode compatibility.
trigger: review, diff, check, audit, inspect, verify
---

# Code Review

Run this checklist after implementing a task, before committing.

## Correctness
- All public functions have corresponding tests
- Error paths are tested (not just happy path)
- `#[non_exhaustive]` enums have `_` match arms in all consumers
- No `.unwrap()` or `.expect()` in library code (only in tests or main entry points)

## Performance
- No blocking calls inside `tokio::spawn` async tasks — use `spawn_blocking` for CPU-heavy work
- `flush_task` batches PTY events at 16ms; no per-event channel sends in hot paths
- `DashMap` used for concurrent session state; no `Mutex<HashMap>` under async

## API Consistency
- New `AgentEvent` variants are `#[non_exhaustive]`-safe (additive only)
- `IpcAgentEvent` mirror type updated whenever `AgentEvent` changes
- `RunTurnArgs` struct-literal updated in all call sites when fields are added
- TypeScript bindings regenerated (`cargo test -p kay-tauri -- export_tauri_bindings`) after Rust type changes

## ForgeCode Compatibility
- JSON schema hardening applied: `required` array before `properties` object
- Tool schemas use explicit truncation reminders for large output fields
- `ToolRegistry::register` called with `Arc<dyn Tool>` — not boxed trait objects

## Kay-Specific Checks
- Every commit has `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`
- No direct `main` branch commits — branch + PR only
- RC crate versions pinned with `=` prefix in Cargo.toml
