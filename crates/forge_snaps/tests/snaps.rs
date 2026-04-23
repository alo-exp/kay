/// Integration test for forge_snaps
use forge_snaps::SnapshotService;
use std::path::PathBuf;

#[test]
fn snapshot_service_debug_non_empty() {
    let service = SnapshotService::new(PathBuf::from("/tmp/snaps-test"));
    let debug_str = format!("{:?}", service);
    assert!(
        !debug_str.is_empty(),
        "SnapshotService Debug output should not be empty, got: {}",
        debug_str
    );
}

#[test]
fn snapshot_service_new_accepts_path() {
    // Smoke test: constructing with a temp path should not panic
    let _service = SnapshotService::new(PathBuf::from("/tmp"));
    let _service2 = SnapshotService::new(PathBuf::from("."));
}
