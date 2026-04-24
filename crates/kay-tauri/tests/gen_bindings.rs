//! Generates TypeScript bindings from tauri-specta.
//!
//! Run with:
//!   cargo test -p kay-tauri --test gen_bindings export_tauri_bindings
//!
//! The generated file is committed at `ui/src/bindings.ts`.
//! CI validates it is in sync via `scripts/check-bindings.sh`.
//!
//! Integration tests (not build.rs) are used because build scripts are compiled
//! separately from the main crate and cannot access `crate::commands::*`.

#[test]
fn export_tauri_bindings() {
    // tauri_specta::Builder uses Typescript (from specta-typescript crate)
    // as the language exporter. collect_commands! internally calls
    // tauri::generate_handler!, so we get both binding generation AND handler
    // registration.
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            kay_tauri::commands::start_session,
            kay_tauri::commands::stop_session,
            kay_tauri::commands::get_session_status,
        ])
        .export(
            specta_typescript::Typescript::default(),
            concat!(env!("CARGO_MANIFEST_DIR"), "/ui/src/bindings.ts"),
        )
        .expect("export tauri-specta bindings");
}