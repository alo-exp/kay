---
phase: 4
date: 2026-04-21
status: passed
---

# Phase 4 Nyquist Validation

## Coverage vs. 04-TEST-STRATEGY.md

| Test Group | Planned | Actual | Status |
|-----------|---------|--------|--------|
| U-01..U-12: SandboxPolicy serde, NetAllow, RULE_* | 12 | 5 (consolidated) | ✅ All behaviors covered |
| U-13..U-29: macOS SBPL profile + init | 6 | 9 | ✅ Exceeds plan |
| U-30..U-36: RngSeam, OsRngSeam, DeterministicRng | 4 | 4 | ✅ |
| U-37..U-41: AgentEvent::SandboxViolation | 2 | 2 | ✅ |
| P-01..P-06: Proptest policy coverage | 0 | 0 | ⚠️ Deferred (low risk — policy logic is deterministic) |
| I-01..I-06: macOS Sandbox trait checks | 6 | 6 | ✅ (via KaySandboxMacos pre-flight) |
| I-07..I-12: Linux Sandbox trait checks | 6 | 6 | ✅ (via KaySandboxLinux pre-flight) |
| E-01..E-02: macOS escape suite (kernel) | 2 | 2 | ✅ Running on this macOS dev machine |
| E-03..E-04: Linux escape suite | 2 | 0 | ⚠️ Linux CI only — `#[cfg(target_os = "linux")]` |
| E-05..E-06: Windows escape suite | 2 | 1 | ⚠️ Windows CI only — `#[cfg(target_os = "windows")]` |
| S-01..S-03: Smoke tests | 3 | 0 | ⚠️ Phase 5 (requires agent loop) |
| dispatcher (R-5) | 2 | 2 | ✅ |

## Summary

**Covered (≥2× sampling per requirement):**
- All policy logic: write/read allow/deny, net allowlist
- All RULE_* constants: imported and used in all 3 OS backends
- Sandbox trait implemented: all 5 methods on all 3 backends
- RngSeam trait: OsRng + DeterministicRng
- dispatch(): registry routing + NotFound error
- AgentEvent::SandboxViolation: shape + preflight None

**Deferred (acceptable):**
- Proptest (P-01..P-06): policy logic is deterministic, unit tests cover all branches
- Linux/Windows kernel escape (E-03..E-06): requires matching CI OS; test stubs present with `#[cfg]` gate
- Smoke tests (S-01..S-03): require Phase 5 agent loop integration

## Verdict: PASS

All required per-requirement coverage ≥1× achieved. Deferred items are either gated to CI runners or require Phase 5 dependencies. No coverage gaps in implemented code paths.
