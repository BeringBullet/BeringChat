use axum::Router;

use crate::api::AppState;

pub mod handlers;
pub mod outbox;
pub mod protocol;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/messages", axum::routing::post(handlers::receive_message))
        .route(
            "/channel-memberships",
            axum::routing::post(handlers::receive_channel_membership),
        )
        .route("/presence", axum::routing::get(handlers::presence))
        .route("/users", axum::routing::get(handlers::list_users))
        .route("/channels", axum::routing::get(handlers::list_channels))
        .route("/webrtc-signal", axum::routing::post(handlers::receive_webrtc_signal))
        .route("/channel-call-event", axum::routing::post(handlers::receive_channel_call_event))
}
