#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dialog;
mod network;
mod server;
mod session;
mod transfer;

use network::NetworkState;
use server::{server_info, ServerConfig, ServerInfoPayload, SharedState};
use session::SessionManager;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;
use transfer::TransferStore;

#[derive(Clone)]
struct RuntimeState {
    shared: SharedState,
}

#[tauri::command]
fn get_server_info(state: tauri::State<'_, RuntimeState>) -> ServerInfoPayload {
    server_info(&state.shared)
}

#[tauri::command]
fn ensure_save_directory(state: tauri::State<'_, RuntimeState>) -> Result<String, String> {
    let mut guard = state
        .shared
        .save_directory
        .lock()
        .map_err(|_| "保存目录状态不可用".to_string())?;

    if let Some(path) = guard.clone() {
        return Ok(path.to_string_lossy().to_string());
    }

    let selected =
        dialog::pick_save_directory().ok_or_else(|| "用户取消选择保存目录".to_string())?;
    *guard = Some(selected.clone());
    Ok(selected.to_string_lossy().to_string())
}

#[tauri::command]
async fn cancel_transfer(
    transfer_id: String,
    state: tauri::State<'_, RuntimeState>,
) -> Result<(), String> {
    if state.shared.transfers.cancel(&transfer_id).await {
        Ok(())
    } else {
        Err("未找到正在进行的传输任务".into())
    }
}

#[tauri::command]
fn open_directory(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        Command::new("explorer")
            .arg(path)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("打开目录失败: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("打开目录失败: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("打开目录失败: {error}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("当前平台不支持打开目录".into())
}

#[tauri::command]
fn refresh_network_info(state: tauri::State<'_, RuntimeState>) -> ServerInfoPayload {
    if let Ok(mut network) = state.shared.network.lock() {
        network.refresh();
    }

    server_info(&state.shared)
}

#[tauri::command]
fn select_network_interface(
    interface_id: String,
    state: tauri::State<'_, RuntimeState>,
) -> Result<ServerInfoPayload, String> {
    let mut network = state
        .shared
        .network
        .lock()
        .map_err(|_| "网络状态不可用".to_string())?;

    network.refresh();
    if !network.select_interface(&interface_id) {
        return Err("未找到指定网卡地址".into());
    }

    Ok(server_info(&state.shared))
}

fn find_available_port(start: u16) -> u16 {
    for port in start..65535 {
        if TcpListener::bind(("0.0.0.0", port)).is_ok() {
            return port;
        }
    }

    8080
}

fn build_runtime_state() -> RuntimeState {
    let port = find_available_port(8080);
    let token = uuid::Uuid::new_v4().to_string();
    let mut network = NetworkState::default();
    network.refresh();

    let shared = SharedState {
        server: ServerConfig {
            port,
            token,
        },
        network: Arc::new(Mutex::new(network)),
        save_directory: Arc::new(Mutex::new(None::<PathBuf>)),
        sessions: Arc::new(Mutex::new(SessionManager::new(Duration::from_secs(600)))),
        transfers: TransferStore::default(),
    };

    RuntimeState { shared }
}

fn main() {
    let runtime_state = build_runtime_state();
    let server_state = runtime_state.shared.clone();

    tauri::Builder::default()
        .manage(runtime_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_server_info,
            ensure_save_directory,
            cancel_transfer,
            open_directory,
            refresh_network_info,
            select_network_interface
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state = server_state.clone();
            let initial_info = server_info(&state);

            let _ = app_handle.emit(
                "transfer-event",
                serde_json::json!({
                    "type": "server-ready",
                    "payload": initial_info
                }),
            );

            std::thread::spawn(move || {
                let runtime = actix_web::rt::System::new();
                runtime.block_on(async move {
                    if let Err(error) = server::start_server(app_handle.clone(), state).await {
                        let _ = app_handle.emit(
                            "transfer-event",
                            serde_json::json!({
                                "type": "transfer-error",
                                "payload": {
                                    "id": "server",
                                    "errorMessage": format!("服务器启动失败: {error}")
                                }
                            }),
                        );
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
