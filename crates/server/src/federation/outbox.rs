use reqwest::Client;

use crate::{
    domain::Server,
    error::AppError,
    federation::protocol::{FederatedChannelCallEvent, FederatedChannelMembership, FederatedMessage, FederatedWebRtcSignal},
    storage::SqliteStore,
};

pub async fn send_to_server(
    http: &Client,
    local_token: &str,
    server: &Server,
    message: &FederatedMessage,
) -> Result<(), AppError> {
    let url = format!("{}/federation/messages", server.base_url.trim_end_matches('/'));
    tracing::info!(
        target = "federation",
        server = %server.name,
        url = %url,
        "sending federated message"
    );
    let resp = http
        .post(&url)
        .header("X-Federation-Token", local_token)
        .json(message)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::error!(
            target = "federation",
            server = %server.name,
            url = %url,
            status = %status,
            body = %body,
            "federation request failed"
        );
        return Err(AppError::Internal(format!(
            "federation http error: {}",
            status
        )));
    }
    Ok(())
}

pub async fn send_to_channel_members(
    http: &Client,
    store: &SqliteStore,
    local_server_name: &str,
    local_token: &str,
    channel_id: uuid::Uuid,
    message: &FederatedMessage,
) -> Result<(), AppError> {
    let servers = store.list_channel_member_servers(channel_id)?;
    for server in servers {
        if server.name == local_server_name {
            continue;
        }
        send_to_server(http, local_token, &server, message).await?;
    }
    Ok(())
}

pub async fn send_channel_membership(
    http: &Client,
    local_token: &str,
    server: &Server,
    membership: &FederatedChannelMembership,
) -> Result<(), AppError> {
    let url = format!(
        "{}/federation/channel-memberships",
        server.base_url.trim_end_matches('/')
    );
    http.post(url)
        .header("X-Federation-Token", local_token)
        .json(membership)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn send_channel_call_event(
    http: &Client,
    local_token: &str,
    server: &Server,
    event: &FederatedChannelCallEvent,
) -> Result<(), AppError> {
    let url = format!(
        "{}/federation/channel-call-event",
        server.base_url.trim_end_matches('/')
    );
    let resp = http
        .post(&url)
        .header("X-Federation-Token", local_token)
        .json(event)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        tracing::warn!(target: "federation", "channel-call-event to {} failed: {}", server.name, status);
    }
    Ok(())
}

pub async fn send_webrtc_signal(
    http: &Client,
    local_token: &str,
    server: &Server,
    signal: &FederatedWebRtcSignal,
) -> Result<(), AppError> {
    let url = format!(
        "{}/federation/webrtc-signal",
        server.base_url.trim_end_matches('/')
    );
    let resp = http
        .post(&url)
        .header("X-Federation-Token", local_token)
        .json(signal)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let _body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "federation webrtc signal error: {}",
            status
        )));
    }
    Ok(())
}
