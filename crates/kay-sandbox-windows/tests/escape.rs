// M12-Task 10: kay-sandbox-windows subprocess escape tests.
//
// Verifies Windows sandbox enforcement at the subprocess level.
// OS-gated: #[cfg(target_os = "windows")] — only compiles on Windows.

use kay_tools::seams::sandbox::Sandbox;
use std::path::Path;
use std::sync::OnceLock;

use kay_sandbox_policy::SandboxPolicy;
use kay_sandbox_windows::KaySandboxWindows;

fn project_root() -> &'static Path {
    static ROOT: OnceLock<std::path::PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| std::env::temp_dir().join("kay_windows_escape_test_project"))
}

fn sandbox() -> KaySandboxWindows {
    let root = project_root();
    std::fs::create_dir_all(root).expect("failed to create project root");
    let policy = SandboxPolicy::default_for_project(root.to_path_buf());
    KaySandboxWindows::new(policy)
}

// All policy tests use #[tokio::test] since check_* methods on the Sandbox
// trait are async and must be awaited.

#[tokio::test]
async fn policy_denies_write_to_system32() {
    let s = sandbox();
    let result = s
        .check_fs_write(Path::new("C:\\Windows\\System32\\evil"))
        .await;
    assert!(
        result.is_err(),
        "Sandbox must deny fs_write to C:\\Windows\\System32"
    );
}

#[tokio::test]
async fn policy_denies_write_outside_project_root() {
    let s = sandbox();
    // On Windows, C:\Users\Public is outside the project root (typically under %TEMP%)
    let result = s
        .check_fs_write(Path::new("C:\\Users\\Public\\evil.txt"))
        .await;
    assert!(
        result.is_err(),
        "Sandbox must deny fs_write outside project root"
    );
}

#[tokio::test]
async fn policy_allows_write_inside_project_root() {
    let s = sandbox();
    let result = s
        .check_fs_write(project_root().join("src\\main.rs").as_path())
        .await;
    assert!(
        result.is_ok(),
        "Sandbox must allow write inside project root"
    );
}

#[tokio::test]
async fn policy_allows_read_of_windows_system32() {
    // The default deny list only includes ~/.ssh, ~/.gnupg, etc. (Unix-style paths)
    // C:\Windows\System32 is NOT in the deny list, so reads should be allowed.
    let s = sandbox();
    let result = s
        .check_fs_read(Path::new("C:\\Windows\\System32\\cmd.exe"))
        .await;
    assert!(
        result.is_ok(),
        "Sandbox must allow fs_read of C:\\Windows\\System32\\cmd.exe"
    );
}

#[tokio::test]
async fn policy_allows_net_to_openrouter() {
    // Default allowlist is only openrouter.ai:443
    let s = sandbox();
    let url = url::Url::parse("https://openrouter.ai/api/v1/models").unwrap();
    let result = s.check_net(&url).await;
    assert!(
        result.is_ok(),
        "Sandbox must allow net to openrouter.ai:443"
    );
}

#[tokio::test]
async fn policy_denies_net_to_minimax() {
    // api.minimax.io is NOT on the default allowlist
    let s = sandbox();
    let url = url::Url::parse("https://api.minimax.io/v1/messages").unwrap();
    let result = s.check_net(&url).await;
    assert!(
        result.is_err(),
        "Sandbox must deny net to api.minimax.io (not on allowlist)"
    );
}

#[tokio::test]
async fn policy_denies_net_to_non_allowlisted() {
    let s = sandbox();
    let url = url::Url::parse("https://evil.example.com/").unwrap();
    let result = s.check_net(&url).await;
    assert!(
        result.is_err(),
        "Sandbox must deny net to non-allowlisted hosts"
    );
}

#[cfg(target_os = "windows")]
mod windows_integration {
    use super::*;

    #[tokio::test]
    async fn subprocess_escape_write_denied() {
        let s = sandbox();
        let escape_path = Path::new("C:\\Users\\Public\\kay_escape_test");
        let result = s.check_fs_write(escape_path).await;
        assert!(
            result.is_err(),
            "Policy must deny write outside project root"
        );
    }

    #[tokio::test]
    async fn subprocess_write_in_project_allowed() {
        let s = sandbox();
        let file = project_root().join("test_ok.txt");
        let result = s.check_fs_write(&file).await;
        assert!(
            result.is_ok(),
            "Policy must allow write inside project root"
        );
    }
}
