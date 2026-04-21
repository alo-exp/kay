#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_session::SessionStore;
use kay_session::export::{export_session, import_session};
use kay_session::index::create_session;
use kay_tools::AgentEvent;
use kay_tools::events_wire::AgentEventWire;
use proptest::prelude::*;
use tempfile::TempDir;

proptest! {
    #[test]
    fn export_import_export_stable(n_events in 1usize..10) {
        // Property: export → import → export produces identical transcript.jsonl
        let dir = TempDir::new().unwrap();
        let store = SessionStore::open(dir.path()).unwrap();
        let cwd = dir.path().to_path_buf();
        let mut session = create_session(&store, "prop-test", "forge", "model", &cwd).unwrap();
        let id = session.id;

        for i in 0..n_events {
            let ev = AgentEvent::TextDelta { content: format!("event-{i}") };
            session.append_event(&AgentEventWire::from(&ev)).unwrap();
        }
        drop(session);

        // First export
        let export1 = dir.path().join("export1");
        export_session(&store, &id, &export1).unwrap();
        let first_transcript = std::fs::read(&export1.join("transcript.jsonl")).unwrap();

        // Import → export second time
        let imported = import_session(&store, &export1).unwrap();
        let imported_id = imported.id;
        drop(imported);

        let export2 = dir.path().join("export2");
        export_session(&store, &imported_id, &export2).unwrap();
        let second_transcript = std::fs::read(&export2.join("transcript.jsonl")).unwrap();

        prop_assert_eq!(
            first_transcript, second_transcript,
            "export → import → export must produce identical transcript.jsonl"
        );
    }
}
