use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::communication::transport::CommandTransport;
use crate::room::RoomId;
use crate::utils::AnyResult;

use super::{
    Command, CommandBus, CommandBusError, CommandResponse, ReceivedCommand, ReceivedStream,
    ResponseChannel,
};

/// A command bus that routes messages over a generic [`CommandTransport`].
///
/// Commands are serialized with serde_json and sent through the transport.
/// The transport handles the wire protocol (inbox subjects, native
/// request-reply, …); this bus only handles command / response
/// serialization.
///
/// ## Backend-specific behaviour
///
/// | `T` | `send()` | `subscribe()` |
/// |---|---|---|
/// | `PubSubCommandTransport` | Inbox pub/sub | Pub/sub with envelope |
/// | NATS (future) | Native `conn.request()` | Native inbox |
#[derive(Clone)]
pub struct DistributedCommandBus<T> {
    transport: Arc<T>,
}

impl<T: CommandTransport> DistributedCommandBus<T> {
    pub fn new(transport: Arc<T>) -> Self {
        Self { transport }
    }
}

impl<T: CommandTransport> CommandBus for DistributedCommandBus<T> {
    type Cmd = Command;

    async fn send(
        &self,
        room_id: &RoomId,
        command: Command,
    ) -> Result<CommandResponse, CommandBusError> {
        let payload = serde_json::to_vec(&command).map_err(|e| CommandBusError::Other(e.into()))?;

        let raw = self
            .transport
            .request(room_id, payload, Duration::from_secs(30))
            .await
            .map_err(|e| CommandBusError::Other(e.into()))?;

        serde_json::from_slice(&raw).map_err(|e| CommandBusError::Other(e.into()))
    }

    async fn subscribe(&self, room_id: &RoomId) -> AnyResult<ReceivedStream<Command>> {
        let mut incoming = self.transport.subscribe(room_id).await?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(req) = incoming.next().await {
                let command: Command = match serde_json::from_slice(&req.payload) {
                    Ok(cmd) => cmd,
                    Err(_) => continue,
                };

                let (response_tx, response_rx) = oneshot::channel();

                tokio::spawn(async move {
                    if let Ok(response) = response_rx.await {
                        if let Ok(payload) = serde_json::to_vec(&response) {
                            req.respond(payload).await;
                        }
                    }
                });

                let _ = tx.send(ReceivedCommand {
                    command,
                    response_channel: ResponseChannel::Local(response_tx),
                });
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }
}
