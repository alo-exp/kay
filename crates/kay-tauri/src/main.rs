//! Kay desktop application entry point.
//!
//! Initialises Tauri 2.x with the three Phase 9 IPC commands and shared
//! `AppState`. No externalBin sidecar — all crates compile into this binary
//! (Tauri #11992 blocks macOS notarization on sidecars).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kay_tauri::{commands::specta_builder, state::AppState};
use specta_typescript::Typescript;

fn main() {
    let builder = specta_builder::<tauri::Wry>();

    // Debug: export TypeScript bindings so the React frontend can use them.
    // Release: skip generation (bindings are already committed to git).
    #[cfg(debug_assertions)]
    builder
        .clone()
        .export(Typescript::default(), concat!(env!("CARGO_MANIFEST_DIR"), "/ui/src/bindings.ts"))
        .expect("failed to export tauri-specta bindings");

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}