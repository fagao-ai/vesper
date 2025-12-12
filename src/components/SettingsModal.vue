<script setup lang="ts">
import { ref, watch } from 'vue';
import { useSettingsStore, type AppSettings } from '../stores/settings';
import { useI18n } from '../composables/useI18n';

const { translate, language, setLanguage } = useI18n();

interface Props {
  visible: boolean;
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const settingsStore = useSettingsStore();

// Local form data
const formData = ref<AppSettings>({ ...settingsStore.settings });
const saving = ref(false);

// Watch for prop changes
watch(() => props.visible, (newVal) => {
  if (newVal) {
    formData.value = { ...settingsStore.settings };
  }
});

// Methods
const handleSave = async () => {
  try {
    saving.value = true;

    // 记录保存前的语言设置
    const oldLanguage = settingsStore.settings.language;

    await settingsStore.updateSettings(formData.value);

    // 如果语言设置发生变化，立即应用新语言
    if (formData.value.language !== oldLanguage) {
      setLanguage(formData.value.language as 'zh' | 'en');
    }

    emit('update:visible', false);
  } catch (error) {
    console.error('Failed to save settings:', error);
  } finally {
    saving.value = false;
  }
};

const handleReset = async () => {
  try {
    await settingsStore.resetSettings();
    formData.value = { ...settingsStore.settings };
  } catch (error) {
    console.error('Failed to reset settings:', error);
  }
};

const handleCancel = () => {
  formData.value = { ...settingsStore.settings };
  emit('update:visible', false);
};

// File picker for default key path
const handleSelectKeyPath = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      title: language.value === 'zh' ? '选择默认密钥文件' : 'Select Default Key File',
      multiple: false,
      filters: [
        {
          name: language.value === 'zh' ? '密钥文件' : 'Key Files',
          extensions: ['pem', 'key', 'ppk']
        }
      ]
    });

    if (selected && !Array.isArray(selected)) {
      formData.value.default_key_path = selected;
    }
  } catch (error) {
    console.error('Failed to select file:', error);
  }
};
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    :title="translate('settings')"
    width="600px"
    :before-close="handleCancel"
    :close-on-click-modal="false"
  >
    <div class="settings-form">
      <!-- 外观设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ translate('appearance') }}</h3>

        <div class="form-item">
          <label class="form-label">{{ translate('theme') }}</label>
          <el-select v-model="formData.theme" class="form-control">
            <el-option :label="language === 'zh' ? '浅色' : 'Light'" value="light" />
            <el-option :label="language === 'zh' ? '深色' : 'Dark'" value="dark" />
            <el-option :label="language === 'zh' ? '跟随系统' : 'Follow System'" value="auto" />
          </el-select>
        </div>

        <div class="form-item">
          <label class="form-label">{{ translate('language') }}</label>
          <el-select v-model="formData.language" class="form-control">
            <el-option :label="language === 'zh' ? '中文' : '中文'" value="zh" />
            <el-option :label="language === 'zh' ? '英语' : 'English'" value="en" />
          </el-select>
        </div>
      </div>

      <!-- 行为设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ translate('behavior') }}</h3>

        <div class="form-item">
          <label class="form-label">{{ translate('auto_start') }}</label>
          <el-switch v-model="formData.auto_start" />
        </div>

        <div class="form-item">
          <label class="form-label">{{ translate('default_key_path') }}</label>
          <div class="flex gap-2">
            <el-input
              v-model="formData.default_key_path"
              :placeholder="translate('select_file')"
              readonly
              class="flex-1"
            />
            <el-button @click="handleSelectKeyPath">{{ translate('select_file') }}</el-button>
          </div>
        </div>
      </div>

      <!-- 高级设置 -->
      <div class="settings-section">
        <h3 class="section-title">{{ translate('advanced') }}</h3>

        <div class="form-item">
          <label class="form-label">{{ translate('log_level') }}</label>
          <el-select v-model="formData.log_level" class="form-control">
            <el-option label="Debug" value="debug" />
            <el-option label="Info" value="info" />
            <el-option label="Warning" value="warn" />
            <el-option label="Error" value="error" />
          </el-select>
        </div>

        <div class="form-item">
          <label class="form-label">{{ translate('window_size') }}</label>
          <div class="flex gap-2">
            <el-input-number
              v-model="formData.window_width"
              :min="800"
              :max="1920"
              :placeholder="translate('width')"
              class="flex-1"
            />
            <span class="self-center">×</span>
            <el-input-number
              v-model="formData.window_height"
              :min="600"
              :max="1080"
              :placeholder="translate('height')"
              class="flex-1"
            />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleReset">{{ translate('reset_default') }}</el-button>
        <div class="flex-1"></div>
        <el-button @click="handleCancel">{{ translate('cancel') }}</el-button>
        <el-button
          type="primary"
          @click="handleSave"
          :loading="saving"
        >
          {{ translate('save') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.settings-form {
  max-height: 60vh;
  overflow-y: auto;
}

.settings-section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.settings-section:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 16px;
}

.form-item {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}

.form-item:last-child {
  margin-bottom: 0;
}

.form-label {
  width: 120px;
  font-weight: 500;
  color: var(--el-text-color-regular);
}

.form-control {
  flex: 1;
}

.dialog-footer {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>