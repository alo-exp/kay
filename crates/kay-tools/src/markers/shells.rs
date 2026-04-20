//! Per-OS shell wrap templates (D-03 + D-04).

use super::MarkerContext;

pub fn wrap_unix_sh(_user_cmd: &str, _m: &MarkerContext) -> String {
    todo!("Wave 3 (03-04): ( USER\\n) ; __KAY_EXIT=$? ; printf '\\n__CMDEND_%s_%d__EXITCODE=%d\\n' ...")
}

pub fn wrap_windows_ps(_user_cmd: &str, _m: &MarkerContext) -> String {
    todo!("Wave 3 (03-04): powershell wrap with $LASTEXITCODE")
}
