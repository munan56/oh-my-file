use crate::network::{NetworkInterface, NetworkState};
use crate::dialog;
use crate::session::SessionManager;
use crate::transfer::{self, TransferProgress, TransferStore};
use actix_multipart::Multipart;
use actix_web::web::Bytes;
use actix_web::{error, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use futures_util::StreamExt;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

const INDEX_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phone-www/index.html"));
const STYLE_CSS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phone-www/style.css"));
const APP_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phone-www/app.js"));

#[derive(Clone, Serialize)]
pub struct ServerInfoPayload {
    pub status: &'static str,
    pub port: u16,
    pub token: String,
    pub ip: String,
    pub url: String,
    pub qr_code_data_url: String,
    pub connected_devices: usize,
    pub selected_interface_id: Option<String>,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub token: String,
}

#[derive(Clone)]
pub struct SharedState {
    pub server: ServerConfig,
    pub network: Arc<Mutex<NetworkState>>,
    pub save_directory: Arc<Mutex<Option<PathBuf>>>,
    pub sessions: Arc<Mutex<SessionManager>>,
    pub transfers: TransferStore,
}

#[derive(Clone)]
pub struct HttpServerState {
    pub app_handle: AppHandle,
    pub shared: SharedState,
    pub event_tx: broadcast::Sender<String>,
}

pub async fn start_server(
    app_handle: AppHandle,
    shared: SharedState,
) -> std::io::Result<()> {
    let (event_tx, _) = broadcast::channel(256);
    let state = HttpServerState {
        app_handle,
        shared,
        event_tx,
    };

    let bind_port = state.shared.server.port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(index))
            .route("/style.css", web::get().to(style))
            .route("/app.js", web::get().to(script))
            .route("/health", web::get().to(health_check))
            .route("/api/info", web::get().to(api_info))
            .route("/api/presence", web::post().to(update_presence))
            .route("/api/disconnect", web::post().to(disconnect_device))
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/events", web::get().to(sse_events))
    })
    .bind(("0.0.0.0", bind_port))?
    .run()
    .await
}

pub fn generate_qr_code_data_url(url: &str) -> String {
    let svg_image = QrCode::new(url.as_bytes())
        .expect("valid QR content")
        .render::<svg::Color>()
        .min_dimensions(280, 280)
        .dark_color(svg::Color("#101822"))
        .light_color(svg::Color("#ffffff"))
        .build();

    format!(
        "data:image/svg+xml;utf8,{}",
        urlencoding::encode(&svg_image)
    )
}

pub fn server_info(shared: &SharedState) -> ServerInfoPayload {
    let connected_devices = shared
        .sessions
        .lock()
        .map(|mut sessions| sessions.len())
        .unwrap_or(0);
    let (ip, selected_interface_id, interfaces) = if let Ok(network) = shared.network.lock() {
        let selected = network.selected_interface();
        (
            selected
                .as_ref()
                .map(|item| item.ip.clone())
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            network.selected_interface_id(),
            network.interfaces(),
        )
    } else {
        ("127.0.0.1".to_string(), None, Vec::new())
    };
    let url = format!("http://{}:{}/?token={}", ip, shared.server.port, shared.server.token);

    ServerInfoPayload {
        status: "running",
        port: shared.server.port,
        token: shared.server.token.clone(),
        ip,
        url: url.clone(),
        qr_code_data_url: generate_qr_code_data_url(&url),
        connected_devices,
        selected_interface_id,
        interfaces,
    }
}

fn validate_token(req: &HttpRequest, token: &str) -> Result<(), HttpResponse> {
    let actual = req
        .query_string()
        .split('&')
        .find_map(|pair| pair.strip_prefix("token="))
        .unwrap_or_default();

    if actual == token {
        Ok(())
    } else {
        Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid token"
        })))
    }
}

fn infer_device_name(req: &HttpRequest) -> String {
    req.headers()
        .get("x-device-name")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "Mobile Browser".to_string())
}

fn client_ip(req: &HttpRequest) -> String {
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn looks_like_user_agent(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("mozilla/")
        || lower.contains("applewebkit/")
        || lower.contains("safari/")
        || lower.contains("chrome/")
        || lower.contains("firefox/")
}

fn infer_device_family(user_agent: &str) -> &'static str {
    if user_agent.contains("iPhone") {
        "iPhone"
    } else if user_agent.contains("iPad") {
        "iPad"
    } else if user_agent.contains("Android") && user_agent.contains("Mobile") {
        "Android 手机"
    } else if user_agent.contains("Android") {
        "Android 设备"
    } else if user_agent.contains("Windows") {
        "Windows 设备"
    } else if user_agent.contains("Mac OS X") {
        "Mac 设备"
    } else {
        "移动设备"
    }
}

fn infer_browser_name(user_agent: &str) -> &'static str {
    if user_agent.contains("EdgiOS") || user_agent.contains("EdgA") || user_agent.contains("Edg/") {
        "Edge"
    } else if user_agent.contains("CriOS") || user_agent.contains("Chrome/") {
        "Chrome"
    } else if user_agent.contains("FxiOS") || user_agent.contains("Firefox/") {
        "Firefox"
    } else if user_agent.contains("Version/") && user_agent.contains("Safari/") {
        "Safari"
    } else {
        "浏览器"
    }
}

fn build_device_label(req: &HttpRequest, user_agent: &str, submitted_device_name: Option<&str>) -> String {
    let client_ip = client_ip(req);
    let submitted = submitted_device_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| !looks_like_user_agent(value));

    let base = if let Some(value) = submitted {
        value.to_string()
    } else {
        format!(
            "{} · {}",
            infer_device_family(user_agent),
            infer_browser_name(user_agent)
        )
    };

    format!("{base} · {client_ip}")
}

fn session_key(req: &HttpRequest, user_agent: &str) -> String {
    let remote = client_ip(req);

    format!("{remote}:{user_agent}")
}

async fn emit_devices_changed(app_handle: &AppHandle, count: usize) {
    emit_transfer_event(
        app_handle,
        serde_json::json!({
            "type": "devices-changed",
            "payload": { "connectedDevices": count }
        }),
    )
    .await;
}

fn ensure_save_directory(shared: &SharedState) -> Result<PathBuf, String> {
    let mut guard = shared
        .save_directory
        .lock()
        .map_err(|_| "保存目录状态不可用".to_string())?;

    if let Some(path) = guard.clone() {
        return Ok(path);
    }

    let selected = dialog::pick_save_directory()
        .ok_or_else(|| "用户取消选择保存目录".to_string())?;
    *guard = Some(selected.clone());
    Ok(selected)
}

async fn emit_transfer_event(app_handle: &AppHandle, payload: serde_json::Value) {
    let _ = app_handle.emit("transfer-event", payload);
}

fn emit_sse(tx: &broadcast::Sender<String>, event: &str, data: serde_json::Value) {
    let payload = format!("event: {event}\ndata: {}\n\n", data);
    let _ = tx.send(payload);
}

async fn index(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    let html = INDEX_HTML.replace("__TOKEN__", &state.shared.server.token);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn style(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(STYLE_CSS)
}

async fn script(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .body(APP_JS)
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn api_info(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }
    let info = server_info(&state.shared);

    HttpResponse::Ok().json(serde_json::json!({
        "name": "oh-my-file",
        "version": env!("CARGO_PKG_VERSION"),
        "ip": info.ip,
        "port": info.port
    }))
}

async fn update_presence(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    let device_name = build_device_label(&req, &user_agent, Some(&infer_device_name(&req)));
    let key = session_key(&req, &user_agent);

    if let Ok(mut sessions) = state.shared.sessions.lock() {
        sessions.upsert(key, device_name, user_agent);
        let count = sessions.len();
        emit_devices_changed(&state.app_handle, count).await;
    }

    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn disconnect_device(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    let key = session_key(&req, &user_agent);

    if let Ok(mut sessions) = state.shared.sessions.lock() {
        let _ = sessions.remove(&key);
        let count = sessions.len();
        emit_devices_changed(&state.app_handle, count).await;
    }

    HttpResponse::Ok().json(serde_json::json!({ "status": "disconnected" }))
}

async fn sse_events(req: HttpRequest, state: web::Data<HttpServerState>) -> HttpResponse {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return response;
    }

    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|message| async move {
        match message {
            Ok(chunk) => Some(Ok::<Bytes, Error>(Bytes::from(chunk))),
            Err(_) => None,
        }
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}

async fn upload_file(
    req: HttpRequest,
    mut body: Multipart,
    state: web::Data<HttpServerState>,
) -> Result<HttpResponse, Error> {
    if let Err(response) = validate_token(&req, &state.shared.server.token) {
        return Ok(response);
    }

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    let fallback_device_name = infer_device_name(&req);
    let mut device_name_from_form = build_device_label(&req, &user_agent, Some(&fallback_device_name));
    let key = session_key(&req, &user_agent);

    if let Ok(mut sessions) = state.shared.sessions.lock() {
        sessions.upsert(key.clone(), device_name_from_form.clone(), user_agent.clone());
        let count = sessions.len();
        emit_devices_changed(&state.app_handle, count).await;
    }

    let save_dir = ensure_save_directory(&state.shared).map_err(error::ErrorBadRequest)?;
    let mut handled = false;
    let mut filename_from_form: Option<String> = None;
    let mut filesize_from_form = 0u64;

    while let Some(field_result) = body.next().await {
        let mut field = field_result.map_err(error::ErrorBadRequest)?;
        let content_disposition = field.content_disposition().cloned();
        let field_name = content_disposition
            .as_ref()
            .and_then(|value| value.get_name())
            .unwrap_or_default()
            .to_string();

        if field_name != "file" {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                bytes.extend_from_slice(&chunk.map_err(error::ErrorBadRequest)?);
            }

            if let Ok(value) = String::from_utf8(bytes) {
                match field_name.as_str() {
                    "device" if !value.trim().is_empty() => {
                        device_name_from_form =
                            build_device_label(&req, &user_agent, Some(value.as_str()));
                    }
                    "filename" if !value.trim().is_empty() => {
                        filename_from_form = Some(value);
                    }
                    "filesize" => {
                        filesize_from_form = value.parse::<u64>().unwrap_or(0);
                    }
                    _ => {}
                }
            }
            continue;
        }

        handled = true;

        let filename = content_disposition
            .as_ref()
            .and_then(|value| value.get_filename())
            .map(ToOwned::to_owned)
            .or(filename_from_form.clone())
            .unwrap_or_else(|| format!("upload-{}.bin", Uuid::new_v4()));
        let total_bytes = filesize_from_form;

        let target_path = transfer::resolve_target_path(&save_dir, &filename);
        let transfer_id = Uuid::new_v4().to_string();
        let cancel_token = state
            .shared
            .transfers
            .insert(TransferProgress {
                id: transfer_id.clone(),
                filename: filename.clone(),
                device_name: device_name_from_form.clone(),
                total_bytes,
                received_bytes: 0,
                status: "uploading".into(),
                save_path: None,
                error_message: None,
            })
            .await;

        emit_transfer_event(
            &state.app_handle,
            serde_json::json!({
                "type": "transfer-progress",
                "payload": {
                    "id": transfer_id.clone(),
                    "filename": filename.clone(),
                    "deviceName": device_name_from_form.clone(),
                    "totalBytes": total_bytes,
                    "receivedBytes": 0u64,
                    "status": "uploading"
                }
            }),
        )
        .await;

        emit_sse(
            &state.event_tx,
            "progress",
            serde_json::json!({
                "file_id": transfer_id.clone(),
                "received": 0u64,
                "total": total_bytes
            }),
        );

        let mut file = fs::File::create(&target_path)
            .await
            .map_err(error::ErrorInternalServerError)?;
        let mut received_bytes = 0u64;
        let mut final_error: Option<String> = None;

        while let Some(chunk_result) = field.next().await {
            if cancel_token.is_cancelled() {
                final_error = Some("用户取消传输".into());
                break;
            }

            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(err) => {
                    final_error = Some(format!("上传中断: {err}"));
                    break;
                }
            };

            file.write_all(&chunk)
                .await
                .map_err(error::ErrorInternalServerError)?;
            received_bytes += chunk.len() as u64;
            state
                .shared
                .transfers
                .update_progress(&transfer_id, received_bytes)
                .await;

            emit_transfer_event(
                &state.app_handle,
                serde_json::json!({
                    "type": "transfer-progress",
                    "payload": {
                        "id": transfer_id.clone(),
                        "filename": filename.clone(),
                        "deviceName": device_name_from_form.clone(),
                        "totalBytes": total_bytes,
                        "receivedBytes": received_bytes,
                        "status": "uploading"
                    }
                }),
            )
            .await;

            emit_sse(
                &state.event_tx,
                "progress",
                serde_json::json!({
                    "file_id": transfer_id.clone(),
                    "received": received_bytes,
                    "total": total_bytes
                }),
            );
        }

        if let Some(message) = final_error {
            let _ = fs::remove_file(&target_path).await;
            if message == "用户取消传输" {
                let _ = state.shared.transfers.cancel(&transfer_id).await;
            } else {
                state.shared.transfers.fail(&transfer_id, message.clone()).await;
            }
            emit_transfer_event(
                &state.app_handle,
                serde_json::json!({
                    "type": "transfer-error",
                    "payload": {
                        "id": transfer_id.clone(),
                        "errorMessage": message
                    }
                }),
            )
            .await;
            emit_sse(
                &state.event_tx,
                "error",
                serde_json::json!({
                    "file_id": transfer_id.clone(),
                    "error": state.shared.transfers.snapshot(&transfer_id).await.and_then(|item| item.error_message).unwrap_or_else(|| "上传失败".into())
                }),
            );
            return Ok(HttpResponse::Conflict().json(serde_json::json!({
                "status": "error",
                "file_id": transfer_id
            })));
        }

        file.flush().await.map_err(error::ErrorInternalServerError)?;
        state
            .shared
            .transfers
            .complete(&transfer_id, target_path.to_string_lossy().to_string())
            .await;

        emit_transfer_event(
            &state.app_handle,
            serde_json::json!({
                "type": "transfer-complete",
                "payload": {
                    "id": transfer_id.clone(),
                    "savePath": target_path.to_string_lossy().to_string(),
                    "directoryPath": transfer::directory_path(&target_path).unwrap_or_else(|| save_dir.to_string_lossy().to_string())
                }
            }),
        )
        .await;

        emit_sse(
            &state.event_tx,
            "complete",
            serde_json::json!({
                "file_id": transfer_id,
                "path": target_path.to_string_lossy().to_string()
            }),
        );
    }

    if let Ok(mut sessions) = state.shared.sessions.lock() {
        sessions.touch(&key);
        let count = sessions.len();
        emit_devices_changed(&state.app_handle, count).await;
    }

    if handled {
        Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "received" })))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "missing file field"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::{build_device_label, looks_like_user_agent};
    use actix_web::test::TestRequest;

    #[test]
    fn detects_raw_user_agent_strings() {
        assert!(looks_like_user_agent(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15"
        ));
        assert!(!looks_like_user_agent("iPhone · Safari"));
    }

    #[test]
    fn builds_friendly_label_from_user_agent_and_ip() {
        let request = TestRequest::default()
            .peer_addr("10.31.0.8:50000".parse().expect("peer addr"))
            .to_http_request();
        let label = build_device_label(
            &request,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.4 Mobile/15E148 Safari/604.1",
            Some("Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15"),
        );

        assert_eq!(label, "iPhone · Safari · 10.31.0.8");
    }

    #[test]
    fn preserves_custom_device_name_and_appends_ip() {
        let request = TestRequest::default()
            .peer_addr("10.31.0.9:50000".parse().expect("peer addr"))
            .to_http_request();
        let label = build_device_label(&request, "Mozilla/5.0", Some("小明的 iPhone"));

        assert_eq!(label, "小明的 iPhone · 10.31.0.9");
    }
}
