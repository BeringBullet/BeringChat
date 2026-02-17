use axum::{Json, extract::State, http::HeaderMap};

use crate::{
    api::AppState,
    channel_call::CallParticipant,
    domain::{MessageKind, Server, User},
    error::AppError,
    federation::{outbox, protocol::{FederatedChannel, FederatedChannelCallEvent, FederatedChannelMembership, FederatedMessage, FederatedUser, FederatedWebRtcSignal}},
};

/// Extract the federation token from headers, then validate it against:
/// 1. The `servers` table (known server tokens) — returns `Some(server)` if found
/// 2. The `federation_tokens` table (additional accepted tokens) — returns `None` for server
/// 3. The primary `SERVER_TOKEN` env var — returns `None` for server
/// Returns `Err(Unauthorized)` if neither matches.
fn validate_federation_token(state: &AppState, headers: &HeaderMap) -> Result<Option<Server>, AppError> {
    let token = headers
        .get("x-federation-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    // Check servers table first
    if let Some(server) = state.store.get_server_by_token(token)? {
        return Ok(Some(server));
    }

    // Check federation_tokens table
    if state.store.is_valid_federation_token(token)? {
        return Ok(None);
    }

    // Check primary server token (this server's own token authenticates too)
    if token == state.config.server_token {
        return Ok(None);
    }

    tracing::debug!(target: "federation", "No server or federation token matches provided token");
    Err(AppError::Unauthorized)
}

pub async fn receive_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(message): Json<FederatedMessage>,
) -> Result<Json<&'static str>, AppError> {
    let caller_server = validate_federation_token(&state, &headers)?;

    // Ensure the declared author server exists in the DB (we still need its
    // record to store proper user references). If it's missing, reject to
    // avoid creating orphaned user records.
    let author_server = state
        .store
        .get_server_by_name(&message.author.server)?
        .ok_or_else(|| {
            tracing::warn!(target: "federation", "Author server {} not known", message.author.server);
            AppError::Unauthorized
        })?;

    if let Some(ref caller) = caller_server {
        if caller.name != author_server.name {
            tracing::warn!(target: "federation", "Message forwarded: author server '{}' differs from caller '{}'", author_server.name, caller.name);
        }
    }

    let author_user = ensure_remote_user(&state, &message.author).await?;

    match message.kind {
        MessageKind::Dm => handle_dm(&state, message, author_user).await?,
        MessageKind::Channel => handle_channel(&state, message, author_user).await?,
    };

    Ok(Json("ok"))
}

pub async fn receive_channel_membership(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FederatedChannelMembership>,
) -> Result<Json<&'static str>, AppError> {
    let token = headers
        .get("x-federation-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let origin_server = state
        .store
        .get_server_by_name(&payload.channel.origin_server)?
        .ok_or(AppError::Unauthorized)?;

    // Accept if token matches origin server's token OR is a valid federation token
    if origin_server.token != token
        && !state.store.is_valid_federation_token(token)?
        && token != state.config.server_token
    {
        return Err(AppError::Unauthorized);
    }

    if payload.member.server != state.config.server_name {
        return Err(AppError::BadRequest("membership target mismatch".to_string()));
    }

    let member_user = state
        .store
        .get_user_by_name_and_server(&payload.member.username, None)?
        .ok_or_else(|| AppError::BadRequest("unknown local member".to_string()))?;

    let channel_record = match state
        .store
        .get_channel_by_name_origin(&payload.channel.name, &payload.channel.origin_server)?
    {
        Some(channel) => channel,
        None => state
            .store
            .create_channel(&payload.channel.name, &payload.channel.origin_server)?,
    };

    state
        .store
        .add_channel_member(channel_record.id, member_user.id)?;

    Ok(Json("ok"))
}

#[derive(serde::Serialize)]
pub struct PresenceResponse {
    pub online_users: Vec<String>,
}

pub async fn presence(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PresenceResponse>, AppError> {
    let caller = validate_federation_token(&state, &headers)?;

    if let Some(ref server) = caller {
        tracing::debug!(target: "presence", "Token validated for server: {}", server.name);
    } else {
        tracing::debug!(target: "presence", "Token validated via federation token");
    }

    let hidden_user_ids = if let Some(ref server) = caller {
        state.store.get_hidden_user_ids(server.id)?
    } else {
        Vec::new()
    };

    let mut online_users = Vec::new();
    for user_id in state.presence.online_user_ids() {
        if hidden_user_ids.contains(&user_id.to_string()) {
            continue;
        }
        if let Some(user) = state.store.get_user_by_id(user_id)? {
            if user.is_local {
                online_users.push(user.username.clone());
                tracing::debug!(target: "presence", "  - {} (local user, online)", user.username);
            }
        }
    }

    tracing::debug!(target: "presence", "Responding with {} online users: {:?}", online_users.len(), online_users);

    Ok(Json(PresenceResponse { online_users }))
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<FederatedUser>>, AppError> {
    let caller = validate_federation_token(&state, &headers)?;

    let users = state.store.list_users()?;
    let hidden_user_ids = if let Some(ref server) = caller {
        state.store.get_hidden_user_ids(server.id)?
    } else {
        Vec::new()
    };

    let local_users = users
        .into_iter()
        .filter(|u| u.is_local)
        .filter(|u| !hidden_user_ids.contains(&u.id.to_string()))
        .map(|u| FederatedUser {
            username: u.username,
            server: state.config.server_name.clone(),
            display_name: u.display_name,
        })
        .collect();

    Ok(Json(local_users))
}

pub async fn list_channels(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<FederatedChannel>>, AppError> {
    let caller = validate_federation_token(&state, &headers)?;

    let channels = state.store.list_channels()?;
    let hidden_channel_ids = if let Some(ref server) = caller {
        state.store.get_hidden_channel_ids(server.id)?
    } else {
        Vec::new()
    };

    let local_channels = channels
        .into_iter()
        .filter(|c| c.origin_server == state.config.server_name)
        .filter(|c| !hidden_channel_ids.contains(&c.id.to_string()))
        .map(|c| FederatedChannel {
            name: c.name,
            origin_server: c.origin_server,
        })
        .collect();

    Ok(Json(local_channels))
}

pub async fn receive_webrtc_signal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(signal): Json<FederatedWebRtcSignal>,
) -> Result<Json<&'static str>, AppError> {
    let token = headers
        .get("x-federation-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let from_server = state
        .store
        .get_server_by_name(&signal.from_user.server)?
        .ok_or(AppError::Unauthorized)?;

    // Accept if token matches from_server's token OR is a valid federation token
    if from_server.token != token
        && !state.store.is_valid_federation_token(token)?
        && token != state.config.server_token
    {
        return Err(AppError::Unauthorized);
    }

    if signal.to_user.server != state.config.server_name {
        return Err(AppError::BadRequest(
            "webrtc signal target is not on this server".to_string(),
        ));
    }

    let target_user = state
        .store
        .get_user_by_name_and_server(&signal.to_user.username, None)?
        .ok_or_else(|| AppError::BadRequest("unknown local recipient".to_string()))?;

    let sse_payload = serde_json::to_string(&signal)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    crate::websocket::notify_webrtc_signal(
        &state.message_broadcaster,
        &target_user.id.to_string(),
        &sse_payload,
    );

    Ok(Json("ok"))
}

async fn handle_dm(
    state: &AppState,
    message: FederatedMessage,
    author_user: User,
) -> Result<(), AppError> {
    let recipient = message
        .recipient
        .ok_or_else(|| AppError::BadRequest("missing recipient".to_string()))?;
    let recipient_user = state
        .store
        .get_user_by_name_and_server(&recipient.username, None)?
        .ok_or_else(|| AppError::BadRequest("unknown local recipient".to_string()))?;

    // Insert using the federated message id to avoid duplicate processing.
    let created_opt = state.store.create_message_with_id(
        &message.message_id,
        MessageKind::Dm,
        &message.body,
        author_user.id,
        Some(recipient_user.id),
        None,
        &message.sent_at,
    )?;

    // If this message already exists, skip notification.
    if created_opt.is_none() {
        tracing::warn!(target: "federation", "duplicate DM received, skipping message_id={} to={}", message.message_id, recipient_user.username);
        return Ok(());
    }

    // Notify recipient of new message
    crate::websocket::notify_new_message(
        &state.message_broadcaster,
        Some(recipient_user.id.to_string()),
        None,
    );
    
    Ok(())
}

async fn handle_channel(
    state: &AppState,
    message: FederatedMessage,
    author_user: User,
) -> Result<(), AppError> {
    let channel = message
        .channel
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("missing channel".to_string()))?;
    let channel_record = match state
        .store
        .get_channel_by_name_origin(&channel.name, &channel.origin_server)?
    {
        Some(channel) => channel,
        None => state
            .store
            .create_channel(&channel.name, &channel.origin_server)?,
    };

    // Insert using the federated message id to avoid duplicate processing.
    let created_opt = state.store.create_message_with_id(
        &message.message_id,
        MessageKind::Channel,
        &message.body,
        author_user.id,
        None,
        Some(channel_record.id),
        &message.sent_at,
    )?;

    // If the message already exists, skip notify and fanout.
    if created_opt.is_none() {
        tracing::warn!(target: "federation", "duplicate channel message received, skipping message_id={}", message.message_id);
        return Ok(());
    }

    // Notify channel members of new message
    crate::websocket::notify_new_message(
        &state.message_broadcaster,
        None,
        Some(channel_record.id.to_string()),
    );

    // If this server owns the channel, fan out to all federated servers so
    // offline users still see the full history when they log in.
    if channel_record.origin_server == state.config.server_name {
        let servers = state.store.list_servers()?;
        for server in servers {
            if server.name == message.author.server {
                continue;
            }
            if let Err(e) = outbox::send_to_server(
                &state.http,
                &state.config.server_token,
                &server,
                &message,
            )
            .await
            {
                tracing::error!(target: "federation", server = %server.name, "fanout send failed: {:?}", e);
                // continue with the next server rather than failing the whole handler
            }
        }
    }
    
    Ok(())
}

pub async fn receive_channel_call_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<FederatedChannelCallEvent>,
) -> Result<Json<&'static str>, AppError> {
    let _caller = validate_federation_token(&state, &headers)?;

    // Look up the channel by name + origin_server to get local UUID
    let channel = state
        .store
        .get_channel_by_name_origin(&event.channel.name, &event.channel.origin_server)?
        .ok_or_else(|| {
            tracing::debug!(target: "federation", "Channel call event for unknown channel {}@{}", event.channel.name, event.channel.origin_server);
            AppError::BadRequest("unknown channel".to_string())
        })?;

    let participant = CallParticipant {
        username: event.participant.username.clone(),
        server_name: event.participant.server.clone(),
        user_id: event.participant_user_id.clone(),
    };

    match event.event.as_str() {
        "join" => {
            state.channel_calls.join(channel.id, participant);
        }
        "leave" => {
            state.channel_calls.leave(channel.id, &participant);
        }
        _ => {
            return Err(AppError::BadRequest("invalid event type".to_string()));
        }
    }

    // Broadcast SSE event to local clients so their UI updates
    let payload = serde_json::json!({
        "username": event.participant.username,
        "server_name": event.participant.server,
        "user_id": event.participant_user_id,
    }).to_string();
    let event_name = if event.event == "join" { "channel_call_join" } else { "channel_call_leave" };
    crate::websocket::notify_channel_call_event(
        &state.message_broadcaster,
        event_name,
        &channel.id.to_string(),
        &payload,
    );

    Ok(Json("ok"))
}

async fn ensure_remote_user(state: &AppState, user: &FederatedUser) -> Result<User, AppError> {
    let server = state
        .store
        .get_server_by_name(&user.server)?
        .ok_or(AppError::Unauthorized)?;
    if let Some(existing) = state
        .store
        .get_user_by_name_and_server(&user.username, Some(server.id))?
    {
        return Ok(existing);
    }
    state
        .store
        .create_user(&user.username, false, Some(server.id))
}
