import { ref, computed } from 'vue';

export type Language = 'zh' | 'en';

interface Translations {
  [key: string]: {
    [key: string]: string;
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
    height: '高度'
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
    height: 'Height'
  }
};

export function useI18n() {
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
    localStorage.setItem('language', lang);
    // 设置 HTML lang 属性
    document.documentElement.lang = lang;
  };

  // 初始化语言
  const initLanguage = () => {
    const savedLang = localStorage.getItem('language') as Language;
    if (savedLang && ['zh', 'en'].includes(savedLang)) {
      language.value = savedLang;
    }
    document.documentElement.lang = language.value;
  };

  return {
    language,
    t,
    translate,
    setLanguage,
    initLanguage
  };
}