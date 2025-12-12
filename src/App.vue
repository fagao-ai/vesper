<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useConnectionsStore } from './stores/connections';
import { useSettingsStore } from './stores/settings';
import { useTheme } from './composables/useTheme';
import { useI18n } from './composables/useI18n';
import ConnectionList from './components/ConnectionList.vue';
import ConnectionForm from './components/ConnectionForm.vue';
import ConnectionDetail from './components/ConnectionDetail.vue';
import AddConnectionModal from './components/AddConnectionModal.vue';
import TunnelModal from './components/TunnelModal.vue';
import SettingsModal from './components/SettingsModal.vue';
import TitleBar from './components/TitleBar.vue';
import type { SSHConnection, SSHTunnel } from './types';

const connectionsStore = useConnectionsStore();
const settingsStore = useSettingsStore();
const { theme, resolvedTheme, setTheme, initTheme } = useTheme();
const { language, translate, setLanguage, initLanguage } = useI18n();

// Initialize the stores with data from the backend
onMounted(async () => {
  try {
    // 先初始化 settings store 来获取语言和主题设置
    await settingsStore.initialize();

    // 然后初始化 theme 和 language，使用 backend 的设置
    initTheme();
    await initLanguage(); // 现在是异步的

    // 确保本地状态与 backend 设置同步
    if (settingsStore.settings.theme !== theme.value) {
      setTheme(settingsStore.settings.theme);
    }
    if (settingsStore.settings.language !== language.value) {
      setLanguage(settingsStore.settings.language as 'zh' | 'en');
    }

    // 最后初始化 connections store
    await connectionsStore.initialize();
  } catch (error) {
    console.error('Failed to initialize stores:', error);
  }
});

// Watch settings changes and apply them
watch(() => settingsStore.settings.theme, (newTheme) => {
  setTheme(newTheme as 'light' | 'dark' | 'auto');
});

watch(() => settingsStore.settings.language, (newLanguage) => {
  setLanguage(newLanguage as 'zh' | 'en');
});

// UI State
const selectedConnectionId = ref<string | null>(null);
const showAddModal = ref(false);
const editingConnection = ref<SSHConnection | null>(null);
const showTunnelModal = ref(false);
const tunnelConnectionId = ref('');
const showSettingsModal = ref(false);
const leftPanelWidth = ref(240); // 默认左侧面板宽度
const isDragging = ref(false);

const selectedConnection = computed(() => {
  if (!selectedConnectionId.value) return null;
  const connection = connectionsStore.getConnectionById(selectedConnectionId.value);
  if (!connection) {
    // 如果连接不存在，清除选择状态
    selectedConnectionId.value = null;
    return null;
  }
  return connection;
});

// 拖动调整分割条
const handleMouseDown = (e: MouseEvent) => {
  isDragging.value = true;
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';

  const startX = e.clientX;
  const startWidth = leftPanelWidth.value;

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging.value) return;

    const deltaX = e.clientX - startX;
    const newWidth = startWidth + deltaX;

    // 限制最小和最大宽度
    if (newWidth >= 200 && newWidth <= 360) {
      leftPanelWidth.value = newWidth;
    }
  };

  const handleMouseUp = () => {
    isDragging.value = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

// Actions
const handleSelectConnection = (id: string) => {
  // 确保选择的连接存在
  const connection = connectionsStore.getConnectionById(id);
  if (connection) {
    selectedConnectionId.value = id;
  } else {
    selectedConnectionId.value = null;
  }
};

const handleAddConnection = () => {
  showAddModal.value = true;
  editingConnection.value = null;
};

const handleEditConnection = (id: string) => {
  const connection = connectionsStore.getConnectionById(id);
  if (connection) {
    editingConnection.value = connection;
    showAddModal.value = true;
  }
};

const handleDeleteConnection = async (id: string) => {
  try {
    await connectionsStore.removeConnection(id);

    // 如果删除的是当前选中的连接，清除选择状态
    if (selectedConnectionId.value === id) {
      selectedConnectionId.value = null;
    }
  } catch (error) {
    console.error('Failed to delete connection:', error);

    // 如果是"Connection not found"错误，说明连接已经被删除了，不需要显示错误
    const errorMessage = String(error);
    if (errorMessage.includes('Connection not found')) {
      // 更新UI状态，移除本地状态中的连接
      if (selectedConnectionId.value === id) {
        selectedConnectionId.value = null;
      }
    } else {
      alert(`删除连接失败: ${error}`);
    }
  }
};

const handleConnectionSubmit = async (connectionData: Omit<SSHConnection, 'id' | 'status' | 'lastConnected'>) => {
  try {
    if (editingConnection.value) {
      await connectionsStore.updateConnection(editingConnection.value.id, connectionData);
    } else {
      await connectionsStore.addConnection(connectionData);
    }
    showAddModal.value = false;
    editingConnection.value = null;
  } catch (error) {
    console.error('Failed to save connection:', error);
    alert(`保存连接失败: ${error}`);
  }
};

const handleModalCancel = () => {
  showAddModal.value = false;
  editingConnection.value = null;
};

const handleConnect = async (id: string) => {
  try {
    connectionsStore.updateConnection(id, { status: 'connecting' });
    const result = await connectionsStore.connectSSH(id);
    if (!result.success) {
      alert(`连接失败: ${result.message}`);
    }
  } catch (error) {
    console.error('Failed to connect:', error);
    alert(`连接失败: ${error}`);
  }
};

const handleDisconnect = async (id: string) => {
  try {
    const result = await connectionsStore.disconnectSSH(id);
    if (!result.success) {
      alert(`断开连接失败: ${result.message}`);
    }
  } catch (error) {
    console.error('Failed to disconnect:', error);
    alert(`断开连接失败: ${error}`);
  }
};

// Tunnel handlers
const handleAddTunnel = (connectionId: string) => {
  tunnelConnectionId.value = connectionId;
  showTunnelModal.value = true;
};

const handleTunnelSubmit = async (tunnelData: Omit<SSHTunnel, 'id' | 'status'>) => {
  try {
    await connectionsStore.addTunnel(tunnelData);
    showTunnelModal.value = false;
    tunnelConnectionId.value = '';
  } catch (error) {
    console.error('Failed to create tunnel:', error);
    alert(`创建隧道失败: ${error}`);
  }
};

const handleStartTunnel = async (id: string) => {
  try {
    const result = await connectionsStore.startTunnel(id);
    if (!result.success) {
      alert(`启动隧道失败: ${result.message}`);
    }
  } catch (error) {
    console.error('Failed to start tunnel:', error);
    alert(`启动隧道失败: ${error}`);
  }
};

const handleStopTunnel = async (id: string) => {
  try {
    const result = await connectionsStore.stopTunnel(id);
    if (!result.success) {
      alert(`停止隧道失败: ${result.message}`);
    }
  } catch (error) {
    console.error('Failed to stop tunnel:', error);
    alert(`停止隧道失败: ${error}`);
  }
};

const handleRemoveTunnel = async (id: string) => {
  try {
    await connectionsStore.removeTunnel(id);
  } catch (error) {
    console.error('Failed to remove tunnel:', error);
    alert(`删除隧道失败: ${error}`);
  }
};

const openGitHub = async () => {
  try {
    // 先尝试Tauri的opener插件
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl('https://github.com/fagao-ai/vesper');
  } catch (error) {
    console.warn('Tauri opener failed, falling back to window.open:', error);
    // 降级到普通的window.open
    window.open('https://github.com/fagao-ai/vesper', '_blank');
  }
};

</script>

<template>
  <div class="h-screen gradient-bg flex flex-col">
    <!-- Header -->
    <header class="glass-effect border-b border-white/20 shadow-soft z-10">
      <div class="px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
              <i class="el-icon-connection text-white text-lg"></i>
            </div>
            <div>
              <h1 class="text-xl font-bold text-gray-800">{{ translate('app_title') }}</h1>
            </div>
          </div>
          <div class="flex items-center space-x-3">
            <!-- Settings button -->
            <el-button type="text" size="default" @click="showSettingsModal = true">
              <el-icon><Setting /></el-icon>
              {{ translate('settings') }}
            </el-button>

            <!-- GitHub button -->
            <el-button type="text" size="default" @click="openGitHub">
              <el-icon><Link /></el-icon>
              {{ translate('github') }}
            </el-button>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content - Left/Right Layout -->
    <main class="flex-1 overflow-hidden">
      <!-- Loading State -->
      <div v-if="connectionsStore.loading" class="flex justify-center items-center h-full">
        <el-loading :loading="true" :text="translate('loading')" />
      </div>

      <!-- Error State -->
      <el-alert
        v-else-if="connectionsStore.error"
        :title="connectionsStore.error"
        type="error"
        show-icon
        class="mb-4"
        @close="connectionsStore.error = null"
      />

      <!-- Main Layout -->
      <div v-else class="flex h-full">
        <!-- Left Panel - Connection List -->
        <div
          class="flex flex-col bg-white border-r border-gray-200 overflow-hidden"
          :style="{ width: leftPanelWidth + 'px' }"
        >
          <!-- Panel Header -->
          <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-gray-50">
            <h2 class="text-base font-semibold text-gray-800">{{ translate('ssh_connections') }}</h2>
            <el-badge :value="connectionsStore.connections.length" type="primary">
              <el-icon class="text-gray-500"><Monitor /></el-icon>
            </el-badge>
          </div>

          <!-- Connection List -->
          <div class="flex-1 overflow-y-auto">
            <ConnectionList
              :connections="connectionsStore.connections"
              :tunnels="connectionsStore.tunnels"
              :selected-id="selectedConnectionId"
              @select-connection="handleSelectConnection"
              @add-connection="handleAddConnection"
              @edit-connection="handleEditConnection"
              @delete-connection="handleDeleteConnection"
              @connect="handleConnect"
              @disconnect="handleDisconnect"
            />
          </div>
        </div>

        <!-- Resizable Divider -->
        <div
          class="w-1 bg-gray-100 hover:bg-gray-200 cursor-col-resize transition-colors duration-150 relative"
          @mousedown="handleMouseDown"
        >
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="w-1 h-8 bg-gray-300 rounded-full hover:bg-gray-400 transition-colors duration-150"></div>
          </div>
        </div>

        <!-- Right Panel - Connection Details -->
        <div class="flex-1 flex flex-col bg-white border border-gray-200 rounded-lg shadow-sm overflow-hidden">
          <div class="flex-1 overflow-hidden">
            <ConnectionDetail
              :connection="selectedConnection"
              :tunnels="selectedConnection ? connectionsStore.tunnels.filter(t => t.connectionId === selectedConnection.id) : []"
              @add-tunnel="handleAddTunnel"
              @start-tunnel="handleStartTunnel"
              @stop-tunnel="handleStopTunnel"
              @remove-tunnel="handleRemoveTunnel"
            />
          </div>
        </div>
      </div>
    </main>

    <!-- Add/Edit Connection Modal -->
    <AddConnectionModal
      v-model:visible="showAddModal"
      :connection="editingConnection || undefined"
      @submit="handleConnectionSubmit"
      @cancel="handleModalCancel"
    />

    <!-- Add Tunnel Modal -->
    <TunnelModal
      v-model:visible="showTunnelModal"
      :connection-id="tunnelConnectionId"
      @submit="handleTunnelSubmit"
    />

    <!-- Settings Modal -->
    <SettingsModal
      v-model:visible="showSettingsModal"
    />
  </div>
</template>

<style scoped>
/* Fade transition for view changes */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>