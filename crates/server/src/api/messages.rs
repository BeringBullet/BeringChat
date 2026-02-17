use axum::{extract::{Path, Query}, routing::{delete, get, post, put}, Json, Router};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use std::collections::HashMap;

use crate::{
    api::AppState,
    auth::UserGuard,
    channel_call::CallParticipant,
    domain::{Channel, MessageKind, User},
    error::AppError,
    federation::{outbox, protocol::{FederatedChannel, FederatedChannelCallEvent, FederatedMessage, FederatedUser, FederatedWebRtcSignal}},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(user_login))
        .route("/messages/dm", post(send_dm))
        .route("/messages/channel", post(send_channel))
        .route("/messages/inbox", get(get_inbox))
        .route("/messages/channel/:channel_id", get(get_channel_messages))
        .route("/messages/dm/:user_id", get(get_dm_messages))
        .route("/users", get(list_all_users))
        .route("/channels", get(list_all_channels))
        .route("/channels", post(create_channel_user))
        .route("/channels/active-calls", get(channel_active_calls))
        .route("/channels/:channel_id/members", post(add_channel_member_user))
        .route("/channels/:channel_id/members/:user_id", delete(remove_channel_member))
        .route("/channels/:channel_id/call/join", post(channel_call_join))
        .route("/channels/:channel_id/call/leave", post(channel_call_leave))
        .route("/channels/:channel_id/call/participants", get(channel_call_participants))
        .route("/call/signal", post(call_signal))
        .route("/profile", put(update_profile))
        .route("/profile/password", put(change_password))
        .route("/gif/search", get(gif_search))
}

#[derive(Deserialize)]
struct UserLoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct UserLoginResponse {
    user_id: String,
    username: String,
    token: String,
    display_name: Option<String>,
}

async fn user_login(
    state: axum::extract::State<AppState>,
    Json(payload): Json<UserLoginRequest>,
) -> Result<Json<UserLoginResponse>, AppError> {
    let user = state
        .store
        .get_user_by_name_and_server(&payload.username, None)?
        .ok_or(AppError::Unauthorized)?;

    // Verify password if the user has one set
    if let Some(hash) = state.store.get_user_password_hash(&user.id)? {
        let valid = bcrypt::verify(&payload.password, &hash)
            .map_err(|_| AppError::Internal("password verification failed".to_string()))?;
        if !valid {
            return Err(AppError::Unauthorized);
        }
    }
    // If no password_hash set, allow login without password (migration grace period)

    // Create a 24-hour user session
    let session = state.sessions.create_user_session(user.id, 86400);

    Ok(Json(UserLoginResponse {
        user_id: user.id.to_string(),
        display_name: user.display_name,
        username: user.username,
        token: session.token,
    }))
}

#[derive(Deserialize)]
struct SendDmRequest {
    recipient: String,
    body: String,
}

#[derive(Serialize)]
struct SendMessageResponse {
    message_id: String,
}

async fn send_dm(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<SendDmRequest>,
) -> Result<Json<SendMessageResponse>, AppError> {
    let (recipient_name, recipient_server_name) = split_recipient(&payload.recipient, &state.config.server_name);
    
    let (recipient_user, recipient_server) = if recipient_server_name == state.config.server_name {
        // Local recipient
        let u = if let Some(u) = state.store.get_user_by_name_and_server(&recipient_name, None)? {
            u
        } else {
            // Auto-create local user if not found (matching original logic but safer)
            state.store.create_user(&recipient_name, true, None)?
        };
        (u, None)
    } else {
        // Remote recipient
        let server = state
            .store
            .get_server_by_name(&recipient_server_name)?
            .ok_or_else(|| AppError::BadRequest(format!("unknown server: {}", recipient_server_name)))?;
        
        let u = if let Some(u) = state.store.get_user_by_name_and_server(&recipient_name, Some(server.id))? {
            u
        } else {
            // Create remote user reference
            state.store.create_user(&recipient_name, false, Some(server.id))?
        };
        (u, Some(server))
    };
    
    let sent_at = OffsetDateTime::now_utc().format(&Rfc3339).map_err(|e| AppError::Internal(e.to_string()))?;
    let message = state.store.create_message(
        MessageKind::Dm,
        &payload.body,
        user.id,
        Some(recipient_user.id),
        None,
        &sent_at,
    )?;

    // Notify recipient of new message
    crate::websocket::notify_new_message(
        &state.message_broadcaster,
        Some(recipient_user.id.to_string()),
        None,
    );

    if let Some(server) = recipient_server {
        let fed_message = FederatedMessage {
            message_id: message.id.to_string(),
            sent_at,
            kind: MessageKind::Dm,
            body: payload.body,
            author: FederatedUser {
                username: user.username,
                server: state.config.server_name.clone(),
                display_name: None,
            },
            recipient: Some(FederatedUser {
                username: recipient_user.username,
                server: recipient_server_name,
                display_name: None,
            }),
            channel: None,
        };
        outbox::send_to_server(
            &state.http,
            &state.config.server_token,
            &server,
            &fed_message,
        )
        .await?;
    }

    Ok(Json(SendMessageResponse {
        message_id: message.id.to_string(),
    }))
}

#[derive(Deserialize)]
struct SendChannelRequest {
    channel: String,
    origin_server: Option<String>,
    body: String,
}

async fn send_channel(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<SendChannelRequest>,
) -> Result<Json<SendMessageResponse>, AppError> {
    let origin_server = payload.origin_server.as_deref().unwrap_or(&state.config.server_name);
    let channel = state
        .store
        .get_channel_by_name_origin(&payload.channel, origin_server)?
        .ok_or_else(|| AppError::BadRequest("unknown channel".to_string()))?;

    let sent_at = OffsetDateTime::now_utc().format(&Rfc3339).map_err(|e| AppError::Internal(e.to_string()))?;
    let message = state.store.create_message(
        MessageKind::Channel,
        &payload.body,
        user.id,
        None,
        Some(channel.id),
        &sent_at,
    )?;

    // Notify local channel members of new message
    crate::websocket::notify_new_message(
        &state.message_broadcaster,
        None,
        Some(channel.id.to_string()),
    );

    let fed_message = FederatedMessage {
        message_id: message.id.to_string(),
        sent_at,
        kind: MessageKind::Channel,
        body: payload.body,
        author: FederatedUser {
            username: user.username,
            server: state.config.server_name.clone(),
            display_name: None,
        },
        recipient: None,
        channel: Some(FederatedChannel {
            name: channel.name,
            origin_server: channel.origin_server.clone(),
        }),
    };

    // If we are not the origin server, we should also send it to the origin server
    // (In this simple model, we send to ALL members' servers, which includes origin)
    outbox::send_to_channel_members(
        &state.http,
        &state.store,
        &state.config.server_name,
        &state.config.server_token,
        channel.id,
        &fed_message,
    )
    .await?;
    
    // If it's a remote channel, ensure we send to origin server even if no other local users are in it
    if channel.origin_server != state.config.server_name {
        if let Some(server) = state.store.get_server_by_name(&channel.origin_server)? {
            // Only send if not already sent by send_to_channel_members
            let member_servers = state.store.list_channel_member_servers(channel.id)?;
            if !member_servers.iter().any(|s| s.name == server.name) {
                outbox::send_to_server(
                    &state.http,
                    &state.config.server_token,
                    &server,
                    &fed_message,
                ).await?;
            }
        }
    }

    Ok(Json(SendMessageResponse {
        message_id: message.id.to_string(),
    }))
}

fn split_recipient(recipient: &str, default_server: &str) -> (String, String) {
    if let Some((user, server)) = recipient.split_once('@') {
        (user.to_string(), server.to_string())
    } else {
        (recipient.to_string(), default_server.to_string())
    }
}

#[derive(Serialize)]
struct InboxMessage {
    message_id: String,
    kind: MessageKind,
    body: String,
    author_user_id: String,
    recipient_user_id: Option<String>,
    channel_id: Option<String>,
    sent_at: String,
}

async fn get_inbox(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<InboxMessage>>, AppError> {
    let messages = state.store.list_messages_for_user(user.id, 50)?;
    let inbox = messages
        .into_iter()
        .map(|message| InboxMessage {
            message_id: message.id.to_string(),
            kind: message.kind,
            body: message.body,
            author_user_id: message.author_user_id.to_string(),
            recipient_user_id: message.recipient_user_id.map(|id| id.to_string()),
            channel_id: message.channel_id.map(|id| id.to_string()),
            sent_at: message.sent_at,
        })
        .collect();
    Ok(Json(inbox))
}

#[derive(Serialize)]
struct MessageRecord {
    message_id: String,
    body: String,
    author_user_id: String,
    author_username: String,
    author_display_name: Option<String>,
    sent_at: String,
}

#[derive(Serialize)]
struct UserListItem {
    id: String,
    username: String,
    token: String,
    server_id: Option<String>,
    is_local: bool,
    server_name: Option<String>,
    is_online: bool,
    display_name: Option<String>,
}

async fn get_channel_messages(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<MessageRecord>>, AppError> {
    let id = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
    let messages = state.store.list_channel_messages(id)?;
    let records = messages
        .into_iter()
        .map(|msg| {
            let author_user = state.store.get_user_by_id(msg.author_user_id)
                .ok()
                .flatten();
            MessageRecord {
                message_id: msg.id.to_string(),
                body: msg.body,
                author_user_id: msg.author_user_id.to_string(),
                author_username: author_user.as_ref().map(|u| u.username.clone()).unwrap_or_default(),
                author_display_name: author_user.as_ref().and_then(|u| u.display_name.clone()),
                sent_at: msg.sent_at,
            }
        })
        .collect();
    Ok(Json(records))
}

async fn get_dm_messages(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(other_user_id): Path<String>,
) -> Result<Json<Vec<MessageRecord>>, AppError> {
    let other_id = Uuid::parse_str(&other_user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let messages = state.store.list_dm_messages(user.id, other_id)?;
    let records = messages
        .into_iter()
        .map(|msg| {
            let author_user = state.store.get_user_by_id(msg.author_user_id)
                .ok()
                .flatten();
            MessageRecord {
                message_id: msg.id.to_string(),
                body: msg.body,
                author_user_id: msg.author_user_id.to_string(),
                author_username: author_user.as_ref().map(|u| u.username.clone()).unwrap_or_default(),
                author_display_name: author_user.as_ref().and_then(|u| u.display_name.clone()),
                sent_at: msg.sent_at,
            }
        })
        .collect();
    Ok(Json(records))
}

async fn list_all_users(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<UserListItem>>, AppError> {
    let users = state.store.list_users()?;
    let mut results = Vec::with_capacity(users.len());
    for user in users {
        let server_name = match user.server_id {
            Some(server_id) => state.store.get_server_by_id(&server_id)?.map(|s| s.name),
            None => None,
        };
        
        // Check local presence for local users, remote presence for remote users
        let is_online = if user.is_local {
            state.presence.is_online(user.id)
        } else {
            // For remote users, check using username@server format
            match &server_name {
                Some(srv) => {
                    let remote_key = format!("{}@{}", user.username, srv);
                    state.presence.is_remote_user_online(&remote_key)
                }
                None => false,
            }
        };
        
        results.push(UserListItem {
            id: user.id.to_string(),
            username: user.username,
            token: user.token,
            server_id: user.server_id.map(|id| id.to_string()),
            is_local: user.is_local,
            server_name,
            is_online,
            display_name: user.display_name,
        });
    }
    Ok(Json(results))
}

async fn list_all_channels(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<Channel>>, AppError> {
    let channels = state.store.list_channels()?;
    Ok(Json(channels))
}

#[derive(Deserialize)]
struct CreateChannelRequest {
    name: String,
}

async fn create_channel_user(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<Channel>, AppError> {
    let channel = state
        .store
        .create_channel(&payload.name, &state.config.server_name)?;
    Ok(Json(channel))
}

#[derive(Deserialize)]
struct AddMemberRequest {
    user_id: String,
}

async fn add_channel_member_user(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(channel_id): Path<String>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<Json<User>, AppError> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
    let user_uuid = Uuid::parse_str(&payload.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    
    let user = state.store.get_user_by_id(user_uuid)?
        .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;
    
    state.store.add_channel_member(channel_uuid, user_uuid)?;
    Ok(Json(user))
}

async fn remove_channel_member(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Path((channel_id, user_id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    
    state.store.remove_channel_member(channel_uuid, user_uuid)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
struct CallSignalRequest {
    target: String,
    signal_type: String,
    payload: String,
}

async fn call_signal(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(req): Json<CallSignalRequest>,
) -> Result<Json<&'static str>, AppError> {
    let (target_name, target_server_name) =
        split_recipient(&req.target, &state.config.server_name);

    let signal = FederatedWebRtcSignal {
        from_user: FederatedUser {
            username: user.username.clone(),
            server: state.config.server_name.clone(),
            display_name: None,
        },
        to_user: FederatedUser {
            username: target_name.clone(),
            server: target_server_name.clone(),
            display_name: None,
        },
        signal_type: req.signal_type,
        payload: req.payload,
    };

    if target_server_name == state.config.server_name {
        let target_user = state
            .store
            .get_user_by_name_and_server(&target_name, None)?
            .ok_or_else(|| AppError::BadRequest("unknown user".to_string()))?;

        let sse_payload = serde_json::to_string(&signal)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        crate::websocket::notify_webrtc_signal(
            &state.message_broadcaster,
            &target_user.id.to_string(),
            &sse_payload,
        );
    } else {
        let server = state
            .store
            .get_server_by_name(&target_server_name)?
            .ok_or_else(|| {
                AppError::BadRequest(format!("unknown server: {}", target_server_name))
            })?;

        outbox::send_webrtc_signal(
            &state.http,
            &state.config.server_token,
            &server,
            &signal,
        )
        .await?;
    }

    Ok(Json("ok"))
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    display_name: Option<String>,
}

async fn update_profile(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<User>, AppError> {
    let updated = state.store.update_user(&user.id, &user.username, payload.display_name.as_deref())?;
    Ok(Json(updated))
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    current_password: Option<String>,
    new_password: String,
}

async fn change_password(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<&'static str>, AppError> {
    // Verify current password if user has one
    if let Some(hash) = state.store.get_user_password_hash(&user.id)? {
        let current = payload.current_password.as_deref().unwrap_or("");
        let valid = bcrypt::verify(current, &hash)
            .map_err(|_| AppError::Internal("password verification failed".to_string()))?;
        if !valid {
            return Err(AppError::Unauthorized);
        }
    }

    let new_hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::Internal("password hashing failed".to_string()))?;
    state.store.set_user_password(&user.id, &new_hash)?;

    Ok(Json("ok"))
}

// --- Channel Group Call Endpoints ---

#[derive(Serialize)]
struct CallParticipantResponse {
    username: String,
    server_name: String,
    user_id: String,
}

#[derive(Serialize)]
struct ChannelCallJoinResponse {
    participants: Vec<CallParticipantResponse>,
}

async fn channel_call_join(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(channel_id): Path<String>,
) -> Result<Json<ChannelCallJoinResponse>, AppError> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;

    let participant = CallParticipant {
        username: user.username.clone(),
        server_name: state.config.server_name.clone(),
        user_id: user.id.to_string(),
    };

    let participants = state.channel_calls.join(channel_uuid, participant);

    // Broadcast join event to all SSE clients
    let payload = serde_json::json!({
        "username": user.username,
        "server_name": state.config.server_name,
        "user_id": user.id.to_string(),
    }).to_string();
    crate::websocket::notify_channel_call_event(
        &state.message_broadcaster,
        "channel_call_join",
        &channel_id,
        &payload,
    );

    // Broadcast to federated servers
    if let Ok(Some(channel)) = state.store.get_channel_by_id(channel_uuid) {
        let fed_event = FederatedChannelCallEvent {
            channel: FederatedChannel {
                name: channel.name,
                origin_server: channel.origin_server,
            },
            event: "join".to_string(),
            participant: FederatedUser {
                username: user.username.clone(),
                server: state.config.server_name.clone(),
                display_name: user.display_name.clone(),
            },
            participant_user_id: user.id.to_string(),
        };
        let servers = state.store.list_servers().unwrap_or_default();
        for server in servers {
            let _ = outbox::send_channel_call_event(
                &state.http,
                &state.config.server_token,
                &server,
                &fed_event,
            ).await;
        }
    }

    let resp = participants.into_iter().map(|p| CallParticipantResponse {
        username: p.username,
        server_name: p.server_name,
        user_id: p.user_id,
    }).collect();

    Ok(Json(ChannelCallJoinResponse { participants: resp }))
}

async fn channel_call_leave(
    UserGuard(user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(channel_id): Path<String>,
) -> Result<Json<&'static str>, AppError> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;

    let participant = CallParticipant {
        username: user.username.clone(),
        server_name: state.config.server_name.clone(),
        user_id: user.id.to_string(),
    };

    state.channel_calls.leave(channel_uuid, &participant);

    let payload = serde_json::json!({
        "username": user.username,
        "server_name": state.config.server_name,
        "user_id": user.id.to_string(),
    }).to_string();
    crate::websocket::notify_channel_call_event(
        &state.message_broadcaster,
        "channel_call_leave",
        &channel_id,
        &payload,
    );

    // Broadcast to federated servers
    if let Ok(Some(channel)) = state.store.get_channel_by_id(channel_uuid) {
        let fed_event = FederatedChannelCallEvent {
            channel: FederatedChannel {
                name: channel.name,
                origin_server: channel.origin_server,
            },
            event: "leave".to_string(),
            participant: FederatedUser {
                username: user.username.clone(),
                server: state.config.server_name.clone(),
                display_name: user.display_name.clone(),
            },
            participant_user_id: user.id.to_string(),
        };
        let servers = state.store.list_servers().unwrap_or_default();
        for server in servers {
            let _ = outbox::send_channel_call_event(
                &state.http,
                &state.config.server_token,
                &server,
                &fed_event,
            ).await;
        }
    }

    Ok(Json("ok"))
}

async fn channel_call_participants(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<CallParticipantResponse>>, AppError> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;

    let participants = state.channel_calls.participants(channel_uuid);
    let resp = participants.into_iter().map(|p| CallParticipantResponse {
        username: p.username,
        server_name: p.server_name,
        user_id: p.user_id,
    }).collect();

    Ok(Json(resp))
}

async fn channel_active_calls(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
) -> Result<Json<HashMap<String, usize>>, AppError> {
    Ok(Json(state.channel_calls.all_active_calls()))
}

#[derive(Deserialize)]
struct GifSearchQuery {
    q: String,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct GifResult {
    url: String,
    preview_url: String,
    width: u32,
    height: u32,
}

async fn gif_search(
    UserGuard(_user): UserGuard,
    state: axum::extract::State<AppState>,
    Query(params): Query<GifSearchQuery>,
) -> Result<Json<Vec<GifResult>>, AppError> {
    let api_key = state
        .config
        .tenor_api_key
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("GIF search is not configured. Set TENOR_API_KEY on the server.".to_string()))?;

    let limit = params.limit.unwrap_or(20).min(50);
    let url = format!(
        "https://tenor.googleapis.com/v2/search?q={}&key={}&limit={}&media_filter=gif,tinygif",
        urlencoding::encode(&params.q),
        urlencoding::encode(api_key),
        limit,
    );

    let resp = state.http.get(&url).send().await?;
    let body: serde_json::Value = resp.json().await.map_err(|e| AppError::Internal(e.to_string()))?;

    let results = body
        .get("results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let media = item.get("media_formats")?;
                    let gif = media.get("gif")?;
                    let tinygif = media.get("tinygif").unwrap_or(gif);
                    Some(GifResult {
                        url: gif.get("url")?.as_str()?.to_string(),
                        preview_url: tinygif.get("url")?.as_str()?.to_string(),
                        width: gif.get("dims")?.as_array()?.first()?.as_u64()? as u32,
                        height: gif.get("dims")?.as_array()?.get(1)?.as_u64()? as u32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(results))
}

