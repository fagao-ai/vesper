# Vesper - SSH隧道管理器

![Tauri](https://img.shields.io/badge/Tauri-FFC131?logo=tauri&logoColor=black)
![Vue.js](https://img.shields.io/badge/Vue.js-4FC08D?logo=vuedotjs&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)

Vesper 是一个现代化的 SSH 隧道管理器，提供直观的界面来管理和配置 SSH 连接与隧道。基于 Tauri 框架构建，具有跨平台、高性能和轻量级的特点。

## 🎯 当前状态

**已完成核心功能：**
- ✅ SSH 连接的增删改查
- ✅ 支持密码和密钥认证
- ✅ 隧道管理（本地/远程/动态转发）
- ✅ 隧道的完整生命周期管理（创建、编辑、删除、启动、停止）
- ✅ 数据持久化存储
- ✅ 实时状态监控
## ✨ 特性

### SSH 连接管理
- [x] 支持 SSH 密钥和密码认证
- [x] 多服务器配置管理
- [x] 连接状态实时监控
- [ ] 连接历史记录
- [ ] 服务器分组和标签

### 隧道管理
- [x] 本地端口转发 (Local Port Forwarding)
- [x] 远程端口转发 (Remote Port Forwarding)
- [ ] 动态端口转发 (Dynamic Port Forwarding/SOCKS)
- [x] 隧道 CRUD 操作（创建、查看、编辑、删除）
- [x] 隧道状态管理（启动/停止状态切换）
- [x] 实际隧道连接建立与关闭
- [ ] 隧道配置文件导入/导出
- [ ] 隧道批量操作

### 用户界面
- [x] 现代化的 Vue 3 + TypeScript 界面
- [x] 深色/浅色主题
- [x] 响应式设计
- [ ] 实时日志查看器
- [x] 连接状态可视化

### 高级功能
- [x] 隧道自动重连配置
- [x] 配置文件本地存储
- [ ] 快捷命令生成
- [ ] 性能监控
- [ ] 多窗口支持
## 📦 安装

### 下载预构建版本
从 [Releases 页面](https://github.com/fagao-ai/vesper/releases) 下载对应平台的安装包：

#### macOS
- **Intel 芯片**: `Vesper-{version}-macOS-x86_64.dmg` - 适用于 Intel Mac (2015-2020)
- **Apple Silicon**: `Vesper-{version}-macOS-arm64.dmg` - 适用于 Apple Silicon Mac (M1/M2/M3)

#### Windows
- `Vesper-{version}-Windows.msi` - 适用于 Windows 10/11

#### Linux
- `Vesper-{version}-Linux.AppImage` - 通用可执行文件
- `Vesper-{version}-Linux.deb` - Debian/Ubuntu 系发行版

### 从源码构建

#### 先决条件
- Node.js 18+ 和 npm/pnpm
- Rust 1.70+

#### 系统依赖
- **Windows**: 安装 WebView2
- **macOS**: Xcode 命令行工具 + OpenSSL
- **Linux**: WebKit2GTK, libayatana-appindicator, libssl-dev, 及其他依赖

#### 构建步骤
```bash
# 克隆仓库
git clone https://github.com/fagao-ai/vesper.git
cd vesper

# 安装依赖（推荐使用 pnpm）
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建应用
pnpm tauri build

# 构建特定平台版本
pnpm tauri build --target x86_64-apple-darwin    # macOS Intel
pnpm tauri build --target aarch64-apple-darwin    # macOS Apple Silicon
```

### GitHub Actions 自动构建

项目使用 GitHub Actions 进行自动化构建和发布：

#### 构建矩阵
- **Windows**: `windows-2022` - 构建 Windows MSI 安装包
- **Linux**: `ubuntu-22.04` - 构建 Linux AppImage 和 DEB 包
- **macOS Intel**: `macos-15-intel` - 在 Intel runner 上构建 x86_64 版本
- **macOS Apple Silicon**: `macos-14` - 通过交叉编译构建 arm64 版本

#### 构建流程
1. **并行构建**: 所有平台的构建任务并行执行
2. **独立构建**: 每个平台的构建过程完全独立，避免架构冲突
3. **自动发布**: 成功构建后自动创建 GitHub Release 并上传所有构建产物
4. **版本管理**: 基于 Git 标签自动创建版本发布

#### 发布产物
- `Vesper-{version}-macOS-x86_64.dmg` - macOS Intel 版本
- `Vesper-{version}-macOS-arm64.dmg` - macOS Apple Silicon 版本
- `Vesper-{version}-Windows.msi` - Windows 安装包
- `Vesper-{version}-Linux.AppImage` - Linux AppImage
- `Vesper-{version}-Linux.deb` - Linux DEB 包
## 🚀 快速开始

### 添加 SSH 服务器
1. 点击 "添加服务器" 按钮
2. 填写主机名、端口、用户名
3. 选择认证方式（密码或密钥）

### 创建隧道
1. 选择目标服务器
2. 点击 "添加隧道"
3. 配置隧道类型和端口映射
4. 保存并启动

### 管理连接
- 在仪表板查看所有连接状态
- 点击连接/断开按钮控制隧道
- 查看实时日志
## 📁 项目结构

```
vesper/
├── src-tauri/           # Tauri 后端
│   ├── src/
│   │   ├── commands.rs  # Tauri 命令
│   │   ├── ssh.rs       # SSH 连接逻辑
│   │   └── main.rs      # 主入口
│   ├── icons/           # 应用图标
│   ├── Cargo.toml       # Rust 依赖
│   └── tauri.conf.json  # Tauri 配置
├── src/                 # 前端源码
│   ├── components/      # Vue 组件
│   ├── views/          # 页面视图
│   ├── stores/        # Pinia 状态管理
│   ├── types/          # TypeScript 类型定义
│   ├── utils/          # 工具函数
│   ├── assets/         # 静态资源
│   ├── main.ts         # 前端入口
│   └── App.vue         # 根组件
├── public/             # 公共资源
├── index.html
├── package.json        # 前端依赖
├── tsconfig.json       # TypeScript 配置
├── vite.config.ts      # Vite 配置
├── CLAUDE.md           # Claude Code 开发指南
└── README.md
```
## 🛠️ 技术栈

### 前端
- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - 类型安全的 JavaScript
- **Vite** - 下一代前端构建工具
- **Pinia** - Vue 状态管理
- **Element Plus** - Vue 3 组件库
- **VueUse** - Vue 组合式 API 工具集

### 后端
- **Tauri** - 构建小型、快速的跨平台应用
- **Rust** - 系统编程语言
- **russh** - Rust SSH 客户端库
- **serde** - Rust 序列化框架
- **tokio** - Rust 异步运行时

### 开发工具
- **TypeScript** - 类型检查 (`vue-tsc`)
- **Vite** - 快速开发服务器和构建工具
- **Tailwind CSS** - 实用优先的 CSS 框架
- **PostCSS** - CSS 转换工具
- **pnpm** - 快速、节省磁盘空间的包管理器

## 🔧 配置

### 应用配置
配置文件位置：
- Windows: `%APPDATA%/vesper/config.json`
- macOS: `~/Library/Application Support/vesper/config.json`
- Linux: `~/.config/vesper/config.json`

示例配置：
```json
{
  "theme": "dark",
  "language": "zh-CN",
  "autoStart": false,
  "logLevel": "info",
  "defaultKeyPath": "~/.ssh/id_rsa"
}
```


## 🔄 持续集成/持续部署 (CI/CD)

### GitHub Actions 工作流

项目配置了完整的 CI/CD 流水线，位于 `.github/workflows/` 目录：

#### `release.yml` - 自动发布工作流
- **触发条件**: 推送以 `v` 开头的标签 (如 `v1.0.0`)
- **构建策略**: 使用矩阵策略并行构建多个平台
- **自动发布**: 成功构建后自动创建 GitHub Release

#### 构建环境配置
```yaml
strategy:
  matrix:
    include:
      - os: windows-2022
      - os: ubuntu-22.04
      - os: macos-15-intel    # Intel Mac 原生构建
        arch: x86_64
      - os: macos-14          # Apple Silicon 交叉编译
        arch: arm64
```

#### 关键依赖管理
- **macOS**: 使用 `openssl@3` 处理 SSH 相关依赖
- **Linux**: 安装完整的 GTK 和 WebKit 开发环境
- **Windows**: 使用 MSVC 工具链

### 开发与发布流程

1. **开发阶段**
   ```bash
   # 创建功能分支
   git checkout -b feature/new-feature

   # 开发和测试
   pnpm tauri dev

   # 提交更改
   git commit -m "Add new feature"
   git push origin feature/new-feature
   ```

2. **发布版本**
   ```bash
   # 合并到 main 分支后，创建版本标签
   git tag v1.0.0
   git push origin v1.0.0

   # GitHub Actions 会自动触发构建和发布
   ```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何开始。

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 🐛 问题反馈

请使用 [GitHub Issues](https://github.com/fagao-ai/vesper/issues) 报告 bug 或提出功能建议。

## 📈 开发路线图

- [x] v1.0.0: 正式发布
  - ✅ 基础 SSH 连接管理
  - ✅ 隧道管理功能
  - ✅ 深色/浅色主题
  - ✅ 配置文件本地存储
  - ✅ 跨平台支持（Windows、macOS、Linux）

### 未来版本计划
- [ ] v1.1.0: 增强功能
  - [ ] 配置文件导入/导出
  - [ ] 连接历史记录
  - [ ] 服务器分组和标签
- [ ] v1.2.0: 高级功能
  - [ ] 动态端口转发 (SOCKS)
  - [ ] 快捷命令生成
  - [ ] 实时日志查看器
- [ ] v2.0.0: 专业版功能
  - [ ] 高级功能（代理链、跳板机）
  - [ ] 性能监控
  - [ ] 多窗口支持

## 🙏 致谢

- [Tauri](https://tauri.app/) - 提供优秀的跨平台应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- 所有贡献者和用户

## 📞 支持

- 📧 邮箱：support@vesper.app
- 💬 Discord 社区
- 📖 文档网站
