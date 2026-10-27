//! ztk as a desktop app. Bootstrap only: one window, one command. The plan is
//! to start ztk's REST API inside the app and point the window at it; see
//! `desktop/README.md`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// The app's version, so the page can show that the shell talks to Rust.
#[tauri::command]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![version])
        .run(tauri::generate_context!())
        .expect("ztk desktop failed to start");
}
