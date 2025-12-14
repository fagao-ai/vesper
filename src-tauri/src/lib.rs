mod ssh;
mod commands;
mod storage;
mod settings;

use ssh::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConnectionManager::new())
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
