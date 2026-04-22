use kay_context::watcher::FileWatcher;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn watcher_triggers_on_create() {
    let dir = TempDir::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _watcher = FileWatcher::new(dir.path(), move || {
        c.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await; // let watcher settle
    std::fs::write(dir.path().join("new_file.rs"), "fn foo() {}").unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await; // wait debounce+buffer
    assert!(
        counter.load(Ordering::SeqCst) >= 1,
        "should have triggered on .rs create"
    );
}

#[tokio::test]
async fn watcher_triggers_on_modify() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("existing.rs");
    std::fs::write(&path, "fn original() {}").unwrap();

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _watcher = FileWatcher::new(dir.path(), move || {
        c.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    std::fs::write(&path, "fn modified() {}").unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(
        counter.load(Ordering::SeqCst) >= 1,
        "should have triggered on .rs modify"
    );
}

#[tokio::test]
async fn watcher_triggers_on_remove() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("to_remove.rs");
    std::fs::write(&path, "fn foo() {}").unwrap();

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _watcher = FileWatcher::new(dir.path(), move || {
        c.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    std::fs::remove_file(&path).unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(
        counter.load(Ordering::SeqCst) >= 1,
        "should have triggered on .rs remove"
    );
}

#[tokio::test]
async fn watcher_ignores_non_source() {
    let dir = TempDir::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _watcher = FileWatcher::new(dir.path(), move || {
        c.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    std::fs::write(dir.path().join("Cargo.lock"), "# lock file").unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "should NOT trigger on .lock file"
    );
}

#[tokio::test]
async fn watcher_debounce_coalesces_events() {
    let dir = TempDir::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _watcher = FileWatcher::new(dir.path(), move || {
        c.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    // Write 3 times within 100ms window — should coalesce to 1 event
    for i in 0..3 {
        std::fs::write(dir.path().join("burst.rs"), format!("fn v{}(){{}}", i)).unwrap();
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    tokio::time::sleep(Duration::from_millis(700)).await;
    let count = counter.load(Ordering::SeqCst);
    assert!(count >= 1, "should trigger at least once");
    // Note: debounce coalesces, but may trigger >1 for rapid writes.
    // The invariant is count < 3 (coalesced, not 1 per event).
    assert!(
        count <= 2,
        "debounce should coalesce, not fire 3x: count={}",
        count
    );
}
