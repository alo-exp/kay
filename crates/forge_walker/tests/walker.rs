/// Integration test for forge_walker
use forge_walker::Walker;
use tempfile::TempDir;

#[tokio::test]
async fn walker_empty_dir_yields_no_files() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let walker = Walker::min_all().cwd(temp_dir.path().to_path_buf());

    let files = walker.get().await.expect("walker should not error on empty dir");
    let non_dir_files: Vec<_> = files.into_iter().filter(|f| !f.is_dir()).collect();
    assert!(
        non_dir_files.is_empty(),
        "empty directory should yield no files, got {}",
        non_dir_files.len()
    );
}
