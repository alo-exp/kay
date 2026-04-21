#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn kay_cmd() -> Command {
    Command::cargo_bin("kay").unwrap()
}

fn with_home(dir: &TempDir) -> Command {
    let mut cmd = kay_cmd();
    cmd.env("KAY_HOME", dir.path());
    cmd
}

#[test]
fn session_list_empty() {
    let dir = TempDir::new().unwrap();
    with_home(&dir)
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(contains("No sessions found"));
}

#[test]
fn session_list_table_format() {
    let dir = TempDir::new().unwrap();
    // First run to create a session via the event-tap in run_async
    with_home(&dir)
        .args(["run", "--prompt", "TEST:done", "--offline"])
        .env("KAY_HOME", dir.path())
        .assert(); // may succeed or fail — session creation is best-effort in offline mode

    // Validate that session list exits 0 regardless of session count
    with_home(&dir)
        .args(["session", "list"])
        .assert()
        .success();
}

#[test]
fn session_list_json_format() {
    let dir = TempDir::new().unwrap();
    let output = with_home(&dir)
        .args(["session", "list", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed.is_array(), "json format must output a JSON array");
}

#[test]
fn session_export_creates_files() {
    use kay_session::index::create_session;
    use kay_session::SessionStore;
    use kay_tools::events_wire::AgentEventWire;
    use kay_tools::AgentEvent;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let mut session = create_session(&store, "e2e export", "forge", "test", dir.path()).unwrap();
    let id = session.id;
    let ev = AgentEvent::TextDelta { content: "hello".into() };
    session.append_event(&AgentEventWire::from(&ev)).unwrap();
    drop(session);
    drop(store);

    let out_dir = dir.path().join("export_out");
    with_home(&dir)
        .args([
            "session", "export", &id.to_string(),
            "--output", out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_dir.join("transcript.jsonl").exists());
    assert!(out_dir.join("manifest.json").exists());
}

#[test]
fn session_import_round_trip() {
    use kay_session::index::create_session;
    use kay_session::SessionStore;
    use kay_tools::events_wire::AgentEventWire;
    use kay_tools::AgentEvent;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let mut session = create_session(&store, "import test", "forge", "model", dir.path()).unwrap();
    let id = session.id;
    let ev = AgentEvent::TextDelta { content: "import me".into() };
    session.append_event(&AgentEventWire::from(&ev)).unwrap();
    drop(session);
    drop(store);

    let out_dir = dir.path().join("exp");
    with_home(&dir)
        .args(["session", "export", &id.to_string(), "--output", out_dir.to_str().unwrap()])
        .assert().success();

    with_home(&dir)
        .args(["session", "import", out_dir.to_str().unwrap()])
        .assert().success();

    let output = with_home(&dir)
        .args(["session", "list", "--format", "json"])
        .assert().success()
        .get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    let sessions: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(sessions.as_array().unwrap().len() >= 2, "original + imported must both appear");
}

#[test]
fn session_replay_emits_jsonl() {
    use kay_session::index::create_session;
    use kay_session::SessionStore;
    use kay_tools::events_wire::AgentEventWire;
    use kay_tools::AgentEvent;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let mut session = create_session(&store, "replay test", "forge", "model", dir.path()).unwrap();
    let id = session.id;
    for i in 0..3 {
        let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
        session.append_event(&AgentEventWire::from(&ev)).unwrap();
    }
    drop(session);
    drop(store);

    let out_dir = dir.path().join("replay_exp");
    with_home(&dir)
        .args(["session", "export", &id.to_string(), "--output", out_dir.to_str().unwrap()])
        .assert().success();

    let output = with_home(&dir)
        .args(["session", "replay", out_dir.to_str().unwrap()])
        .assert().success()
        .get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout.lines().count(), 3, "replay must emit 3 JSONL lines");
}

#[test]
fn rewind_no_snapshot_exit_1() {
    let dir = TempDir::new().unwrap();
    with_home(&dir)
        .args(["rewind"])
        .assert()
        .failure();
}

#[test]
fn session_fork_creates_child() {
    use kay_session::index::create_session;
    use kay_session::SessionStore;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let session = create_session(&store, "parent", "forge", "model", dir.path()).unwrap();
    let parent_id = session.id;
    drop(session);
    drop(store);

    with_home(&dir)
        .args(["session", "fork", &parent_id.to_string()])
        .assert()
        .success();

    let output = with_home(&dir)
        .args(["session", "list", "--format", "json"])
        .assert().success()
        .get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    let sessions: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(sessions.as_array().unwrap().len(), 2);
}

#[test]
fn resume_flag_on_run() {
    use kay_session::index::create_session;
    use kay_session::SessionStore;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let session = create_session(&store, "resume test", "forge", "model", dir.path()).unwrap();
    let id = session.id;
    drop(session);
    drop(store);

    with_home(&dir)
        .args(["run", "--prompt", "TEST:done", "--offline", "--resume", &id.to_string()])
        .assert()
        .success();
}

#[test]
fn rewind_dry_run_no_write() {
    use kay_session::index::create_session;
    use kay_session::snapshot::{record_snapshot, SessConfig};
    use kay_session::SessionStore;

    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    let store = SessionStore::open(&sessions_dir).unwrap();
    let cwd = dir.path().to_path_buf();
    let session = create_session(&store, "rewind test", "forge", "model", &cwd).unwrap();
    let id = session.id;
    drop(session);

    let target = cwd.join("target.txt");
    std::fs::write(&target, b"original").unwrap();
    let config = SessConfig::default();
    record_snapshot(&store, &id, &cwd, 1, &target, b"snapshot content", &config).unwrap();
    drop(store);

    std::fs::write(&target, b"modified").unwrap();

    with_home(&dir)
        .args(["rewind", "--session", &id.to_string(), "--dry-run"])
        .assert()
        .success();

    assert_eq!(
        std::fs::read(&target).unwrap(),
        b"modified",
        "--dry-run must not restore the file"
    );
}
