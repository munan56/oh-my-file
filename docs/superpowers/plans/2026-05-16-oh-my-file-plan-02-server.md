# Phase 2: Rust HTTP 服务器核心

**目标：** 实现局域网 HTTP 服务器，支持静态文件服务、文件上传、SSE 进度事件。

---

### Task 1: 创建 server.rs 模块

- [ ] **创建 `src-tauri/src/server.rs`**

```rust
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, middleware};
use actix_multipart::Multipart;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServerState {
    pub token: String,
    pub port: u16,
}

pub async fn start_server(port: u16, token: String) -> std::io::Result<()> {
    let state = Arc::new(Mutex::new(ServerState {
        token: token.clone(),
        port,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(state.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/info", web::get().to(api_info))
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/events", web::get().to(sse_events))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

async fn api_info() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "name": "oh-my-file",
        "version": "0.1.0"
    }))
}

async fn upload_file(body: Multipart) -> HttpResponse {
    // Phase 2 Task 2 实现
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}

async fn sse_events(req: HttpRequest) -> HttpResponse {
    // Phase 2 Task 3 实现
    HttpResponse::Ok().json(serde_json::json!({"status": "not_implemented"}))
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

---

### Task 2: 实现文件上传处理

- [ ] **在 `server.rs` 中添加文件上传逻辑**（替换 `upload_file`）

```rust
use actix_multipart::{Multipart, Field};
use futures_util::StreamExt;
use std::path::PathBuf;

// 添加到 Cargo.toml: futures-util = "0.3"

async fn upload_file(
    body: Multipart,
    req: HttpRequest,
) -> HttpResponse {
    let mut field: Option<Field> = None;
    let mut filename = String::new();
    let mut filesize: u64 = 0;

    let mut body = body;
    while let Some(Ok(mut f)) = body.next().await {
        let content_disposition = f.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "file" => {
                let temp_dir = std::env::temp_dir().join("oh-my-file");
                std::fs::create_dir_all(&temp_dir).ok();
                
                let file_name = content_disposition
                    .get_filename()
                    .unwrap_or("unknown")
                    .to_string();
                
                let file_path = temp_dir.join(&file_name);
                let mut file = tokio::fs::File::create(&file_path).await.unwrap();
                
                while let Some(Ok(chunk)) = f.next().await {
                    use tokio::io::AsyncWriteExt;
                    file.write_all(&chunk).await.unwrap();
                    filesize += chunk.len() as u64;
                }
                
                filename = file_name;
            }
            _ => {
                while let Some(Ok(_)) = f.next().await {}
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "filename": filename,
        "size": filesize,
        "temp_path": std::env::temp_dir()
            .join("oh-my-file")
            .join(&filename)
            .to_string_lossy()
            .to_string()
    }))
}
```

- [ ] **添加 `futures-util` 到 Cargo.toml**

```toml
futures-util = "0.3"
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

---

### Task 3: 实现 SSE 事件流

- [ ] **在 `server.rs` 中添加 SSE 支持**

```rust
use actix_web::web::Bytes;
use tokio::sync::broadcast;
use std::sync::atomic::{AtomicBool, Ordering};

// 在 ServerState 中添加
pub struct ServerState {
    pub token: String,
    pub port: u16,
    pub event_tx: broadcast::Sender<String>,
}

pub async fn start_server(port: u16, token: String) -> std::io::Result<()> {
    let (event_tx, _) = broadcast::channel(100);
    let state = Arc::new(Mutex::new(ServerState {
        token: token.clone(),
        port,
        event_tx: event_tx.clone(),
    }));
    // ... rest stays the same
}

async fn sse_events(req: HttpRequest) -> HttpResponse {
    // 验证 token
    let token = req.query_string()
        .split('&')
        .find_map(|s| s.strip_prefix("token="))
        .unwrap_or("");

    // 简化版：本实现将在后续阶段完善
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(futures_util::stream::once(async move {
            Ok::<Bytes, actix_web::Error>(
                Bytes::from("data: {\"connected\":true}\n\n")
            )
        }))
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

---

### Task 4: 生成二维码

- [ ] **在 `server.rs` 中添加二维码生成函数**

```rust
use qrcode::QrCode;
use image::Luma;

pub fn generate_qr_code_data_url(url: &str) -> String {
    let code = QrCode::new(url.as_bytes()).unwrap();
    let image = code.render::<Luma<u8>>().build();
    let width = image.width() as u32;
    let height = image.height() as u32;
    
    // 编码为 PNG 的 base64 data URL
    let mut png_bytes = std::io::Cursor::new(Vec::new());
    {
        let mut encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .encode(&image, width, height, image::ColorType::L8)
            .unwrap();
    }
    let b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes.into_inner(),
    );
    format!("data:image/png;base64,{}", b64)
}
```

- [ ] **添加 base64 依赖到 Cargo.toml**

```toml
base64 = "0.22"
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

---

### Task 5: 集成到 main.rs

- [ ] **编辑 `src-tauri/src/main.rs`**，启动 HTTP 服务器

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;

use std::net::TcpListener;

#[tauri::command]
fn get_server_info() -> String {
    serde_json::json!({
        "status": "running",
        "port": 0, // 将在后续实现中动态获取
    }).to_string()
}

fn find_available_port(start: u16) -> u16 {
    for port in start..65535 {
        if TcpListener::bind(("0.0.0.0", port)).is_ok() {
            return port;
        }
    }
    8080
}

fn main() {
    let port = find_available_port(8080);
    let token = uuid::Uuid::new_v4().to_string();

    // 在后台线程启动 HTTP 服务器
    let server_port = port;
    let server_token = token.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            server::start_server(server_port, server_token).await.unwrap();
        });
    });

    // 给服务器一点时间启动
    std::thread::sleep(std::time::Duration::from_millis(100));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_server_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **验证编译和运行**

```bash
cd src-tauri && cargo check
```

---

### 验证清单

- [ ] `cargo check` 无错误
- [ ] 服务器模块编译通过
- [ ] 文件上传函数签名正确
- [ ] SSE 端点可用
- [ ] 二维码生成函数正常编译