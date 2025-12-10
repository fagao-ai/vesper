<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div v-if="connection" class="p-6 border-b border-gray-100 bg-gradient-to-r from-blue-50 to-purple-50">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center">
            <el-icon class="text-white text-xl"><Monitor /></el-icon>
          </div>
          <div>
            <h3 class="text-xl font-bold text-gray-800">{{ connection.name }}</h3>
            <div class="flex items-center space-x-3 mt-1">
              <el-tag
                :type="getStatusType(connection.status)"
                size="small"
                effect="light"
              >
                <el-icon class="mr-1" v-if="connection.status === 'connected'"><Check /></el-icon>
                <el-icon class="mr-1" v-else-if="connection.status === 'connecting'"><Loading /></el-icon>
                <el-icon class="mr-1" v-else-if="connection.status === 'error'"><Close /></el-icon>
                {{ getStatusText(connection.status) }}
              </el-tag>
              <span class="text-sm text-gray-500">{{ connection.authMethod === 'password' ? '密码认证' : '密钥认证' }}</span>
            </div>
          </div>
        </div>

        <div class="flex items-center space-x-3">
          <el-button
            v-if="connection.status === 'disconnected' || connection.status === 'error'"
            type="success"
            size="default"
            @click="$emit('connect', connection.id)"
          >
            <el-icon class="mr-1"><VideoPlay /></el-icon>
            连接
          </el-button>
          <el-button
            v-else-if="connection.status === 'connected'"
            type="danger"
            size="default"
            @click="$emit('disconnect', connection.id)"
          >
            <el-icon class="mr-1"><VideoPause /></el-icon>
            断开
          </el-button>
          <el-button
            v-else-if="connection.status === 'connecting'"
            loading
            disabled
          >
            连接中...
          </el-button>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="flex-1 flex items-center justify-center">
      <el-empty description="请选择一个SSH连接查看详情" />
    </div>

    <!-- Content -->
    <div v-if="connection" class="flex-1 overflow-y-auto">
      <el-tabs v-model="activeTab" class="h-full flex flex-col">
        <el-tab-pane label="连接信息" name="info">
          <div class="p-6 space-y-6">
            <!-- Basic Info -->
            <div>
              <h4 class="text-base font-semibold text-gray-800 mb-4 flex items-center">
                <el-icon class="mr-2 text-blue-500"><InfoFilled /></el-icon>
                基本信息
              </h4>
              <div class="bg-gray-50 rounded-lg p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <span class="text-gray-600 flex items-center">
                    <el-icon class="mr-2"><Location /></el-icon>
                    主机地址
                  </span>
                  <span class="font-mono text-sm font-medium">{{ connection.host }}:{{ connection.port }}</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-gray-600 flex items-center">
                    <el-icon class="mr-2"><User /></el-icon>
                    用户名
                  </span>
                  <span class="font-mono text-sm font-medium">{{ connection.username }}</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-gray-600 flex items-center">
                    <el-icon class="mr-2"><Key /></el-icon>
                    认证方式
                  </span>
                  <span class="text-sm font-medium">{{ connection.authMethod === 'password' ? '密码认证' : '密钥认证' }}</span>
                </div>
                <div v-if="connection.lastConnected" class="flex items-center justify-between">
                  <span class="text-gray-600 flex items-center">
                    <el-icon class="mr-2"><Clock /></el-icon>
                    最后连接
                  </span>
                  <span class="text-sm font-medium">{{ formatDate(connection.lastConnected) }}</span>
                </div>
              </div>
            </div>

            <!-- Advanced Info -->
            <div>
              <h4 class="text-base font-semibold text-gray-800 mb-4 flex items-center">
                <el-icon class="mr-2 text-purple-500"><Setting /></el-icon>
                高级信息
              </h4>
              <div class="bg-gray-50 rounded-lg p-4">
                <div class="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span class="text-gray-600">连接ID:</span>
                    <span class="ml-2 font-mono font-medium">{{ connection.id.slice(0, 8) }}...</span>
                  </div>
                  <div>
                    <span class="text-gray-600">创建时间:</span>
                    <span class="ml-2 font-medium">{{ formatDate(connection.createdAt) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <el-tab-pane name="tunnels">
          <template #label>
            <span class="flex items-center">
              隧道管理
              <el-badge v-if="tunnels.length > 0" :value="tunnels.length" class="ml-2" />
            </span>
          </template>

          <div class="p-6">
            <!-- Tunnels Header -->
            <div class="flex items-center justify-between mb-6">
              <h4 class="text-base font-semibold text-gray-800 flex items-center">
                <el-icon class="mr-2 text-green-500"><Link /></el-icon>
                SSH 隧道
              </h4>
              <el-button type="primary" size="small" @click="$emit('add-tunnel', connection.id)">
                <el-icon class="mr-1"><Plus /></el-icon>
                添加隧道
              </el-button>
            </div>

            <!-- Tunnels List -->
            <div v-if="tunnels.length === 0" class="text-center py-12">
              <el-icon class="text-6xl text-gray-300 mb-4"><Link /></el-icon>
              <p class="text-gray-500">暂无隧道配置</p>
              <el-button type="text" @click="$emit('add-tunnel', connection.id)">创建第一个隧道</el-button>
            </div>

            <div v-else class="space-y-4">
              <div
                v-for="tunnel in tunnels"
                :key="tunnel.id"
                class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition-all duration-200"
                :class="{
                  'border-green-200 bg-green-50': tunnel.status === 'active'
                }"
              >
                <div class="flex items-center justify-between">
                  <div class="flex-1">
                    <div class="flex items-center space-x-3 mb-2">
                      <h5 class="font-medium text-gray-800">{{ tunnel.name }}</h5>
                      <el-tag
                        :type="tunnel.status === 'active' ? 'success' : 'info'"
                        size="small"
                      >
                        {{ tunnel.status === 'active' ? '运行中' : '已停止' }}
                      </el-tag>
                      <el-tag size="small" effect="plain">
                        {{ getTunnelTypeText(tunnel.type) }}
                      </el-tag>
                    </div>
                    <div class="text-sm text-gray-600 font-mono">
                      {{ formatTunnelConfig(tunnel) }}
                    </div>
                  </div>

                  <div class="flex items-center space-x-2">
                    <el-button
                      v-if="tunnel.status === 'inactive'"
                      type="success"
                      size="small"
                      @click="$emit('start-tunnel', tunnel.id)"
                    >
                      <el-icon><VideoPlay /></el-icon>
                    </el-button>
                    <el-button
                      v-else-if="tunnel.status === 'active'"
                      type="warning"
                      size="small"
                      @click="$emit('stop-tunnel', tunnel.id)"
                    >
                      <el-icon><VideoPause /></el-icon>
                    </el-button>
                    <el-dropdown trigger="click" @command="handleTunnelAction">
                      <el-button type="text" size="small">
                        <el-icon><MoreFilled /></el-icon>
                      </el-button>
                      <template #dropdown>
                        <el-dropdown-menu>
                          <el-dropdown-item :command="`edit-${tunnel.id}`">
                            <el-icon class="mr-2"><Edit /></el-icon>
                            编辑
                          </el-dropdown-item>
                          <el-dropdown-item :command="`remove-${tunnel.id}`" divided class="text-red-600">
                            <el-icon class="mr-2"><Delete /></el-icon>
                            删除
                          </el-dropdown-item>
                        </el-dropdown-menu>
                      </template>
                    </el-dropdown>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { ElMessageBox, ElMessage } from 'element-plus';
import type { SSHConnection, SSHTunnel } from '../types';

interface Props {
  connection: SSHConnection | null;
  tunnels: SSHTunnel[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  connect: [id: string];
  disconnect: [id: string];
  'add-tunnel': [connectionId: string];
  'start-tunnel': [id: string];
  'stop-tunnel': [id: string];
  'remove-tunnel': [id: string];
}>();

const activeTab = ref('info');

const getStatusText = (status: string) => {
  const statusMap = {
    'disconnected': '已断开',
    'connecting': '连接中',
    'connected': '已连接',
    'error': '连接错误'
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

const formatDate = (dateString?: string) => {
  if (!dateString) return '从未';
  try {
    return new Date(dateString).toLocaleString('zh-CN');
  } catch {
    return '无效日期';
  }
};

const handleTunnelAction = async (command: string) => {
  const [action, id] = command.split('-');

  switch (action) {
    case 'edit':
      // TODO: Implement edit tunnel functionality
      ElMessage.info('编辑隧道功能即将推出');
      break;
    case 'remove':
      await handleRemoveTunnel(id);
      break;
  }
};

const handleRemoveTunnel = async (id: string) => {
  try {
    await ElMessageBox.confirm(
      '确定要删除这个隧道吗？',
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    emit('remove-tunnel', id);
    ElMessage.success('隧道删除成功');
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to remove tunnel:', error);
      ElMessage.error(`删除隧道失败: ${error}`);
    }
  }
};
</script>

<style scoped>
:deep(.el-tabs__content) {
  flex: 1;
  overflow: auto;
}

:deep(.el-tab-pane) {
  height: 100%;
}
</style>