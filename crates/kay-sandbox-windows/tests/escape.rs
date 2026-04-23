#[cfg(target_os = "windows")]
mod tests {
    use kay_sandbox_policy::SandboxPolicy;

    #[test]
    fn sandbox_blocks_write_outside_allowed_dir() {
        let policy = SandboxPolicy::default_for_project(std::path::PathBuf::from("C:\\Users\\test\\project"));
        // Policy should deny writes outside the project root
        assert!(
            !policy.allows_write(Path::new("C:\\Windows\\System32\\config")),
            "Policy should block writes outside allowed dir"
        );
        assert!(
            !policy.allows_write(Path::new("C:\\Users\\test\\allowed\\..\\..\\Windows")),
            "Policy should block writes outside project root"
        );
    }

    #[test]
    fn sandbox_allows_write_inside_allowed_dir() {
        let policy = SandboxPolicy::default_for_project(std::path::PathBuf::from("C:\\Users\\test\\project"));
        // Policy should allow writes within the project root
        assert!(
            policy.allows_write(Path::new("C:\\Users\\test\\project\\src\\main.rs")),
            "Policy should allow writes inside project root"
        );
        assert!(
            policy.allows_write(Path::new("C:\\Users\\test\\project\\Cargo.toml")),
            "Policy should allow writes to Cargo.toml"
        );
    }
}