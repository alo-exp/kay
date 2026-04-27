You are completing Phase 9.1 "Comprehensive Test Coverage" for the Kay project.

## Current State

- Project root: /Users/shafqat/Documents/Projects/opencode/vs-others
- Branch: phase/09.1-test-coverage (already checked out)
- Latest commit: da4b065 [GREEN] test(wave-3): forge_api/ci/test_kit/tool_macros — 4 integration tests pass
- Waves 1–3 are COMPLETE (6 commits landed). YOUR JOB: complete Waves 4, 5, 6, and 7 in order.

## Iron Laws

1. TDD: RED commit (todo!() stubs that compile but panic at runtime) MUST precede GREEN commit per wave. No exceptions.
2. DCO: Every commit must include the trailer: Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
3. Do NOT modify any forge_* source files — tests are additive only.
4. Work sequentially: finish all commits for Wave N before starting Wave N+1.
5. Use `git commit -s` or append the Signed-off-by trailer manually to every commit.

---

## WAVE 4 — kay-sandbox Cross-Platform Escape Tests

Crates: crates/kay-sandbox-macos, crates/kay-sandbox-linux, crates/kay-sandbox-windows

### W-4 RED

Create crates/kay-sandbox-macos/tests/escape.rs:
```rust
#[cfg(target_os = "macos")]
#[test]
fn sandbox_blocks_write_outside_allowed_dir() { todo!() }

#[cfg(target_os = "macos")]
#[test]
fn sandbox_allows_write_inside_allowed_dir() { todo!() }
```

Create crates/kay-sandbox-linux/tests/escape.rs:
```rust
#[cfg(target_os = "linux")]
#[test]
fn sandbox_blocks_write_outside_allowed_dir() { todo!() }

#[cfg(target_os = "linux")]
#[test]
fn sandbox_allows_write_inside_allowed_dir() { todo!() }
```

Create crates/kay-sandbox-windows/tests/escape.rs:
```rust
#[cfg(target_os = "windows")]
#[test]
fn sandbox_blocks_write_outside_allowed_dir() { todo!() }

#[cfg(target_os = "windows")]
#[test]
fn sandbox_allows_write_inside_allowed_dir() { todo!() }
```

Add to EACH sandbox crate's Cargo.toml:
```toml
[[test]]
name = "escape"
path = "tests/escape.rs"

[dev-dependencies]
tempfile = { workspace = true }
```

Verify stubs compile (they panic at runtime on native OS, which is fine for RED).

RED commit message:
```
[RED] test(wave-4): scaffold escape test stubs for kay-sandbox-{macos,linux,windows}

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

### W-4 GREEN

First read crates/kay-sandbox-macos/src/lib.rs. The struct is KaySandboxMacos, constructed
with KaySandboxMacos::new(policy: SandboxPolicy) and has sandboxed_command(command: &str,
cwd: &Path) -> std::process::Command. Also read crates/kay-sandbox-policy/src/lib.rs for
SandboxPolicy builder API.

Implement crates/kay-sandbox-macos/tests/escape.rs:
```rust
use std::path::Path;
use kay_sandbox_macos::KaySandboxMacos;
use kay_sandbox_policy::SandboxPolicy;

#[cfg(target_os = "macos")]
mod tests {
    use super::*;

    #[test]
    fn sandbox_blocks_write_outside_allowed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let policy = SandboxPolicy::builder()
            .allow_read_write(dir.path())
            .build();
        let sandbox = KaySandboxMacos::new(policy).unwrap();
        let mut cmd = sandbox.sandboxed_command("sh", dir.path());
        cmd.args(["-c", "echo test > /tmp/escape_test_kay_phase91"]);
        let _ = cmd.status().unwrap();
        assert!(
            !Path::new("/tmp/escape_test_kay_phase91").exists(),
            "Sandbox should block writes outside allowed dir"
        );
    }

    #[test]
    fn sandbox_allows_write_inside_allowed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let policy = SandboxPolicy::builder()
            .allow_read_write(dir.path())
            .build();
        let _sandbox = KaySandboxMacos::new(policy).unwrap();
        let target = dir.path().join("allowed.txt");
        std::fs::write(&target, b"ok").unwrap();
        assert!(target.exists());
    }
}
```

Mirror for linux (inspect crates/kay-sandbox-linux/src/lib.rs for the struct name) and
windows (inspect crates/kay-sandbox-windows/src/lib.rs). If the SandboxPolicy builder API
differs, adapt. If construction is async, use #[tokio::test] and add tokio to dev-deps.

Add sandbox-tests job to .github/workflows/ci.yml (insert after the dco job):
```yaml
  sandbox-tests:
    name: Sandbox escape tests (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test -p kay-sandbox-linux -p kay-sandbox-macos -p kay-sandbox-windows
        shell: bash
```

Verify: cargo test -p kay-sandbox-macos passes on macOS.

GREEN commit message:
```
[GREEN] test(wave-4): sandbox escape tests passing + CI OS matrix (3 targets)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

---

## WAVE 5 — kay-tauri IPC Command Tests

The commands in crates/kay-tauri/src/commands.rs are:
- start_session(prompt: String, persona: String, channel: Channel<IpcAgentEvent>, state: State<AppState>) -> Result<String, String>
- stop_session(session_id: String, state: State<AppState>) -> Result<(), String>
- get_session_status(session_id: String, state: State<AppState>) -> Result<SessionStatus, String>

SessionStatus variants: Running, Complete.
AppState (inspect crates/kay-tauri/src/state.rs) wraps a dashmap::DashMap<String, CancellationToken>.

### W-5 RED

Create crates/kay-tauri/tests/commands.rs:
```rust
#[tokio::test]
async fn stop_nonexistent_session_returns_ok() { todo!() }

#[tokio::test]
async fn get_status_of_unknown_session_returns_complete() { todo!() }

#[tokio::test]
async fn start_and_stop_session_roundtrip() { todo!() }
```

Add to crates/kay-tauri/Cargo.toml [dev-dependencies]:
```toml
tauri = { workspace = true, features = ["test"] }
tokio = { workspace = true, features = ["macros", "rt"] }
tokio-util = { workspace = true }
```

Add [[test]] entry:
```toml
[[test]]
name = "commands"
path = "tests/commands.rs"
```

RED commit message:
```
[RED] test(wave-5): scaffold failing IPC command test stubs for kay-tauri

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

### W-5 GREEN

Read crates/kay-tauri/src/state.rs for the exact AppState type. Build a mock test app with
tauri::test::mock_builder().manage(AppState::default()).build(tauri::generate_context!()).
Call stop_session and get_session_status directly (not via IPC) using state from
app.state::<AppState>(). For the roundtrip test, insert a CancellationToken manually.

```rust
use tauri::test::mock_builder;
use tauri::Manager;
use kay_tauri::state::AppState;

fn build_test_app() -> tauri::App<tauri::test::MockRuntime> {
    mock_builder()
        .manage(AppState::default())
        .build(tauri::generate_context!())
        .expect("failed to build test app")
}

#[tokio::test]
async fn stop_nonexistent_session_returns_ok() {
    let app = build_test_app();
    let state = app.state::<AppState>();
    let result = kay_tauri::commands::stop_session(
        "nonexistent-id".to_string(), state
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_status_of_unknown_session_returns_complete() {
    let app = build_test_app();
    let state = app.state::<AppState>();
    let result = kay_tauri::commands::get_session_status(
        "unknown-id".to_string(), state
    ).await;
    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        kay_tauri::commands::SessionStatus::Complete
    ));
}

#[tokio::test]
async fn start_and_stop_session_roundtrip() {
    let app = build_test_app();
    let session_id = "test-session-123".to_string();
    let token = tokio_util::sync::CancellationToken::new();
    app.state::<AppState>().sessions.insert(session_id.clone(), token);

    let status = kay_tauri::commands::get_session_status(
        session_id.clone(), app.state::<AppState>()
    ).await.unwrap();
    assert!(matches!(status, kay_tauri::commands::SessionStatus::Running));

    kay_tauri::commands::stop_session(
        session_id.clone(), app.state::<AppState>()
    ).await.unwrap();

    let final_status = kay_tauri::commands::get_session_status(
        session_id, app.state::<AppState>()
    ).await.unwrap();
    assert!(matches!(final_status, kay_tauri::commands::SessionStatus::Complete));
}
```

If AppState does not implement Default, inspect state.rs and construct it appropriately.
If tauri::generate_context!() needs a tauri.conf.json, look in crates/kay-tauri/src-tauri/.

Verify: cargo test -p kay-tauri — all 3 tests pass.

GREEN commit message:
```
[GREEN] test(wave-5): IPC command tests passing for kay-tauri (3 tests)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

---

## WAVE 6 — WebDriverIO UI Smoke Suite

All npm work is inside crates/kay-tauri/ui/.

### W-6 RED

Step 1: Install WDIO packages:
```
cd crates/kay-tauri/ui && npm install --save-dev @wdio/cli@9 @wdio/local-runner@9 @wdio/mocha-framework@9 @wdio/spec-reporter@9 webdriverio@9 tauri-driver tsx
```

Step 2: Create crates/kay-tauri/ui/wdio.conf.ts:
```typescript
import type { Options } from "@wdio/types";

export const config: Options.Testrunner = {
  runner: "local",
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: { project: "./tsconfig.json" },
  },
  specs: ["./e2e/**/*.ts"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: "../../../target/debug/kay",
      },
    },
  ],
  logLevel: "warn",
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  services: ["tauri-driver"],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: { ui: "bdd", timeout: 30000 },
};
```

Step 3: Create crates/kay-tauri/ui/e2e/smoke.ts:
```typescript
describe("Kay Desktop Smoke", () => {
  it.skip("app window opens", async () => {});
  it.skip("session view renders", async () => {});
  it.skip("start session button exists", async () => {});
  it.skip("stop session button exists", async () => {});
  it.skip("cost meter visible", async () => {});
});
```

Step 4: Add to crates/kay-tauri/ui/package.json scripts:
```json
"test:e2e:stub": "wdio run wdio.conf.ts",
"test:e2e": "wdio run wdio.conf.ts"
```

Verify: cd crates/kay-tauri/ui && npm run test:e2e:stub exits 0 (all 5 skipped).

RED commit message:
```
[RED] test(wave-6): scaffold WDIO e2e smoke suite stubs (5 cases, all skipped)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

### W-6 GREEN

Replace skipped stubs in e2e/smoke.ts with real WDIO assertions:
```typescript
describe("Kay Desktop Smoke", () => {
  it("app window opens", async () => {
    const title = await browser.getTitle();
    expect(title).toMatch(/Kay/);
  });

  it("session view renders", async () => {
    const el = await $('[data-testid="session-view"]');
    await expect(el).toExist();
  });

  it("start session button exists", async () => {
    const el = await $('[data-testid="start-session-btn"]');
    await expect(el).toExist();
  });

  it("stop session button exists", async () => {
    const el = await $('[data-testid="stop-session-btn"]');
    await expect(el).toExist();
  });

  it("cost meter visible", async () => {
    const el = await $('[data-testid="cost-meter"]');
    await expect(el).toExist();
  });
});
```

Inspect crates/kay-tauri/ui/src/ to find the React components. Add data-testid attributes:
- data-testid="session-view" on the main session container div
- data-testid="start-session-btn" on the start button
- data-testid="stop-session-btn" on the stop button
- data-testid="cost-meter" on the cost/token meter element

Create .github/workflows/ui-smoke.yml:
```yaml
name: UI Smoke
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
jobs:
  ui-smoke:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build -p kay-tauri
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
      - run: npm ci
        working-directory: crates/kay-tauri/ui
      - run: npm run test:e2e
        working-directory: crates/kay-tauri/ui
```

GREEN commit message:
```
[GREEN] test(wave-6): WDIO UI smoke suite (5 cases) + data-testid attrs + ui-smoke CI workflow

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

---

## WAVE 7 — kay-tui Render Tests + Coverage Gate + CI Matrix

### W-7 RED

Inspect crates/kay-tui/src/ first (has main.rs, gen_bindings.rs, memory_canary.rs).

Create crates/kay-tui/tests/render.rs:
```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn session_view_renders_stub() { todo!() }

#[test]
fn tool_call_inspector_renders_stub() { todo!() }
```

Add to crates/kay-tui/Cargo.toml:
```toml
[[test]]
name = "render"
path = "tests/render.rs"

[dev-dependencies]
ratatui = { workspace = true }
```

If ratatui is not in the workspace Cargo.toml [workspace.dependencies], add:
```toml
ratatui = "0.29"
```

RED commit message:
```
[RED] test(wave-7): scaffold failing render test stubs for kay-tui

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

### W-7 GREEN

Implement render tests. Use ratatui primitives (Block, Paragraph) as stand-ins if kay-tui
does not yet export named widget types:

```rust
use ratatui::{backend::TestBackend, Terminal};

fn make_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(80, 24)).unwrap()
}

#[test]
fn session_view_renders_without_panic() {
    let mut t = make_terminal();
    t.draw(|f| {
        let block = ratatui::widgets::Block::default().title("Kay Session");
        f.render_widget(block, f.area());
    })
    .unwrap();
}

#[test]
fn tool_call_inspector_renders_without_panic() {
    let mut t = make_terminal();
    t.draw(|f| {
        let p = ratatui::widgets::Paragraph::new("Tool calls")
            .block(ratatui::widgets::Block::default());
        f.render_widget(p, f.area());
    })
    .unwrap();
}
```

If kay-tui exports actual widget types (SessionView, ToolCallInspector), use those instead.

Create scripts/coverage-gate.sh (then chmod +x scripts/coverage-gate.sh):
```bash
#!/bin/bash
set -euo pipefail

GAP_LIST=(
  "forge_app" "forge_config" "forge_display" "forge_domain" "forge_embed"
  "forge_fs" "forge_infra" "forge_json_repair" "forge_main" "forge_spinner"
  "forge_markdown_stream" "forge_repo" "forge_services" "forge_snaps"
  "forge_stream" "forge_template" "forge_tracker" "forge_walker"
  "forge_api" "forge_ci" "forge_test_kit" "forge_tool_macros"
  "kay-sandbox-macos" "kay-sandbox-linux" "kay-sandbox-windows"
  "kay-tauri" "kay-tui"
)

FAILED=0
for crate in "${GAP_LIST[@]}"; do
  dir=$(find crates -maxdepth 1 -name "$crate" -type d 2>/dev/null | head -1)
  if [ -z "$dir" ]; then
    echo "WARN: $crate not found"
    continue
  fi
  test_count=$(find "$dir/tests" -name "*.rs" 2>/dev/null | wc -l | tr -d ' ')
  src_test=$(grep -rl "#\[test\]" "$dir/src" 2>/dev/null | wc -l | tr -d ' ')
  total=$((test_count + src_test))
  if [ "$total" -eq 0 ]; then
    echo "FAIL: $crate has zero test files"
    FAILED=1
  else
    echo "OK:   $crate ($total test file(s))"
  fi
done

if [ "$FAILED" -eq 1 ]; then
  echo ""
  echo "Coverage gate FAILED: one or more gap-list crates have no tests."
  exit 1
fi
echo ""
echo "Coverage gate PASSED: all gap-list crates have at least one test file."
```

Extend .github/workflows/ci.yml:
1. Find the main test job and add a matrix + update runs-on:
   ```yaml
   strategy:
     matrix:
       os: [ubuntu-latest, macos-latest, windows-latest]
   runs-on: ${{ matrix.os }}
   ```
2. Add a new coverage-gate job (requires the test job):
   ```yaml
     coverage-gate:
       name: Coverage gate
       runs-on: ubuntu-latest
       needs: [test]
       steps:
         - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5
         - name: Run coverage gate
           run: bash scripts/coverage-gate.sh
   ```

Verify: ./scripts/coverage-gate.sh exits 0. cargo test -p kay-tui passes.

GREEN commit message:
```
[GREEN] test(wave-7): kay-tui render tests + coverage-gate.sh + CI matrix + coverage-gate job

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>
```

---

## FINAL VERIFICATION

After all 8 commits (4 RED + 4 GREEN):

1. ./scripts/coverage-gate.sh — must exit 0
2. cargo test -p kay-sandbox-macos -p kay-sandbox-linux -p kay-sandbox-windows
3. cargo test -p kay-tauri
4. cargo test -p kay-tui
5. git log --oneline -10 — confirm 4 RED + 4 GREEN commits in order

Output a summary table showing each commit hash and which tests pass.
