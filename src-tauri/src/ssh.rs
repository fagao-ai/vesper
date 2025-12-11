use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use std::path::Path;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub status: ConnectionStatus,
    pub last_connected: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Password,
    Key,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHTunnel {
    pub id: String,
    pub name: String,
    pub connection_id: String,
    pub tunnel_type: TunnelType,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub status: TunnelStatus,
    pub auto_reconnect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Inactive,
    Active,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
    pub error_code: Option<String>,
}

pub struct ConnectionManager {
    connections: RwLock<HashMap<String, SSHConnection>>,
    tunnels: RwLock<HashMap<String, SSHTunnel>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            tunnels: RwLock::new(HashMap::new()),
        }
    }

    async fn load_from_storage(&self) -> Result<(), String> {
        use crate::storage::DataManager;
        let data_manager = DataManager::new()?;
        let (connections, tunnels) = data_manager.load_connections_and_tunnels().await?;
        *self.connections.write().await = connections;
        *self.tunnels.write().await = tunnels;
        Ok(())
    }

    async fn save_to_storage(&self) -> Result<(), String> {
        use crate::storage::DataManager;
        let data_manager = DataManager::new()?;
        // Clone the data to avoid lifetime issues
        let connections = self.connections.read().await.clone();
        let tunnels = self.tunnels.read().await.clone();
        data_manager.save_connections_and_tunnels(&connections, &tunnels).await
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Load data from storage on startup
        self.load_from_storage().await
    }

    pub async fn add_connection(&self, connection: SSHConnection) -> Result<String, String> {
        let mut connections = self.connections.write().await;
        let id = connection.id.clone();
        connections.insert(id.clone(), connection);
        drop(connections); // Release the lock

        // Save to storage
        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        Ok(id)
    }

    pub async fn get_connections(&self) -> Vec<SSHConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    pub async fn get_connection(&self, id: &str) -> Option<SSHConnection> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    pub async fn update_connection(&self, id: &str, updates: SSHConnection) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if connections.contains_key(id) {
            connections.insert(id.to_string(), updates);
            drop(connections); // Release the lock

            // Save to storage
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn remove_connection(&self, id: &str) -> Result<(), String> {
        // First remove the connection
        {
            let mut connections = self.connections.write().await;
            connections.remove(id);
        } // Release connections lock

        // Then remove related tunnels
        {
            let mut tunnels = self.tunnels.write().await;
            tunnels.retain(|_, tunnel| tunnel.connection_id != id);
        } // Release tunnels lock

        // Save to storage
        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        Ok(())
    }

    pub async fn test_connection(&self, _connection: &SSHConnection) -> ConnectionResult {
        // This is a placeholder for actual SSH connection testing
        // In a real implementation, you would use the ssh2 crate to test the connection

        // Simulate connection attempt
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // For demo purposes, we'll simulate a successful connection
        // In reality, you would implement actual SSH connection logic here
        ConnectionResult {
            success: true,
            message: "Connection successful".to_string(),
            error_code: None,
        }
    }

    pub async fn connect_ssh(&self, id: &str) -> ConnectionResult {
        let mut connections = self.connections.write().await;

        if let Some(connection) = connections.get_mut(id) {
            connection.status = ConnectionStatus::Connecting;

            // Simulate connection process
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // For demo purposes, we'll simulate a successful connection
            connection.status = ConnectionStatus::Connected;
            connection.last_connected = Some(SystemTime::now());
            drop(connections); // Release the lock

            // Save to storage
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            ConnectionResult {
                success: true,
                message: "SSH connection established".to_string(),
                error_code: None,
            }
        } else {
            ConnectionResult {
                success: false,
                message: "Connection not found".to_string(),
                error_code: Some("NOT_FOUND".to_string()),
            }
        }
    }

    pub async fn disconnect_ssh(&self, id: &str) -> ConnectionResult {
        let mut connections = self.connections.write().await;

        if let Some(connection) = connections.get_mut(id) {
            connection.status = ConnectionStatus::Disconnected;
            drop(connections); // Release the lock

            // Save to storage
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            ConnectionResult {
                success: true,
                message: "SSH connection closed".to_string(),
                error_code: None,
            }
        } else {
            ConnectionResult {
                success: false,
                message: "Connection not found".to_string(),
                error_code: Some("NOT_FOUND".to_string()),
            }
        }
    }

    // Tunnel management methods
    pub async fn add_tunnel(&self, tunnel: SSHTunnel) -> Result<String, String> {
        let mut tunnels = self.tunnels.write().await;
        let id = tunnel.id.clone();
        tunnels.insert(id.clone(), tunnel);
        drop(tunnels); // Release the lock

        // Save to storage
        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        Ok(id)
    }

    pub async fn get_tunnels(&self) -> Vec<SSHTunnel> {
        let tunnels = self.tunnels.read().await;
        tunnels.values().cloned().collect()
    }

    pub async fn get_tunnels_by_connection(&self, connection_id: &str) -> Vec<SSHTunnel> {
        let tunnels = self.tunnels.read().await;
        tunnels
            .values()
            .filter(|tunnel| tunnel.connection_id == connection_id)
            .cloned()
            .collect()
    }

    pub async fn remove_tunnel(&self, id: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.write().await;
        tunnels.remove(id).ok_or("Tunnel not found")?;

        // Save to storage
        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        Ok(())
    }

    pub async fn start_tunnel(&self, id: &str) -> ConnectionResult {
        let mut tunnels = self.tunnels.write().await;
        let connections = self.connections.read().await;

        if let Some(tunnel) = tunnels.get_mut(id) {
            // Clone the tunnel name before we potentially drop the lock
            let tunnel_name = tunnel.name.clone();

            // Verify the associated connection is active
            if let Some(connection) = connections.get(&tunnel.connection_id) {
                match connection.status {
                    ConnectionStatus::Connected => {
                        tunnel.status = TunnelStatus::Active;
                        drop(tunnels); // Release the write lock
                        drop(connections); // Release the read lock

                        // Save to storage
                        if let Err(e) = self.save_to_storage().await {
                            eprintln!("Failed to save data: {}", e);
                        }

                        ConnectionResult {
                            success: true,
                            message: format!("Tunnel '{}' started successfully", tunnel_name),
                            error_code: None,
                        }
                    },
                    _ => ConnectionResult {
                        success: false,
                        message: "SSH connection must be active to start tunnel".to_string(),
                        error_code: Some("CONNECTION_NOT_ACTIVE".to_string()),
                    }
                }
            } else {
                ConnectionResult {
                    success: false,
                    message: "Associated SSH connection not found".to_string(),
                    error_code: Some("CONNECTION_NOT_FOUND".to_string()),
                }
            }
        } else {
            ConnectionResult {
                success: false,
                message: "Tunnel not found".to_string(),
                error_code: Some("TUNNEL_NOT_FOUND".to_string()),
            }
        }
    }

    pub async fn stop_tunnel(&self, id: &str) -> ConnectionResult {
        let mut tunnels = self.tunnels.write().await;

        if let Some(tunnel) = tunnels.get_mut(id) {
            // Clone the tunnel name before dropping the lock
            let tunnel_name = tunnel.name.clone();

            tunnel.status = TunnelStatus::Inactive;
            drop(tunnels); // Release the lock

            // Save to storage
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            ConnectionResult {
                success: true,
                message: format!("Tunnel '{}' stopped successfully", tunnel_name),
                error_code: None,
            }
        } else {
            ConnectionResult {
                success: false,
                message: "Tunnel not found".to_string(),
                error_code: Some("TUNNEL_NOT_FOUND".to_string()),
            }
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// 独立的SSH连接测试函数，用于测试连接数据而不需要保存到数据库
pub async fn test_ssh_connection(connection: &SSHConnection) -> ConnectionResult {
    // 设置5秒超时，避免长时间卡住
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::task::spawn_blocking({
            let connection = connection.clone();
            move || {
                test_ssh_connection_sync(&connection)
            }
        })
    ).await {
        Ok(result) => {
            result.unwrap_or_else(|_| {
                ConnectionResult {
                    success: false,
                    message: "SSH测试任务执行失败".to_string(),
                    error_code: Some("TASK_EXECUTION_ERROR".to_string()),
                }
            })
        },
        Err(_) => {
            ConnectionResult {
                success: false,
                message: "SSH连接测试超时".to_string(),
                error_code: Some("TIMEOUT".to_string()),
            }
        }
    }
}

// 同步的SSH连接测试函数
fn test_ssh_connection_sync(connection: &SSHConnection) -> ConnectionResult {
    // 首先验证密钥文件路径（如果使用密钥认证）
    if let AuthMethod::Key = connection.auth_method {
        if let Some(key_path) = &connection.key_path {
            if !Path::new(key_path).exists() {
                return ConnectionResult {
                    success: false,
                    message: format!("密钥文件不存在: {}", key_path),
                    error_code: Some("KEY_FILE_NOT_FOUND".to_string()),
                };
            }
        } else {
            return ConnectionResult {
                success: false,
                message: "密钥认证需要指定密钥文件路径".to_string(),
                error_code: Some("KEY_PATH_MISSING".to_string()),
            };
        }
    }

    // 尝试建立TCP连接
    let tcp_addr = format!("{}:{}", connection.host, connection.port);
    let tcp = match TcpStream::connect(&tcp_addr) {
        Ok(stream) => stream,
        Err(e) => {
            let error_code = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => "CONNECTION_REFUSED",
                std::io::ErrorKind::TimedOut => "CONNECTION_TIMEOUT",
                std::io::ErrorKind::HostUnreachable => "HOST_UNREACHABLE",
                _ => "TCP_CONNECTION_ERROR",
            };

            return ConnectionResult {
                success: false,
                message: format!("无法连接到服务器 {}:{} - {}", connection.host, connection.port, e),
                error_code: Some(error_code.to_string()),
            };
        }
    };

    // 设置TCP超时（缩短为3秒）
    if let Err(e) = tcp.set_read_timeout(Some(std::time::Duration::from_secs(3))) {
        return ConnectionResult {
            success: false,
            message: format!("设置连接超时失败: {}", e),
            error_code: Some("TIMEOUT_SETUP_ERROR".to_string()),
        };
    }

    if let Err(e) = tcp.set_write_timeout(Some(std::time::Duration::from_secs(3))) {
        return ConnectionResult {
            success: false,
            message: format!("设置写入超时失败: {}", e),
            error_code: Some("TIMEOUT_SETUP_ERROR".to_string()),
        };
    }

    // 尝试建立SSH会话
    let mut sess = match ssh2::Session::new() {
        Ok(session) => session,
        Err(e) => {
            return ConnectionResult {
                success: false,
                message: format!("创建SSH会话失败: {}", e),
                error_code: Some("SSH_SESSION_ERROR".to_string()),
            };
        }
    };

    sess.set_tcp_stream(tcp);

    if let Err(e) = sess.handshake() {
        return ConnectionResult {
            success: false,
            message: format!("SSH握手失败: {}", e),
            error_code: Some("SSH_HANDSHAKE_ERROR".to_string()),
        };
    }

    // 尝试用户认证
    let auth_result = match connection.auth_method {
        AuthMethod::Password => {
            if let Some(password) = &connection.password {
                sess.userauth_password(&connection.username, password)
            } else {
                return ConnectionResult {
                    success: false,
                    message: "密码认证需要提供密码".to_string(),
                    error_code: Some("PASSWORD_MISSING".to_string()),
                };
            }
        },
        AuthMethod::Key => {
            if let Some(key_path) = &connection.key_path {
                // 尝试读取私钥文件
                let private_key = match fs::read_to_string(key_path) {
                    Ok(key) => key,
                    Err(e) => {
                        return ConnectionResult {
                            success: false,
                            message: format!("读取私钥文件失败 {}: {}", key_path, e),
                            error_code: Some("KEY_FILE_READ_ERROR".to_string()),
                        };
                    }
                };

                // 尝试使用私钥认证
                sess.userauth_pubkey_memory(&connection.username, None, &private_key, None)
            } else {
                return ConnectionResult {
                    success: false,
                    message: "密钥认证需要提供密钥文件路径".to_string(),
                    error_code: Some("KEY_PATH_MISSING".to_string()),
                };
            }
        }
    };

    match auth_result {
        Ok(_) => {
            ConnectionResult {
                success: true,
                message: "SSH连接测试成功".to_string(),
                error_code: None,
            }
        },
        Err(e) => {
            // 简化错误处理，直接使用错误消息
            let message = format!("SSH认证失败: {}", e);
            let error_code = Some("SSH_AUTH_ERROR".to_string());

            ConnectionResult {
                success: false,
                message,
                error_code,
            }
        }
    }
}

// Helper function to generate UUID
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}