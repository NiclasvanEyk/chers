use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::room::RoomId;
use crate::utils::AnyResult;

use super::{
    Command, CommandBus, CommandBusError, CommandResponse, ReceivedCommand, ReceivedStream,
    ResponseChannel,
};

/// An in-memory command bus using mpsc channels + oneshot replies.
///
/// Commands are delivered via an unbounded channel per room. The actor
/// receives the command together with a [`oneshot::Sender`] that it uses
/// to respond directly to the caller — zero serialization, same-process.
#[derive(Clone)]
pub struct LocalCommandBus {
    rooms: Arc<RwLock<HashMap<RoomId, mpsc::UnboundedSender<Envelope>>>>,
}

struct Envelope {
    command: Command,
    response_tx: oneshot::Sender<CommandResponse>,
}

impl LocalCommandBus {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for LocalCommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBus for LocalCommandBus {
    type Cmd = Command;

    async fn send(
        &self,
        room_id: &RoomId,
        command: Command,
    ) -> Result<CommandResponse, CommandBusError> {
        let (tx, rx) = oneshot::channel();
        let sender = self
            .rooms
            .read()
            .get(room_id.as_str())
            .cloned()
            .ok_or(CommandBusError::RoomNotFound)?;
        sender
            .send(Envelope {
                command,
                response_tx: tx,
            })
            .map_err(|_| CommandBusError::ActorDropped)?;
        rx.await.map_err(|_| CommandBusError::ActorDropped)
    }

    async fn subscribe(&self, room_id: &RoomId) -> AnyResult<ReceivedStream<Command>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.rooms.write().insert(room_id.clone(), tx);

        let stream = UnboundedReceiverStream::new(rx).map(
            |Envelope {
                 command,
                 response_tx,
             }| ReceivedCommand {
                command,
                response_channel: ResponseChannel::Local(response_tx),
            },
        );

        Ok(Box::pin(stream) as ReceivedStream<Command>)
    }
}
