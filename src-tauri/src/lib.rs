mod commands;
mod settings;
mod ssh;
mod storage;
// mod tray; // TODO: Re-enable when Tauri v2 tray API stabilizes

use ssh::ConnectionManager;
use std::sync::Arc;
use tauri::Manager;
use tauri::Listener;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create shared ConnectionManager
    let connection_manager = Arc::new(ConnectionManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::clone(&connection_manager))
        .setup(|app| {
            let manager = app.state::<Arc<ConnectionManager>>().inner().clone();
            tauri::async_runtime::spawn(async move {
                manager.start_health_monitoring().await;
            });

            // TODO: Initialize system tray when API stabilizes
            // tray::create_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Data Storage
            commands::initialize_storage,
            // SSH Connection Commands
            commands::create_connection,
            commands::get_connections,
            commands::get_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::test_connection_data,
            commands::connect_ssh,
            commands::disconnect_ssh,
            // SSH Tunnel Commands
            commands::create_tunnel,
            commands::update_tunnel,
            commands::get_tunnels,
            commands::get_tunnels_by_connection,
            commands::stop_tunnel,
            commands::delete_tunnel,
            // Settings Commands
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
