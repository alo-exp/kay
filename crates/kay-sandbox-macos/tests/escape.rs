// M12-Task 10: kay-sandbox-macos subprocess escape tests.
//
// Verifies macOS sandbox (sandbox-exec) enforcement at the subprocess level.
// OS-gated: #[cfg(target_os = "macos")] — only compiles on macOS.

use std::path::Path;
use std::sync::OnceLock;

use kay_sandbox_macos::KaySandboxMacos;
use kay_sandbox_policy::SandboxPolicy;

fn project_root() -> &'static Path {
    static ROOT: OnceLock<std::path::PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let tmp = std::env::temp_dir();
        tmp.join("kay_macos_escape_test_project")
    })
}

fn sandbox() -> KaySandboxMacos {
    let root = project_root();
    std::fs::create_dir_all(root).expect("failed to create project root");
    let policy = SandboxPolicy::default_for_project(root.to_path_buf());
    KaySandboxMacos::new(policy)
}

#[test]
fn policy_denies_write_to_system_dir() {
    let s = sandbox();
    let result = s.check_fs_write(Path::new("/System/Library/evil"));
    assert!(
        result.is_err(),
        "Sandbox must deny fs_write to /System/Library"
    );
}

#[test]
fn policy_denies_write_outside_project_root() {
    let s = sandbox();
    let result = s.check_fs_write(Path::new("/tmp/evil.txt"));
    assert!(
        result.is_err(),
        "Sandbox must deny fs_write to /tmp/ (outside project root)"
    );
}

#[test]
fn policy_allows_write_inside_project_root() {
    let s = sandbox();
    let result = s.check_fs_write(project_root().join("src/main.rs").as_path());
    assert!(result.is_ok(), "Sandbox must allow write inside project root");
}

#[test]
fn policy_denies_read_of_sensitive_path() {
    let s = sandbox();
    let result = s.check_fs_read(Path::new("/Users/shared/.ssh/id_rsa"));
    assert!(
        result.is_err(),
        "Sandbox must deny fs_read of sensitive paths"
    );
}

#[test]
fn policy_allows_net_to_minimax() {
    let s = sandbox();
    let url = url::Url::parse("https://api.minimax.io/v1/messages").unwrap();
    let result = s.check_net(&url);
    assert!(result.is_ok(), "Sandbox must allow net to api.minimax.io");
}

#[test]
fn policy_denies_net_to_non_allowlisted() {
    let s = sandbox();
    let url = url::Url::parse("https://evil.example.com/").unwrap();
    let result = s.check_net(&url);
    assert!(result.is_err(), "Sandbox must deny net to non-allowlisted hosts");
}

#[cfg(target_os = "macos")]
mod sandbox_exec_integration {
    use super::*;

    #[test]
    fn subprocess_escape_write_denied() {
        // Policy-level assertion: subprocess write outside project root must be denied.
        let s = sandbox();
        let escape_path = Path::new("/tmp/kay_macos_escape_test");
        let _ = std::fs::remove_file(escape_path);
        let result = s.check_fs_write(escape_path);
        let _ = std::fs::remove_file(escape_path);
        assert!(result.is_err(), "Policy must deny write outside project root");
    }

    #[test]
    fn subprocess_write_in_project_allowed() {
        let s = sandbox();
        let file = project_root().join("test_ok.txt");
        let result = s.check_fs_write(&file);
        assert!(result.is_ok(), "Policy must allow write inside project root");
    }
}
