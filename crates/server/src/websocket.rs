use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse, KeepAlive},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use futures_util::stream::unfold;

use crate::{api::AppState, channel_call::ChannelCallStore, config::Config, error::AppError, presence::PresenceGuard, storage::SqliteStore};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageNotification {
    pub event: String,
    pub user_id: Option<String>,
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

#[derive(Deserialize)]
pub struct SseQuery {
    token: Option<String>,
}

pub type MessageBroadcaster = Arc<broadcast::Sender<MessageNotification>>;

pub fn create_broadcaster() -> MessageBroadcaster {
    let (tx, _) = broadcast::channel::<MessageNotification>(100);
    Arc::new(tx)
}

pub async fn sse_handler(
    State(state): State<AppState>,
    Query(params): Query<SseQuery>,
) -> Result<impl IntoResponse, AppError> {
    let token = params.token.ok_or(AppError::Unauthorized)?;

    // Check user session first, fall back to DB token
    let user = if let Some(user_id) = state.sessions.validate_user_session(&token) {
        state.store.get_user_by_id(user_id)?.ok_or(AppError::Unauthorized)?
    } else {
        state.store.get_user_by_token(&token)?.ok_or(AppError::Unauthorized)?
    };
    
    tracing::info!(target: "presence", "🟢 SSE user '{}' (id: {}) coming ONLINE", user.username, user.id);
    
    // Broadcast that someone came online
    notify_presence_changed(&state.message_broadcaster);
    
    let guard = PresenceGuard::new(state.presence.clone(), user.id);
    let rx = state.message_broadcaster.subscribe();
    
    // Helper to notify when the stream is closed
    struct NotifyDrop {
        broadcaster: MessageBroadcaster,
        username: String,
        server_name: String,
        channel_calls: ChannelCallStore,
        user_id: String,
        display_name: Option<String>,
        store: SqliteStore,
        config: Config,
        http: reqwest::Client,
    }
    impl Drop for NotifyDrop {
        fn drop(&mut self) {
            tracing::info!(target: "presence", "🔴 SSE connection closed for '{}', broadcasting presence change", self.username);
            // Clean up any group calls this user was in
            let affected_channels = self.channel_calls.leave_all(&self.username, &self.server_name);
            for channel_id in &affected_channels {
                let payload = serde_json::json!({
                    "username": self.username,
                    "server_name": self.server_name,
                }).to_string();
                notify_channel_call_event(
                    &self.broadcaster,
                    "channel_call_leave",
                    &channel_id.to_string(),
                    &payload,
                );
            }
            // Broadcast leave events to federated servers
            if !affected_channels.is_empty() {
                let store = self.store.clone();
                let config = self.config.clone();
                let http = self.http.clone();
                let username = self.username.clone();
                let server_name = self.server_name.clone();
                let user_id = self.user_id.clone();
                let display_name = self.display_name.clone();
                let affected = affected_channels.clone();
                tokio::spawn(async move {
                    let servers = store.list_servers().unwrap_or_default();
                    for channel_id in affected {
                        if let Ok(Some(channel)) = store.get_channel_by_id(channel_id) {
                            let fed_event = crate::federation::protocol::FederatedChannelCallEvent {
                                channel: crate::federation::protocol::FederatedChannel {
                                    name: channel.name,
                                    origin_server: channel.origin_server,
                                },
                                event: "leave".to_string(),
                                participant: crate::federation::protocol::FederatedUser {
                                    username: username.clone(),
                                    server: server_name.clone(),
                                    display_name: display_name.clone(),
                                },
                                participant_user_id: user_id.clone(),
                            };
                            for server in &servers {
                                let _ = crate::federation::outbox::send_channel_call_event(
                                    &http,
                                    &config.server_token,
                                    server,
                                    &fed_event,
                                ).await;
                            }
                        }
                    }
                });
            }
            notify_presence_changed(&self.broadcaster);
        }
    }

    let notify_drop = NotifyDrop {
        broadcaster: state.message_broadcaster.clone(),
        username: user.username.clone(),
        server_name: state.config.server_name.clone(),
        channel_calls: state.channel_calls.clone(),
        user_id: user.id.to_string(),
        display_name: user.display_name.clone(),
        store: state.store.clone(),
        config: state.config.clone(),
        http: state.http.clone(),
    };
    
    // Use futures stream to convert async recv operations to a stream
    let stream = unfold((rx, guard, notify_drop), |(mut rx, guard, nd)| async move {
        match rx.recv().await {
            Ok(notification) => {
                if let Ok(json) = serde_json::to_string(&notification) {
                    Some((Ok::<Event, std::io::Error>(Event::default().data(json)), (rx, guard, nd)))
                } else {
                    Some((Ok(Event::default().data("{}")), (rx, guard, nd)))
                }
            }
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub fn notify_new_message(
    broadcaster: &MessageBroadcaster,
    user_id: Option<String>,
    channel_id: Option<String>,
) {
    let notification = MessageNotification {
        event: "new_message".to_string(),
        user_id,
        channel_id,
        target_user_id: None,
        payload: None,
    };
    
    // Ignore if no subscribers
    let _ = broadcaster.send(notification);
}

pub fn notify_presence_changed(
    broadcaster: &MessageBroadcaster,
) {
    let notification = MessageNotification {
        event: "presence_changed".to_string(),
        user_id: None,
        channel_id: None,
        target_user_id: None,
        payload: None,
    };
    tracing::debug!("Broadcasting presence_changed event");
    let _ = broadcaster.send(notification);
}

pub fn notify_webrtc_signal(
    broadcaster: &MessageBroadcaster,
    target_user_id: &str,
    payload: &str,
) {
    let notification = MessageNotification {
        event: "webrtc_signal".to_string(),
        user_id: None,
        channel_id: None,
        target_user_id: Some(target_user_id.to_string()),
        payload: Some(payload.to_string()),
    };
    let _ = broadcaster.send(notification);
}

pub fn notify_channel_call_event(
    broadcaster: &MessageBroadcaster,
    event: &str,
    channel_id: &str,
    payload: &str,
) {
    let notification = MessageNotification {
        event: event.to_string(),
        user_id: None,
        channel_id: Some(channel_id.to_string()),
        target_user_id: None,
        payload: Some(payload.to_string()),
    };
    let _ = broadcaster.send(notification);
}

