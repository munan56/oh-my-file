# Phase 4: 会话管理、传输控制与联调验收

**目标：** 完成目录选择、设备会话、上传取消、文件落盘与冲突处理，并收尾文档与验收测试。

---

### Task 1: 创建会话与传输管理模块

- [ ] **创建 `src-tauri/src/session.rs`**

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DeviceSession {
    pub token: String,
    pub device_name: String,
    pub user_agent: String,
    pub connected_at: Instant,
    pub last_seen_at: Instant,
}

pub struct SessionManager {
    sessions: HashMap<String, DeviceSession>,
    timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout: Duration) -> Self {
        Self {
            sessions: HashMap::new(),
            timeout,
        }
    }

    pub fn upsert(&mut self, token: String, device_name: String, user_agent: String) {
        let now = Instant::now();
        self.sessions.insert(
          token.clone(),
          DeviceSession {
            token,
            device_name,
            user_agent,
            connected_at: now,
            last_seen_at: now,
          },
        );
    }

    pub fn touch(&mut self, token: &str) {
        if let Some(session) = self.sessions.get_mut(token) {
            session.last_seen_at = Instant::now();
        }
    }

    pub fn cleanup_expired(&mut self) {
        let timeout = self.timeout;
        self.sessions.retain(|_, session| session.last_seen_at.elapsed() < timeout);
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }
}
```

- [ ] **创建 `src-tauri/src/transfer.rs`**

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, serde::Serialize)]
pub struct TransferProgress {
    pub id: String,
    pub filename: String,
    pub device_name: String,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub status: String,
    pub save_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Default)]
pub struct TransferStore {
    inner: Arc<RwLock<HashMap<String, TransferProgress>>>,
}

impl TransferStore {
    pub async fn insert(&self, progress: TransferProgress) {
        self.inner.write().await.insert(progress.id.clone(), progress);
    }

    pub async fn update<F>(&self, id: &str, updater: F)
    where
        F: FnOnce(&mut TransferProgress),
    {
        if let Some(item) = self.inner.write().await.get_mut(id) {
            updater(item);
        }
    }

    pub async fn remove(&self, id: &str) {
        self.inner.write().await.remove(id);
    }
}

pub fn resolve_target_path(base_dir: &PathBuf, filename: &str) -> PathBuf {
    let mut candidate = base_dir.join(filename);

    if !candidate.exists() {
        return candidate;
    }

    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    for index in 1..1000 {
        let next = if ext.is_empty() {
            base_dir.join(format!("{stem}({index})"))
        } else {
            base_dir.join(format!("{stem}({index}).{ext}"))
        };

        if !next.exists() {
            candidate = next;
            break;
        }
    }

    candidate
}
```

- [ ] **验证 Rust 编译**

```bash
cd src-tauri
cargo check
```

---

### Task 2: 实现目录选择与应用状态

- [ ] **创建 `src-tauri/src/dialog.rs`**

```rust
use std::path::PathBuf;

pub fn pick_save_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择接收文件保存目录")
        .pick_folder()
}
```

- [ ] **在 `src-tauri/src/main.rs` 中补充共享状态**

```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

mod dialog;
mod server;
mod session;
mod transfer;

#[derive(Clone)]
pub struct AppState {
    pub save_directory: Arc<RwLock<Option<PathBuf>>>,
    pub transfer_store: transfer::TransferStore,
    pub session_manager: Arc<RwLock<session::SessionManager>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            save_directory: Arc::new(RwLock::new(None)),
            transfer_store: transfer::TransferStore::default(),
            session_manager: Arc::new(RwLock::new(session::SessionManager::new(
                std::time::Duration::from_secs(600),
            ))),
        }
    }
}
```

- [ ] **增加 Tauri 命令**

```rust
#[tauri::command]
async fn ensure_save_directory(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some(path) = state.save_directory.read().await.clone() {
        return Ok(path.to_string_lossy().to_string());
    }

    let selected = dialog::pick_save_directory()
        .ok_or_else(|| "用户取消选择保存目录".to_string())?;

    *state.save_directory.write().await = Some(selected.clone());
    Ok(selected.to_string_lossy().to_string())
}
```

- [ ] **验证命令可编译**

```bash
cd src-tauri
cargo check
```

---

### Task 3: 完善上传写入、取消与事件广播

- [ ] **扩展 `src-tauri/src/server.rs` 的共享状态**

```rust
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct ServerState {
    pub token: String,
    pub app_handle: AppHandle,
    pub save_directory: Arc<RwLock<Option<PathBuf>>>,
    pub transfer_store: transfer::TransferStore,
    pub session_manager: Arc<RwLock<session::SessionManager>>,
}
```

- [ ] **在 `upload_file` 中接入目录选择、进度广播与冲突重命名**

```rust
let transfer_id = uuid::Uuid::new_v4().to_string();
let save_dir = state
    .save_directory
    .read()
    .await
    .clone()
    .ok_or_else(|| error::ErrorBadRequest("save directory not selected"))?;

let target_path = transfer::resolve_target_path(&save_dir, &filename);
let mut file = tokio::fs::File::create(&target_path)
    .await
    .map_err(error::ErrorInternalServerError)?;

state
    .transfer_store
    .insert(transfer::TransferProgress {
        id: transfer_id.clone(),
        filename: filename.clone(),
        device_name: device_name.clone(),
        total_bytes,
        received_bytes: 0,
        status: "uploading".into(),
        save_path: None,
        error_message: None,
    })
    .await;

while let Some(chunk) = field.next().await {
    let bytes = chunk.map_err(error::ErrorBadRequest)?;
    file.write_all(&bytes)
        .await
        .map_err(error::ErrorInternalServerError)?;

    received_bytes += bytes.len() as u64;

    state
        .transfer_store
        .update(&transfer_id, |item| {
            item.received_bytes = received_bytes;
        })
        .await;

    state.app_handle.emit(
        "transfer-event",
        serde_json::json!({
            "type": "transfer-progress",
            "payload": {
                "id": transfer_id,
                "filename": filename,
                "deviceName": device_name,
                "totalBytes": total_bytes,
                "receivedBytes": received_bytes,
                "status": "uploading"
            }
        }),
    ).ok();
}
```

- [ ] **补充完成与错误事件**

```rust
state.app_handle.emit(
    "transfer-event",
    serde_json::json!({
        "type": "transfer-complete",
        "payload": {
            "id": transfer_id,
            "savePath": target_path.to_string_lossy().to_string()
        }
    }),
).ok();
```

- [ ] **新增取消命令**

```rust
#[tauri::command]
async fn cancel_transfer(
    transfer_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .transfer_store
        .update(&transfer_id, |item| {
            item.status = "cancelled".into();
            item.error_message = Some("用户取消传输".into());
        })
        .await;

    Ok(())
}
```

---

### Task 4: 实现连接信息与前后端桥接

- [ ] **重写 `get_server_info` 返回结构化 JSON**

```rust
#[derive(serde::Serialize)]
struct ServerInfoPayload {
    status: &'static str,
    port: u16,
    token: String,
    ip: String,
    url: String,
    qr_code_data_url: String,
    connected_devices: usize,
}

#[tauri::command]
async fn get_server_info(state: tauri::State<'_, RuntimeState>) -> Result<ServerInfoPayload, String> {
    let url = format!("http://{}:{}/?token={}", state.ip, state.port, state.token);

    Ok(ServerInfoPayload {
        status: "running",
        port: state.port,
        token: state.token.clone(),
        ip: state.ip.clone(),
        url: url.clone(),
        qr_code_data_url: server::generate_qr_code_data_url(&url),
        connected_devices: state.app_state.session_manager.read().await.len(),
    })
}
```

- [ ] **补充 `RuntimeState` 结构，避免把端口/token/ip 分散在多个全局变量**

```rust
#[derive(Clone)]
pub struct RuntimeState {
    pub port: u16,
    pub token: String,
    pub ip: String,
    pub app_state: AppState,
}
```

- [ ] **在前端把 Rust snake_case 字段映射成 TS camelCase**

```ts
const raw = await invoke<{
  status: 'starting' | 'running' | 'error';
  port: number;
  token: string;
  ip: string;
  url: string;
  qr_code_data_url: string;
  connected_devices: number;
}>('get_server_info');

setServerInfo({
  status: raw.status,
  port: raw.port,
  token: raw.token,
  ip: raw.ip,
  url: raw.url,
  qrCodeDataUrl: raw.qr_code_data_url,
  connectedDevices: raw.connected_devices,
});
```

- [ ] **验证二维码与 URL 一致**

```bash
npm run tauri dev
```

预期：二维码与页面展示地址一致，手机扫码可直接打开上传页。

---

### Task 5: 补充手机端体验与异常处理

- [ ] **在 `phone-www/app.js` 中增加服务信息加载**

```javascript
async function loadInfo() {
  const response = await fetch(`/api/info?token=${encodeURIComponent(token || '')}`);
  const info = await response.json();
  document.getElementById('server-name').textContent = `${info.name} ${info.version}`;
}

loadInfo().catch(() => {
  document.getElementById('server-name').textContent = '服务信息读取失败';
});
```

- [ ] **在 `phone-www/app.js` 中加入简单重试策略**

```javascript
function sendWithRetry(file, attempt = 0) {
  return new Promise((resolve, reject) => {
    const item = createUploadItem(file);
    const xhr = new XMLHttpRequest();

    xhr.open('POST', `/api/upload?token=${encodeURIComponent(token || '')}`);
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve();
        return;
      }

      if (attempt >= 9) {
        reject(new Error('upload failed'));
        return;
      }

      setTimeout(() => {
        sendWithRetry(file, attempt + 1).then(resolve).catch(reject);
      }, 1000);
    };

    xhr.onerror = () => {
      if (attempt >= 9) {
        reject(new Error('network error'));
        return;
      }

      setTimeout(() => {
        sendWithRetry(file, attempt + 1).then(resolve).catch(reject);
      }, 1000);
    };

    const formData = new FormData();
    formData.append('file', file);
    formData.append('device', navigator.userAgent);
    xhr.send(formData);
  });
}
```

- [ ] **将文件选择回调改为走重试入口**

```javascript
fileInput.addEventListener('change', () => {
  const files = Array.from(fileInput.files || []);
  files.forEach((file) => {
    sendWithRetry(file).catch(() => {
      console.error(`上传失败: ${file.name}`);
    });
  });
  fileInput.value = '';
});
```

- [ ] **补充手机端样式**

```css
.upload-item {
  padding: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(10px);
}

.upload-progress {
  overflow: hidden;
  height: 8px;
  margin: 10px 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
}

.upload-progress > div {
  width: 0;
  height: 100%;
  background: linear-gradient(90deg, #58b8ff, #6affbf);
}
```

---

### Task 6: 联调、文档与验收

- [ ] **补充 README 使用说明**

```md
## 使用方式

1. 启动桌面端应用
2. 允许应用在本机局域网监听端口
3. 手机扫描桌面端二维码
4. 首次上传时在桌面端选择保存目录
5. 在手机端选择文件并等待完成
```

- [ ] **执行联调检查**

```bash
npm install
npm run build
cd src-tauri
cargo check
```

- [ ] **手动验收场景**

```text
1. 单文件上传成功，桌面端显示 100%
2. 多文件连续上传，列表顺序正确
3. 重名文件自动改名保存
4. 手机端取消上传，桌面端状态更新为 cancelled
5. 电脑端取消上传，不再继续写入磁盘
6. 10 分钟无操作后，会话从已连接设备数中清理
```

---

### 验证清单

- [ ] 保存目录首次上传时弹出，后续沿用同一路径
- [ ] 会话数可反映手机端连接/超时状态
- [ ] 文件冲突会自动重命名
- [ ] 取消上传后临时文件被清理
- [ ] README 补充了启动与使用说明
