#[cfg(target_os = "linux")]
mod tests {
    use kay_sandbox_policy::SandboxPolicy;

    #[test]
    fn sandbox_blocks_write_outside_allowed_dir() {
        let policy = SandboxPolicy::default_for_project(std::path::PathBuf::from("/tmp/kay_test_project"));
        // Policy should deny writes outside the project root
        assert!(
            !policy.allows_write(Path::new("/tmp/escape_test_kay_phase91")),
            "Policy should block writes outside allowed dir"
        );
        assert!(
            !policy.allows_write(Path::new("/etc/passwd")),
            "Policy should block writes to /etc"
        );
    }

    #[test]
    fn sandbox_allows_write_inside_allowed_dir() {
        let policy = SandboxPolicy::default_for_project(std::path::PathBuf::from("/tmp/kay_test_project"));
        // Policy should allow writes within the project root
        assert!(
            policy.allows_write(Path::new("/tmp/kay_test_project/src/main.rs")),
            "Policy should allow writes inside project root"
        );
        assert!(
            policy.allows_write(Path::new("/tmp/kay_test_project/Cargo.toml")),
            "Policy should allow writes to Cargo.toml"
        );
    }
}