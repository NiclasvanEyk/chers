use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;

pub async fn announce(to: &mut WebSocket, what: impl Serialize) {
    let Ok(serialized) = serde_json::to_string(&what) else {
        tracing::error!("Failed to serialize anouncement!");
        return;
    };

    to.send(Message::Text(serialized)).await;
}
