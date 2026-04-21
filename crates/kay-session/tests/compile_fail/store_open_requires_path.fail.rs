// Canary: SessionStore::open requires &Path, not &str.
// Expected error: mismatched types — expected `&Path`, found `&str`
fn main() {
    let _ = kay_session::SessionStore::open("~/.kay/sessions");
}
