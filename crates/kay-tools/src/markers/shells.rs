//! Shell-specific command wrapping. Produces a command string that,
//! when executed by the host shell, emits a single marker line on a
//! fresh line of its own AFTER the user command completes. See D-03.

use super::MarkerContext;

/// Wrap a user command for `sh -c` / `bash -c` / `zsh -c`. The subshell
/// guarantees multi-command scripts still propagate their final exit
/// code via `$?`. Leading `\n` before the marker ensures the marker is
/// on its own line even if the user command ends without a newline.
pub fn wrap_unix_sh(user_cmd: &str, m: &MarkerContext) -> String {
    format!(
        "( {user_cmd}\n) ; __KAY_EXIT=$? ; printf '\\n__CMDEND_%s_%d__EXITCODE=%d\\n' '{}' {} \"$__KAY_EXIT\"",
        m.nonce_hex, m.seq
    )
}

/// Wrap a user command for `powershell -NoProfile -Command`. Uses
/// `$LASTEXITCODE` in lieu of `$?` (PowerShell sets LASTEXITCODE only
/// for native invocations; cmdlet-only pipelines leave it untouched —
/// documented limitation).
pub fn wrap_windows_ps(user_cmd: &str, m: &MarkerContext) -> String {
    format!(
        "& {{ {user_cmd} ; $kay_exit = $LASTEXITCODE }} ; Write-Host \"`n__CMDEND_{}_{}__EXITCODE=$kay_exit\"",
        m.nonce_hex, m.seq
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::*;

    fn mk() -> MarkerContext {
        let c = AtomicU64::new(7);
        MarkerContext::new(&c).expect("SysRng must succeed in tests")
    }

    #[test]
    fn unix_wrap_contains_user_cmd_and_printf() {
        let m = mk();
        let wrapped = wrap_unix_sh("echo hi", &m);
        assert!(
            wrapped.contains("( echo hi\n) ;"),
            "missing subshell: {wrapped}"
        );
        assert!(
            wrapped.contains("__KAY_EXIT=$?"),
            "missing exit capture: {wrapped}"
        );
        assert!(
            wrapped.contains(&format!("'{}' {}", m.nonce_hex, m.seq)),
            "missing nonce+seq injection: {wrapped}"
        );
    }

    #[test]
    fn windows_wrap_contains_subexpression_and_lastexitcode() {
        let m = mk();
        let wrapped = wrap_windows_ps("Get-Item .", &m);
        assert!(
            wrapped.contains("& { Get-Item . ; $kay_exit = $LASTEXITCODE }"),
            "missing subexpr: {wrapped}"
        );
        assert!(
            wrapped.contains(&format!(
                "__CMDEND_{}_{}__EXITCODE=$kay_exit",
                m.nonce_hex, m.seq
            )),
            "missing marker template: {wrapped}"
        );
    }
}
