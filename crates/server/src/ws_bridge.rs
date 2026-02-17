use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use axum::http::HeaderMap;
use futures_util::StreamExt;
use serde_json::json;

use crate::api::AppState;

async fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("x-admin-token") {
        if let Ok(s) = v.to_str() {
            return Some(s.to_string());
        }
    }
    if let Some(v) = headers.get("authorization") {
        if let Ok(s) = v.to_str() {
            let s = s.trim();
            if let Some(t) = s.strip_prefix("Bearer ") {
                return Some(t.to_string());
            }
        }
    }
    None
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract token and validate user
    let token = match extract_token_from_headers(&headers).await {
        Some(t) => t,
        None => return (axum::http::StatusCode::UNAUTHORIZED, "Missing token").into_response(),
    };

    // Check user session first, fall back to DB token
    let user = if let Some(user_id) = state.sessions.validate_user_session(&token) {
        match state.store.get_user_by_id(user_id).ok().flatten() {
            Some(u) => u,
            None => return (axum::http::StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
        }
    } else {
        match state.store.get_user_by_token(&token).ok().flatten() {
            Some(u) => u,
            None => return (axum::http::StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
        }
    };

    ws.on_upgrade(move |socket| bridge(socket, state, token, user.id.to_string())).into_response()
}

async fn bridge(mut socket: WebSocket, state: AppState, token: String, user_id: String) {
    // Subscribe to internal broadcaster (same source as SSE)
    let mut rx = state.message_broadcaster.subscribe();

    // Send a welcome message
    let welcome = json!({"type":"connected","user_id": user_id});
    if socket.send(WsMessage::Text(welcome.to_string())).await.is_err() {
        return; // client disconnected immediately
    }

    loop {
        tokio::select! {
            biased;
            // Messages from SSE broadcaster
            msg = rx.recv() => {
                match msg {
                    Ok(notification) => {
                        if let Ok(text) = serde_json::to_string(&notification) {
                            if socket.send(WsMessage::Text(text)).await.is_err() {
                                // client disconnected
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        // broadcaster closed
                        let _ = socket.send(WsMessage::Text(json!({"type":"_closed"}).to_string())).await;
                        return;
                    }
                }
            }
            // Messages from client
            client_msg = socket.next() => {
                match client_msg {
                    Some(Ok(WsMessage::Text(t))) => {
                        // Try to parse an outgoing message envelope and forward to HTTP API
                        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&t) {
                            if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
                                match kind {
                                    "send_channel_message" => {
                                        if let (Some(channel_id), Some(body)) = (obj.get("channel_id").and_then(|v| v.as_str()), obj.get("body")) {
                                            let url = format!("{}/api/channels/{}/messages", state.config.base_url.trim_end_matches('/'), channel_id);
                                            let _ = state.http.post(&url)
                                                .bearer_auth(&token)
                                                .json(&json!({"body": body}))
                                                .send().await;
                                        }
                                    }
                                    "send_dm" => {
                                        if let (Some(target_user), Some(body)) = (obj.get("target_user").and_then(|v| v.as_str()), obj.get("body")) {
                                            let url = format!("{}/api/messages/dm", state.config.base_url.trim_end_matches('/'));
                                            let _ = state.http.post(&url)
                                                .bearer_auth(&token)
                                                .json(&json!({"to": target_user, "body": body}))
                                                .send().await;
                                        }
                                    }
                                    _ => {
                                        // unknown kind - ignore or log
                                        tracing::debug!(target: "ws_bridge", "Unknown client ws message kind: {}", kind);
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(WsMessage::Ping(p))) => {
                        let _ = socket.send(WsMessage::Pong(p)).await;
                    }
                    Some(Ok(WsMessage::Close(_))) => {
                        return;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        tracing::warn!(target: "ws_bridge", "websocket error: {}", e);
                        return;
                    }
                    None => return, // stream ended
                }
            }
        }
    }
}
