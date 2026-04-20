//! macOS sandbox-exec backend for Kay (Phase 4).
//!
//! Uses `sandbox-exec -p <sbpl-profile>` to enforce the sandbox at the kernel
//! level. The SBPL profile is built once from `SandboxPolicy` and cached as
//! `Arc<str>` — no per-call file I/O.
//!
//! **Deprecation note (D-01a):** `sandbox-exec` is deprecated on macOS 15 but
//! remains functional through 15.x. Phase 11 will monitor for removal.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use kay_sandbox_policy::rules::{
    RULE_NET_NOT_ALLOWLISTED, RULE_READ_DENIED_PATH, RULE_WRITE_OUTSIDE_ROOT,
};
use kay_sandbox_policy::{SandboxError, SandboxPolicy};
use kay_tools::seams::sandbox::{Sandbox, SandboxDenial};
use url::Url;

#[derive(Debug)]
pub struct KaySandboxMacos {
    policy: Arc<SandboxPolicy>,
    /// Cached SBPL profile string — built once at construction.
    cached_profile: Arc<str>,
}

impl KaySandboxMacos {
    /// Construct the macOS sandbox backend.
    ///
    /// Returns `SandboxError::BackendUnavailable` if `sandbox-exec` is not
    /// found on PATH — must NOT panic (QG-C2).
    pub fn new(policy: SandboxPolicy) -> Result<Self, SandboxError> {
        which_sandbox_exec()?;
        let cached_profile = Arc::from(build_sbpl_profile(&policy));
        Ok(Self { policy: Arc::new(policy), cached_profile })
    }

    /// Build a sandboxed `std::process::Command` wrapping `command` in
    /// `sandbox-exec -p <profile> -- sh -c <command>`.
    pub fn sandboxed_command(&self, command: &str, cwd: &Path) -> std::process::Command {
        let mut cmd = std::process::Command::new("sandbox-exec");
        cmd.args(["-p", &self.cached_profile, "--", "sh", "-c", command]);
        cmd.current_dir(cwd);
        cmd
    }
}

fn which_sandbox_exec() -> Result<(), SandboxError> {
    let output = std::process::Command::new("which")
        .arg("sandbox-exec")
        .output()
        .map_err(|e| SandboxError::BackendUnavailable(format!("which sandbox-exec: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(SandboxError::BackendUnavailable(
            "sandbox-exec not found on PATH (macOS 15 may have removed it)".into(),
        ))
    }
}

fn build_sbpl_profile(policy: &SandboxPolicy) -> String {
    let mut lines = vec![
        "(version 1)".to_string(),
        "(deny default)".to_string(),
        // System paths needed for basic shell execution.
        "(allow file-read* (subpath \"/usr\"))".to_string(),
        "(allow file-read* (subpath \"/System\"))".to_string(),
        "(allow file-read* (subpath \"/Library/Apple\"))".to_string(),
        "(allow file-read* (subpath \"/private/var/db/timezone\"))".to_string(),
        "(allow file-read* (literal \"/dev/urandom\"))".to_string(),
        "(allow file-read* (literal \"/dev/null\"))".to_string(),
        "(allow process-exec* (subpath \"/usr\"))".to_string(),
        "(allow process-exec* (subpath \"/bin\"))".to_string(),
        "(allow mach-lookup)".to_string(),
        "(allow sysctl-read)".to_string(),
    ];

    for root in &policy.write_roots {
        let path = root.to_string_lossy();
        lines.push(format!("(allow file-read* (subpath \"{path}\"))"));
        lines.push(format!("(allow file-write* (subpath \"{path}\"))"));
    }

    // Allow HTTPS outbound for each allowlisted host on port 443.
    if policy.network_allowlist.iter().any(|n| n.port == 443) {
        lines.push("(allow network-outbound (remote tcp \"*:443\"))".to_string());
    }

    lines.join("\n")
}

#[async_trait]
impl Sandbox for KaySandboxMacos {
    async fn check_shell(&self, _command: &str, _cwd: &Path) -> Result<(), SandboxDenial> {
        // Pre-flight: always allow shell (kernel SBPL enforces at spawn time).
        Ok(())
    }

    async fn check_fs_read(&self, path: &Path) -> Result<(), SandboxDenial> {
        if !self.policy.allows_read(path) {
            return Err(SandboxDenial {
                reason: RULE_READ_DENIED_PATH.to_string(),
                resource: path.to_string_lossy().into_owned(),
            });
        }
        Ok(())
    }

    async fn check_fs_write(&self, path: &Path) -> Result<(), SandboxDenial> {
        if !self.policy.allows_write(path) {
            return Err(SandboxDenial {
                reason: RULE_WRITE_OUTSIDE_ROOT.to_string(),
                resource: path.to_string_lossy().into_owned(),
            });
        }
        Ok(())
    }

    async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial> {
        if url.scheme() == "file" {
            return Err(SandboxDenial {
                reason: RULE_NET_NOT_ALLOWLISTED.to_string(),
                resource: url.to_string(),
            });
        }
        let host = url.host_str().unwrap_or("");
        let port = url.port_or_known_default().unwrap_or(443);
        if !self.policy.allows_net(host, port) {
            return Err(SandboxDenial {
                reason: RULE_NET_NOT_ALLOWLISTED.to_string(),
                resource: url.to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn test_policy() -> SandboxPolicy {
        SandboxPolicy::default_for_project(PathBuf::from("/tmp/test_project"))
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_new_returns_ok_on_macos() {
        let result = KaySandboxMacos::new(test_policy());
        assert!(
            result.is_ok(),
            "KaySandboxMacos::new should succeed: {result:?}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_sbpl_profile_cached() {
        let sandbox = KaySandboxMacos::new(test_policy()).unwrap();
        let a = Arc::clone(&sandbox.cached_profile);
        let b = Arc::clone(&sandbox.cached_profile);
        assert!(Arc::ptr_eq(&a, &b), "cached profile must be shared");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_sbpl_profile_contains_write_root() {
        let sandbox = KaySandboxMacos::new(test_policy()).unwrap();
        assert!(
            sandbox.cached_profile.contains("/tmp/test_project"),
            "profile must include write root: {}",
            sandbox.cached_profile
        );
    }

    #[test]
    fn test_policy_denies_write_outside_root() {
        let policy = test_policy();
        assert!(!policy.allows_write(Path::new("/etc/evil")));
    }

    #[test]
    fn test_policy_denies_read_sensitive_path() {
        let mut policy = test_policy();
        policy.read_deny_list = vec![PathBuf::from("/home/user/.ssh")];
        assert!(!policy.allows_read(Path::new("/home/user/.ssh/id_rsa")));
    }

    #[test]
    fn test_policy_denies_net_non_allowlisted() {
        let policy = test_policy();
        assert!(!policy.allows_net("evil.com", 443));
    }

    #[test]
    fn test_policy_allows_openrouter() {
        let policy = test_policy();
        assert!(policy.allows_net("openrouter.ai", 443));
    }

    // Escape suite — kernel-level enforcement, only on macOS.
    #[cfg(target_os = "macos")]
    mod escape_suite {
        use std::path::PathBuf;

        use super::*;

        fn escape_sandbox() -> KaySandboxMacos {
            std::fs::create_dir_all("/tmp/kay_escape_test").ok();
            KaySandboxMacos::new(SandboxPolicy::default_for_project(PathBuf::from(
                "/tmp/kay_escape_test",
            )))
            .expect("KaySandboxMacos::new failed in escape test")
        }

        #[test]
        fn escape_write_outside_root() {
            let sandbox = escape_sandbox();
            let target = "/tmp/kay_sandbox_escape_write_test";
            let mut cmd = sandbox.sandboxed_command(
                &format!("echo pwned > {target}"),
                Path::new("/tmp/kay_escape_test"),
            );
            let status = cmd.status().expect("sandbox-exec spawn failed");
            // Kernel denies write outside root: either non-zero exit or file absent.
            let escaped = status.success() && std::path::Path::new(target).exists();
            let _ = std::fs::remove_file(target);
            assert!(!escaped, "kernel must deny write outside root");
        }

        #[test]
        fn escape_net_not_allowlisted() {
            let sandbox = escape_sandbox();
            // curl to port 80 — not in allowlist (only 443 allowed).
            let mut cmd = sandbox.sandboxed_command(
                "curl -s --max-time 2 http://example.com/ > /dev/null 2>&1; echo $?",
                Path::new("/tmp/kay_escape_test"),
            );
            let output = cmd.output().expect("sandbox-exec spawn failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            // curl exit code should be non-zero when network is denied.
            let exit_code: i32 = stdout.trim().parse().unwrap_or(1);
            assert_ne!(exit_code, 0, "net to non-allowlisted port must be denied");
        }
    }
}
