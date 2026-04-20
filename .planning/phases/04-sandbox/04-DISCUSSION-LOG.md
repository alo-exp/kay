---
phase: 4
date: 2026-04-21
mode: autonomous
---

# Phase 4 Discussion Log

**Mode:** autonomous (--auto, no stall, all gray areas auto-resolved from brainstorm)

## Gray Areas Resolved

All 10 engineering gray areas (E1..E10) from 04-BRAINSTORM.md §Engineering-Lens were auto-resolved using prior session decisions. No interactive clarification was required.

| ID | Topic | Resolution |
|----|-------|------------|
| E1 | macOS profile delivery | Inline `-p` flag, SBPL cached as `Arc<str>` at construction |
| E2 | Linux policy | Landlock + seccomp BPF; ENOSYS → warn + seccomp-only fallback |
| E3 | Windows API | `windows-sys` raw bindings (already in workspace) |
| E4 | AgentEvent::SandboxViolation shape | 5-field struct (call_id, tool_name, resource, policy_rule, os_error) |
| E5 | SandboxPolicy location | New `kay-sandbox-policy` crate (zero OS deps) |
| E6 | dispatcher.rs | `dispatch()` → R-5 closure + Phase 5 entry point |
| E7 | rng.rs | `RngSeam` trait + `OsRngSeam` + `DeterministicRng` |
| E8 | CI matrix | `macos-14` / `ubuntu-latest` / `windows-latest` |
| E9 | Landlock degradation | `tracing::warn!` + seccomp-only; no error propagation |
| E10 | Escape test isolation | `std::process::Command` user-space, no root, `tempfile` dirs |

## Planning Constraints Captured

QG-C1..QG-C4 from quality-gates design-time review are recorded in CONTEXT.md and must appear in PLAN.md.

## Deferred Ideas

- Per-tool policy override (Phase 5+)
- Policy hot-reload without restart (Phase 7+)
- Windows Integrity Level below Medium (Phase 4+ optional)
- Audit log file for SandboxViolation events (Phase 6+)
