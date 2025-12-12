<template>
  <div class="h-full flex flex-col">
    <!-- Empty State -->
    <div v-if="connections.length === 0" class="flex-1 flex items-center justify-center">
      <el-empty :description="translate('no_connections')" class="text-center">
        <el-button type="primary" @click="debugEmit('add-connection')">
          <el-icon class="mr-2"><Plus /></el-icon>
          {{ translate('add_first_connection') }}
        </el-button>
      </el-empty>
    </div>

    <!-- Connection List -->
    <div v-else class="flex flex-col h-full">
      <!-- Add Connection Button -->
      <div class="p-3">
        <el-button
          type="primary"
          class="w-full"
          @click="debugEmit('add-connection')"
          size="default"
        >
          <el-icon class="mr-2"><Plus /></el-icon>
          {{ translate('add_new_connection') }}
        </el-button>
      </div>

      <!-- Connections -->
      <div class="flex-1 overflow-y-auto px-3 pt-0 pb-2 space-y-2">
        <TransitionGroup name="list-fade" tag="div">
          <div
            v-for="connection in connections"
            :key="connection.id"
            class="connection-item mb-2"
            :class="{
              'selected': selectedId === connection.id,
              'connected': connection.status === 'connected',
              'error': connection.status === 'error'
            }"
            @click="$emit('select-connection', connection.id)"
          >
            <!-- Connection Main Info -->
            <div class="flex items-center space-x-2 p-2">
              <div class="flex-shrink-0">
                <div
                  class="w-7 h-7 rounded-lg flex items-center justify-center transition-all duration-200"
                  :class="{
                    'bg-gradient-to-br from-blue-500 to-purple-600': connection.status !== 'connected',
                    'bg-gradient-to-br from-green-500 to-emerald-600': connection.status === 'connected',
                    'bg-gradient-to-br from-red-500 to-pink-600': connection.status === 'error'
                  }"
                >
                  <el-icon class="text-white text-xs"><Monitor /></el-icon>
                </div>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h3 class="font-semibold text-gray-800 text-sm leading-tight truncate pr-2">{{ connection.name }}</h3>
                  <el-tag
                    :type="getStatusType(connection.status)"
                    size="small"
                    effect="light"
                    class="flex-shrink-0 min-w-0"
                  >
                    <el-icon class="mr-0.5 scale-75" v-if="connection.status === 'connected'"><Check /></el-icon>
                    <el-icon class="mr-0.5 scale-75" v-else-if="connection.status === 'connecting'"><Loading /></el-icon>
                    <el-icon class="mr-0.5 scale-75" v-else-if="connection.status === 'error'"><Close /></el-icon>
                    <el-icon class="mr-0.5 scale-75" v-else><Remove /></el-icon>
                    <span class="text-xs truncate inline-block max-w-12">{{ getStatusText(connection.status) }}</span>
                  </el-tag>
                </div>

                <div class="mt-1 text-xs text-gray-600 truncate font-mono">
                  {{ connection.username }}@{{ connection.host }}:{{ connection.port }}
                </div>

                <!-- Tunnel Count -->
                <div v-if="getTunnels(connection.id).length > 0" class="mt-2 flex items-center text-xs text-gray-500">
                  <el-icon class="mr-1 scale-90"><Link /></el-icon>
                  <span>{{ getTunnels(connection.id).length }}{{ translate('tunnel_count') }}</span>
                  <el-tag
                    v-for="tunnel in getTunnels(connection.id).filter(t => t.status === 'active')"
                    :key="tunnel.id"
                    type="success"
                    size="small"
                    effect="plain"
                    class="ml-1 h-4"
                  >
                    <span class="text-xs">{{ tunnel.name }}</span>
                  </el-tag>
                </div>
              </div>

              <!-- Quick Actions -->
              <div class="flex items-center space-x-1 opacity-0 hover:opacity-100 transition-opacity duration-200">
                <el-button
                  v-if="connection.status === 'disconnected' || connection.status === 'error'"
                  type="success"
                  size="small"
                  circle
                  @click.stop="$emit('connect', connection.id)"
                >
                  <el-icon><VideoPlay /></el-icon>
                </el-button>
                <el-button
                  v-else-if="connection.status === 'connected'"
                  type="danger"
                  size="small"
                  circle
                  @click.stop="$emit('disconnect', connection.id)"
                >
                  <el-icon><VideoPause /></el-icon>
                </el-button>
                <el-dropdown trigger="click" @command="handleAction" @click.stop>
                  <el-button type="text" size="small" circle>
                    <el-icon><MoreFilled /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item :command="`test-${connection.id}`">
                        <el-icon class="mr-2"><Connection /></el-icon>
                        {{ translate('test_connection') }}
                      </el-dropdown-item>
                      <el-dropdown-item :command="`edit-${connection.id}`">
                        <el-icon class="mr-2"><Edit /></el-icon>
                        {{ translate('edit_connection') }}
                      </el-dropdown-item>
                      <el-dropdown-item :command="`tunnel-${connection.id}`">
                        <el-icon class="mr-2"><Link /></el-icon>
                        {{ translate('add_tunnel') }}
                      </el-dropdown-item>
                      <el-dropdown-item
                        :command="`delete-${connection.id}`"
                        divided
                        class="text-red-600"
                      >
                        <el-icon class="mr-2"><Delete /></el-icon>
                        {{ translate('delete_connection') }}
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { ElMessageBox, ElMessage } from 'element-plus';
import { useConnectionsStore } from '../stores/connections';
import { useI18n } from '../composables/useI18n';
import type { SSHConnection, SSHTunnel } from '../types';

interface Props {
  connections: SSHConnection[];
  tunnels: SSHTunnel[];
  selectedId?: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'select-connection': [id: string];
  'add-connection': [];
  'edit-connection': [id: string];
  'delete-connection': [id: string];
  'connect': [id: string];
  'disconnect': [id: string];
  'add-tunnel': [connectionId: string];
  'start-tunnel': [id: string];
  'stop-tunnel': [id: string];
}>();

const { translate } = useI18n();
const connectionsStore = useConnectionsStore();

const getTunnels = (connectionId: string) => {
  return props.tunnels.filter(tunnel => tunnel.connectionId === connectionId);
};

const getStatusText = (status: string) => {
  const statusMap = {
    'disconnected': translate('status_disconnected'),
    'connecting': translate('status_connecting'),
    'connected': translate('status_connected'),
    'error': translate('status_error')
  };
  return statusMap[status as keyof typeof statusMap] || status;
};

const getStatusType = (status: string) => {
  const typeMap = {
    'disconnected': 'info',
    'connecting': 'warning',
    'connected': 'success',
    'error': 'danger'
  };
  return typeMap[status as keyof typeof typeMap] || 'info';
};

const getTunnelTypeText = (type: string) => {
  const typeMap = {
    'local': '本地转发',
    'remote': '远程转发',
    'dynamic': '动态转发'
  };
  return typeMap[type as keyof typeof typeMap] || type;
};

const formatTunnelConfig = (tunnel: SSHTunnel) => {
  switch (tunnel.type) {
    case 'local':
      return `本地 ${tunnel.localPort} → ${tunnel.remoteHost}:${tunnel.remotePort}`;
    case 'remote':
      return `远程 ${tunnel.localPort} → ${tunnel.remoteHost}:${tunnel.remotePort}`;
    case 'dynamic':
      return `SOCKS 代理 localhost:${tunnel.localPort}`;
    default:
      return '';
  }
};

const handleAction = async (command: string) => {
  // 修复：正确处理UUID中的连字符，只分割第一个连字符
  const dashIndex = command.indexOf('-');
  if (dashIndex === -1) return;

  const action = command.substring(0, dashIndex);
  const id = command.substring(dashIndex + 1);

  switch (action) {
    case 'test':
      await handleTestConnection(id);
      break;
    case 'edit':
      emit('edit-connection', id);
      break;
    case 'tunnel':
      emit('add-tunnel', id);
      break;
    case 'delete':
      await handleDelete(id);
      break;
  }
};

// 添加调试函数
const debugEmit = (event: string, ...args: any[]) => {
  emit(event as any, ...(args as any));
};

const handleTunnelAction = async (command: string) => {
  const [action, id] = command.split('-');

  switch (action) {
    case 'start':
      emit('start-tunnel', id);
      break;
    case 'stop':
      emit('stop-tunnel', id);
      break;
    case 'remove':
      await handleRemoveTunnel(id);
      break;
  }
};

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm(
      translate('confirm_delete_message'),
      translate('confirm_delete'),
      {
        confirmButtonText: translate('delete_connection'),
        cancelButtonText: translate('cancel'),
        type: 'warning',
      }
    );

    // 触发父组件的删除事件
    emit('delete-connection', id);
    ElMessage.success(translate('connection_deleted_successfully'));
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete connection:', error);
      ElMessage.error(`${translate('delete_connection')}失败: ${error}`);
    }
  }
};

const handleTestConnection = async (id: string) => {
  try {
    const result = await connectionsStore.testConnection(id);
    if (result.success) {
      ElMessage.success(translate('connection_test_successful'));
    } else {
      ElMessage.error(`${translate('test_connection')}失败: ${result.message}`);
    }
  } catch (error) {
    console.error('Failed to test connection:', error);
    ElMessage.error(`测试${translate('test_connection')}失败: ${error}`);
  }
};

const handleRemoveTunnel = async (id: string) => {
  try {
    await ElMessageBox.confirm(
      translate('confirm_delete_tunnel_message'),
      translate('confirm_delete'),
      {
        confirmButtonText: translate('delete_connection'),
        cancelButtonText: translate('cancel'),
        type: 'warning',
      }
    );

    await connectionsStore.removeTunnel(id);
    ElMessage.success(translate('tunnel_deleted_successfully'));
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to remove tunnel:', error);
      ElMessage.error(`删除隧道失败: ${error}`);
    }
  }
};
</script>

<style scoped>
/* Connection item styles */
.connection-item {
  @apply bg-white rounded-lg border border-gray-200 cursor-pointer transition-all duration-200;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.connection-item:hover {
  @apply border-blue-200;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.connection-item.selected {
  @apply border-blue-400 bg-blue-50;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.25);
}

.connection-item.connected {
  @apply border-green-200;
}

.connection-item.connected.selected {
  @apply border-green-400 bg-green-50;
}

.connection-item.error {
  @apply border-red-200;
}

.connection-item.error.selected {
  @apply border-red-400 bg-red-50;
}

/* List transition animation */
.list-fade-enter-active,
.list-fade-leave-active {
  transition: all 0.3s ease;
}

.list-fade-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-fade-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.list-fade-move {
  transition: transform 0.3s ease;
}

/* Dropdown menu customization */
:deep(.el-dropdown-menu__item.text-red-600) {
  color: #dc2626;
}

:deep(.el-dropdown-menu__item.text-red-600:hover) {
  background-color: #fef2f2;
  color: #dc2626;
}

/* Hover effect for action buttons */
.connection-item:hover .opacity-0 {
  opacity: 1 !important;
}
</style>