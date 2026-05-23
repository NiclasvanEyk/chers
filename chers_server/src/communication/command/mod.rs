pub mod distributed;
pub mod in_memory;

use std::future::Future;
use std::pin::Pin;
use tokio::sync::oneshot;
use tokio_stream::Stream;

pub use distributed::DistributedCommandBus;
pub use in_memory::LocalCommandBus;

// Re-export command types from the shared API crate
pub use chers_server_api::v2::commands::{
    Command, CommandResponse, CommandResult, GameCommand, LobbyCommand, PostGameCommand,
};

// Re-export sync types
pub use chers_server_api::v2::sync::RoomStateMirror;

use crate::room::RoomId;
use crate::utils::AnyResult;

// ---------------------------------------------------------------------------
// CommandBus trait
// ---------------------------------------------------------------------------

/// Point-to-point bus for sending commands to room actors and
/// receiving back a response.
pub trait CommandBus: Clone + Send + Sync + 'static {
    type Cmd: Send;

    /// Send a command to the actor responsible for `room_id` and await its
    /// response.
    fn send(
        &self,
        room_id: &RoomId,
        command: Self::Cmd,
    ) -> impl Future<Output = Result<CommandResponse, CommandBusError>> + Send;

    /// Subscribe to commands addressed to `room_id` (used by the actor loop).
    fn subscribe(
        &self,
        room_id: &RoomId,
    ) -> impl Future<Output = AnyResult<ReceivedStream<Self::Cmd>>> + Send;
}

/// A stream of received commands, each carrying a response channel back to
/// the original sender.
pub type ReceivedStream<Cmd> = Pin<Box<dyn Stream<Item = ReceivedCommand<Cmd>> + Send>>;

/// A command that was delivered to an actor, together with a channel to
/// respond to the original sender.
pub struct ReceivedCommand<Cmd> {
    pub command: Cmd,
    pub response_channel: ResponseChannel,
}

// ---------------------------------------------------------------------------
// Response channel
// ---------------------------------------------------------------------------

/// Mechanism used to send a response back to the caller.
///
/// For the local in-memory bus this wraps a [`oneshot::Sender`];
/// for the distributed bus the transport internally bridges the response
/// back through the wire.
pub enum ResponseChannel {
    Local(oneshot::Sender<CommandResponse>),
}

impl ResponseChannel {
    pub async fn respond(self, response: CommandResponse) {
        match self {
            Self::Local(tx) => {
                let _ = tx.send(response);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum CommandBusError {
    RoomNotFound,
    ActorDropped,
    Timeout,
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for CommandBusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoomNotFound => write!(f, "room not found"),
            Self::ActorDropped => write!(f, "actor dropped before responding"),
            Self::Timeout => write!(f, "response timed out"),
            Self::Other(err) => write!(f, "bus error: {err}"),
        }
    }
}

impl std::error::Error for CommandBusError {}

// ---------------------------------------------------------------------------
// AnyCommandBus — enum dispatch for CommandBus
// ---------------------------------------------------------------------------

/// Type-erased bus that dispatches to [`LocalCommandBus`] or
/// [`DistributedCommandBus`] at runtime.
pub enum AnyCommandBus {
    Local(LocalCommandBus),
    Distributed(
        DistributedCommandBus<
            crate::communication::transport::pubsub::PubSubCommandTransport<
                crate::communication::bus::AnyEventBus<Vec<u8>>,
            >,
        >,
    ),
    #[cfg(feature = "nats")]
    Nats(DistributedCommandBus<crate::communication::transport::nats::NatsCommandTransport>),
}

impl Clone for AnyCommandBus {
    fn clone(&self) -> Self {
        match self {
            Self::Local(t) => Self::Local(t.clone()),
            Self::Distributed(t) => Self::Distributed(t.clone()),
            #[cfg(feature = "nats")]
            Self::Nats(t) => Self::Nats(t.clone()),
        }
    }
}

impl CommandBus for AnyCommandBus {
    type Cmd = Command;

    fn send(
        &self,
        room_id: &RoomId,
        command: Self::Cmd,
    ) -> impl Future<Output = Result<CommandResponse, CommandBusError>> + Send {
        async move {
            match self {
                Self::Local(t) => t.send(room_id, command).await,
                Self::Distributed(t) => t.send(room_id, command).await,
                #[cfg(feature = "nats")]
                Self::Nats(t) => t.send(room_id, command).await,
            }
        }
    }

    fn subscribe(
        &self,
        room_id: &RoomId,
    ) -> impl Future<Output = AnyResult<ReceivedStream<Self::Cmd>>> + Send {
        async move {
            match self {
                Self::Local(t) => t.subscribe(room_id).await,
                Self::Distributed(t) => t.subscribe(room_id).await,
                #[cfg(feature = "nats")]
                Self::Nats(t) => t.subscribe(room_id).await,
            }
        }
    }
}
