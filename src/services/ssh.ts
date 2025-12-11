import { invoke } from '@tauri-apps/api/core';
import type { SSHConnection, SSHTunnel } from '../types';

// API Response Types
export interface ConnectionResult {
  success: boolean;
  message: string;
  error_code?: string;
}

export interface CreateConnectionRequest {
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: 'password' | 'key';
  password?: string;
  key_path?: string;
}

export interface UpdateConnectionRequest {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: 'password' | 'key';
  password?: string;
  key_path?: string;
}

export interface CreateTunnelRequest {
  name: string;
  connection_id: string;
  tunnel_type: 'local' | 'remote' | 'dynamic';
  local_port: number;
  remote_host: string;
  remote_port: number;
  auto_reconnect: boolean;
}

// SSH Connection API
export const sshApi = {
  // Storage initialization
  async initializeStorage(): Promise<void> {
    return await invoke('initialize_storage');
  },

  // Connection CRUD operations
  async createConnection(connection: CreateConnectionRequest): Promise<string> {
    return await invoke('create_connection', { request: connection });
  },

  async getConnections(): Promise<SSHConnection[]> {
    return await invoke('get_connections');
  },

  async getConnection(id: string): Promise<SSHConnection | null> {
    return await invoke('get_connection', { id });
  },

  async updateConnection(connection: UpdateConnectionRequest): Promise<void> {
    return await invoke('update_connection', { request: connection });
  },

  async deleteConnection(id: string): Promise<void> {
    return await invoke('delete_connection', { id });
  },

  // Connection operations
  async testConnection(id: string): Promise<ConnectionResult> {
    return await invoke('test_connection', { id });
  },

  async testConnectionData(connectionData: CreateConnectionRequest): Promise<ConnectionResult> {
    return await invoke('test_connection_data', { request: connectionData });
  },

  async connectSSH(id: string): Promise<ConnectionResult> {
    return await invoke('connect_ssh', { id });
  },

  async disconnectSSH(id: string): Promise<ConnectionResult> {
    return await invoke('disconnect_ssh', { id });
  },

  // Tunnel CRUD operations
  async createTunnel(tunnel: CreateTunnelRequest): Promise<string> {
    return await invoke('create_tunnel', { request: tunnel });
  },

  async getTunnels(): Promise<SSHTunnel[]> {
    return await invoke('get_tunnels');
  },

  async getTunnelsByConnection(connectionId: string): Promise<SSHTunnel[]> {
    return await invoke('get_tunnels_by_connection', { connection_id: connectionId });
  },

  async deleteTunnel(id: string): Promise<void> {
    return await invoke('delete_tunnel', { id });
  },

  // Tunnel control operations
  async startTunnel(id: string): Promise<ConnectionResult> {
    return await invoke('start_tunnel', { id });
  },

  async stopTunnel(id: string): Promise<ConnectionResult> {
    return await invoke('stop_tunnel', { id });
  },

  // Settings operations
  async getSettings(): Promise<any> {
    return await invoke('get_settings');
  },

  async updateSettings(settings: any): Promise<void> {
    return await invoke('update_settings', { settings });
  },

  async resetSettings(): Promise<any> {
    return await invoke('reset_settings');
  }
};