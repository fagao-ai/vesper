<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useConnectionsStore } from './stores/connections';
import { useSettingsStore } from './stores/settings';
import { useTheme } from './composables/useTheme';
import { useI18n } from './composables/useI18n';
import ConnectionList from './components/ConnectionList.vue';
import ConnectionDetail from './components/ConnectionDetail.vue';
import AddConnectionModal from './components/AddConnectionModal.vue';
import TunnelModal from './components/TunnelModal.vue';
import EditTunnelModal from './components/EditTunnelModal.vue';
import SettingsModal from './components/SettingsModal.vue';
import type { SSHConnection, SSHTunnel } from './types';

const connectionsStore = useConnectionsStore();
const settingsStore = useSettingsStore();
const { setTheme, initTheme } = useTheme();
const { translate, setLanguage, initLanguage } = useI18n();

// Initialize the stores with data from the backend
onMounted(async () => {
  try {
    // 先初始化 settings store 来获取语言和主题设置
    await settingsStore.initialize();

    // 确保使用从 backend 获取的语言设置
    const backendLanguage = settingsStore.settings.language as 'zh' | 'en';
    console.log('Backend language:', backendLanguage);

    // 初始化 theme，language 已经在 useI18n 中从 localStorage 初始化了
    initTheme();
    // 只需要同步 backend 设置（如果不同于当前的）
    await initLanguage(backendLanguage);

    // 设置主题
    setTheme(settingsStore.settings.theme);

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
const showEditTunnelModal = ref(false);
const editingTunnel = ref<SSHTunnel | null>(null);
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


const handleEditTunnel = (tunnel: SSHTunnel) => {
  editingTunnel.value = tunnel;
  showEditTunnelModal.value = true;
};

const handleEditTunnelSubmit = async (tunnelData: SSHTunnel) => {
  try {
    if (!tunnelData.id) {
      throw new Error('隧道ID不能为空');
    }

    await connectionsStore.updateTunnel(tunnelData.id, {
      name: tunnelData.name,
      tunnel_type: tunnelData.tunnel_type,
      local_port: tunnelData.local_port,
      remote_host: tunnelData.remote_host,
      remote_port: tunnelData.remote_port,
      auto_reconnect: tunnelData.auto_reconnect
    });

    showEditTunnelModal.value = false;
    editingTunnel.value = null;
  } catch (error) {
    console.error('Failed to update tunnel:', error);
    alert(`更新隧道失败: ${error}`);
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
            <el-button type="text" size="default" @click="showSettingsModal = true" class="settings-btn">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" class="settings-icon">
                <path d="M12 15.5A3.5 3.5 0 0 1 8.5 12A3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5a3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97c0-.33-.03-.65-.07-.97l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65A.488.488 0 0 0 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.23-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.97c0 .33.03.65.07.97l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.23.09.49 0 .61-.22l2-3.46c.13-.22.07-.49-.12-.64l-2.11-1.65Z" fill="currentColor"/>
              </svg>
            </el-button>

            <!-- GitHub button -->
            <el-button type="text" size="default" @click="openGitHub" class="github-btn">
              <svg width="20" height="20" viewBox="0 -0.5 25 25" fill="currentColor" class="github-icon">
                <path d="m12.301 0h.093c2.242 0 4.34.613 6.137 1.68l-.055-.031c1.871 1.094 3.386 2.609 4.449 4.422l.031.058c1.04 1.769 1.654 3.896 1.654 6.166 0 5.406-3.483 10-8.327 11.658l-.087.026c-.063.02-.135.031-.209.031-.162 0-.312-.054-.433-.144l.002.001c-.128-.115-.208-.281-.208-.466 0-.005 0-.01 0-.014v.001q0-.048.008-1.226t.008-2.154c.007-.075.011-.161.011-.249 0-.792-.323-1.508-.844-2.025.618-.061 1.176-.163 1.718-.305l-.076.017c.573-.16 1.073-.373 1.537-.642l-.031.017c.508-.28.938-.636 1.292-1.058l.006-.007c.372-.476.663-1.036.84-1.645l.009-.035c.209-.683.329-1.468.329-2.281 0-.045 0-.091-.001-.136v.007c0-.022.001-.047.001-.072 0-1.248-.482-2.383-1.269-3.23l.003.003c.168-.44.265-.948.265-1.479 0-.649-.145-1.263-.404-1.814l.011.026c-.115-.022-.246-.035-.381-.035-.334 0-.649.078-.929.216l.012-.005c-.568.21-1.054.448-1.512.726l.038-.022-.609.384c-.922-.264-1.981-.416-3.075-.416s-2.153.152-3.157.436l.081-.02q-.256-.176-.681-.433c-.373-.214-.814-.421-1.272-.595l-.066-.022c-.293-.154-.64-.244-1.009-.244-.124 0-.246.01-.364.03l.013-.002c-.248.524-.393 1.139-.393 1.788 0 .531.097 1.04.275 1.509l-.01-.029c-.785.844-1.266 1.979-1.266 3.227 0 .025 0 .051.001.076v-.004c-.001.039-.001.084-.001.13 0 .809.12 1.591.344 2.327l-.015-.057c.189.643.476 1.202.85 1.693l-.009-.013c.354.435.782.793 1.267 1.062l.022.011c.432.252.933.465 1.46.614l.046.011c.466.125 1.024.227 1.595.284l.046.004c-.431.428-.718 1-.784 1.638l-.001.012c-.207.101-.448.183-.699.236l-.021.004c-.256.051-.549.08-.85.08-.022 0-.044 0-.066 0h.003c-.394-.008-.756-.136-1.055-.348l.006.004c-.371-.259-.671-.595-.881-.986l-.007-.015c-.198-.336-.459-.614-.768-.827l-.009-.006c-.225-.169-.49-.301-.776-.38l-.016-.004-.32-.048c-.023-.002-.05-.003-.077-.003-.14 0-.273.028-.394.077l.007-.003q-.128.072-.08.184c.039.086.087.16.145.225l-.001-.001c.061.072.13.135.205.19l.003.002.112.08c.283.148.516.354.693.603l.004.006c.191.237.359.505.494.792l.01.024.16.368c.135.402.38.738.7.981l.005.004c.3.234.662.402 1.057.478l.016.002c.33.064.714.104 1.106.112h.007c.045.002.097.002.15.002.261 0 .517-.021.767-.062l-.027.004.368-.064q0 .609.008 1.418t.008.873v.014c0 .185-.08.351-.208.466h-.001c-.119.089-.268.143-.431.143-.075 0-.147-.011-.214-.032l.005.001c-4.929-1.689-8.409-6.283-8.409-11.69 0-2.268.612-4.393 1.681-6.219l-.032.058c1.094-1.871 2.609-3.386 4.422-4.449l.058-.031c1.739-1.034 3.835-1.645 6.073-1.645h.098-.005zm-7.64 17.666q.048-.112-.112-.192-.16-.048-.208.032-.048.112.112.192.144.096.208-.032zm.497.545q.112-.08-.032-.256-.16-.144-.256-.048-.112.08.032.256.159.157.256.047zm.48.72q.144-.112 0-.304-.128-.208-.272-.096-.144.08 0 .288t.272.112zm.672.673q.128-.128-.064-.304-.192-.192-.32-.048-.144.128.064.304.192.192.32.044zm.913.4q.048-.176-.208-.256-.24-.064-.304.112t.208.24q.24.097.304-.096zm1.009.08q0-.208-.272-.176-.256 0-.256.176 0 .208.272.176.256.001.256-.175zm.929-.16q-.032-.176-.288-.144-.256.048-.224.24t.288.128.225-.224z"/>
              </svg>
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
              :tunnels="selectedConnection ? connectionsStore.tunnels.filter(t => t.connection_id === selectedConnection!.id) : []"
              @connect="handleConnect"
              @disconnect="handleDisconnect"
              @add-tunnel="handleAddTunnel"
              @edit-tunnel="handleEditTunnel"
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

    <!-- Edit Tunnel Modal -->
    <EditTunnelModal
      v-model:visible="showEditTunnelModal"
      :tunnel="editingTunnel"
      @submit="handleEditTunnelSubmit"
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

/* Settings button styling */
.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  min-width: auto;
}

.settings-btn .settings-icon {
  color: #606266;
  transition: color 0.2s ease;
}

.settings-btn:hover .settings-icon {
  color: #409eff;
}

/* GitHub button styling */
.github-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  min-width: auto;
}

.github-btn .github-icon {
  color: #606266;
  transition: color 0.2s ease;
}

.github-btn:hover .github-icon {
  color: #409eff;
}
</style>