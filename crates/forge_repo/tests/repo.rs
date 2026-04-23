/// Integration test for forge_repo
use forge_snaps::SnapshotService;

#[test]
fn snapshot_service_debug_non_empty() {
    let service = SnapshotService::new(std::path::PathBuf::from("/tmp/test"));
    let debug_str = format!("{:?}", service);
    assert!(
        !debug_str.is_empty(),
        "SnapshotService Debug output should not be empty"
    );
}
