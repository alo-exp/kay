OBJECTIVE: Create the Phase 9.1 testing strategy document and PLAN.md for the Kay workspace comprehensive test coverage phase.

CONTEXT:
- Spec doc already written at: docs/superpowers/specs/2026-04-23-phase9.1-comprehensive-test-coverage-design.md
- Read it first to extract all decisions before writing the two artifacts
- Phase dir to create: .planning/phases/09.1-test-coverage/
- Existing PLAN.md example to match style: .planning/phases/08-kira-critics/ (read any PLAN.md file in there)
- Existing TEST-STRATEGY.md example: .planning/phases/05-agent-loop/05-TEST-STRATEGY.md (read it for format)

DESIRED STATE:

1. File: .planning/phases/09.1-test-coverage/09.1-TEST-STRATEGY.md
   Content must include:
   - Test pyramid breakdown (unit%, integration%, property%, compile-fail%, UI automation%)
   - Tooling table (tool | purpose | crates | notes)
   - TDD discipline section: RED-GREEN-REFACTOR per wave; commit message prefixes [RED]/[GREEN]
   - Coverage targets per crate area (matching spec §8)
   - CI strategy: matrix (ubuntu-latest + macos-14 + windows-latest), per-crate cargo test
   - Gap-closure verification: coverage-gate.sh script enforcing zero-tests crates fail CI
   - All 7 waves with their specific tooling

2. File: .planning/phases/09.1-test-coverage/09.1-PLAN.md
   Full implementation plan with:
   - Frontmatter: phase: 9.1, goal, depends_on: [Phase 9], requirements: TEST-01..TEST-07
   - Goal statement (match spec overview)
   - Requirements mapping (TEST-01 through TEST-07)
   - 7 waves as tasks with explicit sub-steps:
     Wave 1: forge_app, forge_config, forge_display, forge_domain, forge_embed(partial), forge_fs, forge_infra, forge_json_repair, forge_main — tests/ dir + integration tests
     Wave 2: forge_markdown_stream, forge_repo, forge_services, forge_snaps, forge_spinner, forge_stream, forge_template, forge_tracker, forge_walker — tests/ dir + integration tests
     Wave 3: forge_api, forge_ci, forge_embed(full), forge_test_kit, forge_tool_macros — proptest + trybuild
     Wave 4: kay-sandbox-linux, kay-sandbox-macos, kay-sandbox-windows — real subprocess escape tests + OS-gated CI
     Wave 5: kay-tauri commands.rs tests — tauri::test::mock_builder() for start_session, stop_session, get_session_status
     Wave 6: kay-tauri UI — @wdio/cli setup, wdio.conf.ts, e2e/smoke.ts smoke test
     Wave 7: kay-tui render test, coverage-gate.sh script, CI matrix wiring in .github/workflows/ci.yml
   - Verification steps per wave (cargo test -p <crate>)
   - Success criteria (matches spec TEST-01..07 acceptance criteria)
   - Non-negotiables (DCO, TDD, additive-only, 3-OS CI)
   - Branch: phase/09.1-test-coverage

SUCCESS CRITERIA:
- Both files written to the exact paths
- PLAN.md has all 7 waves with explicit crate lists
- TEST-STRATEGY.md has all tooling rows and TDD discipline section
- Style matches existing .planning/phases/ artifacts

INJECTED SKILLS: testing-strategy, quality-gates
