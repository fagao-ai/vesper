<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

// 窗口控制函数
const appWindow = getCurrentWindow();
const minimizeWindow = async () => {
  await appWindow.minimize();
};

const maximizeWindow = async () => {
  const maximized = await appWindow.isMaximized();
  if (maximized) {
    await appWindow.unmaximize();
  } else {
    await appWindow.maximize();
  }
};

const closeWindow = async () => {
  await appWindow.close();
};
</script>

<template>
  <div class="title-bar" data-tauri-drag-region>
    <div class="window-title">Vesper</div>
    <div class="drag-area" data-tauri-drag-region></div>
    <div class="window-controls">
      <button class="window-button minimize" @click="minimizeWindow">
        <svg viewBox="0 0 24 24" width="10" height="10">
          <line x1="2" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>

      <button class="window-button maximize" @click="maximizeWindow">
        <svg viewBox="0 0 24 24" width="10" height="10">
          <rect x="2" y="2" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>

      <button class="window-button close" @click="closeWindow">
        <svg viewBox="0 0 24 24" width="10" height="10">
          <line x1="2" y1="2" x2="22" y2="22" stroke="currentColor" stroke-width="2"/>
          <line x1="22" y1="2" x2="2" y2="22" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  height: 24px;
  width: 100%;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  -webkit-app-region: drag;
  background: rgba(249, 250, 251, 0.98);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(229, 231, 235, 0.8);
}

.window-title {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  font-size: 14px;
  font-weight: 500;
  color: #333;
  pointer-events: none;
}

.drag-area {
  position: absolute;
  top: 0;
  left: 12px; /* 左侧留出一点空间 */
  right: 120px; /* 留出窗口控制按钮的空间 */
  bottom: 0;
  -webkit-app-region: drag;
}

.window-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.window-button {
  width: 40px;
  height: 24px;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #666;
  transition: all 0.15s ease;
}

.window-button:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #333;
}

.window-button.close:hover {
  background: #e81123;
  color: white;
}

.window-button svg {
  width: 10px;
  height: 10px;
}
</style>