mod ssh;
mod commands;

use ssh::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConnectionManager::new())
        .invoke_handler(tauri::generate_handler![
            // SSH Connection Commands
            commands::create_connection,
            commands::get_connections,
            commands::get_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::connect_ssh,
            commands::disconnect_ssh,
            // SSH Tunnel Commands
            commands::create_tunnel,
            commands::get_tunnels,
            commands::get_tunnels_by_connection,
            commands::delete_tunnel,
            commands::start_tunnel,
            commands::stop_tunnel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
