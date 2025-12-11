<template>
  <div class="drag-handle" data-tauri-drag-region></div>
</template>

<style scoped>
.drag-handle {
  height: 8px;
  width: 100%;
  background: transparent;
  cursor: move;
  -webkit-app-region: drag;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
}
</style>