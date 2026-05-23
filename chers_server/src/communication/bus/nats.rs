use std::marker::PhantomData;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::utils::AnyResult;

use super::EventBus;

#[derive(Clone)]
pub struct NatsEventBus<T> {
    client: async_nats::Client,
    _marker: PhantomData<T>,
}

impl<T> NatsEventBus<T> {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static> EventBus
    for NatsEventBus<T>
{
    type Item = T;

    async fn publish(&self, subject: &str, item: T) -> AnyResult<()> {
        let payload = serde_json::to_vec(&item)?;
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> AnyResult<Pin<Box<dyn Stream<Item = T> + Send>>> {
        let mut subscriber = self.client.subscribe(subject.to_string()).await?;
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                match serde_json::from_slice::<T>(&msg.payload) {
                    Ok(item) => {
                        if tx.send(item).is_err() {
                            break;
                        }
                    }
                    Err(e) => tracing::warn!("nats event deserialization error: {e}"),
                }
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }

    async fn remove(&self, _subject: &str) {
        // NATS subjects need no explicit cleanup.
    }
}
