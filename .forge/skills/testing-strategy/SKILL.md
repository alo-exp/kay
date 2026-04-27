---
id: testing-strategy
title: Testing Strategy
description: TDD red-green-refactor discipline for Kay — unit tests, integration tests, memory canary, and binding generation tests.
trigger: test, tdd, spec, coverage, canary, assert, expect, mock
---

# Testing Strategy

Follow red-green-refactor: write a failing test first, then make it pass, then refactor.

## Test Levels

### Unit Tests (in-file)
- Place in `#[cfg(test)] mod tests { ... }` within the source file
- Test one function at a time; use `assert_eq!` / `assert!` / `assert_matches!`
- No network, no filesystem, no tokio runtime unless the function explicitly requires it
- Use `tokio::test` for async unit tests

### Integration Tests (`crates/<crate>/tests/`)
- One file per feature area; no spaces in filenames
- May use real filesystem, real tokio runtime
- `gen_bindings.rs`: runs `tauri_specta::Builder` to export TypeScript bindings — fails if types diverge
- `memory_canary.rs`: `#[ignore]` 4-hour RSS test; always-runs `process_rss_is_nonzero` sanity check

### Property Tests
- Use `proptest` for schema hardening (required-before-properties, flattening)
- Adversarial inputs: empty strings, 65535-char strings, null bytes, deeply nested JSON

## RED/GREEN Commit Pattern
- RED commit: test file added, implementation stub (returns `todo!()` or compile error), message prefix `[RED]`
- GREEN commit: implementation complete, all tests pass, message prefix `[GREEN]`
- REFACTOR commit (optional): clean up without changing behavior, message prefix `[REFACTOR]`

## Coverage Targets
- `kay-tools`: ≥90% line coverage on tool implementations
- `kay-core`: ≥85% on loop.rs and run_with_rework
- `kay-tauri`: flush_task drain behavior + IPC round-trip + memory canary
