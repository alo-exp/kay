use forge_fs::is_binary;
use tempfile::NamedTempFile;
use tokio::fs;

#[tokio::test]
async fn is_binary_detects_text() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), b"plain text content").await.unwrap();
    let result = is_binary(file.path()).await.unwrap();
    assert!(!result, "plain text should not be detected as binary");
}

#[tokio::test]
async fn is_binary_detects_binary_file() {
    let file = NamedTempFile::new().unwrap();
    // 0x00 byte inside content marks file as binary
    fs::write(file.path(), &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0x57]).await.unwrap();
    let result = is_binary(file.path()).await.unwrap();
    assert!(result, "file with zero byte should be detected as binary");
}
