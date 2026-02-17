use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub token: String,
    pub server_id: Option<Uuid>,
    pub is_local: bool,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub origin_server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub kind: MessageKind,
    pub body: String,
    pub author_user_id: Uuid,
    pub recipient_user_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,
    pub sent_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationToken {
    pub id: Uuid,
    pub token: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    Dm,
    Channel,
}
