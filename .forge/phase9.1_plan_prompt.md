OBJECTIVE: Write .planning/phases/09.1-test-coverage/09.1-PLAN.md for the Phase 9.1 comprehensive test coverage phase of the Kay Rust workspace.

CONTEXT:
- Read .planning/phases/08-multi-perspective-verification/PLAN.md for exact format/style to match
- Spec: docs/superpowers/specs/2026-04-23-phase9.1-comprehensive-test-coverage-design.md (read §9 Wave Structure and §10 Non-Negotiables)
- Branch: phase/09.1-test-coverage
- Phase number: 9.1
- Depends on: Phase 9 (Tauri Desktop Shell)

DESIRED STATE:
File: .planning/phases/09.1-test-coverage/09.1-PLAN.md

The PLAN.md must have the same structure as the Phase 8 PLAN.md. Include:

1. Frontmatter block:
   - phase: 9.1
   - name: Comprehensive Test Coverage
   - branch: phase/09.1-test-coverage
   - depends_on: Phase 9
   - requirements: TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06, TEST-07
   - status: draft

2. Goal section (1-2 sentences from spec overview)

3. Success Criteria (7 items matching TEST-01..TEST-07 acceptance criteria)

4. Wave breakdown — 7 waves as numbered plan sections (W-1 through W-7):

   W-1 (forge_* batch 1):
   - Crates: forge_app, forge_config, forge_display, forge_domain, forge_fs, forge_infra, forge_json_repair, forge_main, forge_spinner
   - Tasks: [RED] create tests/ dir + failing integration test stubs; [GREEN] implement tests; cargo test -p <crate> per crate
   - Test files: tests/ping.rs (forge_app), tests/load.rs (forge_config), tests/render.rs (forge_display + forge_spinner), tests/entity.rs (forge_domain), tests/read.rs + tests/write.rs (forge_fs), tests/health.rs (forge_infra), tests/repair.rs (forge_json_repair), tests/entry.rs (forge_main)
   - Commit pattern: [RED] "test(forge-batch-1): add failing integration test stubs" then [GREEN] "test(forge-batch-1): implement integration tests, all passing"

   W-2 (forge_* batch 2):
   - Crates: forge_embed, forge_markdown_stream, forge_repo, forge_services, forge_snaps, forge_stream, forge_template, forge_tracker, forge_walker
   - Tasks: [RED] create tests/ dir + failing stubs; [GREEN] implement tests; cargo test -p <crate> per crate
   - Test files: tests/encode.rs (forge_embed), tests/stream.rs (forge_markdown_stream + forge_stream), tests/discover.rs (forge_repo), tests/plugin.rs (forge_services), tests/snapshot.rs (forge_snaps), tests/render.rs (forge_template), tests/track.rs (forge_tracker), tests/walk.rs (forge_walker)

   W-3 (forge_api, forge_ci, forge_test_kit, forge_tool_macros):
   - Crates: forge_api, forge_ci, forge_test_kit, forge_tool_macros
   - Tasks: [RED] create tests/ + trybuild stubs for forge_tool_macros; [GREEN] implement all tests
   - forge_tool_macros: tests/expand.rs using trybuild; tests/ui/ subdir with .rs/.stderr pairs
   - forge_api: tests/request.rs + tests/response.rs using proptest
   - forge_ci: tests/build.rs using assert_cmd
   - forge_test_kit: tests/suite.rs using assert_cmd

   W-4 (sandbox cross-platform):
   - Crates: kay-sandbox-linux, kay-sandbox-macos, kay-sandbox-windows
   - Tasks: [RED] create tests/escape.rs per crate with failing subprocess tests; [GREEN] implement escape assertions
   - Each test: spawn real std::process::Command, assert sandbox denies forbidden op (write /etc/passwd, exec /bin/sh)
   - OS gates: #[cfg(target_os = "linux")] / "macos" / "windows"
   - CI: add matrix runs-on: [ubuntu-latest, macos-14, windows-latest] for sandbox jobs
   - Commit: [RED] then [GREEN] per platform batch

   W-5 (kay-tauri IPC unit tests):
   - Crate: kay-tauri
   - Tasks: [RED] create tests/commands.rs with failing tauri::test::mock_builder() tests; [GREEN] implement
   - Tests: start_session (session_id returned, AppState populated), stop_session (CancellationToken cancelled), get_session_status (Running vs Complete)
   - Add tauri = {workspace=true, features=["test"]} to [dev-dependencies] in kay-tauri/Cargo.toml
   - cargo test -p kay-tauri

   W-6 (tauri-driver + WebDriverIO UI):
   - Directory: crates/kay-tauri/ui/
   - Tasks: install @wdio/cli @wdio/local-runner webdriverio; create wdio.conf.ts; create e2e/smoke.ts
   - Smoke tests (>=5): app launches without crash, prompt input visible, Run button clickable, event renders, stop button works
   - CI: add .github/workflows/ui-smoke.yml running on macos-14 + ubuntu-latest
   - Commit: [RED] wdio setup + failing smoke test; [GREEN] passing smoke suite

   W-7 (kay-tui + coverage gate + CI matrix):
   - Tasks:
     a) kay-tui: add ratatui::backend::TestBackend to [dev-dependencies]; create tests/render.rs; cargo test -p kay-tui
     b) coverage gate: create scripts/coverage-gate.sh; list all gap-list crates; check tests/ dir + #[test] presence; exit 1 on failure
     c) CI matrix: add coverage-gate job to .github/workflows/ci.yml running on ubuntu-latest after tests pass
   - Commit: [RED] then [GREEN]

5. Verification steps:
   - Per wave: cargo test -p <crate> passes
   - End of phase: scripts/coverage-gate.sh exits 0 on all gap-list crates
   - CI: all 3 OS matrix jobs green

6. Non-Negotiables (same 7 as spec §10)

7. DCO trailer on every commit:
   Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>

SUCCESS CRITERIA:
- File written at exact path: .planning/phases/09.1-test-coverage/09.1-PLAN.md
- All 7 waves present with crate lists and commit patterns
- Style matches Phase 8 PLAN.md

INJECTED SKILLS: testing-strategy, quality-gates
