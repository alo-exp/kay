// Canary: Session cannot be used after drop (move semantics enforce exclusive ownership).
// Expected error: E0382 use of moved value: `session`
fn main() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = kay_session::SessionStore::open(dir.path()).unwrap();
    let session = kay_session::index::create_session(
        &store, "title", "forge", "model", dir.path()
    ).unwrap();
    drop(session);
    // Attempting to use session after drop must fail
    let _ = session.id; // E0382: use of moved value
}
