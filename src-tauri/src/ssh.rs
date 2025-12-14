use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use async_ssh2_lite::{AsyncSession, TokioTcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(rename = "auth_method")]
    pub auth_method: AuthMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "key_path", skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
    pub status: ConnectionStatus,
    #[serde(rename = "last_connected", skip_serializing_if = "Option::is_none")]
    pub last_connected: Option<SystemTime>,
    #[serde(rename = "created_at", default = "default_created_at")]
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
    #[serde(rename = "connection_id")]
    pub connection_id: String,
    pub name: String,
    #[serde(rename = "tunnel_type")]
    pub tunnel_type: TunnelType,
    #[serde(rename = "local_port")]
    pub local_port: u16,
    #[serde(rename = "remote_host")]
    pub remote_host: String,
    #[serde(rename = "remote_port")]
    pub remote_port: u16,
    pub status: TunnelStatus,
    #[serde(rename = "auto_reconnect")]
    pub auto_reconnect: bool,
}

// Active tunnel handle for managing port forwarding
pub enum TunnelHandle {
    Local {
        _task_handle: tokio::task::JoinHandle<()>,
    },
    Remote {
        _task_handle: tokio::task::JoinHandle<()>,
    },
}

pub struct ActiveTunnel {
    pub tunnel: SSHTunnel,
    pub handle: TunnelHandle,
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

pub struct ConnectionManager {
    connections: RwLock<HashMap<String, SSHConnection>>,
    tunnels: RwLock<HashMap<String, SSHTunnel>>,
    ssh_sessions: RwLock<HashMap<String, Arc<AsyncSession<TokioTcpStream>>>>,
    active_tunnels: RwLock<HashMap<String, ActiveTunnel>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            tunnels: RwLock::new(HashMap::new()),
            ssh_sessions: RwLock::new(HashMap::new()),
            active_tunnels: RwLock::new(HashMap::new()),
        }
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
        use crate::storage::DataManager;
        let data_manager = DataManager::new()?;

        let connections = self.connections.read().await;
        let tunnels = self.tunnels.read().await;

        data_manager
            .save_connections_and_tunnels(&*connections, &*tunnels)
            .await?;

        Ok(())
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
        // Stop all active tunnels for this connection first
        {
            let mut active_tunnels = self.active_tunnels.write().await;
            let tunnels_to_remove: Vec<String> = active_tunnels
                .iter()
                .filter(|(_, tunnel)| tunnel.tunnel.connection_id == id)
                .map(|(tunnel_id, _)| tunnel_id.clone())
                .collect();

            for tunnel_id in tunnels_to_remove {
                if let Some(active_tunnel) = active_tunnels.remove(&tunnel_id) {
                    match active_tunnel.handle {
                        TunnelHandle::Local { _task_handle } => {
                            println!("Stopping local tunnel {} for connection {}", tunnel_id, id);
                            _task_handle.abort();
                        }
                        TunnelHandle::Remote { _task_handle } => {
                            println!("Stopping remote tunnel {} for connection {}", tunnel_id, id);
                            _task_handle.abort();
                        }
                    }
                }
            }
        }

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
        let mut connections = self.connections.write().await;
        let connection_clone = connections.get(id).cloned();

        if connection_clone.is_none() {
            return ConnectionResult {
                success: false,
                message: "Connection not found".to_string(),
                error_code: None,
            };
        }

        if let Some(connection) = connections.get_mut(id) {
            connection.status = ConnectionStatus::Connecting;
            drop(connections); // Release the lock

            // Test the connection first
            let test_result = test_ssh_connection(&connection_clone.as_ref().unwrap()).await;

            if !test_result.success {
                // Update connection status to error
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(id) {
                    conn.status = ConnectionStatus::Error;
                }
                return test_result;
            }

            // If test passes, establish real SSH connection and store session
            match establish_ssh_session(&connection_clone.as_ref().unwrap()).await {
                Ok(session) => {
                    // Store the SSH session
                    let mut sessions = self.ssh_sessions.write().await;
                    sessions.insert(id.to_string(), Arc::new(session));
                    drop(sessions);

                    // Update connection status
                    let mut connections = self.connections.write().await;
                    if let Some(conn) = connections.get_mut(id) {
                        conn.status = ConnectionStatus::Connected;
                        conn.last_connected = Some(SystemTime::now());
                    }
                    drop(connections);

                    // Start all tunnels for this connection
                    if let Err(e) = self.start_all_tunnels_for_connection(id).await {
                        eprintln!("Failed to start tunnels: {}", e);
                    }

                    // Save to storage
                    if let Err(e) = self.save_to_storage().await {
                        eprintln!("Failed to save data: {}", e);
                    }

                    ConnectionResult {
                        success: true,
                        message: "SSH connection established".to_string(),
                        error_code: None,
                    }
                }
                Err(e) => {
                    // Update connection status to error
                    let mut connections = self.connections.write().await;
                    if let Some(conn) = connections.get_mut(id) {
                        conn.status = ConnectionStatus::Error;
                    }
                    drop(connections);

                    ConnectionResult {
                        success: false,
                        message: format!("Failed to establish SSH connection: {}", e),
                        error_code: Some("CONNECTION_FAILED".to_string()),
                    }
                }
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
        let mut sessions = self.ssh_sessions.write().await;

        if let Some(connection) = connections.get_mut(id) {
            connection.status = ConnectionStatus::Disconnected;

            // First, gracefully close all active tunnels for this connection
            let mut active_tunnels = self.active_tunnels.write().await;
            let tunnel_ids_to_remove: Vec<String> = active_tunnels
                .iter()
                .filter(|(_, tunnel)| tunnel.tunnel.connection_id == id)
                .map(|(id, _)| id.clone())
                .collect();

            for tunnel_id in tunnel_ids_to_remove {
                println!("Gracefully stopping tunnel: {}", tunnel_id);

                // Get the active tunnel before removing it
                if let Some(active_tunnel) = active_tunnels.remove(&tunnel_id) {
                    // Gracefully shutdown the tunnel task
                    match active_tunnel.handle {
                        TunnelHandle::Local { _task_handle } => {
                            // Try to gracefully shutdown the task first
                            _task_handle.abort();
                        }
                        TunnelHandle::Remote { _task_handle } => {
                            // Try to gracefully shutdown the task first
                            _task_handle.abort();
                        }
                    }
                }
            }
            drop(active_tunnels);

            // Update tunnel statuses to inactive
            let connection_tunnels = self.get_tunnels_by_connection(id).await;
            if !connection_tunnels.is_empty() {
                let mut tunnels = self.tunnels.write().await;
                for tunnel in connection_tunnels {
                    if let Some(t) = tunnels.get_mut(&tunnel.id) {
                        t.status = TunnelStatus::Inactive;
                    }
                }
                drop(tunnels);
            }

            // Finally close SSH session
            if let Some(session_arc) = sessions.remove(id) {
                drop(session_arc); // Session will be dropped when Arc count goes to 0
            }

            drop(connections); // Release the lock
            drop(sessions); // Release sessions lock

            // Save to storage
            if let Err(e) = self.save_to_storage().await {
                eprintln!("Failed to save data: {}", e);
            }

            ConnectionResult {
                success: true,
                message: "SSH connection and all tunnels closed gracefully".to_string(),
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
        let mut tunnels = self.tunnels.write().await;
        let mut active_tunnels = self.active_tunnels.write().await;

        // Stop the active tunnel if it exists
        if let Some(active_tunnel) = active_tunnels.remove(&id) {
            match active_tunnel.handle {
                TunnelHandle::Local { _task_handle } => {
                    // The task will be aborted when the handle is dropped
                    println!("Stopping local tunnel: {}", id);
                    _task_handle.abort();
                }
                TunnelHandle::Remote { _task_handle } => {
                    // The task will be aborted when the handle is dropped
                    println!("Stopping remote tunnel: {}", id);
                    _task_handle.abort();
                }
            }
        }

        tunnels.remove(&id);

        drop(tunnels);
        drop(active_tunnels);

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
        let mut active_tunnels = self.active_tunnels.write().await;

        // Stop the active tunnel if it exists
        if let Some(active_tunnel) = active_tunnels.remove(&id) {
            match active_tunnel.handle {
                TunnelHandle::Local { _task_handle } => {
                    println!("Stopping local tunnel: {}", id);
                    _task_handle.abort();
                }
                TunnelHandle::Remote { _task_handle } => {
                    println!("Stopping remote tunnel: {}", id);
                    _task_handle.abort();
                }
            }
        }

        // Update tunnel status to inactive
        {
            let mut tunnels = self.tunnels.write().await;
            if let Some(tunnel) = tunnels.get_mut(&id) {
                tunnel.status = TunnelStatus::Inactive;
            }
        }

        drop(active_tunnels);
        self.save_to_storage().await?;
        Ok(())
    }

    // Start all tunnels for a given connection
    async fn start_all_tunnels_for_connection(&self, connection_id: &str) -> Result<(), String> {
        let connection_tunnels = self.get_tunnels_by_connection(connection_id).await;

        for tunnel in connection_tunnels {
            let tunnel_id = tunnel.id.clone();

            // Get session for each tunnel separately
            let sessions = self.ssh_sessions.read().await;
            if let Some(session_arc) = sessions.get(connection_id) {
                let session = Arc::clone(session_arc);
                drop(sessions);

                match tunnel.tunnel_type {
                    TunnelType::Local => {
                        // Start local forwarding in a new task
                        let handle = start_local_forwarding(&tunnel, session).await?;

                        // Save the task handle in active_tunnels
                        let active_tunnel = ActiveTunnel {
                            tunnel: tunnel.clone(),
                            handle: TunnelHandle::Local {
                                _task_handle: handle,
                            },
                        };

                        let mut active_tunnels = self.active_tunnels.write().await;
                        active_tunnels.insert(tunnel_id.clone(), active_tunnel);

                        // Update tunnel status to Active
                        drop(active_tunnels);
                        let mut tunnels = self.tunnels.write().await;
                        if let Some(t) = tunnels.get_mut(&tunnel_id) {
                            t.status = TunnelStatus::Active;
                        }
                    }
                    TunnelType::Remote => {
                        // Start remote forwarding in a new task
                        let handle = start_remote_forwarding(&tunnel, session).await?;

                        // Save the task handle in active_tunnels
                        let active_tunnel = ActiveTunnel {
                            tunnel: tunnel.clone(),
                            handle: TunnelHandle::Remote {
                                _task_handle: handle,
                            },
                        };

                        let mut active_tunnels = self.active_tunnels.write().await;
                        active_tunnels.insert(tunnel_id.clone(), active_tunnel);

                        // Update tunnel status to Active
                        drop(active_tunnels);
                        let mut tunnels = self.tunnels.write().await;
                        if let Some(t) = tunnels.get_mut(&tunnel_id) {
                            t.status = TunnelStatus::Active;
                        }
                    }
                }
            }
        }

        Ok(())
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

    // Try to establish SSH session
    let mut session = match AsyncSession::new(tcp, None) {
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
                // Read private key file
                let private_key = match std::fs::read_to_string(key_path) {
                    Ok(key) => key,
                    Err(e) => {
                        return Err(format!(
                            "Failed to read private key file {}: {}",
                            key_path, e
                        ));
                    }
                };

                session
                    .userauth_pubkey_memory(&connection.username, None, &private_key, None)
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

// Start local port forwarding - creates a task that handles the forwarding
async fn start_local_forwarding(
    tunnel: &SSHTunnel,
    session: Arc<AsyncSession<TokioTcpStream>>,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let local_addr = format!("0.0.0.0:{}", tunnel.local_port);
    let remote_host = tunnel.remote_host.clone();
    let remote_port = tunnel.remote_port;
    let tunnel_id = tunnel.id.clone();
    let tunnel_name = tunnel.name.clone();

    println!(
        "Creating SSH tunnel: {} -> {}:{} (tunnel: {})",
        tunnel_name, remote_host, remote_port, tunnel_name
    );

    // Create TCP listener
    let listener = match TcpListener::bind(&local_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(format!("Failed to bind to {}: {}", local_addr, e));
        }
    };

    let local_addr = listener.local_addr().unwrap();
    println!("Local tunnel listening on {}", local_addr);

    // tunnel_id will be used in the println! below

    // Spawn a task to handle incoming connections
    let tunnel_id_clone = tunnel_id.clone();
    let handle = tokio::spawn(async move {
        // Create a shutdown signal that will be triggered when the task is aborted
        tokio::select! {
            _result = async {
                loop {
                    match listener.accept().await {
                        Ok((mut local_stream, _)) => {
                            let session_clone = session.clone();
                            let remote_host = remote_host.clone();

                            tokio::spawn(async move {
                                if let Err(e) = handle_local_connection(
                                    session_clone,
                                    &mut local_stream,
                                    &remote_host,
                                    remote_port,
                                )
                                .await
                                {
                                    eprintln!("Tunnel connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                            break;
                        }
                    }
                }
            } => {
                println!("Tunnel {} listener loop ended", tunnel_id_clone);
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Tunnel {} received shutdown signal", tunnel_id_clone);
            }
        }

        println!("Tunnel {} gracefully shut down", tunnel_id_clone);
    });

    println!(
        "SSH tunnel created successfully for {}: {}",
        tunnel_name, tunnel_id
    );
    Ok(handle)
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

// Start remote port forwarding - forwards remote port to local host
async fn start_remote_forwarding(
    tunnel: &SSHTunnel,
    session: Arc<AsyncSession<TokioTcpStream>>,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let local_addr = format!("127.0.0.1:{}", tunnel.local_port);
    let remote_port = tunnel.remote_port;
    let tunnel_id = tunnel.id.clone();
    let tunnel_name = tunnel.name.clone();

    println!(
        "Creating remote forwarding: remote:{} -> local:{} (tunnel: {})",
        remote_port, tunnel.local_port, tunnel_name
    );

    // Create listener on remote side
    let (mut listener, _) = match session
        .channel_forward_listen(remote_port, None, None)
        .await
    {
        Ok(listener) => listener,
        Err(e) => {
            let error_msg = format!(
                "Failed to create remote forwarding for tunnel {}: {}",
                tunnel_name, e
            );
            eprintln!("{}", error_msg);
            return Err(error_msg);
        }
    };

    let tunnel_id_clone = tunnel_id.clone();

    // Spawn a task to handle incoming remote connections
    let handle = tokio::spawn(async move {
        // Create a shutdown signal that will be triggered when the task is aborted
        tokio::select! {
            _result = async {
                loop {
                    match listener.accept().await {
                        Ok(channel) => {
                            let local_addr = local_addr.clone();

                            tokio::spawn(async move {
                                if let Err(e) = handle_remote_connection(channel, &local_addr).await {
                                    eprintln!("Remote tunnel error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to accept remote connection: {}", e);
                            break;
                        }
                    }
                }
            } => {
                println!("Remote tunnel {} listener loop ended", tunnel_id_clone);
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Remote tunnel {} received shutdown signal", tunnel_id_clone);
            }
        }

        println!("Remote tunnel {} gracefully shut down", tunnel_id_clone);
    });

    println!(
        "Remote forwarding created successfully for {}: {}",
        tunnel_name, tunnel_id
    );
    Ok(handle)
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
    let mut session = match AsyncSession::new(tcp, None) {
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
                // Try to read private key file
                let private_key = match fs::read_to_string(key_path) {
                    Ok(key) => key,
                    Err(e) => {
                        return ConnectionResult {
                            success: false,
                            message: format!("Failed to read private key file {}: {}", key_path, e),
                            error_code: Some("KEY_FILE_READ_ERROR".to_string()),
                        };
                    }
                };

                // Try to authenticate using private key
                session
                    .userauth_pubkey_memory(&connection.username, None, &private_key, None)
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
