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
/// `$LASTEXITCODE` in lieu of `$?`.
///
/// Two PowerShell gotchas this wrapper navigates:
/// 1. `$LASTEXITCODE` is `$null` for cmdlet-only pipelines (PowerShell
///    only touches it for native invocations). We pre-seed it to 0 and
///    fall back to 0 if it's still `$null` after the user command.
/// 2. `& { }` creates a child variable scope — assignments inside are
///    invisible to the outer command line. The marker emit must happen
///    INSIDE the block to see `$kay_exit`.
pub fn wrap_windows_ps(user_cmd: &str, m: &MarkerContext) -> String {
    format!(
        "& {{ $LASTEXITCODE = 0 ; {user_cmd} ; $kay_exit = if ($null -eq $LASTEXITCODE) {{ 0 }} else {{ $LASTEXITCODE }} ; Write-Host \"`n__CMDEND_{}_{}__EXITCODE=$kay_exit\" }}",
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
        // Script block must pre-seed LASTEXITCODE, run user cmd, and fall
        // back to 0 when LASTEXITCODE is still $null after a cmdlet-only
        // pipeline (cmdlets never touch LASTEXITCODE).
        assert!(
            wrapped.contains("$LASTEXITCODE = 0"),
            "missing LASTEXITCODE seed: {wrapped}"
        );
        assert!(
            wrapped.contains("Get-Item ."),
            "missing user command: {wrapped}"
        );
        assert!(
            wrapped.contains("if ($null -eq $LASTEXITCODE) { 0 } else { $LASTEXITCODE }"),
            "missing null-fallback: {wrapped}"
        );
        assert!(
            wrapped.contains(&format!(
                "__CMDEND_{}_{}__EXITCODE=$kay_exit",
                m.nonce_hex, m.seq
            )),
            "missing marker template: {wrapped}"
        );
        // Write-Host must be INSIDE the script block so $kay_exit is visible.
        // Trailing `}` closes the block; no outer statements after it.
        assert!(
            wrapped.trim_end().ends_with('}'),
            "Write-Host must live inside the script block so kay_exit is in scope: {wrapped}"
        );
    }

    /// Regression: an outer `Write-Host` sitting after `}` would see
    /// `$kay_exit` as `$null` (child-scope isolation in `& { }`),
    /// producing `EXITCODE=` which `scan_line` rejects as ForgedMarker —
    /// the exact Phase 4 Windows CI failure on commit 468486c.
    #[test]
    fn windows_wrap_has_no_outer_write_host_after_block() {
        let m = mk();
        let wrapped = wrap_windows_ps("echo hi", &m);
        // Find the closing `}` and assert nothing meaningful follows.
        let last_brace = wrapped.rfind('}').expect("must contain closing }");
        let tail = wrapped[last_brace + 1..].trim();
        assert!(
            tail.is_empty(),
            "found content after closing block — Write-Host would run in outer scope and lose $kay_exit: tail={tail:?}"
        );
    }
}
