<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    :title="isEditing ? '编辑SSH连接' : '添加新的SSH连接'"
    width="700px"
    :before-close="handleClose"
    destroy-on-close
  >
    <ConnectionForm
      :connection="connection"
      @submit="handleSubmit"
      @cancel="handleCancel"
    />
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import ConnectionForm from './ConnectionForm.vue';
import type { SSHConnection } from '../types';

interface Props {
  visible: boolean;
  connection?: SSHConnection;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  submit: [connection: Omit<SSHConnection, 'id' | 'status' | 'lastConnected'>];
  cancel: [];
}>();

// 添加调试
watch(() => props.visible, (newValue) => {
  console.log('AddConnectionModal visible changed to:', newValue);
});

const isEditing = computed(() => !!props.connection);

console.log('AddConnectionModal initial props.visible:', props.visible);

const handleSubmit = async (connectionData: Omit<SSHConnection, 'id' | 'status' | 'lastConnected'>) => {
  emit('submit', connectionData);
};

const handleCancel = () => {
  emit('cancel');
};

const handleClose = () => {
  handleCancel();
};
</script>

<style scoped>
:deep(.el-dialog) {
  border-radius: 1rem;
}

:deep(.el-dialog__header) {
  padding: 1.5rem 1.5rem 0;
  border-bottom: none;
}

:deep(.el-dialog__body) {
  padding: 0 1.5rem 1.5rem;
}

:deep(.el-dialog__title) {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
}
</style>