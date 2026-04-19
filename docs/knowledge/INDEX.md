# Project Knowledge Index

Quick-reference pointer to current project docs. Updated when docs are added/removed.

| Doc | Path | Purpose |
|-----|------|---------|
| Product | `docs/PRD-Overview.md` | Vision, core value, requirement areas, 3-frontend architecture |
| Architecture | `docs/ARCHITECTURE.md` | 8-crate workspace, CLI/TUI/GUI over one core, design principles |
| Testing | `docs/TESTING.md` | Test pyramid, coverage goals, governance invariants |
| CI/CD | `docs/CICD.md` | Pipeline stages — 6 CI jobs + nightly audit + governance checker |
| Task Log | `docs/CHANGELOG.md` | Rolling task log (50 entries max) |
| Session Logs | `docs/sessions/` | Per-session work logs (30 max) |
| Design Specs | `docs/specs/` | Point-in-time design specs |
| Active Workflow | `docs/workflows/full-dev-cycle.md` | Dev cycle steps |
| Git Repo | https://github.com/alo-exp/kay | — |

**Planning artifacts (durable reference):**
- `.planning/PROJECT.md` — authoritative project definition + architectural amendments
- `.planning/REQUIREMENTS.md` — all REQ-IDs with traceability to phases
- `.planning/ROADMAP.md` — 13 phases (12 integer + Phase 9.5 TUI insertion)
- `.planning/STATE.md` — current-position cursor
- `.planning/research/` — project-level research (STACK, FEATURES, ARCHITECTURE, PITFALLS, SUMMARY)
- `.planning/phases/01-fork-governance-infrastructure/` — Phase 1 completed artifacts (CONTEXT, RESEARCH, VALIDATION, 6 PLANs, 6 SUMMARYs, VERIFICATION, REVIEW-ROUNDS, parity-baseline archive)

**Governance artifacts:**
- `LICENSE`, `NOTICE`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `ATTRIBUTIONS.md`
- `.github/workflows/ci.yml` + `audit.yml`
- `tests/governance/check_attribution.sh` (run locally: 35 grep-based invariants)
- `forgecode-parity-baseline` git tag (annotated, unsigned per D-OP-04 deferral)

Latest knowledge: `docs/knowledge/2026-04.md`
Latest lessons: `docs/lessons/2026-04.md`
