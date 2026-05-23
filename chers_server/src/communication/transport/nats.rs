use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::oneshot;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::room::RoomId;
use crate::utils::AnyResult;

use super::{CommandStream, CommandTransport, IncomingRequest};

/// A [`CommandTransport`] backed by NATS native request-reply.
///
/// Unlike [`PubSubCommandTransport`](super::pubsub::PubSubCommandTransport),
/// this implementation does **not** create inbox subjects or JSON envelopes.
/// NATS handles inbox creation at the protocol level — `request()` uses
/// the built-in `Client::request()`, and `subscribe()` replies via the
/// `reply` field that NATS sets on incoming request messages.
///
/// No per-request subscription churn, no application-level envelope, no
/// reply-subject naming convention.
#[derive(Clone)]
pub struct NatsCommandTransport {
    client: Arc<async_nats::Client>,
}

impl NatsCommandTransport {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

impl CommandTransport for NatsCommandTransport {
    async fn request(
        &self,
        room: &RoomId,
        payload: Vec<u8>,
        timeout: Duration,
    ) -> AnyResult<Vec<u8>> {
        let subject = format!("cmd.{room}");
        let msg = tokio::time::timeout(timeout, self.client.request(subject, payload.into()))
            .await
            .map_err(|_| "nats request timed out")?
            .map_err(|e| format!("nats request failed: {e}"))?;

        Ok(msg.payload.to_vec())
    }

    async fn subscribe(&self, room: &RoomId) -> AnyResult<CommandStream> {
        let mut subscriber = self
            .client
            .subscribe(format!("cmd.{room}"))
            .await
            .map_err(|e| format!("nats subscribe failed: {e}"))?;

        let client = Arc::clone(&self.client);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                let reply_to = match msg.reply {
                    Some(r) => r,
                    None => continue,
                };

                let (respond_tx, respond_rx) = oneshot::channel::<Vec<u8>>();
                let client = Arc::clone(&client);

                tokio::spawn(async move {
                    if let Ok(response) = respond_rx.await {
                        let _ = client.publish(reply_to, response.into()).await;
                    }
                });

                let _ = tx.send(IncomingRequest::new(msg.payload.to_vec(), respond_tx));
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)) as CommandStream)
    }
}
