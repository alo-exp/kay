//! KIRA marker protocol (SHELL-01 / SHELL-05 / D-03).

pub mod shells;

pub struct MarkerContext {
    pub nonce_hex: String,
    pub seq: u64,
    pub line_prefix: String,
}

impl MarkerContext {
    pub fn new(_counter: &std::sync::atomic::AtomicU64) -> Self {
        todo!("Wave 3 (03-04): OsRng 128-bit nonce, fetch_add seq, build line_prefix")
    }
}

pub enum ScanResult {
    NotMarker,
    Marker { exit_code: i32 },
    ForgedMarker,
}

pub fn scan_line(_line: &str, _m: &MarkerContext) -> ScanResult {
    todo!("Wave 3 (03-04): starts_with __CMDEND_ → subtle::ConstantTimeEq prefix → parse EXITCODE=N")
}
