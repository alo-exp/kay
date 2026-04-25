OBJECTIVE: Write the design spec document for Phase 9.1 — Comprehensive Test Coverage — for the Kay Rust workspace.

CONTEXT:
- Project: Kay, a Rust terminal coding agent (fork of ForgeCode). Workspace at /Users/shafqat/Documents/Projects/opencode/vs-others
- Existing spec format: see /Users/shafqat/Documents/Projects/opencode/vs-others/docs/superpowers/specs/ for style reference
- Crates workspace: kay-core, kay-cli, kay-tui, kay-tauri, kay-provider-openrouter, kay-verifier, kay-context, kay-session, kay-tools, kay-sandbox-{macos,linux,windows,policy}, plus ~18 forge_* sub-crates (ForgeCode-inherited)
- Well-tested crates (no action needed): kay-core, kay-tools, kay-cli, kay-verifier, kay-session, kay-context, kay-provider-openrouter
- Test gaps to close:
  1. forge_* crates (18): inline unit tests only, zero integration test dirs
  2. kay-sandbox-{linux,macos,windows}: 1 unit test each, no cross-platform integration tests
  3. forge_api, forge_ci, forge_embed, forge_test_kit, forge_tool_macros: zero coverage
  4. kay-tauri: only gen_bindings + memory_canary; no IPC command tests, no UI automation
  5. kay-tui: stub only, zero tests

DESIGN DECISIONS (pre-approved — transcribe faithfully):
1. Phase name: "Phase 9.1 — Comprehensive Test Coverage" (inserted between Phase 9 and 9.5 in ROADMAP.md)
2. Coverage scope: BOTH internal forge_* behavior AND Kay-native boundary contracts
3. Test tooling:
   - Rust: cargo test, proptest, trybuild, assert_cmd, insta
   - Tauri IPC unit tests: tauri::test::mock_builder() for command handlers
   - Tauri UI automation: tauri-driver + WebDriverIO (@wdio/cli, @wdio/local-runner)
   - kay-tui: ratatui::backend::TestBackend for render tests
4. Coverage targets:
   - forge_* crates: tests/ integration dir per crate; cover all pub API functions called by Kay code
   - forge_api/ci/embed/test_kit: min 1 integration test per exported pub function
   - forge_tool_macros: trybuild proc-macro expansion tests
   - kay-sandbox-*: real subprocess tests on macOS + Linux + Windows CI matrix
   - kay-tauri: tauri::test unit tests for start_session, stop_session, get_session_status; tauri-driver WDIO smoke suite (app launches, session starts, event stream renders)
   - kay-tui: compile test + basic TestBackend render test
5. Wave structure (7 waves):
   - Wave 1: forge_* crates batch 1 (first ~9 alphabetically)
   - Wave 2: forge_* crates batch 2 (remaining ~9)
   - Wave 3: forge_api, forge_ci, forge_embed, forge_test_kit, forge_tool_macros
   - Wave 4: kay-sandbox cross-platform integration tests
   - Wave 5: kay-tauri IPC command unit tests (tauri::test::mock_builder)
   - Wave 6: tauri-driver + WebDriverIO setup + UI smoke suite
   - Wave 7: kay-tui tests + CI matrix wiring + coverage report gate
6. New requirements: TEST-01 through TEST-07
7. Non-negotiables: DCO on every commit, no direct main commits, RC pins with = prefix, TDD red-green per wave

DESIRED STATE:
Write file: /Users/shafqat/Documents/Projects/opencode/vs-others/docs/superpowers/specs/2026-04-23-phase9.1-comprehensive-test-coverage-design.md

The spec must include these sections:
- Frontmatter (date, phase, type)
- Overview (2-3 sentences)
- Goals (bulleted)
- Non-Goals (bulleted — explicitly exclude: forge_* source changes, new features, performance optimization)
- Personas & Value (contributor, CI operator, benchmark runner)
- Success Metrics (measurable: zero untested crates, CI green on 3 OSes, WDIO suite passing)
- Scope (table: area | current state | target state | test type)
- Test Tooling (table: level | tool | crates | coverage target) — as designed above
- Requirements: TEST-01 through TEST-07 with acceptance criteria
  - TEST-01: Every crate has >= 1 integration test in tests/ dir
  - TEST-02: All forge_* pub API functions called by Kay have test coverage
  - TEST-03: Sandbox crates have real subprocess escape tests on macOS + Linux + Windows
  - TEST-04: kay-tauri commands tested via tauri::test::mock_builder()
  - TEST-05: kay-tauri UI has tauri-driver + WDIO smoke suite passing in CI
  - TEST-06: CI test matrix runs on ubuntu-latest + macos-14 + windows-latest
  - TEST-07: Coverage gate: CI fails if any gap-list crate has zero tests post-merge
- Wave Structure (table: wave | crates | test types | RED/GREEN commits)
- Non-Negotiables (7 items including DCO, no direct main, TDD discipline)
- Open Questions (tauri-driver macOS notarization in CI?, WDIO version pinning strategy?)

SUCCESS CRITERIA:
- File written at exact path
- All 7 waves documented with crate lists
- TEST-01..07 defined with clear acceptance criteria
- Scope table covers all 5 gap areas
- Style matches existing specs in the directory

INJECTED SKILLS: testing-strategy, quality-gates
