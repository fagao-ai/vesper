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
    tunnels.value.filter(tunnel => tunnel.connection_id === connectionId);

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
      const tunnelsData = await sshApi.getTunnels();
      console.log('Fetched tunnels from backend:', tunnelsData);
      tunnels.value = tunnelsData;
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
        console.warn(`Attempted to update non-existent connection: ${id}. Skipping update.`);
        return; // 静默处理，不抛出错误
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

      // 先从本地状态中移除，立即更新UI
      connections.value = connections.value.filter(conn => conn.id !== id);
      tunnels.value = tunnels.value.filter(tunnel => tunnel.connection_id !== id);

      // 然后调用后端删除
      await sshApi.deleteConnection(id);

      // 短暂延迟后重新获取数据以确保同步
      setTimeout(async () => {
        try {
          await fetchConnections();
          await fetchTunnels();
        } catch (refreshErr) {
          console.warn('Failed to refresh data after deletion:', refreshErr);
        }
      }, 100);

    } catch (err) {
      error.value = err as string;
      console.error('Failed to delete connection:', err);

      // 如果是"Connection not found"错误，先检查本地是否还有这个连接
      const errorMessage = String(err);
      if (errorMessage.includes('Connection not found')) {
        // 从本地状态中移除连接，即使后端删除失败
        connections.value = connections.value.filter(conn => conn.id !== id);
        tunnels.value = tunnels.value.filter(tunnel => tunnel.connection_id !== id);

        // 短暂延迟后重新获取数据
        setTimeout(async () => {
          try {
            await fetchConnections();
            await fetchTunnels();
          } catch (refreshErr) {
            console.warn('Failed to refresh data after connection not found:', refreshErr);
          }
        }, 100);

        // 不抛出错误，让UI继续正常工作
        return;
      }

      throw err;
    }
  };

  const testConnection = async (id: string) => {
    try {
      error.value = null;
      const connection = getConnectionById(id);
      if (!connection) {
        console.warn(`Attempted to test non-existent connection: ${id}. Skipping test.`);
        return { success: false, message: 'Connection not found', error_code: 'NOT_FOUND' };
      }
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
      const connection = getConnectionById(id);
      if (!connection) {
        console.warn(`Attempted to connect to non-existent connection: ${id}. Skipping connection.`);
        return { success: false, message: 'Connection not found', error_code: 'NOT_FOUND' };
      }

      const result = await sshApi.connectSSH(id);

      // Update connection status locally
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
      const connection = getConnectionById(id);
      if (!connection) {
        console.warn(`Attempted to disconnect from non-existent connection: ${id}. Skipping disconnection.`);
        return { success: false, message: 'Connection not found', error_code: 'NOT_FOUND' };
      }

      const result = await sshApi.disconnectSSH(id);

      // Update connection status locally
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
      console.log('addTunnel - tunnel object:', tunnel);
      console.log('addTunnel - tunnel.connection_id:', tunnel.connection_id);

      const id = await sshApi.createTunnel({
        name: tunnel.name,
        connection_id: tunnel.connection_id,
        tunnel_type: tunnel.tunnel_type,
        local_port: tunnel.local_port,
        remote_host: tunnel.remote_host,
        remote_port: tunnel.remote_port,
        auto_reconnect: tunnel.auto_reconnect
      });

      await fetchTunnels(); // Refresh the list
      return id;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to create tunnel:', err);
      throw err;
    }
  };

  const updateTunnel = async (id: string, updates: Partial<SSHTunnel>) => {
    try {
      error.value = null;

      const currentTunnel = tunnels.value.find(t => t.id === id);
      if (!currentTunnel) {
        throw new Error('Tunnel not found');
      }

      await sshApi.updateTunnel({
        id,
        name: updates.name || currentTunnel.name,
        connection_id: updates.connection_id || currentTunnel.connection_id,
        tunnel_type: updates.tunnel_type || currentTunnel.tunnel_type,
        local_port: updates.local_port || currentTunnel.local_port,
        remote_host: updates.remote_host || currentTunnel.remote_host,
        remote_port: updates.remote_port || currentTunnel.remote_port,
        auto_reconnect: updates.auto_reconnect !== undefined ? updates.auto_reconnect : currentTunnel.auto_reconnect
      });

      // Update local state
      const tunnelIndex = tunnels.value.findIndex(t => t.id === id);
      if (tunnelIndex !== -1) {
        Object.assign(tunnels.value[tunnelIndex], updates);
      }
    } catch (err) {
      error.value = err as string;
      console.error('Failed to update tunnel:', err);
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
      tunnels.value = tunnels.value.filter(tunnel => tunnel.connection_id !== connectionId);
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
    try {
      loading.value = true;
      error.value = null;

      // Initialize storage to load persisted data
      await sshApi.initializeStorage();

      // 清空本地状态，然后从后端获取数据
      connections.value = [];
      tunnels.value = [];
      error.value = null;

      await Promise.all([
        fetchConnections(),
        fetchTunnels()
      ]);
    } catch (err) {
      error.value = err as string;
      console.error('Failed to initialize store:', err);
      throw err;
    } finally {
      loading.value = false;
    }
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
    updateTunnel,
    removeTunnel,
    loadTunnelsByConnection,
    startTunnel,
    stopTunnel
  };
});