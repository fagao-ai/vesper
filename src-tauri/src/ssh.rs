use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::sync::{oneshot, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{interval, timeout, Duration};
use uuid::Uuid;

use async_ssh2_lite::{AsyncListener, AsyncSession, SessionConfiguration, TokioTcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
    pub status: ConnectionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_connected: Option<SystemTime>,
    #[serde(default = "default_created_at")]
    pub created_at: SystemTime,
}

fn default_created_at() -> SystemTime {
    SystemTime::now()
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
    pub connection_id: String,
    pub name: String,
    pub tunnel_type: TunnelType,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub status: TunnelStatus,
    pub auto_reconnect: bool,
}

pub struct ActiveTunnel {
    pub tunnel: SSHTunnel,
    shutdown_tx: Option<oneshot::Sender<TunnelControl>>,
    task_handle: JoinHandle<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Local,
    Remote,
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

#[derive(Debug, Clone)]
enum TunnelControl {
    Stop,
    ConnectionLost(String),
}

#[derive(Debug)]
enum TunnelExitReason {
    Stopped,
    ConnectionLost(String),
    TunnelError(String),
}

const HEALTH_CHECK_INTERVAL_SECS: u64 = 60;
const SSH_KEEPALIVE_INTERVAL_SECS: u64 = 30;
const SSH_KEEPALIVE_FAILURE_THRESHOLD: u8 = 3;
const TUNNEL_STOP_TIMEOUT_SECS: u64 = 5;
const RECONNECT_DELAY_SECS: u64 = 5;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, SSHConnection>>>,
    tunnels: Arc<RwLock<HashMap<String, SSHTunnel>>>,
    ssh_sessions: Arc<RwLock<HashMap<String, Arc<AsyncSession<TokioTcpStream>>>>>,
    active_tunnels: Arc<RwLock<HashMap<String, ActiveTunnel>>>,
    reconnecting_connections: Arc<RwLock<HashSet<String>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            tunnels: Arc::new(RwLock::new(HashMap::new())),
            ssh_sessions: Arc::new(RwLock::new(HashMap::new())),
            active_tunnels: Arc::new(RwLock::new(HashMap::new())),
            reconnecting_connections: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    // Start health monitoring task
    pub async fn start_health_monitoring(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS));
            loop {
                interval.tick().await;

                // Get list of connected connections
                let connections = manager.connections.read().await;
                let connected_ids: Vec<String> = connections
                    .iter()
                    .filter(|(_, conn)| matches!(conn.status, ConnectionStatus::Connected))
                    .map(|(id, _)| id.clone())
                    .collect();
                drop(connections);

                // Check health of each connected connection
                for id in connected_ids {
                    manager.check_connection_health(&id).await;
                }
            }
        });
    }

    pub async fn initialize(&self) -> Result<(), String> {
        self.load_from_storage().await
    }

    async fn load_from_storage(&self) -> Result<(), String> {
        use crate::storage::DataManager;
        let data_manager = DataManager::new()?;

        let (connections, tunnels) = data_manager.load_connections_and_tunnels().await?;

        let mut connections_map = self.connections.write().await;
        *connections_map = connections;

        let mut tunnels_map = self.tunnels.write().await;
        *tunnels_map = tunnels;

        Ok(())
    }

    async fn save_to_storage(&self) -> Result<(), String> {
        #[cfg(test)]
        {
            Ok(())
        }

        #[cfg(not(test))]
        {
            use crate::storage::DataManager;

            let data_manager = DataManager::new()?;
            let connections = self.connections.read().await;
            let tunnels = self.tunnels.read().await;

            data_manager
                .save_connections_and_tunnels(&*connections, &*tunnels)
                .await?;

            Ok(())
        }
    }

    pub async fn add_connection(&self, connection: SSHConnection) -> Result<String, String> {
        let id = generate_id();
        let mut connection = connection;
        connection.id = id.clone();
        connection.created_at = SystemTime::now();
        connection.status = ConnectionStatus::Disconnected;

        let mut connections = self.connections.write().await;
        connections.insert(id.clone(), connection);
        drop(connections);

        self.save_to_storage().await?;

        Ok(id)
    }

    pub async fn update_connection(
        &self,
        id: String,
        updates: SSHConnection,
    ) -> Result<(), String> {
        let mut connections = self.connections.write().await;

        if let Some(connection) = connections.get_mut(&id) {
            connection.name = updates.name;
            connection.host = updates.host;
            connection.port = updates.port;
            connection.username = updates.username;
            connection.auth_method = updates.auth_method;
            connection.password = updates.password;
            connection.key_path = updates.key_path;

            drop(connections);
            self.save_to_storage().await?;
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn delete_connection(&self, id: String) -> Result<(), String> {
        self.stop_tunnels_for_connection(&id, TunnelControl::Stop)
            .await;
        self.close_ssh_session(&id, "Connection deleted").await;

        {
            let mut connections = self.connections.write().await;
            connections.remove(&id);
        }

        {
            let mut tunnels = self.tunnels.write().await;
            tunnels.retain(|_, tunnel| tunnel.connection_id != id);
        }

        self.save_to_storage().await?;
        Ok(())
    }

    pub async fn get_connections(&self) -> Vec<SSHConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    pub async fn get_connection(&self, id: &str) -> Option<SSHConnection> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    pub async fn test_connection(&self, connection: &SSHConnection) -> ConnectionResult {
        test_ssh_connection(connection).await
    }

    pub async fn connect_ssh(&self, id: &str) -> ConnectionResult {
        let connect_result = self.ensure_ssh_session(id).await;
        if !connect_result.success {
            return connect_result;
        }

        let tunnel_start_error = match self.start_all_tunnels_for_connection(id).await {
            Ok(()) => None,
            Err(error) => {
                eprintln!("Failed to start tunnels: {}", error);
                Some(error)
            }
        };

        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        if let Some(error) = tunnel_start_error {
            ConnectionResult {
                success: true,
                message: format!(
                    "SSH connection established, but some tunnels failed to start: {}",
                    error
                ),
                error_code: Some("TUNNEL_START_FAILED".to_string()),
            }
        } else {
            ConnectionResult {
                success: true,
                message: "SSH connection established".to_string(),
                error_code: None,
            }
        }
    }

    pub async fn start_tunnel(&self, id: &str) -> ConnectionResult {
        let tunnel = {
            let tunnels = self.tunnels.read().await;
            tunnels.get(id).cloned()
        };

        let Some(tunnel) = tunnel else {
            return ConnectionResult {
                success: false,
                message: "Tunnel not found".to_string(),
                error_code: Some("NOT_FOUND".to_string()),
            };
        };

        let connect_result = self.ensure_ssh_session(&tunnel.connection_id).await;
        if !connect_result.success {
            return connect_result;
        }

        match self.start_tunnels_by_ids(&tunnel.connection_id, &[tunnel.id.clone()]).await {
            Ok(()) => {
                if let Err(e) = self.save_to_storage().await {
                    eprintln!("Failed to save data: {}", e);
                }

                ConnectionResult {
                    success: true,
                    message: format!("Tunnel {} started", tunnel.name),
                    error_code: None,
                }
            }
            Err(error) => {
                self.set_tunnel_status(&tunnel.id, TunnelStatus::Error).await;
                if let Err(e) = self.save_to_storage().await {
                    eprintln!("Failed to save data: {}", e);
                }

                ConnectionResult {
                    success: false,
                    message: error,
                    error_code: Some("TUNNEL_START_FAILED".to_string()),
                }
            }
        }
    }

    async fn ensure_ssh_session(&self, id: &str) -> ConnectionResult {
        let connection = {
            let connections = self.connections.read().await;
            connections.get(id).cloned()
        };

        let Some(connection) = connection else {
            return ConnectionResult {
                success: false,
                message: "Connection not found".to_string(),
                error_code: Some("NOT_FOUND".to_string()),
            };
        };

        let existing_session = {
            let sessions = self.ssh_sessions.read().await;
            sessions.get(id).cloned()
        };

        if existing_session.is_some() {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.status = ConnectionStatus::Connected;
                if conn.last_connected.is_none() {
                    conn.last_connected = Some(SystemTime::now());
                }
            }

            return ConnectionResult {
                success: true,
                message: "SSH connection already established".to_string(),
                error_code: None,
            };
        }

        {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.status = ConnectionStatus::Connecting;
            }
        }

        let test_result = test_ssh_connection(&connection).await;
        if !test_result.success {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.status = ConnectionStatus::Error;
            }
            return test_result;
        }

        match establish_ssh_session(&connection).await {
            Ok(session) => {
                self.close_ssh_session(id, "Replacing existing SSH session")
                    .await;

                let mut sessions = self.ssh_sessions.write().await;
                sessions.insert(id.to_string(), Arc::new(session));
                drop(sessions);

                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(id) {
                    conn.status = ConnectionStatus::Connected;
                    conn.last_connected = Some(SystemTime::now());
                }

                ConnectionResult {
                    success: true,
                    message: "SSH connection established".to_string(),
                    error_code: None,
                }
            }
            Err(e) => {
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(id) {
                    conn.status = ConnectionStatus::Error;
                }

                ConnectionResult {
                    success: false,
                    message: format!("Failed to establish SSH connection: {}", e),
                    error_code: Some("CONNECTION_FAILED".to_string()),
                }
            }
        }
    }

    pub async fn disconnect_ssh(&self, id: &str) -> ConnectionResult {
        let connection_exists = {
            let connections = self.connections.read().await;
            connections.contains_key(id)
        };

        if !connection_exists {
            return ConnectionResult {
                success: false,
                message: "Connection not found".to_string(),
                error_code: Some("NOT_FOUND".to_string()),
            };
        } else {
            self.stop_tunnels_for_connection(id, TunnelControl::Stop)
                .await;
            self.close_ssh_session(id, "User disconnected SSH session")
                .await;

            {
                let mut connections = self.connections.write().await;
                if let Some(connection) = connections.get_mut(id) {
                    connection.status = ConnectionStatus::Disconnected;
                }
            }

            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            ConnectionResult {
                success: true,
                message: "SSH connection and all tunnels closed gracefully".to_string(),
                error_code: None,
            }
        }
    }

    // Check if a connection is still alive and attempt reconnection if needed
    pub async fn check_connection_health(&self, id: &str) {
        let session = {
            let sessions = self.ssh_sessions.read().await;
            sessions.get(id).cloned()
        };

        let Some(session) = session else {
            let is_connected = {
                let connections = self.connections.read().await;
                connections
                    .get(id)
                    .map(|connection| matches!(connection.status, ConnectionStatus::Connected))
                    .unwrap_or(false)
            };

            if is_connected {
                self.handle_connection_failure(
                    id,
                    format!(
                        "Connection {} is marked connected but has no SSH session",
                        id
                    ),
                )
                .await;
            }
            return;
        };

        if let Err(err) = session.keepalive_send().await {
            let reason = format!("SSH keepalive failed for connection {}: {}", id, err);
            eprintln!("{}", reason);
            self.handle_connection_failure(id, reason).await;
        }
    }

    pub async fn add_tunnel(&self, tunnel: SSHTunnel) -> Result<String, String> {
        let id = generate_id();
        let mut tunnel = tunnel;
        tunnel.id = id.clone();
        tunnel.status = TunnelStatus::Inactive;

        let mut tunnels = self.tunnels.write().await;
        tunnels.insert(id.clone(), tunnel);
        drop(tunnels);

        self.save_to_storage().await?;

        Ok(id)
    }

    pub async fn update_tunnel(&self, id: String, updates: SSHTunnel) -> Result<(), String> {
        let mut tunnels = self.tunnels.write().await;

        if let Some(tunnel) = tunnels.get_mut(&id) {
            tunnel.name = updates.name;
            tunnel.tunnel_type = updates.tunnel_type;
            tunnel.local_port = updates.local_port;
            tunnel.remote_host = updates.remote_host;
            tunnel.remote_port = updates.remote_port;
            tunnel.auto_reconnect = updates.auto_reconnect;

            drop(tunnels);
            self.save_to_storage().await?;
            Ok(())
        } else {
            Err("Tunnel not found".to_string())
        }
    }

    pub async fn delete_tunnel(&self, id: String) -> Result<(), String> {
        self.stop_active_tunnel(&id, TunnelControl::Stop).await;

        let mut tunnels = self.tunnels.write().await;
        tunnels.remove(&id);
        drop(tunnels);

        self.save_to_storage().await?;
        Ok(())
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

    // Stop a tunnel without deleting it
    pub async fn stop_tunnel(&self, id: String) -> Result<(), String> {
        let stopped = self.stop_active_tunnel(&id, TunnelControl::Stop).await;
        if !stopped {
            self.set_tunnel_status(&id, TunnelStatus::Inactive).await;
        }

        self.save_to_storage().await?;
        Ok(())
    }

    // Start all tunnels for a given connection
    async fn start_all_tunnels_for_connection(&self, connection_id: &str) -> Result<(), String> {
        let tunnel_ids: Vec<String> = self
            .get_tunnels_by_connection(connection_id)
            .await
            .into_iter()
            .map(|tunnel| tunnel.id)
            .collect();

        self.start_tunnels_by_ids(connection_id, &tunnel_ids).await
    }

    async fn start_tunnels_by_ids(
        &self,
        connection_id: &str,
        tunnel_ids: &[String],
    ) -> Result<(), String> {
        let id_set: HashSet<&str> = tunnel_ids.iter().map(String::as_str).collect();
        let connection_tunnels: Vec<SSHTunnel> = self
            .get_tunnels_by_connection(connection_id)
            .await
            .into_iter()
            .filter(|tunnel| id_set.contains(tunnel.id.as_str()))
            .collect();

        let session = {
            let sessions = self.ssh_sessions.read().await;
            sessions.get(connection_id).cloned()
        };

        let Some(session) = session else {
            for tunnel in &connection_tunnels {
                self.set_tunnel_status(&tunnel.id, TunnelStatus::Error)
                    .await;
            }
            let error = format!(
                "No active SSH session found for connection {}",
                connection_id
            );
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }
            return Err(error);
        };

        self.start_tunnels_with_session(connection_tunnels, session).await
    }

    async fn start_tunnels_with_session(
        &self,
        tunnels_to_start: Vec<SSHTunnel>,
        session: Arc<AsyncSession<TokioTcpStream>>,
    ) -> Result<(), String> {
        let mut first_error = None;

        for tunnel in tunnels_to_start {
            self.stop_active_tunnel(&tunnel.id, TunnelControl::Stop)
                .await;

            match self
                .start_tunnel_with_session(tunnel.clone(), session.clone())
                .await
            {
                Ok(active_tunnel) => {
                    let tunnel_id = tunnel.id.clone();
                    let mut active_tunnels = self.active_tunnels.write().await;
                    active_tunnels.insert(tunnel_id.clone(), active_tunnel);
                    drop(active_tunnels);
                    self.set_tunnel_status(&tunnel_id, TunnelStatus::Active)
                        .await;
                }
                Err(err) => {
                    eprintln!("Failed to start tunnel {}: {}", tunnel.id, err);
                    self.set_tunnel_status(&tunnel.id, TunnelStatus::Error)
                        .await;
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if let Err(e) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", e);
        }

        if let Some(err) = first_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    async fn start_tunnel_with_session(
        &self,
        tunnel: SSHTunnel,
        session: Arc<AsyncSession<TokioTcpStream>>,
    ) -> Result<ActiveTunnel, String> {
        match tunnel.tunnel_type {
            TunnelType::Local => start_local_forwarding(self.clone(), tunnel, session).await,
            TunnelType::Remote => start_remote_forwarding(self.clone(), tunnel, session).await,
        }
    }

    async fn set_tunnel_status(&self, id: &str, status: TunnelStatus) {
        let mut tunnels = self.tunnels.write().await;
        if let Some(tunnel) = tunnels.get_mut(id) {
            tunnel.status = status;
        }
    }

    async fn close_ssh_session(&self, id: &str, description: &str) {
        let session = {
            let mut sessions = self.ssh_sessions.write().await;
            sessions.remove(id)
        };

        if let Some(session) = session {
            if let Err(err) = session.disconnect(None, description, None).await {
                eprintln!("Failed to disconnect SSH session {} cleanly: {}", id, err);
            }
        }
    }

    async fn stop_tunnels_for_connection(&self, connection_id: &str, signal: TunnelControl) {
        let tunnel_ids: Vec<String> = {
            let active_tunnels = self.active_tunnels.read().await;
            active_tunnels
                .iter()
                .filter(|(_, tunnel)| tunnel.tunnel.connection_id == connection_id)
                .map(|(tunnel_id, _)| tunnel_id.clone())
                .collect()
        };

        for tunnel_id in tunnel_ids {
            self.stop_active_tunnel(&tunnel_id, signal.clone()).await;
        }
    }

    async fn stop_active_tunnel(&self, tunnel_id: &str, signal: TunnelControl) -> bool {
        let active_tunnel = {
            let mut active_tunnels = self.active_tunnels.write().await;
            active_tunnels.remove(tunnel_id)
        };

        let Some(mut active_tunnel) = active_tunnel else {
            return false;
        };

        println!(
            "Stopping {:?} tunnel: {}",
            active_tunnel.tunnel.tunnel_type, tunnel_id
        );

        if let Some(shutdown_tx) = active_tunnel.shutdown_tx.take() {
            let _ = shutdown_tx.send(signal);
        }

        let mut task_handle = active_tunnel.task_handle;
        match timeout(
            Duration::from_secs(TUNNEL_STOP_TIMEOUT_SECS),
            &mut task_handle,
        )
        .await
        {
            Ok(join_result) => {
                if let Err(err) = join_result {
                    if !err.is_cancelled() {
                        eprintln!("Tunnel task {} exited with error: {}", tunnel_id, err);
                    }
                }
            }
            Err(_) => {
                eprintln!(
                    "Tunnel {} did not exit in time, aborting the task",
                    tunnel_id
                );
                task_handle.abort();
                let _ = task_handle.await;
            }
        }

        true
    }

    async fn handle_tunnel_runtime_exit(&self, tunnel: SSHTunnel, exit_reason: TunnelExitReason) {
        {
            let mut active_tunnels = self.active_tunnels.write().await;
            active_tunnels.remove(&tunnel.id);
        }

        match exit_reason {
            TunnelExitReason::Stopped => {
                self.set_tunnel_status(&tunnel.id, TunnelStatus::Inactive)
                    .await;
            }
            TunnelExitReason::TunnelError(message) => {
                eprintln!("Tunnel {} exited with an error: {}", tunnel.id, message);
                self.set_tunnel_status(&tunnel.id, TunnelStatus::Error)
                    .await;
            }
            TunnelExitReason::ConnectionLost(message) => {
                eprintln!(
                    "Tunnel {} detected SSH session loss: {}",
                    tunnel.id, message
                );
                self.set_tunnel_status(&tunnel.id, TunnelStatus::Error)
                    .await;
                self.handle_connection_failure(&tunnel.connection_id, message)
                    .await;
            }
        }

        if let Err(err) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", err);
        }
    }

    async fn handle_connection_failure(&self, id: &str, reason: String) {
        let restart_tunnel_ids: Vec<String> = {
            let active_tunnels = self.active_tunnels.read().await;
            active_tunnels
                .values()
                .filter(|active_tunnel| {
                    active_tunnel.tunnel.connection_id == id && active_tunnel.tunnel.auto_reconnect
                })
                .map(|active_tunnel| active_tunnel.tunnel.id.clone())
                .collect()
        };

        self.stop_tunnels_for_connection(id, TunnelControl::ConnectionLost(reason.clone()))
            .await;
        self.close_ssh_session(id, "SSH session lost").await;

        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(id) {
                connection.status = ConnectionStatus::Error;
            } else {
                return;
            }
        }

        if let Err(err) = self.save_to_storage().await {
            eprintln!("Failed to save data: {}", err);
        }

        if !restart_tunnel_ids.is_empty() {
            self.spawn_connection_reconnect(id.to_string(), reason, restart_tunnel_ids);
        }
    }

    fn spawn_connection_reconnect(
        &self,
        id: String,
        reason: String,
        restart_tunnel_ids: Vec<String>,
    ) {
        let manager = self.clone();
        tokio::spawn(async move {
            let should_reconnect = {
                let mut reconnecting_connections = manager.reconnecting_connections.write().await;
                reconnecting_connections.insert(id.clone())
            };

            if !should_reconnect {
                eprintln!("Reconnect for connection {} is already in progress", id);
                return;
            }

            eprintln!(
                "Attempting to reconnect connection {} after failure: {}",
                id, reason
            );

            {
                let mut connections = manager.connections.write().await;
                if let Some(connection) = connections.get_mut(&id) {
                    connection.status = ConnectionStatus::Connecting;
                } else {
                    manager.reconnecting_connections.write().await.remove(&id);
                    return;
                }
            }

            if let Err(err) = manager.save_to_storage().await {
                eprintln!("Failed to save data: {}", err);
            }

            tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;

            let reconnect_result = manager.ensure_ssh_session(&id).await;
            if !reconnect_result.success {
                eprintln!(
                    "Failed to reconnect connection {}: {}",
                    id, reconnect_result.message
                );
                let mut connections = manager.connections.write().await;
                if let Some(connection) = connections.get_mut(&id) {
                    connection.status = ConnectionStatus::Error;
                }
            } else if let Err(error) = manager.start_tunnels_by_ids(&id, &restart_tunnel_ids).await {
                eprintln!(
                    "SSH reconnected for connection {}, but failed to restart tunnels: {}",
                    id, error
                );
            } else {
                eprintln!("Successfully reconnected connection {}", id);
            }

            if let Err(err) = manager.save_to_storage().await {
                eprintln!("Failed to save data: {}", err);
            }

            manager.reconnecting_connections.write().await.remove(&id);
        });
    }
}

// Establish a real SSH session
async fn establish_ssh_session(
    connection: &SSHConnection,
) -> Result<AsyncSession<TokioTcpStream>, String> {
    let connection = connection.clone();

    // Try to establish TCP connection
    let tcp_addr = format!("{}:{}", connection.host, connection.port);
    let tcp = match TcpStream::connect(&tcp_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!("TCP connection failed {}: {}", tcp_addr, e));
        }
    };

    let mut session = match AsyncSession::new(tcp, Some(build_session_configuration())) {
        Ok(session) => session,
        Err(e) => {
            return Err(format!("Failed to create SSH session: {}", e));
        }
    };

    // Perform SSH handshake
    if let Err(e) = session.handshake().await {
        return Err(format!("SSH handshake failed: {}", e));
    }

    // Try user authentication
    let auth_result = match connection.auth_method {
        AuthMethod::Password => {
            if let Some(password) = &connection.password {
                session
                    .userauth_password(&connection.username, password)
                    .await
            } else {
                return Err("Password authentication requires a password".to_string());
            }
        }
        AuthMethod::Key => {
            if let Some(key_path) = &connection.key_path {
                // Check if the key file exists
                if !Path::new(key_path).exists() {
                    return Err(format!("Private key file not found: {}", key_path));
                }

                // Try to authenticate using private key file
                session
                    .userauth_pubkey_file(&connection.username, None, Path::new(key_path), None)
                    .await
            } else {
                return Err("Key authentication requires a key file path".to_string());
            }
        }
    };

    if let Err(e) = auth_result {
        return Err(format!("SSH authentication failed: {}", e));
    }

    // Verify authentication
    if !session.authenticated() {
        return Err("SSH authentication failed".to_string());
    }

    Ok(session)
}

fn build_session_configuration() -> SessionConfiguration {
    let mut configuration = SessionConfiguration::new();
    configuration.set_keepalive(true, SSH_KEEPALIVE_INTERVAL_SECS as u32);
    configuration
}

fn create_local_tunnel_listener(port: u16) -> std::io::Result<TcpListener> {
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(false)?;
    socket.bind(std::net::SocketAddr::from(([0, 0, 0, 0], port)))?;
    socket.listen(1024)
}

async fn start_local_forwarding(
    manager: ConnectionManager,
    tunnel: SSHTunnel,
    session: Arc<AsyncSession<TokioTcpStream>>,
) -> Result<ActiveTunnel, String> {
    println!(
        "Creating SSH tunnel: {} -> {}:{} (tunnel: {})",
        tunnel.name, tunnel.remote_host, tunnel.remote_port, tunnel.name
    );

    let listener = match create_local_tunnel_listener(tunnel.local_port) {
        Ok(listener) => listener,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                return Err(format!(
                    "Local port {} is already in use",
                    tunnel.local_port
                ));
            }
            return Err(format!(
                "Failed to bind local tunnel port {}: {}",
                tunnel.local_port, e
            ));
        }
    };

    let local_addr = listener.local_addr().unwrap();
    println!("Local tunnel listening on {}", local_addr);

    let tunnel_for_task = tunnel.clone();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        run_local_forwarding_loop(manager, tunnel_for_task, listener, session, shutdown_rx).await;
    });

    println!(
        "SSH tunnel created successfully for {}: {}",
        tunnel.name, tunnel.id
    );

    Ok(ActiveTunnel {
        tunnel,
        shutdown_tx: Some(shutdown_tx),
        task_handle: handle,
    })
}

// Handle a single local forwarding connection
async fn handle_local_connection(
    session: Arc<AsyncSession<TokioTcpStream>>,
    local_stream: &mut TcpStream,
    remote_host: &str,
    remote_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create SSH channel to remote host
    let mut channel = session
        .channel_direct_tcpip(remote_host, remote_port, None)
        .await?;

    // Copy data bidirectionally
    if let Err(err) = tokio::io::copy_bidirectional(&mut channel, local_stream).await {
        eprintln!(
            "Copying data between local stream and SSH tunnel failed: {:?}",
            err
        );
    }

    Ok(())
}

async fn start_remote_forwarding(
    manager: ConnectionManager,
    tunnel: SSHTunnel,
    session: Arc<AsyncSession<TokioTcpStream>>,
) -> Result<ActiveTunnel, String> {
    let local_addr = format!("127.0.0.1:{}", tunnel.local_port);

    println!(
        "Creating remote forwarding: remote:{} -> local:{} (tunnel: {})",
        tunnel.remote_port, tunnel.local_port, tunnel.name
    );

    let (listener, _) = match session
        .channel_forward_listen(tunnel.remote_port, None, None)
        .await
    {
        Ok(listener) => listener,
        Err(e) => {
            let raw_error = e.to_string();
            let error_msg = if raw_error
                .to_ascii_lowercase()
                .contains("address already in use")
            {
                format!("Remote port {} is already in use", tunnel.remote_port)
            } else {
                format!(
                    "Failed to create remote forwarding for tunnel {}: {}",
                    tunnel.name, raw_error
                )
            };
            eprintln!("{}", error_msg);
            return Err(error_msg);
        }
    };

    let tunnel_for_task = tunnel.clone();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        run_remote_forwarding_loop(
            manager,
            tunnel_for_task,
            listener,
            local_addr,
            session,
            shutdown_rx,
        )
        .await;
    });

    println!(
        "Remote forwarding created successfully for {}: {}",
        tunnel.name, tunnel.id
    );

    Ok(ActiveTunnel {
        tunnel,
        shutdown_tx: Some(shutdown_tx),
        task_handle: handle,
    })
}

// Handle a single remote forwarding connection
async fn handle_remote_connection(
    mut channel: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    local_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to local service
    let mut local_stream = TcpStream::connect(local_addr).await?;

    // Copy data bidirectionally
    if let Err(err) = tokio::io::copy_bidirectional(&mut channel, &mut local_stream).await {
        eprintln!(
            "Copying data between Unix domain socket A and SSH tunnel failed: {:?}",
            err
        );
    }

    Ok(())
}

async fn run_local_forwarding_loop(
    manager: ConnectionManager,
    tunnel: SSHTunnel,
    listener: TcpListener,
    session: Arc<AsyncSession<TokioTcpStream>>,
    mut shutdown_rx: oneshot::Receiver<TunnelControl>,
) {
    let mut heartbeat = interval(Duration::from_secs(SSH_KEEPALIVE_INTERVAL_SECS));
    heartbeat.tick().await;
    let mut failure_count = 0;
    let mut workers = JoinSet::new();
    let remote_host = tunnel.remote_host.clone();
    let remote_port = tunnel.remote_port;

    let exit_reason = loop {
        tokio::select! {
            signal = &mut shutdown_rx => {
                match signal {
                    Ok(TunnelControl::Stop) | Err(_) => break TunnelExitReason::Stopped,
                    Ok(TunnelControl::ConnectionLost(message)) => break TunnelExitReason::ConnectionLost(message),
                }
            }
            _ = heartbeat.tick() => {
                match session.keepalive_send().await {
                    Ok(_) => {
                        failure_count = 0;
                    }
                    Err(err) => {
                        failure_count += 1;
                        eprintln!(
                            "SSH keepalive failed for tunnel {} (attempt {}): {}",
                            tunnel.id, failure_count, err
                        );
                        if failure_count >= SSH_KEEPALIVE_FAILURE_THRESHOLD {
                            break TunnelExitReason::ConnectionLost(format!("SSH keepalive failed: {}", err));
                        }
                    }
                }
            }
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((local_stream, _)) => {
                        let session = session.clone();
                        let remote_host = remote_host.clone();
                        workers.spawn(async move {
                            let mut local_stream = local_stream;
                            if let Err(err) = handle_local_connection(
                                session,
                                &mut local_stream,
                                &remote_host,
                                remote_port,
                            )
                            .await
                            {
                                eprintln!("Tunnel connection error: {}", err);
                            }
                        });
                    }
                    Err(err) => {
                        break TunnelExitReason::TunnelError(format!(
                            "Failed to accept local connection: {}",
                            err
                        ));
                    }
                }
            }
            join_result = workers.join_next(), if !workers.is_empty() => {
                if let Some(Err(err)) = join_result {
                    if !err.is_cancelled() {
                        eprintln!("Tunnel worker for {} exited unexpectedly: {}", tunnel.id, err);
                    }
                }
            }
        }
    };

    workers.abort_all();
    while let Some(join_result) = workers.join_next().await {
        if let Err(err) = join_result {
            if !err.is_cancelled() {
                eprintln!(
                    "Tunnel worker for {} exited unexpectedly: {}",
                    tunnel.id, err
                );
            }
        }
    }

    manager
        .handle_tunnel_runtime_exit(tunnel, exit_reason)
        .await;
}

async fn run_remote_forwarding_loop(
    manager: ConnectionManager,
    tunnel: SSHTunnel,
    mut listener: AsyncListener<TokioTcpStream>,
    local_addr: String,
    session: Arc<AsyncSession<TokioTcpStream>>,
    mut shutdown_rx: oneshot::Receiver<TunnelControl>,
) {
    let mut heartbeat = interval(Duration::from_secs(SSH_KEEPALIVE_INTERVAL_SECS));
    heartbeat.tick().await;
    let mut failure_count = 0;
    let mut workers = JoinSet::new();

    let exit_reason = loop {
        tokio::select! {
            signal = &mut shutdown_rx => {
                match signal {
                    Ok(TunnelControl::Stop) | Err(_) => break TunnelExitReason::Stopped,
                    Ok(TunnelControl::ConnectionLost(message)) => break TunnelExitReason::ConnectionLost(message),
                }
            }
            _ = heartbeat.tick() => {
                match session.keepalive_send().await {
                    Ok(_) => {
                        failure_count = 0;
                    }
                    Err(err) => {
                        failure_count += 1;
                        eprintln!(
                            "SSH keepalive failed for tunnel {} (attempt {}): {}",
                            tunnel.id, failure_count, err
                        );
                        if failure_count >= SSH_KEEPALIVE_FAILURE_THRESHOLD {
                            break TunnelExitReason::ConnectionLost(format!("SSH keepalive failed: {}", err));
                        }
                    }
                }
            }
            accept_result = listener.accept() => {
                match accept_result {
                    Ok(channel) => {
                        let local_addr = local_addr.clone();
                        workers.spawn(async move {
                            if let Err(err) = handle_remote_connection(channel, &local_addr).await {
                                eprintln!("Remote tunnel error: {}", err);
                            }
                        });
                    }
                    Err(err) => {
                        break TunnelExitReason::TunnelError(format!(
                            "Failed to accept remote connection: {}",
                            err
                        ));
                    }
                }
            }
            join_result = workers.join_next(), if !workers.is_empty() => {
                if let Some(Err(err)) = join_result {
                    if !err.is_cancelled() {
                        eprintln!("Remote tunnel worker for {} exited unexpectedly: {}", tunnel.id, err);
                    }
                }
            }
        }
    };

    workers.abort_all();
    while let Some(join_result) = workers.join_next().await {
        if let Err(err) = join_result {
            if !err.is_cancelled() {
                eprintln!(
                    "Remote tunnel worker for {} exited unexpectedly: {}",
                    tunnel.id, err
                );
            }
        }
    }

    manager
        .handle_tunnel_runtime_exit(tunnel, exit_reason)
        .await;
}

// Test SSH connection
pub async fn test_ssh_connection(connection: &SSHConnection) -> ConnectionResult {
    // Set 5 second timeout
    match timeout(
        Duration::from_secs(5),
        test_ssh_connection_async(connection),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => ConnectionResult {
            success: false,
            message: "SSH connection test timed out".to_string(),
            error_code: Some("TIMEOUT".to_string()),
        },
    }
}

// Async SSH connection test function
async fn test_ssh_connection_async(connection: &SSHConnection) -> ConnectionResult {
    // First validate key file path (if using key authentication)
    if let AuthMethod::Key = connection.auth_method {
        if let Some(key_path) = &connection.key_path {
            if !Path::new(key_path).exists() {
                return ConnectionResult {
                    success: false,
                    message: format!("Key file does not exist: {}", key_path),
                    error_code: Some("KEY_FILE_NOT_FOUND".to_string()),
                };
            }
        } else {
            return ConnectionResult {
                success: false,
                message: "Key authentication requires specifying a key file path".to_string(),
                error_code: Some("KEY_PATH_MISSING".to_string()),
            };
        }
    }

    // Try to establish TCP connection
    let tcp_addr = format!("{}:{}", connection.host, connection.port);
    let tcp = match TcpStream::connect(&tcp_addr).await {
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
                message: format!(
                    "Unable to connect to server {}:{} - {}",
                    connection.host, connection.port, e
                ),
                error_code: Some(error_code.to_string()),
            };
        }
    };

    // Try to establish SSH session
    let mut session = match AsyncSession::new(tcp, Some(build_session_configuration())) {
        Ok(session) => session,
        Err(e) => {
            return ConnectionResult {
                success: false,
                message: format!("Failed to create SSH session: {}", e),
                error_code: Some("SSH_SESSION_ERROR".to_string()),
            };
        }
    };

    // Perform SSH handshake
    if let Err(e) = session.handshake().await {
        return ConnectionResult {
            success: false,
            message: format!("SSH handshake failed: {}", e),
            error_code: Some("SSH_HANDSHAKE_ERROR".to_string()),
        };
    }

    // Try user authentication
    let auth_result = match connection.auth_method {
        AuthMethod::Password => {
            if let Some(password) = &connection.password {
                session
                    .userauth_password(&connection.username, password)
                    .await
            } else {
                return ConnectionResult {
                    success: false,
                    message: "Password authentication requires providing a password".to_string(),
                    error_code: Some("PASSWORD_MISSING".to_string()),
                };
            }
        }
        AuthMethod::Key => {
            if let Some(key_path) = &connection.key_path {
                // Check if the key file exists
                if !Path::new(key_path).exists() {
                    return ConnectionResult {
                        success: false,
                        message: format!("Private key file not found: {}", key_path),
                        error_code: Some("KEY_FILE_NOT_FOUND".to_string()),
                    };
                }

                // Try to authenticate using private key file
                session
                    .userauth_pubkey_file(&connection.username, None, Path::new(key_path), None)
                    .await
            } else {
                return ConnectionResult {
                    success: false,
                    message: "Key authentication requires providing a key file path".to_string(),
                    error_code: Some("KEY_PATH_MISSING".to_string()),
                };
            }
        }
    };

    match auth_result {
        Ok(_) => {
            // Verify authentication
            if session.authenticated() {
                ConnectionResult {
                    success: true,
                    message: "SSH connection test successful".to_string(),
                    error_code: None,
                }
            } else {
                ConnectionResult {
                    success: false,
                    message: "SSH authentication failed".to_string(),
                    error_code: Some("SSH_AUTH_ERROR".to_string()),
                }
            }
        }
        Err(e) => {
            // Simplify error handling, directly use error message
            let message = format!("SSH authentication failed: {}", e);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_connection(id: &str, status: ConnectionStatus) -> SSHConnection {
        SSHConnection {
            id: id.to_string(),
            name: "Test Connection".to_string(),
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "tester".to_string(),
            auth_method: AuthMethod::Password,
            password: Some("secret".to_string()),
            key_path: None,
            status,
            last_connected: None,
            created_at: SystemTime::now(),
        }
    }

    fn sample_tunnel(
        id: &str,
        connection_id: &str,
        status: TunnelStatus,
        auto_reconnect: bool,
    ) -> SSHTunnel {
        SSHTunnel {
            id: id.to_string(),
            connection_id: connection_id.to_string(),
            name: "Test Tunnel".to_string(),
            tunnel_type: TunnelType::Local,
            local_port: 8080,
            remote_host: "127.0.0.1".to_string(),
            remote_port: 80,
            status,
            auto_reconnect,
        }
    }

    fn spawn_dummy_active_tunnel(manager: ConnectionManager, tunnel: SSHTunnel) -> ActiveTunnel {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let tunnel_for_task = tunnel.clone();
        let task_handle = tokio::spawn(async move {
            let exit_reason = match shutdown_rx.await {
                Ok(TunnelControl::Stop) | Err(_) => TunnelExitReason::Stopped,
                Ok(TunnelControl::ConnectionLost(message)) => {
                    TunnelExitReason::ConnectionLost(message)
                }
            };
            manager
                .handle_tunnel_runtime_exit(tunnel_for_task, exit_reason)
                .await;
        });

        ActiveTunnel {
            tunnel,
            shutdown_tx: Some(shutdown_tx),
            task_handle,
        }
    }

    #[tokio::test]
    async fn stop_active_tunnel_marks_it_inactive() {
        let manager = ConnectionManager::new();
        let connection = sample_connection("conn-stop", ConnectionStatus::Connected);
        let tunnel = sample_tunnel("tunnel-stop", "conn-stop", TunnelStatus::Active, false);

        manager
            .connections
            .write()
            .await
            .insert(connection.id.clone(), connection);
        manager
            .tunnels
            .write()
            .await
            .insert(tunnel.id.clone(), tunnel.clone());
        manager.active_tunnels.write().await.insert(
            tunnel.id.clone(),
            spawn_dummy_active_tunnel(manager.clone(), tunnel.clone()),
        );

        assert!(
            manager
                .stop_active_tunnel(&tunnel.id, TunnelControl::Stop)
                .await
        );
        assert!(manager.active_tunnels.read().await.is_empty());

        let tunnels = manager.tunnels.read().await;
        assert!(matches!(
            tunnels.get(&tunnel.id).map(|tunnel| &tunnel.status),
            Some(TunnelStatus::Inactive)
        ));
    }

    #[tokio::test]
    async fn health_check_without_session_marks_connection_and_tunnel_error() {
        let manager = ConnectionManager::new();
        let connection = sample_connection("conn-health", ConnectionStatus::Connected);
        let tunnel = sample_tunnel("tunnel-health", "conn-health", TunnelStatus::Active, false);

        manager
            .connections
            .write()
            .await
            .insert(connection.id.clone(), connection.clone());
        manager
            .tunnels
            .write()
            .await
            .insert(tunnel.id.clone(), tunnel.clone());
        manager.active_tunnels.write().await.insert(
            tunnel.id.clone(),
            spawn_dummy_active_tunnel(manager.clone(), tunnel.clone()),
        );

        manager.check_connection_health(&connection.id).await;

        let connections = manager.connections.read().await;
        assert!(matches!(
            connections
                .get(&connection.id)
                .map(|connection| &connection.status),
            Some(ConnectionStatus::Error)
        ));
        drop(connections);

        let tunnels = manager.tunnels.read().await;
        assert!(matches!(
            tunnels.get(&tunnel.id).map(|tunnel| &tunnel.status),
            Some(TunnelStatus::Error)
        ));
        drop(tunnels);

        assert!(manager.active_tunnels.read().await.is_empty());
    }

    #[test]
    fn local_listener_fails_when_loopback_port_is_already_bound() {
        let occupied_listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = occupied_listener.local_addr().unwrap().port();

        let bind_result = create_local_tunnel_listener(port);

        assert!(matches!(
            bind_result.as_ref().map_err(|error| error.kind()),
            Err(std::io::ErrorKind::AddrInUse)
        ));
    }
}
