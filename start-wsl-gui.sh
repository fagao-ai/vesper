#!/bin/bash
# WSL GUI 环境变量设置
export DISPLAY=$DISPLAY
export LIBGL_ALWAYS_INDIRECT=1
export GDK_BACKEND=wayland
export QT_QPA_PLATFORM=wayland
export NO_AT_BRIDGE=1
# 启动应用
exec "$@"