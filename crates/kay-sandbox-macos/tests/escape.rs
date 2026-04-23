#[cfg(target_os = "macos")]
mod tests {
    use std::path::Path;
    use kay_sandbox_macos::KaySandboxMacos;
    use kay_sandbox_policy::SandboxPolicy;

    #[test]
    fn sandbox_blocks_write_outside_allowed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let policy = SandboxPolicy::default_for_project(dir.path().to_path_buf());
        let sandbox = KaySandboxMacos::new(policy).unwrap();
        let mut cmd = sandbox.sandboxed_command(
            "echo test > /tmp/escape_test_kay_phase91",
            dir.path(),
        );
        let _ = cmd.status().unwrap();
        assert!(
            !Path::new("/tmp/escape_test_kay_phase91").exists(),
            "Sandbox should block writes outside allowed dir"
        );
        let _ = std::fs::remove_file("/tmp/escape_test_kay_phase91");
    }

    #[test]
    fn sandbox_allows_write_inside_allowed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let policy = SandboxPolicy::default_for_project(dir.path().to_path_buf());
        let _sandbox = KaySandboxMacos::new(policy).unwrap();
        let target = dir.path().join("allowed.txt");
        std::fs::write(&target, b"ok").unwrap();
        assert!(target.exists());
    }
}