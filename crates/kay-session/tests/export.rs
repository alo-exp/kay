#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use kay_session::export::{export_session, import_session, replay};
use kay_session::index::{create_session, list_sessions};
use kay_tools::AgentEvent;
use kay_tools::events_wire::AgentEventWire;
use tempfile::TempDir;

fn make_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)
}

fn populate_session(store: &SessionStore, cwd: &std::path::Path, n_events: usize) -> uuid::Uuid {
    let mut session = create_session(store, "export test", "forge", "test-model", cwd).unwrap();
    let id = session.id;
    for i in 0..n_events {
        let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
        session.append_event(&AgentEventWire::from(&ev)).unwrap();
    }
    drop(session);
    id
}

#[test]
fn export_creates_transcript_and_manifest() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 3);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    assert!(
        out_dir.join("transcript.jsonl").exists(),
        "transcript.jsonl must exist"
    );
    assert!(
        out_dir.join("manifest.json").exists(),
        "manifest.json must exist"
    );
}

#[test]
fn manifest_has_required_fields() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 2);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let manifest_str = std::fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let manifest: kay_session::export::ExportManifest =
        serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest.session_id, id);
    assert_eq!(
        manifest.schema_version, 1,
        "schema_version must be 1 in Phase 6"
    );
    assert_eq!(manifest.turn_count, 2);
    assert!(!manifest.model.is_empty());
}

#[test]
fn export_does_not_include_snapshots() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 1);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let entries: Vec<_> = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "export must contain only transcript.jsonl + manifest.json"
    );
}

#[test]
fn import_creates_new_session() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 3);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let imported = import_session(&store, &out_dir).unwrap();
    let sessions = list_sessions(&store, 10).unwrap();
    assert_eq!(sessions.len(), 2, "import must add a new session row");
    drop(imported);
}

#[test]
fn import_new_uuid_not_original() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let original_id = populate_session(&store, &cwd, 2);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &original_id, &out_dir).unwrap();

    let imported = import_session(&store, &out_dir).unwrap();
    assert_ne!(
        imported.id, original_id,
        "imported session must have a new UUID"
    );
}

#[test]
fn import_transcript_matches_original() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 5);

    let original_path: String = store
        .conn
        .query_row(
            "SELECT jsonl_path FROM sessions WHERE id = ?1",
            rusqlite::params![id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let original_lines = std::fs::read_to_string(&original_path)
        .unwrap()
        .lines()
        .count();

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let imported = import_session(&store, &out_dir).unwrap();
    let imported_contents = std::fs::read_to_string(&imported.jsonl_path).unwrap();
    let imported_lines = imported_contents.lines().count();
    assert_eq!(
        imported_lines, original_lines,
        "imported transcript must have same line count"
    );
}

#[test]
fn replay_emits_events_to_dest() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let id = populate_session(&store, &cwd, 4);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let mut buf = Vec::new();
    let count = replay(&out_dir.join("transcript.jsonl"), &mut buf).unwrap();
    assert_eq!(count, 4, "replay must emit 4 events");
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output.lines().count(), 4, "replay output must have 4 lines");
}

#[test]
fn replay_preserves_event_order() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();

    let mut session = create_session(&store, "ordered", "forge", "model", &cwd).unwrap();
    let id = session.id;
    for i in 0..3 {
        let ev = AgentEvent::TextDelta { content: format!("ordered-{i}") };
        session.append_event(&AgentEventWire::from(&ev)).unwrap();
    }
    drop(session);

    let out_dir = dir.path().join("export_out");
    export_session(&store, &id, &out_dir).unwrap();

    let mut buf = Vec::new();
    replay(&out_dir.join("transcript.jsonl"), &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    let contents: Vec<String> = lines
        .iter()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["content"].as_str().unwrap_or("").to_string()
        })
        .collect();
    assert_eq!(contents, vec!["ordered-0", "ordered-1", "ordered-2"]);
}

// ─── T-10 QG-C4 smoke guard ──────────────────────────────────────────────
//
// Verifies: SandboxViolation events are STORED in the transcript (write side
// is correct) AND are NOT re-injected into model context (QG-C4 contract).
// The "not re-injected" property is guaranteed by event_filter.rs (Phase 5,
// unchanged in Phase 6). This test verifies the write side only.
#[test]
fn sandbox_violation_stored_not_re_injected() {
    let (dir, store) = make_store();
    let cwd = dir.path().to_path_buf();
    let mut session = create_session(&store, "smoke", "forge", "model", &cwd).unwrap();
    let id = session.id;

    let ev = AgentEvent::SandboxViolation {
        call_id: "call-test-01".into(),
        tool_name: "fs_write".into(),
        resource: "/outside/project/evil.txt".into(),
        policy_rule: "project_root_only".into(),
        os_error: Some(13),
    };
    session.append_event(&AgentEventWire::from(&ev)).unwrap();
    let path = session.jsonl_path.clone();
    drop(session);

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.contains("SandboxViolation") || contents.contains("sandbox_violation"),
        "SandboxViolation must be stored in transcript (write-side correct)"
    );

    let out_dir = dir.path().join("smoke_export");
    export_session(&store, &id, &out_dir).unwrap();
    let exported = std::fs::read_to_string(out_dir.join("transcript.jsonl")).unwrap();
    assert!(
        !exported.contains("model_input"),
        "exported transcript must not contain model_input wrapper (QG-C5)"
    );
}
