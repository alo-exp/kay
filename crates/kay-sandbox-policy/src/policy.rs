use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetAllow {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SandboxPolicy {
    pub write_roots: Vec<PathBuf>,
    pub read_deny_list: Vec<PathBuf>,
    pub network_allowlist: Vec<NetAllow>,
}

impl SandboxPolicy {
    pub fn default_for_project(project_root: PathBuf) -> Self {
        let home = dirs_home();
        let read_deny_list = home
            .map(|h| {
                vec![
                    h.join(".aws"),
                    h.join(".ssh"),
                    h.join(".gnupg"),
                    h.join(".gnupg2"),
                ]
            })
            .unwrap_or_default();

        Self {
            write_roots: vec![project_root],
            read_deny_list,
            network_allowlist: vec![NetAllow { host: "openrouter.ai".to_string(), port: 443 }],
        }
    }

    pub fn allows_write(&self, path: &Path) -> bool {
        self.write_roots.iter().any(|root| path.starts_with(root))
    }

    pub fn allows_read(&self, path: &Path) -> bool {
        !self
            .read_deny_list
            .iter()
            .any(|deny| path.starts_with(deny))
    }

    pub fn allows_net(&self, host: &str, port: u16) -> bool {
        self.network_allowlist
            .iter()
            .any(|a| a.host == host && a.port == port)
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_default_for_project() {
        let root = PathBuf::from("/home/user/project");
        let policy = SandboxPolicy::default_for_project(root.clone());
        assert_eq!(policy.write_roots, vec![root]);
        assert!(!policy.network_allowlist.is_empty());
        assert_eq!(policy.network_allowlist[0].host, "openrouter.ai");
        assert_eq!(policy.network_allowlist[0].port, 443);
    }

    #[test]
    fn test_allows_write_inside_root() {
        let root = PathBuf::from("/project");
        let policy = SandboxPolicy::default_for_project(root.clone());
        assert!(policy.allows_write(&root.join("src/main.rs")));
        assert!(!policy.allows_write(Path::new("/tmp/evil")));
    }

    #[test]
    fn test_allows_read_deny_list() {
        let root = PathBuf::from("/project");
        let mut policy = SandboxPolicy::default_for_project(root);
        policy.read_deny_list = vec![PathBuf::from("/home/user/.ssh")];
        assert!(!policy.allows_read(Path::new("/home/user/.ssh/id_rsa")));
        assert!(policy.allows_read(Path::new("/home/user/other_file")));
    }

    #[test]
    fn test_net_allow_matching() {
        let policy = SandboxPolicy::default_for_project(PathBuf::from("/project"));
        assert!(policy.allows_net("openrouter.ai", 443));
        assert!(!policy.allows_net("evil.com", 80));
        assert!(!policy.allows_net("openrouter.ai", 80));
    }

    #[test]
    fn test_policy_serde_roundtrip() {
        let policy = SandboxPolicy::default_for_project(PathBuf::from("/project"));
        let json = serde_json::to_string(&policy).unwrap();
        let decoded: SandboxPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.write_roots, policy.write_roots);
        assert_eq!(
            decoded.network_allowlist.len(),
            policy.network_allowlist.len()
        );
    }
}
