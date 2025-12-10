import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { sshApi } from '../services/ssh';
import type { SSHConnection, SSHTunnel } from '../types';

export const useConnectionsStore = defineStore('connections', () => {
  // State
  const connections = ref<SSHConnection[]>([]);
  const tunnels = ref<SSHTunnel[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const connectedConnections = computed(() =>
    connections.value.filter(conn => conn.status === 'connected')
  );

  const activeTunnels = computed(() =>
    tunnels.value.filter(tunnel => tunnel.status === 'active')
  );

  const getConnectionById = (id: string) =>
    connections.value.find(conn => conn.id === id);

  const getTunnelsByConnectionId = (connectionId: string) =>
    tunnels.value.filter(tunnel => tunnel.connectionId === connectionId);

  // Actions
  const fetchConnections = async () => {
    try {
      loading.value = true;
      error.value = null;
      connections.value = await sshApi.getConnections();
    } catch (err) {
      error.value = err as string;
      console.error('Failed to fetch connections:', err);
    } finally {
      loading.value = false;
    }
  };

  const fetchTunnels = async () => {
    try {
      loading.value = true;
      error.value = null;
      tunnels.value = await sshApi.getTunnels();
    } catch (err) {
      error.value = err as string;
      console.error('Failed to fetch tunnels:', err);
    } finally {
      loading.value = false;
    }
  };

  const addConnection = async (connection: Omit<SSHConnection, 'id' | 'status' | 'lastConnected'>) => {
    try {
      error.value = null;
      const id = await sshApi.createConnection({
        name: connection.name,
        host: connection.host,
        port: connection.port,
        username: connection.username,
        auth_method: connection.authMethod,
        password: connection.password,
        key_path: connection.keyPath
      });

      await fetchConnections(); // Refresh the list
      return id;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to create connection:', err);
      throw err;
    }
  };

  const updateConnection = async (id: string, updates: Partial<SSHConnection>) => {
    try {
      error.value = null;
      const currentConnection = getConnectionById(id);
      if (!currentConnection) {
        throw new Error('Connection not found');
      }

      await sshApi.updateConnection({
        id,
        name: updates.name || currentConnection.name,
        host: updates.host || currentConnection.host,
        port: updates.port || currentConnection.port,
        username: updates.username || currentConnection.username,
        auth_method: updates.authMethod || currentConnection.authMethod,
        password: updates.password || currentConnection.password,
        key_path: updates.keyPath || currentConnection.keyPath
      });

      await fetchConnections(); // Refresh the list
    } catch (err) {
      error.value = err as string;
      console.error('Failed to update connection:', err);
      throw err;
    }
  };

  const removeConnection = async (id: string) => {
    try {
      error.value = null;
      await sshApi.deleteConnection(id);

      // Update local state
      connections.value = connections.value.filter(conn => conn.id !== id);
      tunnels.value = tunnels.value.filter(tunnel => tunnel.connectionId !== id);
    } catch (err) {
      error.value = err as string;
      console.error('Failed to delete connection:', err);
      throw err;
    }
  };

  const testConnection = async (id: string) => {
    try {
      error.value = null;
      const result = await sshApi.testConnection(id);
      return result;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to test connection:', err);
      throw err;
    }
  };

  const connectSSH = async (id: string) => {
    try {
      error.value = null;
      const result = await sshApi.connectSSH(id);

      // Update connection status locally
      const connection = getConnectionById(id);
      if (connection && result.success) {
        connection.status = 'connected';
        connection.lastConnected = new Date();
      }

      return result;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to connect SSH:', err);

      // Update connection status to error
      const connection = getConnectionById(id);
      if (connection) {
        connection.status = 'error';
      }

      throw err;
    }
  };

  const disconnectSSH = async (id: string) => {
    try {
      error.value = null;
      const result = await sshApi.disconnectSSH(id);

      // Update connection status locally
      const connection = getConnectionById(id);
      if (connection && result.success) {
        connection.status = 'disconnected';
      }

      return result;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to disconnect SSH:', err);
      throw err;
    }
  };

  const addTunnel = async (tunnel: Omit<SSHTunnel, 'id' | 'status'>) => {
    try {
      error.value = null;
      const id = await sshApi.createTunnel({
        name: tunnel.name,
        connection_id: tunnel.connectionId,
        tunnel_type: tunnel.type,
        local_port: tunnel.localPort,
        remote_host: tunnel.remoteHost,
        remote_port: tunnel.remotePort,
        auto_reconnect: tunnel.autoReconnect
      });

      await fetchTunnels(); // Refresh the list
      return id;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to create tunnel:', err);
      throw err;
    }
  };

  const removeTunnel = async (id: string) => {
    try {
      error.value = null;
      await sshApi.deleteTunnel(id);

      // Update local state
      tunnels.value = tunnels.value.filter(tunnel => tunnel.id !== id);
    } catch (err) {
      error.value = err as string;
      console.error('Failed to delete tunnel:', err);
      throw err;
    }
  };

  const loadTunnelsByConnection = async (connectionId: string) => {
    try {
      error.value = null;
      const connectionTunnels = await sshApi.getTunnelsByConnection(connectionId);

      // Update local state for this connection's tunnels
      tunnels.value = tunnels.value.filter(tunnel => tunnel.connectionId !== connectionId);
      tunnels.value.push(...connectionTunnels);

      return connectionTunnels;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to load tunnels for connection:', err);
      throw err;
    }
  };

  const startTunnel = async (id: string) => {
    try {
      error.value = null;
      const result = await sshApi.startTunnel(id);

      // Update tunnel status locally
      const tunnel = tunnels.value.find(t => t.id === id);
      if (tunnel && result.success) {
        tunnel.status = 'active';
      }

      return result;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to start tunnel:', err);

      // Update tunnel status to error
      const tunnel = tunnels.value.find(t => t.id === id);
      if (tunnel) {
        tunnel.status = 'error';
      }

      throw err;
    }
  };

  const stopTunnel = async (id: string) => {
    try {
      error.value = null;
      const result = await sshApi.stopTunnel(id);

      // Update tunnel status locally
      const tunnel = tunnels.value.find(t => t.id === id);
      if (tunnel && result.success) {
        tunnel.status = 'inactive';
      }

      return result;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to stop tunnel:', err);
      throw err;
    }
  };

  // Initialize data on store creation
  const initialize = async () => {
    await Promise.all([
      fetchConnections(),
      fetchTunnels()
    ]);
  };

  return {
    // State
    connections,
    tunnels,
    loading,
    error,

    // Getters
    connectedConnections,
    activeTunnels,
    getConnectionById,
    getTunnelsByConnectionId,

    // Actions
    initialize,
    fetchConnections,
    fetchTunnels,
    addConnection,
    updateConnection,
    removeConnection,
    testConnection,
    connectSSH,
    disconnectSSH,
    addTunnel,
    removeTunnel,
    loadTunnelsByConnection,
    startTunnel,
    stopTunnel
  };
});