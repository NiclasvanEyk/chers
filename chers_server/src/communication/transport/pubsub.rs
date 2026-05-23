use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::room::RoomId;
use crate::utils::AnyResult;

use super::{CommandStream, CommandTransport, IncomingRequest};

static NEXT_INBOX: AtomicU64 = AtomicU64::new(1);

/// A [`CommandTransport`] backed by a subject-based [`EventBus`].
///
/// Serialized command payloads are wrapped in a JSON transport envelope and
/// published to `cmd.{room_id}`.  Responses are routed via per-request inbox
/// subjects (`_inbox.{n}`) — the same pattern that [`DistributedCommandBus`]
/// used directly before the transport abstraction was extracted.
///
/// This is the **default / fallback** implementation.  It works with any
/// `EventBus<Item = Vec<u8>>` (local, Redis, …).
#[derive(Clone)]
pub struct PubSubCommandTransport<B> {
    bus: Arc<B>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TransportEnvelope {
    reply_to: String,
    payload: Vec<u8>,
}

impl<B> PubSubCommandTransport<B> {
    pub fn new(bus: Arc<B>) -> Self {
        Self { bus }
    }
}

impl<B: crate::communication::bus::EventBus<Item = Vec<u8>>> CommandTransport
    for PubSubCommandTransport<B>
{
    async fn request(
        &self,
        room: &RoomId,
        payload: Vec<u8>,
        timeout: Duration,
    ) -> AnyResult<Vec<u8>> {
        let inbox = NEXT_INBOX.fetch_add(1, Ordering::Relaxed);
        let reply_to = format!("_inbox.{inbox}");

        let mut replies = self.bus.subscribe(&reply_to).await?;

        let envelope = TransportEnvelope {
            reply_to: reply_to.clone(),
            payload,
        };
        let raw = serde_json::to_vec(&envelope)?;

        self.bus.publish(&format!("cmd.{room}"), raw).await?;

        let result = tokio::time::timeout(timeout, replies.next()).await;

        self.bus.remove(&reply_to).await;

        match result {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Err("inbox stream ended without response".into()),
            Err(_) => Err("request timed out".into()),
        }
    }

    async fn subscribe(&self, room: &RoomId) -> AnyResult<CommandStream> {
        let mut raw_stream = self.bus.subscribe(&format!("cmd.{room}")).await?;

        let bus = Arc::clone(&self.bus);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(raw) = raw_stream.next().await {
                let envelope: TransportEnvelope = match serde_json::from_slice(&raw) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let (respond_tx, respond_rx) = oneshot::channel();
                let bus = Arc::clone(&bus);

                tokio::spawn(async move {
                    if let Ok(response) = respond_rx.await {
                        let _ = bus.publish(&envelope.reply_to, response).await;
                    }
                });

                let _ = tx.send(IncomingRequest::new(envelope.payload, respond_tx));
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)) as CommandStream)
    }
}
