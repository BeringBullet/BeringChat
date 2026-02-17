use axum::{extract::Path, routing::{delete, get, post, put}, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::AppState,
    auth::AdminGuard,
    domain::{Channel, FederationToken, Server, User},
    error::AppError,
    federation::{outbox, protocol::{FederatedChannel, FederatedChannelMembership, FederatedUser}},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/sync-federated", post(sync_federated_users))
        .route("/debug/sync-federated", get(debug_sync_federated))
        .route("/users/:user_id", delete(delete_user))
        .route("/users/:user_id", put(update_user))
        .route("/servers", post(register_server))
        .route("/servers", get(list_servers))
        .route("/servers/:server_id", delete(delete_server))
        .route("/servers/:server_id", put(update_server))
        .route("/servers/:server_id/visibility", get(get_server_visibility))
        .route("/servers/:server_id/visibility", put(set_server_visibility))
        .route("/channels", post(create_channel))
        .route("/channels", get(list_channels))
        .route("/channels/sync-federated", post(sync_federated_channels))
        .route("/channels/:channel_id", delete(delete_channel))
        .route("/channels/:channel_id", put(update_channel))
        .route("/channels/:channel_id/members", post(add_channel_member))
        .route("/server-info", get(server_info))
        .route("/federation-tokens", get(list_federation_tokens))
        .route("/federation-tokens", post(create_federation_token))
        .route("/federation-tokens/:token_id", delete(delete_federation_token))
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(
    state: axum::extract::State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if payload.username != state.config.admin_username || payload.password != state.config.admin_password {
        return Err(AppError::Unauthorized);
    }
    let session = state.sessions.create(3600); // 1 hour TTL
    Ok(Json(LoginResponse {
        token: session.token,
    }))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    password: Option<String>,
}

#[derive(Serialize)]
struct CreateUserResponse {
    id: Uuid,
    username: String,
    token: String,
}

async fn create_user(
    _admin: AdminGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let password_hash = match payload.password.as_deref() {
        Some(pw) if !pw.is_empty() => {
            let hash = bcrypt::hash(pw, bcrypt::DEFAULT_COST)
                .map_err(|_| AppError::Internal("password hashing failed".to_string()))?;
            Some(hash)
        }
        _ => None,
    };
    let user = state.store.create_user_with_password(
        &payload.username,
        true,
        None,
        password_hash.as_deref(),
    )?;
    Ok(Json(CreateUserResponse {
        id: user.id,
        username: user.username,
        token: user.token,
    }))
}

async fn list_users(
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.store.list_users()?;
    Ok(Json(users))
}

#[derive(Deserialize)]
struct RegisterServerRequest {
    name: String,
    base_url: String,
    token: Option<String>,
}

async fn register_server(
    _admin: AdminGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<RegisterServerRequest>,
) -> Result<Json<Server>, AppError> {
    let token = payload.token.filter(|t| !t.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let server = state
        .store
        .create_server(&payload.name, &payload.base_url, &token)?;
    Ok(Json(server))
}

async fn list_servers(
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<Server>>, AppError> {
    let servers = state.store.list_servers()?;
    Ok(Json(servers))
}

#[derive(Deserialize)]
struct CreateChannelRequest {
    name: String,
}

async fn create_channel(
    _admin: AdminGuard,
    state: axum::extract::State<AppState>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<Channel>, AppError> {
    let channel = state
        .store
        .create_channel(&payload.name, &state.config.server_name)?;
    Ok(Json(channel))
}

async fn list_channels(
    state: axum::extract::State<AppState>,
) -> Result<Json<Vec<Channel>>, AppError> {
    let channels = state.store.list_channels()?;
    Ok(Json(channels))
}

#[derive(Deserialize)]
struct AddMemberRequest {
    username: String,
}

async fn add_channel_member(
    _admin: AdminGuard,
    state: axum::extract::State<AppState>,
    axum::extract::Path(channel_id): axum::extract::Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<Json<User>, AppError> {
    let (member_name, server_name) = if let Some((user, server)) = payload.username.split_once('@') {
        (user.to_string(), Some(server.to_string()))
    } else {
        (payload.username.clone(), None)
    };

    let (server_id, target_server) = if let Some(server_name) = server_name.as_deref() {
        let server = state
            .store
            .get_server_by_name(server_name)?
            .ok_or_else(|| AppError::BadRequest(format!("unknown server: {}", server_name)))?;
        (Some(server.id), Some(server))
    } else {
        (None, None)
    };

    let is_local = server_id.is_none();
    let existing = state
        .store
        .get_user_by_name_and_server(&member_name, server_id)?;
    let user = match existing {
        Some(user) => user,
        None => state
            .store
            .create_user(&member_name, is_local, server_id)?,
    };

    state.store.add_channel_member(channel_id, user.id)?;

    if let Some(server) = target_server {
        let channel = state
            .store
            .get_channel_by_id(channel_id)?
            .ok_or_else(|| AppError::BadRequest("unknown channel".to_string()))?;
        let membership = FederatedChannelMembership {
            channel: FederatedChannel {
                name: channel.name,
                origin_server: channel.origin_server,
            },
            member: FederatedUser {
                username: user.username.clone(),
                server: server.name.clone(),
                display_name: None,
            },
        };
        outbox::send_channel_membership(
            &state.http,
            &state.config.server_token,
            &server,
            &membership,
        )
        .await?;
    }
    Ok(Json(user))
}

async fn delete_user(
    _guard: AdminGuard,
    Path(user_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<()>, AppError> {
    let id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    state.store.delete_user(&id)?;
    Ok(Json(()))
}

async fn delete_server(
    _guard: AdminGuard,
    Path(server_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<()>, AppError> {
    let id = Uuid::parse_str(&server_id)
        .map_err(|_| AppError::BadRequest("Invalid server ID".to_string()))?;
    state.store.delete_server(&id)?;
    Ok(Json(()))
}

async fn delete_channel(
    _guard: AdminGuard,
    Path(channel_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<()>, AppError> {
    let id = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
    state.store.delete_channel(&id)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    username: String,
    display_name: Option<String>,
    password: Option<String>,
}

async fn update_user(
    _guard: AdminGuard,
    Path(user_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    let id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let user = state.store.update_user(&id, &payload.username, payload.display_name.as_deref())?;

    if let Some(pw) = payload.password.as_deref() {
        if !pw.is_empty() {
            let hash = bcrypt::hash(pw, bcrypt::DEFAULT_COST)
                .map_err(|_| AppError::Internal("password hashing failed".to_string()))?;
            state.store.set_user_password(&id, &hash)?;
        }
    }

    Ok(Json(user))
}

#[derive(Deserialize)]
struct UpdateServerRequest {
    name: String,
    base_url: String,
    token: Option<String>,
}

async fn update_server(
    _guard: AdminGuard,
    Path(server_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<Json<Server>, AppError> {
    let id = Uuid::parse_str(&server_id)
        .map_err(|_| AppError::BadRequest("Invalid server ID".to_string()))?;
    let token = payload.token.filter(|t| !t.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let server = state.store.update_server(&id, &payload.name, &payload.base_url, &token)?;
    Ok(Json(server))
}

#[derive(Deserialize)]
struct UpdateChannelRequest {
    name: String,
}

async fn update_channel(
    _guard: AdminGuard,
    Path(channel_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<Json<Channel>, AppError> {
    let id = Uuid::parse_str(&channel_id)
        .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
    let channel = state.store.update_channel(&id, &payload.name)?;
    Ok(Json(channel))
}

async fn sync_federated_users(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let servers = state.store.list_servers()?;
    let mut synced_users = Vec::new();

    for server in servers {
        let url = format!("{}/federation/users", server.base_url.trim_end_matches('/'));
        
        let response = match state
            .http
            .get(&url)
            .header("x-federation-token", &state.config.server_token)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to fetch from {}: {}", url, e);
                return Err(AppError::Internal(format!("Failed to fetch users from {}: {}", server.name, e)));
            }
        };

        if !response.status().is_success() {
            eprintln!("Server {} returned status {}", url, response.status());
            return Err(AppError::Internal(format!("Server {} returned {}", server.name, response.status())));
        }

        match response.json::<Vec<FederatedUser>>().await {
            Ok(users) => {
                for remote_user in users {
                    // Skip if user is already local on this server
                    if state
                        .store
                        .get_user_by_name_and_server(&remote_user.username, None)?
                        .is_some()
                    {
                        continue;
                    }

                    // Update display_name if already synced from this server
                    if let Some(existing) = state
                        .store
                        .get_user_by_name_and_server(&remote_user.username, Some(server.id))?
                    {
                        if existing.display_name != remote_user.display_name {
                            let _ = state.store.update_user_display_name(&existing.id, remote_user.display_name.as_deref());
                        }
                        continue;
                    }

                    let user = state.store.create_user(
                        &remote_user.username,
                        false,
                        Some(server.id),
                    )?;
                    synced_users.push(user);
                }
            }
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", url, e);
                return Err(AppError::Internal(format!("Failed to parse users from {}: {}", server.name, e)));
            }
        }
    }

    Ok(Json(synced_users))
}

async fn sync_federated_channels(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<Channel>>, AppError> {
    let servers = state.store.list_servers()?;
    let mut synced_channels = Vec::new();

    for server in servers {
        let url = format!("{}/federation/channels", server.base_url.trim_end_matches('/'));

        let response = match state
            .http
            .get(&url)
            .header("x-federation-token", &state.config.server_token)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!("Failed to fetch channels from {}: {}", url, e);
                continue;
            }
        };

        if !response.status().is_success() {
            tracing::warn!("Server {} returned status {} for channel sync", server.name, response.status());
            continue;
        }

        match response.json::<Vec<FederatedChannel>>().await {
            Ok(channels) => {
                for remote_channel in channels {
                    // Skip if already exists locally
                    if state
                        .store
                        .get_channel_by_name_origin(&remote_channel.name, &remote_channel.origin_server)?
                        .is_some()
                    {
                        continue;
                    }

                    let channel = state.store.create_channel(
                        &remote_channel.name,
                        &remote_channel.origin_server,
                    )?;
                    synced_channels.push(channel);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse channels from {}: {}", server.name, e);
                continue;
            }
        }
    }

    Ok(Json(synced_channels))
}

#[derive(Serialize)]
struct SyncDiagnostic {
    server_name: String,
    admin_token: String,
    servers: Vec<ServerDiagnostic>,
}

#[derive(Serialize)]
struct ServerDiagnostic {
    name: String,
    base_url: String,
    token: String,
    fetch_url: String,
}

async fn debug_sync_federated(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<SyncDiagnostic>, AppError> {
    let servers = state.store.list_servers()?;
    let server_diags = servers
        .into_iter()
        .map(|s| {
            let fetch_url = format!("{}/federation/users", s.base_url.trim_end_matches('/'));
            ServerDiagnostic {
                name: s.name,
                base_url: s.base_url,
                token: s.token,
                fetch_url,
            }
        })
        .collect();

    Ok(Json(SyncDiagnostic {
        server_name: state.config.server_name.clone(),
        admin_token: state.config.admin_token.clone(),
        servers: server_diags,
    }))
}

// --- Federation Token Management ---

#[derive(Serialize)]
struct ServerInfoResponse {
    server_name: String,
    server_token: String,
}

async fn server_info(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<ServerInfoResponse>, AppError> {
    Ok(Json(ServerInfoResponse {
        server_name: state.config.server_name.clone(),
        server_token: state.config.server_token.clone(),
    }))
}

async fn list_federation_tokens(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<FederationToken>>, AppError> {
    let tokens = state.store.list_federation_tokens()?;
    Ok(Json(tokens))
}

#[derive(Deserialize)]
struct CreateFederationTokenRequest {
    label: String,
}

async fn create_federation_token(
    _admin: AdminGuard,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateFederationTokenRequest>,
) -> Result<Json<FederationToken>, AppError> {
    let label = payload.label.trim().to_string();
    if label.is_empty() {
        return Err(AppError::BadRequest("label is required".to_string()));
    }
    let token = state.store.create_federation_token(&label)?;
    Ok(Json(token))
}

async fn delete_federation_token(
    _admin: AdminGuard,
    Path(token_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<()>, AppError> {
    let id = Uuid::parse_str(&token_id)
        .map_err(|_| AppError::BadRequest("Invalid token ID".to_string()))?;
    state.store.delete_federation_token(&id)?;
    Ok(Json(()))
}

#[derive(Serialize, Deserialize)]
struct ServerVisibility {
    hidden_user_ids: Vec<String>,
    hidden_channel_ids: Vec<String>,
}

async fn get_server_visibility(
    _admin: AdminGuard,
    Path(server_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<ServerVisibility>, AppError> {
    let id = Uuid::parse_str(&server_id)
        .map_err(|_| AppError::BadRequest("Invalid server ID".to_string()))?;
    let hidden_user_ids = state.store.get_hidden_user_ids(id)?;
    let hidden_channel_ids = state.store.get_hidden_channel_ids(id)?;
    Ok(Json(ServerVisibility {
        hidden_user_ids,
        hidden_channel_ids,
    }))
}

async fn set_server_visibility(
    _admin: AdminGuard,
    Path(server_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<ServerVisibility>,
) -> Result<Json<ServerVisibility>, AppError> {
    let id = Uuid::parse_str(&server_id)
        .map_err(|_| AppError::BadRequest("Invalid server ID".to_string()))?;
    state.store.set_hidden_users(id, &payload.hidden_user_ids)?;
    state.store.set_hidden_channels(id, &payload.hidden_channel_ids)?;
    Ok(Json(ServerVisibility {
        hidden_user_ids: payload.hidden_user_ids,
        hidden_channel_ids: payload.hidden_channel_ids,
    }))
}
