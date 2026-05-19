# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**oh-my-file** 是一个局域网单向文件传输工具。核心功能：
- 电脑端运行 Tauri 桌面应用（显示二维码、连接地址、实时传输列表）
- 手机端扫码后在浏览器上传文件（零安装）
- 仅支持手机→电脑传输、仅限同一局域网

**Architecture:**
- **前端**: React 19 + TypeScript + Vite（位置：`src/`）
- **桌面应用**: Tauri 2.0（位置：`src-tauri/`）
- **后端**: Rust（TCP服务器、文件处理、会话管理）
- **手机网页**: 静态HTML文件上传（位置：`phone-www/`）

**核心模块（Rust）:**
- `main.rs` - Tauri命令入口、应用生命周期
- `server.rs` - TCP服务器、HTTP处理、QR码生成
- `network.rs` - IP地址获取、本地网络检测
- `transfer.rs` - 文件传输任务管理
- `session.rs` - 会话状态管理
- `dialog.rs` - 系统文件选择对话框

## Development Commands

### 前端开发
```bash
npm run dev        # 启动前端开发服务器（Vite热更新）
npm run build      # 构建前端生产版本（TypeScript + Vite）
npm run preview    # 预览构建结果
```

### Tauri应用开发
```bash
npm run tauri dev       # 启动Tauri开发模式（含前端热更新）
npm run tauri -- [cmd]  # 运行其他Tauri CLI命令
```

### 打包和清理
```bash
npm run package    # 为当前平台打包应用
npm run clean      # 清理所有构建产物和缓存

# 指定平台打包（需在目标平台运行）
node scripts/package.js win32   # Windows NSIS
node scripts/package.js darwin  # macOS DMG
node scripts/package.js linux   # Linux AppImage/DEB
```

**打包输出位置:** `src-tauri/target/release/bundle/`

## File Structure

```
oh-my-file/
├── src/                     # 前端React代码
│   ├── components/          # React组件
│   ├── hooks/               # 自定义React hooks
│   ├── styles/              # CSS样式
│   ├── types/               # TypeScript类型定义
│   └── App.tsx              # 主应用组件
├── src-tauri/               # Tauri应用和Rust后端
│   ├── src/
│   │   ├── main.rs          # Tauri命令和应用入口
│   │   ├── server.rs        # TCP/HTTP服务器实现
│   │   ├── network.rs       # 网络工具函数
│   │   ├── transfer.rs      # 文件传输状态管理
│   │   ├── session.rs       # 会话管理
│   │   └── dialog.rs        # 系统对话框
│   ├── tauri.conf.json      # Tauri配置（窗口大小、图标、打包目标）
│   ├── icons/               # 应用图标（多平台）
│   └── Cargo.toml           # Rust依赖配置
├── phone-www/               # 手机端网页文件
├── scripts/                 # 跨平台构建脚本
│   ├── dev.js               # 开发服务启动
│   ├── package.js           # 打包脚本
│   └── clean.js             # 清理脚本
├── package.json             # Node.js依赖和npm脚本
├── vite.config.ts           # Vite构建配置
└── tsconfig.json            # TypeScript配置
```

## Key Technologies

| Layer | Technology | Key Details |
|-------|-----------|------------|
| **Desktop** | Tauri 2.0 | 桌面应用框架，基于webview |
| **Frontend** | React 19, TypeScript | UI框架，类型安全 |
| **Build** | Vite 7.0, TypeScript 5.8 | 快速前端构建 |
| **Backend** | Rust, Tauri API | TCP服务器、系统交互 |
| **Dependencies** | @tauri-apps/api, @tauri-apps/cli | Tauri官方库 |

## Important Patterns

### Tauri命令（前端→后端通信）
```rust
#[tauri::command]
fn get_server_info(state: tauri::State<'_, RuntimeState>) -> ServerInfoPayload {
    server_info(&state.shared)
}
```
前端通过 `invoke('get_server_info')` 调用。

### 共享状态（Arc<Mutex>）
后端使用 `Arc<Mutex<T>>` 实现线程安全的共享状态。例如：
- `SharedState::save_directory` - 用户选择的保存目录
- `SharedState::server_config` - 服务器配置
- `SharedState::transfers` - 活跃的文件传输任务

### 文件传输流程
1. 手机上传文件 → TCP服务器接收
2. `TransferStore` 管理传输任务状态
3. `Tauri事件` 实时推送进度给前端
4. 前端显示进度条

## Multi-Platform Packaging

### 前置要求
- **Windows**: Visual Studio Build Tools 或 MSVC
- **macOS**: Xcode Command Line Tools
- **Linux**: `build-essential` (Debian/Ubuntu)

### 交叉编译限制
**不支持交叉编译** - 必须在目标平台运行打包：
```bash
# ✓ 正确
npm run package  # 当前平台

# ✗ 错误 - 在Windows上执行这些会失败
node scripts/package.js darwin
node scripts/package.js linux
```

### 自动打包（GitHub Actions）
`.github/workflows/` 配置了多平台CI/CD，使用 `tauri-action`：
- Windows runner: 构建 `.exe` / `.msi`
- macOS runner: 构建 `.dmg`
- Linux runner: 构建 `.AppImage` / `.deb`

### Icon管理
应用图标位于 `src-tauri/icons/`，需包含：
- `icon.ico` - Windows (多尺寸)
- `icon.icns` - macOS
- `icon-*.png` - Linux (32x32, 64x64, 128x128, 256x256)
- `icon.svg` - 通用矢量格式

Tauri会自动识别这些文件，无需额外配置。

## Configuration Files

### tauri.conf.json
- `productName`, `version`, `identifier` - 应用元数据
- `build.beforeDevCommand` - Tauri开发前的命令
- `build.devUrl` - 前端开发服务器地址
- `app.windows` - 窗口配置（1120×760）
- `bundle.targets` - 打包目标（当前仅Windows）

### vite.config.ts
- 前端开发服务器配置（端口5173）
- Tauri环境变量支持
- 构建目标选择（Windows: chrome105, 其他: safari13）

## Common Workflows

### 本地开发
```bash
npm install
npm run tauri dev  # 启动开发模式
# 修改src或src-tauri中的代码，应用自动重载
```

### 本地打包测试
```bash
npm run build      # 先构建前端
npm run package    # 打包当前平台
# 结果在 src-tauri/target/release/bundle/
```

### 添加新的Tauri命令
1. 在 `src-tauri/src/main.rs` 中定义 `#[tauri::command]` 函数
2. 在前端使用 `invoke('command_name', args)`
3. 记得在 `#[tauri::launch]` 中注册命令

### 修改应用配置
- 窗口大小/标题：编辑 `tauri.conf.json` 的 `app.windows`
- 打包目标：修改 `bundle.targets` 数组
- 代码签名：配置 `build` 部分（暂未实施）

## Known Limitations

- 不支持电脑→手机传输（仅单向）
- 不支持公网访问或账号鉴权
- 不支持会话持久化（重启后丢失历史）
- 跨平台打包必须在目标平台执行

## Recent Changes

- **2026-05-18**: 添加3D拟物化icon设计和Windows打包配置
- **2026-05-18**: 创建多平台打包和icon设计规格

## Resources

- [Tauri官方文档](https://tauri.app)
- [React 19文档](https://react.dev)
- [Vite官方文档](https://vitejs.dev)
- `docs/superpowers/specs/` - 设计规格文档
- `docs/superpowers/plans/` - 实现计划文档
- `scripts/README.md` - 构建脚本详细说明
