use forge_app::{SyncPaths, SyncProgressCounter};

#[test]
fn sync_paths_constructs() {
    // SyncPaths is a simple struct with pub fields — verify it constructs
    let sp = SyncPaths { delete: vec![], upload: vec![] };
    assert!(sp.delete.is_empty());
    assert!(sp.upload.is_empty());
}

#[test]
fn sync_progress_counter_increments() {
    let mut counter = SyncProgressCounter::new(4, 8);
    // Initial state: 0% progress
    let progress = counter.sync_progress();
    assert_eq!(
        format!("{:?}", progress),
        "Syncing { current: 0, total: 4 }",
        "initial progress should be 0"
    );
    // After completing 2 operations, progress should improve
    counter.complete(2);
    let progress = counter.sync_progress();
    assert_eq!(
        format!("{:?}", progress),
        "Syncing { current: 1, total: 4 }",
        "after 2 of 8 ops, current should be 1"
    );
}
