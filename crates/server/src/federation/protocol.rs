use serde::{Deserialize, Serialize};

use crate::domain::MessageKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedUser {
    pub username: String,
    pub server: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedChannel {
    pub name: String,
    pub origin_server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedMessage {
    pub message_id: String,
    pub sent_at: String,
    pub kind: MessageKind,
    pub body: String,
    pub author: FederatedUser,
    pub recipient: Option<FederatedUser>,
    pub channel: Option<FederatedChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedChannelMembership {
    pub channel: FederatedChannel,
    pub member: FederatedUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedWebRtcSignal {
    pub from_user: FederatedUser,
    pub to_user: FederatedUser,
    pub signal_type: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedChannelCallEvent {
    pub channel: FederatedChannel,
    pub event: String, // "join" or "leave"
    pub participant: FederatedUser,
    pub participant_user_id: String,
}
