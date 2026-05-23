pub mod pubsub;

#[cfg(feature = "nats")]
pub mod nats;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio_stream::Stream;

use crate::room::RoomId;
use crate::utils::AnyResult;

// ---------------------------------------------------------------------------
// CommandTransport trait
// ---------------------------------------------------------------------------

/// Low-level request-reply transport for distributed commands.
///
/// Implementations handle the wire protocol — how serialized command bytes
/// reach a room and how the response makes it back.  The trait is generic
/// over bytes; command / response serialization is handled by
/// [`DistributedCommandBus`](super::command::DistributedCommandBus).
///
/// ## Backends
///
/// | Backend | `request` | `subscribe` |
/// |---|---|---|
/// | `PubSubCommandTransport` | Inbox pub/sub | Pub/sub with envelope |
/// | `NatsCommandTransport` | Native `Client::request_with_timeout()` | Native inbox |
pub trait CommandTransport: Clone + Send + Sync + 'static {
    fn request(
        &self,
        room: &RoomId,
        payload: Vec<u8>,
        timeout: Duration,
    ) -> impl Future<Output = AnyResult<Vec<u8>>> + Send;

    fn subscribe(&self, room: &RoomId) -> impl Future<Output = AnyResult<CommandStream>> + Send;
}

// ---------------------------------------------------------------------------
// Incoming request
// ---------------------------------------------------------------------------

/// A command request received from the transport, together with a channel to
/// send the response back through the transport.
pub struct IncomingRequest {
    pub payload: Vec<u8>,
    respond_tx: oneshot::Sender<Vec<u8>>,
}

impl IncomingRequest {
    pub fn new(payload: Vec<u8>, respond_tx: oneshot::Sender<Vec<u8>>) -> Self {
        Self {
            payload,
            respond_tx,
        }
    }

    pub async fn respond(self, response: Vec<u8>) {
        let _ = self.respond_tx.send(response);
    }
}

pub type CommandStream = Pin<Box<dyn Stream<Item = IncomingRequest> + Send>>;
