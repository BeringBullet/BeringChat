use axum::{routing::get, Router};
use reqwest::Client;

use crate::{auth::Sessions, channel_call::ChannelCallStore, config::Config, federation::protocol::{FederatedChannel, FederatedUser}, presence::PresenceStore, storage::SqliteStore, websocket::MessageBroadcaster};

pub mod admin;
pub mod messages;
pub mod web;
// ws_bridge is defined at crate root (`crate::ws_bridge`) and exposed in `src/lib.rs`

#[derive(Clone)]
pub struct AppState {
    pub store: SqliteStore,
    pub config: Config,
    pub http: Client,
    pub sessions: Sessions,
    pub message_broadcaster: MessageBroadcaster,
    pub presence: PresenceStore,
    pub channel_calls: ChannelCallStore,
}

pub fn router(store: SqliteStore, config: Config) -> Router {
    let http = Client::new();
    let sessions = Sessions::new();
    let message_broadcaster = crate::websocket::create_broadcaster();
    let presence = PresenceStore::new();
    let channel_calls = ChannelCallStore::new();
    let state = AppState { store: store.clone(), config: config.clone(), http: http.clone(), sessions, message_broadcaster: message_broadcaster.clone(), presence: presence.clone(), channel_calls };

    // Start background presence sync task
    let server_name = config.server_name.clone();
    let broadcaster_clone = message_broadcaster.clone();
    tokio::spawn(async move {
        tracing::info!(target: "presence", "🔄 Presence sync task started for server '{}'. Will check remote servers every 2 seconds", server_name);
        presence_sync_task(store.clone(), http.clone(), config.clone(), presence.clone(), broadcaster_clone).await
    });

    Router::new()
        .route("/health", get(health))
        .route("/admin/ui", get(web::admin_ui))
        .route("/chat/ui", get(web::chat_ui))
        .route("/chat/settings", get(web::settings_ui))
    .route("/api/events", get(crate::websocket::sse_handler))
    .route("/api/ws", get(crate::ws_bridge::ws_handler))
        .nest("/admin", admin::router())
        .nest("/api", messages::router())
        .nest("/federation", crate::federation::router())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn presence_sync_task(
    store: SqliteStore,
    http: Client,
    config: Config,
    presence: PresenceStore,
    broadcaster: crate::websocket::MessageBroadcaster,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let servers = match store.list_servers() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(target: "presence", "Failed to list servers for presence sync: {}", e);
                continue;
            }
        };

        tracing::debug!(target: "presence", "Starting presence sync for server '{}'. Known servers: {:?}", config.server_name, servers.iter().map(|s| &s.name).collect::<Vec<_>>());

        let mut presence_changed = false;
        for server in &servers {
            let url = format!("{}/federation/presence", server.base_url);
            tracing::debug!(target: "presence", "Querying {} for online users (token: {})", url, config.server_token);

            match http
                .get(&url)
                .header("x-federation-token", &config.server_token)
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    tracing::debug!(target: "presence", "Response from {} {}: {}", server.name, url, status);

                    if status.is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if let Some(online_users) = json.get("online_users").and_then(|v| v.as_array()) {
                                    // Clear old presence for this server
                                    if presence.clear_remote_server(&server.name) {
                                        presence_changed = true;
                                    }

                                    let usernames: Vec<String> = online_users
                                        .iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|username| format!("{}@{}", username, server.name))
                                        .collect();

                                    if !usernames.is_empty() {
                                        tracing::debug!(target: "presence", "Synced {} online users from server '{}': {:?}", usernames.len(), server.name, usernames);
                                        presence.set_remote_users_online(usernames);
                                        presence_changed = true;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!(target: "presence", "Failed to parse presence response from {}: {}", server.name, e);
                            }
                        }
                    } else {
                        let body = response.text().await.unwrap_or_default();
                        tracing::debug!(target: "presence", "Presence query to {} failed with {}: {}", server.name, status, body);
                    }
                }
                Err(e) => {
                    tracing::debug!(target: "presence", "Failed to fetch presence from {} ({}): {}", server.name, url, e);
                }
            }
        }

        // Notify all connected clients if presence changed
        if presence_changed {
            crate::websocket::notify_presence_changed(&broadcaster);
        }

        // Sync channels from federated servers
        for server in &servers {
            let url = format!("{}/federation/channels", server.base_url.trim_end_matches('/'));
            match http
                .get(&url)
                .header("x-federation-token", &config.server_token)
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    match response.json::<Vec<FederatedChannel>>().await {
                        Ok(channels) => {
                            for ch in channels {
                                if store.get_channel_by_name_origin(&ch.name, &ch.origin_server).ok().flatten().is_none() {
                                    match store.create_channel(&ch.name, &ch.origin_server) {
                                        Ok(_) => tracing::info!(target: "federation", "Auto-synced channel '{}' from server '{}'", ch.name, ch.origin_server),
                                        Err(e) => tracing::warn!(target: "federation", "Failed to create synced channel '{}': {}", ch.name, e),
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::warn!(target: "federation", "Failed to parse channels from {}: {}", server.name, e),
                    }
                }
                Ok(response) => {
                    tracing::debug!(target: "federation", "Channel sync from {} returned {}", server.name, response.status());
                }
                Err(e) => {
                    tracing::debug!(target: "federation", "Channel sync from {} failed: {}", server.name, e);
                }
            }
        }

        // Sync display_names from federated servers
        for server in &servers {
            let url = format!("{}/federation/users", server.base_url.trim_end_matches('/'));
            match http
                .get(&url)
                .header("x-federation-token", &config.server_token)
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    match response.json::<Vec<FederatedUser>>().await {
                        Ok(remote_users) => {
                            for ru in remote_users {
                                if let Ok(Some(local_ref)) = store.get_user_by_name_and_server(&ru.username, Some(server.id)) {
                                    if local_ref.display_name != ru.display_name {
                                        let _ = store.update_user_display_name(&local_ref.id, ru.display_name.as_deref());
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::debug!(target: "federation", "Failed to parse users from {}: {}", server.name, e),
                    }
                }
                _ => {}
            }
        }
    }
}
