# Phase 1: 项目脚手架搭建

**目标：** 初始化 Tauri + React + TypeScript 项目，配置所有依赖，确保可以构建运行。

---

### Task 1: 初始化 Tauri 项目

**操作：**

- [ ] **用 Tauri CLI 创建项目**

```bash
cd /d/personal/code/project/oh-my-file
npm create tauri-app@latest oh-my-file -- --template react-ts
```

如果命令交互式运行，选择：
- Project name: `oh-my-file`
- Frontend: `React + TypeScript`
- Package manager: `npm`

- [ ] **验证项目结构**

确认以下目录结构存在：
```
oh-my-file/
├── src/              # React 前端
├── src-tauri/         # Rust 后端
├── package.json
└── ...
```

- [ ] **确认可以构建**

```bash
cd /d/personal/code/project/oh-my-file
npm install
npm run tauri dev
```

预期：Tauri 窗口打开，显示默认 React 页面。

---

### Task 2: 配置 Rust 依赖

- [ ] **编辑 `src-tauri/Cargo.toml`，添加依赖**

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
actix-web = "4"
actix-multipart = "0.7"
tokio = { version = "1", features = ["full"] }
qrcode = "0.14"
image = "0.25"
local-ip-address = "0.6"
rfd = "0.15"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

- [ ] **验证编译**

```bash
cd src-tauri
cargo check
```

预期：编译成功，无错误。

---

### Task 3: 创建 phone-www 目录

- [ ] **创建手机端网页目录**

```bash
mkdir -p /d/personal/code/project/oh-my-file/phone-www
```

- [ ] **创建占位文件 `phone-www/index.html`**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
  <title>oh-my-file 传输</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div id="app">
    <h1>📤 oh-my-file</h1>
    <p id="status">连接中...</p>
  </div>
  <script src="app.js"></script>
</body>
</html>
```

- [ ] **创建 `phone-www/style.css`**

```css
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: #f5f5f5;
  color: #333;
  min-height: 100vh;
  display: flex;
  justify-content: center;
}
#app {
  width: 100%;
  max-width: 480px;
  padding: 24px;
}
h1 { font-size: 24px; margin-bottom: 16px; text-align: center; }
```

- [ ] **创建 `phone-www/app.js`**（空占位）

```javascript
// 手机端上传逻辑 - 将在 Phase 3 实现
```

---

### Task 4: 配置 Tauri 静态文件服务

- [ ] **编辑 `src-tauri/tauri.conf.json`**，配置资源路径确保 phone-www 被包含

```json
{
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "resources": ["../phone-www/*"]
  },
  "app": {
    "windows": [
      {
        "title": "oh-my-file",
        "width": 800,
        "height": 600,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

---

### 验证清单

- [ ] `npm run tauri dev` 启动成功，窗口打开
- [ ] `cargo check` 所有 Rust 依赖编译通过
- [ ] `phone-www/` 目录结构完整