use crate::ssh::{ConnectionManager, SSHConnection, SSHTunnel, AuthMethod, TunnelType, generate_id};
use crate::settings::AppConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConnectionRequest {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTunnelRequest {
    pub name: String,
    pub connection_id: String,
    pub tunnel_type: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub auto_reconnect: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTunnelRequest {
    pub id: String,
    pub name: String,
    pub connection_id: String,
    pub tunnel_type: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub auto_reconnect: bool,
}

// Initialize Data Storage
#[tauri::command]
pub async fn initialize_storage(manager: State<'_, Arc<ConnectionManager>>) -> Result<(), String> {
    eprintln!("initialize_storage command called");
    let result = manager.initialize().await;
    eprintln!("initialize_storage command completed with result: {:?}", result);
    result
}

// SSH Connection Commands
#[tauri::command]
pub async fn create_connection(
    request: CreateConnectionRequest,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<String, String> {
    let auth_method = match request.auth_method.as_str() {
        "password" => AuthMethod::Password,
        "key" => AuthMethod::Key,
        _ => return Err("Invalid auth method".to_string()),
    };

    let connection = SSHConnection {
        id: generate_id(),
        name: request.name,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_method,
        password: request.password,
        key_path: request.key_path,
        status: crate::ssh::ConnectionStatus::Disconnected,
        last_connected: None,
        created_at: std::time::SystemTime::now(),
    };

    manager.add_connection(connection).await
}

#[tauri::command]
pub async fn get_connections(
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<SSHConnection>, String> {
    Ok(manager.get_connections().await)
}

#[tauri::command]
pub async fn get_connection(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Option<SSHConnection>, String> {
    Ok(manager.get_connection(&id).await)
}

#[tauri::command]
pub async fn update_connection(
    request: UpdateConnectionRequest,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    let existing_connection = manager.get_connection(&request.id).await
        .ok_or("Connection not found")?;

    let auth_method = match request.auth_method.as_str() {
        "password" => AuthMethod::Password,
        "key" => AuthMethod::Key,
        _ => return Err("Invalid auth method".to_string()),
    };

    let updated_connection = SSHConnection {
        id: request.id,
        name: request.name,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_method,
        password: request.password,
        key_path: request.key_path,
        status: existing_connection.status,
        last_connected: existing_connection.last_connected,
        created_at: existing_connection.created_at,
    };

    manager.update_connection(updated_connection.id.clone(), updated_connection).await
}

#[tauri::command]
pub async fn delete_connection(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    manager.delete_connection(id).await
}

#[tauri::command]
pub async fn test_connection(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<crate::ssh::ConnectionResult, String> {
    let connection = manager.get_connection(&id).await
        .ok_or("Connection not found")?;

    Ok(manager.test_connection(&connection).await)
}

#[tauri::command]
pub async fn test_connection_data(
    request: CreateConnectionRequest,
) -> Result<crate::ssh::ConnectionResult, String> {
    let auth_method = match request.auth_method.as_str() {
        "password" => AuthMethod::Password,
        "key" => AuthMethod::Key,
        _ => return Err("Invalid auth method".to_string()),
    };

    // 创建临时连接对象用于测试
    let test_connection = SSHConnection {
        id: "test".to_string(), // 临时ID
        name: request.name,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_method,
        password: request.password,
        key_path: request.key_path,
        status: crate::ssh::ConnectionStatus::Disconnected,
        last_connected: None,
        created_at: std::time::SystemTime::now(),
    };

    // 执行连接测试
    Ok(crate::ssh::test_ssh_connection(&test_connection).await)
}

#[tauri::command]
pub async fn connect_ssh(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<crate::ssh::ConnectionResult, String> {
    Ok(manager.connect_ssh(&id).await)
}

#[tauri::command]
pub async fn disconnect_ssh(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<crate::ssh::ConnectionResult, String> {
    Ok(manager.disconnect_ssh(&id).await)
}

// SSH Tunnel Commands
#[tauri::command]
pub async fn create_tunnel(
    request: CreateTunnelRequest,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<String, String> {
    let tunnel_type = match request.tunnel_type.as_str() {
        "local" => TunnelType::Local,
        "remote" => TunnelType::Remote,
        _ => return Err("Invalid tunnel type".to_string()),
    };

    let tunnel = SSHTunnel {
        id: generate_id(),
        name: request.name,
        connection_id: request.connection_id,
        tunnel_type,
        local_port: request.local_port,
        remote_host: request.remote_host,
        remote_port: request.remote_port,
        status: crate::ssh::TunnelStatus::Inactive,
        auto_reconnect: request.auto_reconnect,
    };

    manager.add_tunnel(tunnel).await
}

#[tauri::command]
pub async fn update_tunnel(
    request: UpdateTunnelRequest,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    // Get existing tunnels to find the current status
    let existing_tunnels = manager.get_tunnels().await;
    let existing_tunnel = existing_tunnels.iter()
        .find(|t| t.id == request.id)
        .ok_or("Tunnel not found")?;

    let tunnel_type = match request.tunnel_type.as_str() {
        "local" => TunnelType::Local,
        "remote" => TunnelType::Remote,
        _ => return Err("Invalid tunnel type".to_string()),
    };

    let updated_tunnel = SSHTunnel {
        id: request.id,
        name: request.name,
        connection_id: request.connection_id,
        tunnel_type,
        local_port: request.local_port,
        remote_host: request.remote_host,
        remote_port: request.remote_port,
        status: existing_tunnel.status.clone(), // Preserve the current status
        auto_reconnect: request.auto_reconnect,
    };

    manager.update_tunnel(updated_tunnel.id.clone(), updated_tunnel).await
}

#[tauri::command]
pub async fn get_tunnels(
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<SSHTunnel>, String> {
    Ok(manager.get_tunnels().await)
}

#[tauri::command]
pub async fn get_tunnels_by_connection(
    connection_id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<SSHTunnel>, String> {
    Ok(manager.get_tunnels_by_connection(&connection_id).await)
}

#[tauri::command]
pub async fn delete_tunnel(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    eprintln!("delete_tunnel command received ID: {} (length: {})", id, id.len());
    manager.delete_tunnel(id).await
}

#[tauri::command]
pub async fn stop_tunnel(
    id: String,
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    manager.stop_tunnel(id).await
}


// Settings Commands
#[tauri::command]
pub async fn get_settings() -> Result<AppConfig, String> {
    use crate::storage::DataManager;
    let data_manager = DataManager::new()?;
    data_manager.load_settings().await
}

#[tauri::command]
pub async fn update_settings(settings: AppConfig) -> Result<(), String> {
    use crate::storage::DataManager;
    let data_manager = DataManager::new()?;
    data_manager.save_settings(&settings).await
}

#[tauri::command]
pub async fn reset_settings() -> Result<AppConfig, String> {
    use crate::storage::DataManager;
    let data_manager = DataManager::new()?;
    let default_settings = AppConfig::default();
    data_manager.save_settings(&default_settings).await?;
    Ok(default_settings)
}