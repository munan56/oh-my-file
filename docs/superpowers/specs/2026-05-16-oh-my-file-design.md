# oh-my-file 设计文档

## 概述

oh-my-file 是一个局域网文件传输工具。桌面端运行 Tauri 应用，手机扫码连接后通过浏览器上传文件到电脑。

## 目标平台

- **桌面端**：Windows / macOS / Linux / 麒麟OS（通过 Tauri 跨平台支持）
- **手机端**：任意系统（通过浏览器访问）

## 架构

```
┌─────────────────────────────────────┐      ┌──────────────────┐
│         Tauri Desktop App           │      │   Phone Browser   │
│                                     │      │                  │
│  ┌───────────────────────────────┐  │      │  ┌────────────┐  │
│  │   Tauri Frontend (React)      │  │      │  │ Upload Web │  │
│  │   - 传输列表与进度             │  │      │  │ Page       │  │
│  │   - 文件保存目录选择          │  │      │  │ -选择文件   │  │
│  │   - 二维码展示                │◄─┤ Tauri │  │ -上传进度  │  │
│  │   - 状态栏                    │  │Event  │  └─────┬──────┘  │
│  └───────────────────────────────┘  │      │        │         │
│                   ▲                  │      │   HTTP │ POST    │
│                   │ Tauri Events     │      │  Upload│         │
│  ┌────────────────┴──────────────┐  │      │        │         │
│  │   Rust Backend                │  │      │   QR   │         │
│  │   - HTTP Server (actix-web)   │◄─┼──────┼────────┘         │
│  │   - 文件接收与流式写入        │  │      │                  │
│  │   - 进度追踪与取消            │  │      │  扫码获取 URL    │
│  │   - 会话管理                  │  │      │                  │
│  │   - 系统文件对话框            │  │      └──────────────────┘
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

## 核心流程

### 连接流程

1. 桌面端启动，获取本机局域网 IP
2. 启动 HTTP 服务器（随机端口），生成带令牌的 URL
3. 渲染二维码（编码 URL + 令牌）
4. 手机扫码 → 打开浏览器 → 访问 URL
5. 首次访问时，服务器记录设备信息（User-Agent、设备名），建立会话

### 传输流程

1. 手机端选择文件，发起 POST /upload 请求（multipart/form-data）
2. 服务器识别文件信息（文件名、大小）
3. Tauri 前端弹出保存目录选择对话框（每会话首次传输时弹一次）
4. 用户确认目录后，服务器开始流式写入
5. 传输过程中，服务器通过 Tauri 事件系统实时推送进度到桌面 UI
6. 手机端浏览器通过 XHR progress 事件显示上传进度
7. 传输完成 → 通知双端；取消 → 中断连接 + 清理临时文件

### 取消流程

- **手机端取消**：中断 XHR 请求 → 服务器检测到连接断开 → 清理已接收的临时文件
- **电脑端取消**：桌面 UI 点击取消 → Tauri 命令通知 Rust 后端 → 关闭对应 HTTP 连接 → 清理临时文件

## 桌面端 UI 布局

```
┌──────────────────────────────────────────────┐
│  [Logo] oh-my-file                 [_][□][×]  │
├─────────────────────┬────────────────────────┤
│                     │                        │
│   ┌───────────┐    │  📁 文件传输列表        │
│   │  QR Code   │    │                        │
│   │  ┌───────┐ │    │  photo_001.jpg  ■■■■□  │
│   │  │       │ │    │  iPhone 15     85%     │
│   │  │       │ │    │  [取消]                │
│   │  └───────┘ │    │                        │
│   │            │    │  doc.pdf        ■■■■■  │
│   │  地址:     │    │  Xiaomi 14     100% ✅ │
│   │  192.168.  │    │  [打开文件夹]          │
│   │  1.100:    │    │                        │
│   │  8080      │    │  等待传输...           │
│   └───────────┘    │                        │
│                     │                        │
│  状态: 运行中 🟢   │                        │
│  已连接: 2 台设备   │                        │
└─────────────────────┴────────────────────────┘
```

### 组件结构

| 组件 | 功能 |
|------|------|
| `QRCode.tsx` | 展示二维码和连接地址 |
| `TransferList.tsx` | 文件传输列表容器 |
| `TransferItem.tsx` | 单条传输项（文件名、设备名、进度条、取消按钮） |
| `StatusBar.tsx` | 状态栏（服务器状态、已连接设备数） |

## 手机端上传页面

极简 HTML 页面，扫码即开，零配置。

- "选择文件"按钮 → 系统文件选择器（支持多选）
- 选择后自动开始上传
- 每个文件独立显示进度、独立取消
- 上传完成显示 ✅

## 技术选型

### Rust 后端

| 用途 | 选型 |
|------|------|
| HTTP 服务器 | `actix-web` |
| 二维码生成 | `qrcode` |
| 局域网 IP 获取 | `local-ip-address` |
| 异步文件读写 | `tokio::fs` |
| 系统原生对话框 | `rfd` (Rust File Dialogs) |
| 序列化 | `serde` / `serde_json` |

### 前端

| 用途 | 选型 |
|------|------|
| 桌面 UI 框架 | React + TypeScript |
| 手机网页 | 纯 HTML + CSS + JS（无框架） |
| 构建工具 | Vite |

## 项目结构

```
oh-my-file/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri 入口
│   │   ├── server.rs        # HTTP 服务器
│   │   ├── transfer.rs      # 传输管理（进度、取消）
│   │   ├── session.rs       # 设备会话管理
│   │   └── dialog.rs        # 系统对话框
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/
│   ├── App.tsx
│   ├── components/
│   │   ├── QRCode.tsx
│   │   ├── TransferList.tsx
│   │   ├── TransferItem.tsx
│   │   └── StatusBar.tsx
│   ├── hooks/
│   │   └── useTransfers.ts
│   └── styles/
│       └── global.css
│
├── phone-www/
│   ├── index.html
│   ├── style.css
│   └── app.js
│
├── package.json
└── README.md
```

## API 设计

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 返回上传页面（静态 HTML） |
| GET | `/api/info` | 返回服务器信息（名称、版本） |
| POST | `/api/upload` | 上传文件（multipart/form-data） |
| GET | `/api/events?token=` | SSE 事件流（进度推送） |
| GET | `/health` | 健康检查 |

### 上传请求格式

```
POST /api/upload
Content-Type: multipart/form-data

file: <文件二进制>
filename: photo.jpg
filesize: 1048576
device: iPhone 15
```

### 事件推送格式（SSE）

```
event: progress
data: {"file_id": "abc123", "received": 524288, "total": 1048576, "speed": "2.1 MB/s"}

event: complete
data: {"file_id": "abc123", "path": "/Users/xxx/Downloads/photo.jpg"}

event: error
data: {"file_id": "abc123", "error": "写入磁盘失败"}
```

## 边界情况处理

| 场景 | 处理方式 |
|------|----------|
| 断网中断 | 自动重试，最多 10 次，之后报错 |
| 文件冲突 | 自动重命名（`file(1).jpg`） |
| 连接超时 | 10 分钟无操作自动断开会话 |
| 大文件 | 流式写入磁盘，不占用内存 |
| 并发上传 | actix-web 多线程处理，每个文件独立跟踪 |
| 取消传输 | 中断连接 + 删除已接收的临时文件 |

## 安全性

- HTTP 服务器绑定局域网 IP，不暴露到公网
- URL 携带随机令牌，防止同局域网下的未授权访问
- 会话超时自动清理
- 临时文件及时清理