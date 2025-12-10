use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    pub async fn add_connection(&self, connection: SSHConnection) -> Result<String, String> {
        let mut connections = self.connections.write().await;
        let id = connection.id.clone();
        connections.insert(id.clone(), connection);
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
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn remove_connection(&self, id: &str) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        let mut tunnels = self.tunnels.write().await;

        connections.remove(id).ok_or("Connection not found")?;

        // Remove associated tunnels
        tunnels.retain(|_, tunnel| tunnel.connection_id != id);

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
        Ok(())
    }

    pub async fn start_tunnel(&self, id: &str) -> ConnectionResult {
        let mut tunnels = self.tunnels.write().await;
        let connections = self.connections.read().await;

        if let Some(tunnel) = tunnels.get_mut(id) {
            // Verify the associated connection is active
            if let Some(connection) = connections.get(&tunnel.connection_id) {
                match connection.status {
                    ConnectionStatus::Connected => {
                        tunnel.status = TunnelStatus::Active;

                        // In a real implementation, you would start the actual SSH tunnel here
                        // For example, using ssh2 crate to establish the tunnel

                        ConnectionResult {
                            success: true,
                            message: format!("Tunnel '{}' started successfully", tunnel.name),
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
            tunnel.status = TunnelStatus::Inactive;

            // In a real implementation, you would stop the actual SSH tunnel here
            // Close the socket connection, terminate the background task, etc.

            ConnectionResult {
                success: true,
                message: format!("Tunnel '{}' stopped successfully", tunnel.name),
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

// Helper function to generate UUID
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}