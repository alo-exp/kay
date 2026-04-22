//! Kay desktop application entry point.
//!
//! Initialises Tauri 2.x with the three Phase 9 IPC commands and shared
//! `AppState`. No externalBin sidecar — all crates compile into this binary
//! (Tauri #11992 blocks macOS notarization on sidecars).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kay_tauri::commands::{get_session_status, start_session, stop_session};
use kay_tauri::state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_session,
            stop_session,
            get_session_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
