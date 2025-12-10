<template>
  <div class="max-w-2xl mx-auto">
    <!-- Header -->
    <div class="text-center mb-8">
      <div class="inline-flex items-center justify-center w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl mb-4">
        <el-icon class="text-white text-2xl"><Connection /></el-icon>
      </div>
      <h2 class="text-2xl font-bold text-gray-800 mb-2">
        {{ isEditing ? '编辑SSH连接' : '添加新的SSH连接' }}
      </h2>
      <p class="text-gray-600">
        {{ isEditing ? '修改您的SSH连接配置' : '配置一个新的SSH服务器连接' }}
      </p>
    </div>

    <!-- Form -->
    <el-card class="card-shadow">
      <el-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-position="top"
        size="large"
        @submit.prevent="handleSubmit"
      >
        <!-- Basic Info -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <el-form-item label="连接名称" prop="name">
            <el-input
              v-model="formData.name"
              placeholder="例如: 我的生产服务器"
              :prefix-icon="Monitor"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
            />
          </el-form-item>

          <el-form-item label="主机地址" prop="host">
            <el-input
              v-model="formData.host"
              placeholder="example.com 或 192.168.1.100"
              :prefix-icon="Location"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
            />
          </el-form-item>

          <el-form-item label="端口" prop="port">
            <el-input-number
              v-model="formData.port"
              :min="1"
              :max="65535"
              class="w-full"
            />
          </el-form-item>

          <el-form-item label="用户名" prop="username">
            <el-input
              v-model="formData.username"
              placeholder="root, ubuntu, ec2-user 等"
              :prefix-icon="User"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
            />
          </el-form-item>
        </div>

        <!-- Authentication Method -->
        <el-form-item label="认证方式" prop="authMethod">
          <el-radio-group v-model="formData.authMethod" class="w-full">
            <div class="grid grid-cols-2 gap-4">
              <el-radio-button
                value="password"
                class="flex-1 text-center py-3"
                :class="formData.authMethod === 'password' ? 'bg-blue-50 border-blue-500' : ''"
              >
                <div class="flex flex-col items-center">
                  <el-icon class="text-xl mb-1"><Lock /></el-icon>
                  <span>密码认证</span>
                </div>
              </el-radio-button>

              <el-radio-button
                value="key"
                class="flex-1 text-center py-3"
                :class="formData.authMethod === 'key' ? 'bg-blue-50 border-blue-500' : ''"
              >
                <div class="flex flex-col items-center">
                  <el-icon class="text-xl mb-1"><Key /></el-icon>
                  <span>密钥认证</span>
                </div>
              </el-radio-button>
            </div>
          </el-radio-group>
        </el-form-item>

        <!-- Password Authentication -->
        <el-form-item
          v-if="formData.authMethod === 'password'"
          label="密码"
          prop="password"
        >
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="输入SSH密码"
            show-password
            :prefix-icon="Lock"
            autocomplete="new-password"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
          />
        </el-form-item>

        <!-- Key Authentication -->
        <el-form-item
          v-if="formData.authMethod === 'key'"
          label="密钥文件路径"
          prop="keyPath"
        >
          <el-input
            v-model="formData.keyPath"
            :placeholder="formData.keyPath ? '' : '~/.ssh/id_rsa 或 /path/to/private/key'"
            :prefix-icon="Key"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
          >
            <template #append>
              <el-button type="primary" @click="selectKeyFile">
                <el-icon><FolderOpened /></el-icon>
                选择文件
              </el-button>
            </template>
          </el-input>
        </el-form-item>

        <!-- Optional Notes -->
        <el-form-item label="备注（可选）">
          <el-input
            v-model="formData.notes"
            type="textarea"
            :rows="3"
            placeholder="添加一些关于此连接的备注..."
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
          />
        </el-form-item>

        <!-- Actions -->
        <div class="flex justify-end space-x-4 pt-6 border-t">
          <el-button size="large" @click="$emit('cancel')">
            取消
          </el-button>
          <el-button
            type="primary"
            size="large"
            :loading="submitting"
            @click="handleSubmit"
          >
            <el-icon class="mr-2"><Check /></el-icon>
            {{ isEditing ? '更新连接' : '添加连接' }}
          </el-button>
        </div>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue';
import { ElForm, ElMessage } from 'element-plus';
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';
import type { SSHConnection } from '../types';

interface Props {
  connection?: SSHConnection;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  submit: [connection: Omit<SSHConnection, 'id' | 'status' | 'lastConnected'>];
  cancel: [];
}>();

const formRef = ref<InstanceType<typeof ElForm>>();
const submitting = ref(false);

const isEditing = computed(() => !!props.connection);

const formData = reactive({
  name: '',
  host: '',
  port: 22,
  username: '',
  authMethod: 'password' as 'password' | 'key',
  password: '',
  keyPath: '',
  notes: ''
});

const formRules = computed(() => ({
  name: [
    { required: true, message: '请输入连接名称', trigger: 'blur' },
    { min: 2, max: 50, message: '连接名称长度应在 2 到 50 个字符之间', trigger: 'blur' }
  ],
  host: [
    { required: true, message: '请输入主机地址', trigger: 'blur' },
    { pattern: /^[a-zA-Z0-9.-]+$/, message: '请输入有效的主机地址', trigger: 'blur' }
  ],
  port: [
    { required: true, message: '请输入端口号', trigger: 'blur' },
    { type: 'number', min: 1, max: 65535, message: '端口号应在 1 到 65535 之间', trigger: 'blur' }
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 1, max: 50, message: '用户名长度应在 1 到 50 个字符之间', trigger: 'blur' }
  ],
  password: [
    {
      required: formData.authMethod === 'password',
      message: '请输入密码',
      trigger: 'blur'
    }
  ],
  keyPath: [
    {
      required: formData.authMethod === 'key',
      message: '请输入密钥文件路径',
      trigger: 'blur'
    }
  ]
}));

const selectKeyFile = async () => {
  try {
    console.log('开始选择密钥文件...');

    // 获取用户主目录，尝试打开 .ssh 目录
    const homeDirPath = await homeDir();
    let defaultPath = '';
    if (homeDirPath.includes('\\')) {
      defaultPath = `${homeDirPath}\\.ssh`;
    } else {
      defaultPath = `${homeDirPath}/.ssh`;
    }

    console.log('尝试打开默认路径:', defaultPath);

    // 文件选择对话框，不使用过滤器，允许选择所有文件
    const selected = await open({
      title: '选择SSH私钥文件',
      multiple: false
    });

    console.log('文件选择结果:', selected);

    if (selected && typeof selected === 'string') {
      // 检查是否选择了 .pub 文件，如果是则拒绝
      if (selected.endsWith('.pub')) {
        ElMessage.error('不能选择公钥文件(.pub)，请选择私钥文件');
        console.log('用户试图选择公钥文件，已拒绝:', selected);
        return;
      }

      formData.keyPath = selected;
      ElMessage.success('私钥文件选择成功');
      console.log('私钥文件路径已设置:', selected);

      // 触发表单验证以清除错误状态
      if (formRef.value) {
        formRef.value.validateField('keyPath').catch((error) => {
          // 忽略验证错误，因为此时应该是有效的
        });
      }
    } else {
      console.log('用户取消了文件选择或选择结果无效');
    }
  } catch (error) {
    console.error('选择密钥文件失败:', error);
    ElMessage.error('选择密钥文件失败，请手动输入路径');
  }
};

const handleSubmit = async () => {
  if (!formRef.value) return;

  try {
    const valid = await formRef.value.validate();
    if (!valid) return;

    submitting.value = true;

    emit('submit', {
      name: formData.name.trim(),
      host: formData.host.trim(),
      port: formData.port,
      username: formData.username.trim(),
      authMethod: formData.authMethod,
      ...(formData.authMethod === 'password'
        ? { password: formData.password }
        : { keyPath: formData.keyPath.trim() })
    });
  } catch (error) {
    console.error('Form validation failed:', error);
    ElMessage.error('表单验证失败，请检查输入');
  } finally {
    submitting.value = false;
  }
};

onMounted(() => {
  if (props.connection) {
    Object.assign(formData, {
      name: props.connection.name,
      host: props.connection.host,
      port: props.connection.port,
      username: props.connection.username,
      authMethod: props.connection.authMethod,
      password: props.connection.password || '',
      keyPath: props.connection.keyPath || '',
      notes: ''
    });
  }
});
</script>

<style scoped>
/* Form customization */
:deep(.el-form-item__label) {
  font-weight: 600;
  color: #374151;
}

:deep(.el-input__wrapper) {
  border-radius: 0.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
}

:deep(.el-input__wrapper:hover) {
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.15);
}

:deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

/* 修复输入框自动大写问题 */
:deep(.el-input__inner) {
  text-transform: none !important;
  text-transform: lowercase !important;
}

:deep(.el-textarea__inner) {
  text-transform: none !important;
  text-transform: lowercase !important;
}

/* 确保所有输入都保持原始大小写 */
:deep(input) {
  text-transform: none !important;
}

:deep(textarea) {
  text-transform: none !important;
}

:deep(.el-radio-button__inner) {
  border-radius: 0.5rem;
  padding: 0.75rem 1rem;
  font-weight: 500;
  transition: all 0.2s;
}

:deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
  border-color: #3b82f6;
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

:deep(.el-card) {
  border-radius: 1rem;
  border: none;
  box-shadow: 0 4px 25px -5px rgba(0, 0, 0, 0.1);
}

:deep(.el-button--primary) {
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
  border: none;
  border-radius: 0.5rem;
  font-weight: 600;
  padding: 0.75rem 2rem;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  transition: all 0.2s;
}

:deep(.el-button--primary:hover) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}
</style>