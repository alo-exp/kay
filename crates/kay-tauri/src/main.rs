//! Kay desktop application entry point.
//!
//! Initialises Tauri 2.x with the three Phase 9 IPC commands and shared
//! `AppState`. No externalBin sidecar — all crates compile into this binary
//! (Tauri #11992 blocks macOS notarization on sidecars).
//!
//! Specta v2 builder pattern: `tauri_specta::collect_commands!` internally
//! calls `tauri::generate_handler!`, providing both IPC handler registration
//! AND typescript binding export. In debug builds bindings.ts is written.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kay_tauri::state::AppState;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

fn main() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            // Commands must be exported by kay_tauri (not crate-local).
            kay_tauri::commands::start_session,
            kay_tauri::commands::stop_session,
            kay_tauri::commands::get_session_status,
        ]);

    // Debug: export TypeScript bindings so the React frontend can use them.
    // Release: skip generation (bindings are already committed to git).
    #[cfg(debug_assertions)]
    builder
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