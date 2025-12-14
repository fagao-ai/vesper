<template>
  <el-dialog
    v-model="dialogVisible"
    title="添加隧道"
    width="500px"
    :before-close="handleClose"
  >
    <div class="p-4">
      <!-- Basic Form Structure -->
      <div class="space-y-4">
        <!-- Name Field -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">隧道名称</label>
          <el-input
            v-model="formState.name"
            placeholder="请输入隧道名称"
            clearable
          />
        </div>

        <!-- Type Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">隧道类型</label>
          <el-select
            v-model="formState.type"
            placeholder="选择隧道类型"
            style="width: 100%"
          >
            <el-option
              label="本地转发 (Local)"
              value="local"
            >
              <div class="flex items-center">
                <el-icon class="mr-2"><Position /></el-icon>
                <span>本地转发 - 将远程服务端口映射到本地</span>
              </div>
            </el-option>
            <el-option
              label="远程转发 (Remote)"
              value="remote"
            >
              <div class="flex items-center">
                <el-icon class="mr-2"><Connection /></el-icon>
                <span>远程转发 - 将本地服务端口映射到远程</span>
              </div>
            </el-option>
            </el-select>
        </div>

        <!-- Port Configuration -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">本地端口</label>
          <el-input-number
            v-model="formState.localPort"
            :min="1"
            :max="65535"
            placeholder="本地端口"
            style="width: 100%"
          />
        </div>

        <!-- Remote Configuration -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">远程主机</label>
              <el-input
                v-model="formState.remoteHost"
                placeholder="localhost"
                clearable
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">远程端口</label>
              <el-input-number
                v-model="formState.remotePort"
                :min="1"
                :max="65535"
                placeholder="远程端口"
                style="width: 100%"
              />
            </div>
          </div>

        <!-- Auto Reconnect -->
        <div>
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm font-medium text-gray-700">自动重连</div>
              <div class="text-xs text-gray-500">连接断开时自动尝试重新连接</div>
            </div>
            <el-switch v-model="formState.autoReconnect" />
          </div>
        </div>

        <!-- Configuration Preview -->
        <div class="bg-gray-50 rounded-lg p-3">
          <div class="text-sm font-medium text-gray-700 mb-2">配置预览</div>
          <div class="text-xs font-mono text-gray-600">
            {{ getConfigPreview() }}
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" @click="handleSubmit">创建隧道</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue';
import { ElMessage } from 'element-plus';
import type { SSHTunnel } from '../types';

interface Props {
  visible: boolean;
  connectionId: string;  // 使用 camelCase
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  submit: [data: Omit<SSHTunnel, 'id' | 'status'>];
}>();

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

// 使用简单的响应式对象，避免复杂的watch
const formState = reactive({
  name: '',
  type: 'local' as 'local' | 'remote',
  localPort: 8080,
  remoteHost: 'localhost',
  remotePort: 80,
  autoReconnect: false,
});


const getConfigPreview = () => {
  switch (formState.type) {
    case 'local':
      return `ssh -L ${formState.localPort}:${formState.remoteHost}:${formState.remotePort} user@host`;
    case 'remote':
      return `ssh -R ${formState.localPort}:${formState.remoteHost}:${formState.remotePort} user@host`;
    default:
      return '';
  }
};

const handleSubmit = () => {
  // 基础验证
  if (!formState.name.trim()) {
    ElMessage.error('请输入隧道名称');
    return;
  }

  if (!formState.type) {
    ElMessage.error('请选择隧道类型');
    return;
  }

  if (!formState.remoteHost.trim()) {
    ElMessage.error('请输入远程主机地址');
    return;
  }

  const submitData: Omit<SSHTunnel, 'id' | 'status'> = {
    name: formState.name,
    connection_id: props.connectionId,  // 使用正确的 prop 名称
    tunnel_type: formState.type,
    local_port: formState.localPort,
    remote_host: formState.remoteHost,
    remote_port: formState.remotePort,
    auto_reconnect: formState.autoReconnect,
  };

  console.log('TunnelModal submitData:', submitData);
  console.log('props.connectionId:', props.connectionId);
  emit('submit', submitData);

  // 重置表单
  resetForm();
};

const resetForm = () => {
  formState.name = '';
  formState.type = 'local';
  formState.localPort = 8080;
  formState.remoteHost = 'localhost';
  formState.remotePort = 80;
  formState.autoReconnect = false;
};

const handleClose = () => {
  resetForm();
  emit('update:visible', false);
};
</script>

<style scoped>
.tunnel-dialog :deep(.el-select-dropdown__item) {
  height: auto;
  padding: 12px 20px;
  line-height: 1.5;
}
</style>