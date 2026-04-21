// Canary: Session must NOT derive Clone (exclusive file handle — DL-9).
// Expected error: the trait `Clone` is not implemented for `Session`
fn main() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<kay_session::index::Session>();
}
