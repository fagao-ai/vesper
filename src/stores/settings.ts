import { defineStore } from 'pinia';
import { ref } from 'vue';
import { sshApi } from '../services/ssh';

export interface AppSettings {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  auto_start: boolean;
  log_level: 'debug' | 'info' | 'warn' | 'error';
  default_key_path?: string;
  window_width: number;
  window_height: number;
}

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>({
    theme: 'auto',
    language: 'zh',
    auto_start: false,
    log_level: 'info',
    default_key_path: undefined,
    window_width: 1200,
    window_height: 800,
  });

  const loading = ref(false);
  const error = ref<string | null>(null);

  // Actions
  const fetchSettings = async () => {
    try {
      loading.value = true;
      error.value = null;
      const data = await sshApi.getSettings();
      settings.value = { ...settings.value, ...data };
    } catch (err) {
      error.value = err as string;
      console.error('Failed to fetch settings:', err);
    } finally {
      loading.value = false;
    }
  };

  const updateSettings = async (newSettings: Partial<AppSettings>) => {
    try {
      loading.value = true;
      error.value = null;

      const updatedSettings = { ...settings.value, ...newSettings };
      await sshApi.updateSettings(updatedSettings);
      settings.value = updatedSettings;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to update settings:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const resetSettings = async () => {
    try {
      loading.value = true;
      error.value = null;
      const defaultSettings = await sshApi.resetSettings();
      settings.value = defaultSettings;
    } catch (err) {
      error.value = err as string;
      console.error('Failed to reset settings:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const initialize = async () => {
    await fetchSettings();
  };

  return {
    // State
    settings,
    loading,
    error,

    // Actions
    fetchSettings,
    updateSettings,
    resetSettings,
    initialize,
  };
});