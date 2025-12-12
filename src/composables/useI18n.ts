import { ref, computed, watch } from 'vue';
import { useSettingsStore } from '../stores/settings';

export type Language = 'zh' | 'en';

interface Translations {
  [key: string]: {
    [key: string]: string | { [key: string]: string };
  };
}

const translations: Translations = {
  zh: {
    app_title: 'Vesper',
    ssh_connections: 'SSH 连接',
    add_connection: '添加连接',
    edit_connection: '编辑连接',
    delete_connection: '删除连接',
    connect: '连接',
    disconnect: '断开',
    settings: '设置',
    github: 'GitHub',
    connection_name: '连接名称',
    host: '主机',
    port: '端口',
    username: '用户名',
    auth_method: '认证方式',
    password: '密码',
    key_path: '密钥路径',
    tunnel_name: '隧道名称',
    tunnel_type: '隧道类型',
    local_port: '本地端口',
    remote_host: '远程主机',
    remote_port: '远程端口',
    auto_reconnect: '自动重连',
    save: '保存',
    cancel: '取消',
    delete: '删除',
    start: '启动',
    stop: '停止',
    loading: '正在加载...',
    connected: '已连接',
    disconnected: '已断开',
    error: '错误',
    success: '成功',
    theme: {
      light: '浅色',
      dark: '深色',
      auto: '跟随系统'
    },
    appearance: '外观设置',
    behavior: '行为设置',
    advanced: '高级设置',
    auto_start: '开机自启动',
    log_level: '日志级别',
    window_size: '窗口大小',
    default_key_path: '默认密钥路径',
    select_file: '选择文件',
    reset_default: '重置默认',
    width: '宽度',
    height: '高度',
    language: '语言',
    no_connections: '还没有配置任何 SSH 连接',
    add_first_connection: '添加第一个连接',
    add_connection: '添加连接',
    select_connection: '请选择一个SSH连接查看详情',
    connection_info: '连接信息',
    tunnel_management: '隧道管理',
    ssh_tunnels: 'SSH 隧道',
    auth_method_password: '密码认证',
    auth_method_key: '密钥认证',
    connecting: '连接中...',
    last_connected: '最后连接',
    connection_id: '连接ID',
    created_time: '创建时间',
    basic_info: '基本信息',
    advanced_info: '高级信息',
    host_address: '主机地址',
    auth_method: '认证方式',
    add_new_connection: '添加新连接',
    tunnel_count: '个隧道',
    test_connection: '测试连接',
    edit_connection: '编辑',
    add_tunnel: '添加隧道',
    delete_connection: '删除',
    confirm_delete: '确认删除',
    confirm_delete_message: '确定要删除这个连接吗？相关的隧道也会被删除。',
    confirm_delete_tunnel_message: '确定要删除这个隧道吗？',
    connection_deleted_successfully: '连接删除成功',
    tunnel_deleted_successfully: '隧道删除成功',
    connection_test_successful: '连接测试成功！',
    status_disconnected: '已断开',
    status_connecting: '连接中',
    status_connected: '已连接',
    status_error: '连接错误',
    username: '用户名',
    no_tunnel_config: '暂无隧道配置',
    create_first_tunnel: '创建第一个隧道',
    edit_tunnel_coming_soon: '编辑隧道功能即将推出',
    tunnel_deleted_successfully: '隧道删除成功'
  },
  en: {
    app_title: 'Vesper',
    ssh_connections: 'SSH Connections',
    add_connection: 'Add Connection',
    edit_connection: 'Edit Connection',
    delete_connection: 'Delete Connection',
    connect: 'Connect',
    disconnect: 'Disconnect',
    settings: 'Settings',
    github: 'GitHub',
    connection_name: 'Connection Name',
    host: 'Host',
    port: 'Port',
    username: 'Username',
    auth_method: 'Auth Method',
    password: 'Password',
    key_path: 'Key Path',
    tunnel_name: 'Tunnel Name',
    tunnel_type: 'Tunnel Type',
    local_port: 'Local Port',
    remote_host: 'Remote Host',
    remote_port: 'Remote Port',
    auto_reconnect: 'Auto Reconnect',
    save: 'Save',
    cancel: 'Cancel',
    delete: 'Delete',
    start: 'Start',
    stop: 'Stop',
    loading: 'Loading...',
    connected: 'Connected',
    disconnected: 'Disconnected',
    error: 'Error',
    success: 'Success',
    theme: {
      light: 'Light',
      dark: 'Dark',
      auto: 'Follow System'
    },
    appearance: 'Appearance',
    behavior: 'Behavior',
    advanced: 'Advanced',
    auto_start: 'Auto Start',
    log_level: 'Log Level',
    window_size: 'Window Size',
    default_key_path: 'Default Key Path',
    select_file: 'Select File',
    reset_default: 'Reset Default',
    width: 'Width',
    height: 'Height',
    language: 'Language',
    no_connections: 'No SSH connections configured yet',
    add_first_connection: 'Add First Connection',
    add_connection: 'Add Connection',
    select_connection: 'Please select an SSH connection to view details',
    connection_info: 'Connection Info',
    tunnel_management: 'Tunnel Management',
    ssh_tunnels: 'SSH Tunnels',
    auth_method_password: 'Password Authentication',
    auth_method_key: 'Key Authentication',
    connecting: 'Connecting...',
    last_connected: 'Last Connected',
    connection_id: 'Connection ID',
    created_time: 'Created Time',
    basic_info: 'Basic Info',
    advanced_info: 'Advanced Info',
    host_address: 'Host Address',
    auth_method: 'Auth Method',
    add_new_connection: 'Add New Connection',
    tunnel_count: ' tunnels',
    test_connection: 'Test Connection',
    edit_connection: 'Edit',
    add_tunnel: 'Add Tunnel',
    delete_connection: 'Delete',
    confirm_delete: 'Confirm Delete',
    confirm_delete_message: 'Are you sure to delete this connection? Related tunnels will also be deleted.',
    confirm_delete_tunnel_message: 'Are you sure to delete this tunnel?',
    connection_deleted_successfully: 'Connection deleted successfully',
    tunnel_deleted_successfully: 'Tunnel deleted successfully',
    connection_test_successful: 'Connection test successful!',
    status_disconnected: 'Disconnected',
    status_connecting: 'Connecting',
    status_connected: 'Connected',
    status_error: 'Connection Error',
    username: 'Username',
    no_tunnel_config: 'No tunnel configuration',
    create_first_tunnel: 'Create First Tunnel',
    edit_tunnel_coming_soon: 'Edit tunnel feature coming soon',
    tunnel_deleted_successfully: 'Tunnel deleted successfully'
  }
};

export function useI18n() {
  const settingsStore = useSettingsStore();
  const language = ref<Language>('zh');

  // 获取翻译文本
  const t = computed(() => {
    return translations[language.value];
  });

  // 获取特定的翻译
  const translate = (key: string): string => {
    const keys = key.split('.');
    let value: any = t.value;
    for (const k of keys) {
      value = value?.[k];
    }
    // 如果 value 是对象，返回 key
    if (typeof value === 'object' && value !== null) {
      return key;
    }
    return String(value || key);
  };

  // 设置语言
  const setLanguage = (lang: Language) => {
    language.value = lang;
    // 设置 HTML lang 属性
    document.documentElement.lang = lang;
  };

  // 初始化语言 - 从 backend 设置读取
  const initLanguage = async () => {
    // 首先尝试从 localStorage 读取（作为后备）
    const savedLang = localStorage.getItem('language') as Language;
    if (savedLang && ['zh', 'en'].includes(savedLang)) {
      language.value = savedLang;
    }

    // 然后从 backend 设置读取并覆盖
    try {
      await settingsStore.fetchSettings();
      const backendLang = settingsStore.settings.language as Language;
      if (backendLang && ['zh', 'en'].includes(backendLang)) {
        language.value = backendLang;
      }
    } catch (error) {
      console.warn('Failed to load language from backend settings:', error);
    }

    document.documentElement.lang = language.value;
  };

  // 监听 backend 设置的变化并同步语言
  watch(() => settingsStore.settings.language, (newLanguage) => {
    if (newLanguage && ['zh', 'en'].includes(newLanguage)) {
      setLanguage(newLanguage as Language);
      // 同步更新 localStorage 作为后备
      localStorage.setItem('language', newLanguage);
    }
  });

  return {
    language,
    t,
    translate,
    setLanguage,
    initLanguage
  };
}