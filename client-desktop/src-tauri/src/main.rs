#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures_util::StreamExt;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message as TungMessage;

// Shared, replaceable sender stored in a Mutex. Value is Some(sender) when connected.
static WS_SENDER: OnceCell<Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>> = OnceCell::new();
// Shared connection status
static WS_STATUS: OnceCell<Arc<Mutex<String>>> = OnceCell::new();

#[tauri::command]
fn store_token(token: String) -> Result<(), String> {
    // store in OS keyring using Entry API
    let entry = keyring::Entry::new("beringshare-client", "current_user");
    entry.set_password(&token).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_token() -> Result<String, String> {
    let entry = keyring::Entry::new("beringshare-client", "current_user");
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_token() -> Result<(), String> {
    let entry = keyring::Entry::new("beringshare-client", "current_user");
    entry.delete_password().map_err(|e| e.to_string())
}

#[tauri::command]
fn store_config(base_url: String) -> Result<(), String> {
    use std::fs;
    use std::path::PathBuf;
    let cfg = serde_json::json!({"base_url": base_url});
    let dir: PathBuf = match dirs::config_dir() {
        Some(d) => d.join("beringshare-desktop"),
        None => return Err("Could not determine config dir".into()),
    };
    if let Err(e) = fs::create_dir_all(&dir) {
        return Err(e.to_string());
    }
    let path = dir.join("config.json");
    match fs::write(&path, serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn get_config() -> Result<String, String> {
    use std::fs;
    use std::path::PathBuf;
    let dir: PathBuf = dirs::config_dir().ok_or_else(|| "Could not determine config dir".to_string())?.join("beringshare-desktop");
    let path = dir.join("config.json");
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let base = v.get("base_url").and_then(|b| b.as_str()).ok_or_else(|| "base_url not set".to_string())?;
    Ok(base.to_string())
}

#[tauri::command]
async fn ws_connect_native(window: tauri::Window) -> Result<(), String> {
    // Spawn a background task to establish a native websocket to /api/ws
    tauri::async_runtime::spawn(async move {
        use tokio_tungstenite::connect_async;
        use url::Url;

        let entry = keyring::Entry::new("beringshare-client", "current_user");
        let token = match entry.get_password() {
            Ok(t) => t,
            Err(_) => return,
        };

        let mut url = match Url::parse("ws://127.0.0.1:8080/api/ws") {
            Ok(u) => u,
            Err(_) => return,
        };
        url.query_pairs_mut().append_pair("token", &token);

        // create outgoing channel and store sender for `send_ws_message` command
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let sender_arc = WS_SENDER.get_or_init(|| Arc::new(Mutex::new(None))).clone();
        {
            let mut guard = sender_arc.lock().await;
            *guard = Some(tx);
        }
        let status_arc = WS_STATUS.get_or_init(|| Arc::new(Mutex::new("disconnected".to_string()))).clone();
        {
            let mut s = status_arc.lock().await;
            *s = "connected".to_string();
        }
        let _ = window.emit("ws:status", "connected");

        if let Ok((ws_stream, _)) = connect_async(url.as_str()).await {
            let (mut write, mut read) = ws_stream.split();

            // spawn a task to forward outgoing messages into the ws write sink
            let write_task = tauri::async_runtime::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let _ = write.send(TungMessage::Text(msg)).await;
                }
            });

            // read incoming messages and emit to frontend
            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    if msg.is_text() {
                        let _ = window.emit("ws:event", msg.into_text().unwrap());
                    }
                } else {
                    break;
                }
            }

            // ensure write task is dropped
            write_task.abort();

            // connection ended -- clear sender and update status
            {
                let mut guard = sender_arc.lock().await;
                *guard = None;
            }
            {
                let mut s = status_arc.lock().await;
                *s = "disconnected".to_string();
            }
            let _ = window.emit("ws:status", "disconnected");
        }
    });
    Ok(())
}

#[tauri::command]
fn send_ws_message(message: String) -> Result<(), String> {
    if let Some(sender_arc) = WS_SENDER.get() {
        let rt = tokio::runtime::Handle::current();
        // Acquire lock and send if present
        let res = rt.block_on(async move {
            let guard = sender_arc.lock().await;
            if let Some(s) = &*guard {
                s.send(message).map_err(|e| e.to_string())
            } else {
                Err("ws not connected".to_string())
            }
        });
        res
    } else {
        Err("ws not initialized".into())
    }
}

#[tauri::command]
fn get_ws_status() -> Result<String, String> {
    if let Some(status_arc) = WS_STATUS.get() {
        let rt = tokio::runtime::Handle::current();
        let s = rt.block_on(async move { status_arc.lock().await.clone() });
        Ok(s)
    } else {
        Ok("disconnected".into())
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![store_token, get_token, clear_token, ws_connect_native, send_ws_message, get_ws_status, store_config, get_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
