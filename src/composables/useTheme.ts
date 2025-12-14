import { ref, computed } from 'vue';

export type Theme = 'light' | 'dark' | 'auto';

export function useTheme() {
  const theme = ref<Theme>('auto');

  // 计算实际应用的主题
  const resolvedTheme = computed(() => {
    if (theme.value === 'auto') {
      // 检测系统主题偏好
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      return mediaQuery.matches ? 'dark' : 'light';
    }
    return theme.value;
  });

  // 应用主题到 DOM
  const applyTheme = (newTheme: Theme) => {
    const root = document.documentElement;

    if (newTheme === 'auto') {
      // 移除手动设置的主题类，让系统决定
      root.classList.remove('light', 'dark');
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      root.classList.toggle('dark', mediaQuery.matches);

      // 监听系统主题变化
      mediaQuery.addEventListener('change', (e) => {
        root.classList.toggle('dark', e.matches);
      });
    } else {
      // 应用指定主题
      root.classList.remove('light', 'dark');
      root.classList.add(newTheme);
    }
  };

  // 设置主题
  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme;
    applyTheme(newTheme);
    // 保存到 localStorage 以便下次启动时恢复
    localStorage.setItem('theme', newTheme);
  };

  // 初始化主题
  const initTheme = () => {
    // 从 localStorage 或系统默认值获取主题
    const savedTheme = localStorage.getItem('theme') as Theme;
    if (savedTheme && ['light', 'dark', 'auto'].includes(savedTheme)) {
      theme.value = savedTheme;
    }
    applyTheme(theme.value);
  };

  return {
    theme,
    resolvedTheme,
    setTheme,
    initTheme
  };
}