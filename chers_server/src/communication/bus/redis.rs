use std::marker::PhantomData;
use std::pin::Pin;

use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::utils::AnyResult;

use super::EventBus;

/// A Redis-backed pub/sub bus for typed messages.
///
/// Items are serialized with serde_json and published as raw bytes on Redis
/// pub/sub channels whose names match the subject strings.
///
/// Use `RedisEventBus<Event>` for the event bus and `RedisEventBus<Vec<u8>>`
/// for the command transport's byte-level bus.
#[derive(Clone)]
pub struct RedisEventBus<T> {
    pub(crate) client: redis::Client,
    conn: ConnectionManager,
    _marker: PhantomData<T>,
}

impl<T> RedisEventBus<T> {
    pub fn new(client: redis::Client, conn: ConnectionManager) -> Self {
        Self {
            client,
            conn,
            _marker: PhantomData,
        }
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static> EventBus
    for RedisEventBus<T>
{
    type Item = T;

    async fn publish(&self, subject: &str, item: T) -> AnyResult<()> {
        let payload = serde_json::to_vec(&item)?;
        let mut conn = self.conn.clone();
        let _: () = conn.publish(subject, payload).await?;
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> AnyResult<Pin<Box<dyn Stream<Item = T> + Send>>> {
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe(subject).await?;

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = Box::pin(pubsub.on_message());

            while let Some(msg) = stream.next().await {
                let payload: Vec<u8> = match msg.get_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("redis bus payload error: {e}");
                        continue;
                    }
                };

                match serde_json::from_slice(&payload) {
                    Ok(item) => {
                        if tx.send(item).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("redis bus deserialization error: {e}");
                    }
                }
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }

    async fn remove(&self, _subject: &str) {
        // Redis pub/sub channels are ephemeral — no cleanup needed.
    }
}
