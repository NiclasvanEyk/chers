use std::fmt::Debug;

use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;

#[derive(Debug)]
pub struct Commands<'s> {
    socket: &'s mut WebSocket,
}

impl<'s> Commands<'s> {
    pub fn new(socket: &'s mut WebSocket) -> Self {
        Self { socket }
    }

    #[tracing::instrument]
    pub async fn send(&mut self, command: impl Serialize + Debug) -> Result<(), axum::Error> {
        let serialized = serde_json::to_string(&command).unwrap();
        let message = Message::Text(serialized);
        self.socket.send(message).await
    }
}
