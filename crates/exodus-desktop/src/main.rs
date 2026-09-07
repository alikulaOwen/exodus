#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = DesktopState::new().await?;

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            list_operations,
            get_operation,
            verify_operation,
            approve_operation,
            reject_operation,
            ingest_operation,
            get_sdlc_settings,
            save_sdlc_settings,
            get_sdlc_scaffold,
            get_esg_topology
        ])
        .run(tauri::generate_context!())
        .expect("error while running Exodus Mission Control desktop application");

    Ok(())
}
