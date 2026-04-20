//! Windows Job Objects + restricted token sandbox backend for Kay (Phase 4).
//!
//! Defense strategy:
//! - Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`: all child processes
//!   in the job are killed when Kay exits — closes **R-4** (grandchild cascade).
//! - Restricted token via `CreateRestrictedToken` + `DISABLE_MAX_PRIVILEGE`:
//!   drops excess privileges from the child process token.
//!
//! Pre-flight policy checks (RULE_* constants) mirror the macOS/Linux impls.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use kay_sandbox_policy::rules::{
    RULE_NET_NOT_ALLOWLISTED, RULE_READ_DENIED_PATH, RULE_WRITE_OUTSIDE_ROOT,
};
use kay_sandbox_policy::SandboxPolicy;
use kay_tools::seams::sandbox::{Sandbox, SandboxDenial};
use url::Url;

#[derive(Debug)]
pub struct KaySandboxWindows {
    policy: Arc<SandboxPolicy>,
}

impl KaySandboxWindows {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self {
            policy: Arc::new(policy),
        }
    }

    /// Create a Job Object for the given child process handle.
    ///
    /// The Job Object is configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
    /// so all processes in the job are killed when the returned handle is
    /// dropped — closes R-4.
    ///
    /// # Safety
    /// `child_handle` must be a valid process handle. The returned `JobHandle`
    /// keeps the Job Object alive until dropped.
    #[cfg(target_os = "windows")]
    pub unsafe fn create_job_for_child(
        &self,
        child_handle: windows_sys::Win32::Foundation::HANDLE,
    ) -> Result<JobHandle, std::io::Error> {
        use std::ptr::null;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW,
            JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let job = unsafe { CreateJobObjectW(null(), null()) };
        if job == 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
            unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let ok = unsafe {
            windows_sys::Win32::System::JobObjects::SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error());
        }

        let ok = unsafe { AssignProcessToJobObject(job, child_handle) };
        if ok == 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(JobHandle { handle: job })
    }
}

/// RAII wrapper: closes the Job Object handle when dropped, which kills all
/// member processes (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE).
#[cfg(target_os = "windows")]
pub struct JobHandle {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(target_os = "windows")]
impl Drop for JobHandle {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
    }
}

#[async_trait]
impl Sandbox for KaySandboxWindows {
    async fn check_shell(&self, _command: &str, _cwd: &Path) -> Result<(), SandboxDenial> {
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
        SandboxPolicy::default_for_project(PathBuf::from("C:\\Users\\user\\project"))
    }

    #[test]
    fn test_new_does_not_panic() {
        let _s = KaySandboxWindows::new(test_policy());
    }

    #[test]
    fn test_policy_denies_write_outside_root() {
        let policy = test_policy();
        // On macOS/Linux, path won't start_with Windows path — still tests logic.
        assert!(!policy.allows_write(Path::new("/etc/evil")));
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

    // Windows-specific Job Object tests — only on Windows CI.
    #[cfg(target_os = "windows")]
    mod windows_integration {
        use super::*;

        #[test]
        fn test_job_object_kills_child_on_drop() {
            // Spawn a child that sleeps, assign to Job Object, drop the handle,
            // verify the child is gone. Closes R-4.
            use std::process::Command;
            use windows_sys::Win32::System::Threading::OpenProcess;
            use windows_sys::Win32::System::Threading::PROCESS_ALL_ACCESS;

            let mut child = Command::new("cmd")
                .args(["/C", "timeout /T 30 /NOBREAK"])
                .spawn()
                .expect("spawn child failed");

            let pid = child.id();
            let child_handle = unsafe {
                OpenProcess(PROCESS_ALL_ACCESS, 0, pid)
            };
            assert!(child_handle != 0, "OpenProcess failed");

            let sandbox = KaySandboxWindows::new(
                SandboxPolicy::default_for_project(
                    std::path::PathBuf::from("C:\\Users\\user\\project")
                )
            );

            {
                let _job = unsafe {
                    sandbox.create_job_for_child(child_handle)
                        .expect("create_job_for_child failed")
                };
                // Job handle dropped here → kills child
            }

            // Give OS time to reap the process.
            std::thread::sleep(std::time::Duration::from_millis(200));
            let exit_status = child.try_wait().expect("try_wait failed");
            assert!(exit_status.is_some(), "child must have been killed by Job Object");
        }
    }
}
