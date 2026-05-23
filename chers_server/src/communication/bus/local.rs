use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use parking_lot::RwLock;

use tokio::sync::broadcast;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use crate::utils::AnyResult;

use super::EventBus;

/// The per-subject broadcast channel capacity used by [`InMemoryBus::new`].
const DEFAULT_CHANNEL_CAPACITY: usize = 32;

#[derive(Clone)]
pub struct LocalEventBus<T> {
    subjects: Arc<RwLock<HashMap<String, broadcast::Sender<T>>>>,
    capacity: usize,
}

impl<T> Default for LocalEventBus<T> {
    fn default() -> Self {
        Self {
            subjects: Arc::new(RwLock::new(HashMap::new())),
            capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
}

impl<T> LocalEventBus<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            subjects: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }
}

impl<T: Clone + Send + 'static> EventBus for LocalEventBus<T> {
    type Item = T;

    async fn publish(&self, subject: &str, item: T) -> AnyResult<()> {
        let subjects = self.subjects.read();
        if let Some(sender) = subjects.get(subject) {
            if let Err(err) = sender.send(item) {
                tracing::warn!("published to subject with no receivers: {err}");
            }
        }
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> AnyResult<Pin<Box<dyn Stream<Item = T> + Send>>> {
        let mut subjects = self.subjects.write();
        let sender = subjects
            .entry(subject.to_string())
            .or_insert_with(|| broadcast::channel(self.capacity).0);
        let receiver = sender.subscribe();
        let stream = BroadcastStream::new(receiver).filter_map(|result| match result {
            Ok(item) => Some(item),
            Err(BroadcastStreamRecvError::Lagged(n)) => {
                tracing::warn!("subscriber lagged by {n} events");
                None
            }
        });
        Ok(Box::pin(stream))
    }

    async fn remove(&self, subject: &str) {
        self.subjects.write().remove(subject);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn subscriber_receives_published_event() {
        let bus = LocalEventBus::<&'static str>::new();
        let mut rx = bus.subscribe("greetings").await.unwrap();

        bus.publish("greetings", "hello").await.unwrap();

        assert_eq!(rx.next().await, Some("hello"));
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_events() {
        let bus = LocalEventBus::<&'static str>::new();
        let mut rx1 = bus.subscribe("greetings").await.unwrap();
        let mut rx2 = bus.subscribe("greetings").await.unwrap();

        bus.publish("greetings", "hello").await.unwrap();

        assert_eq!(rx1.next().await, Some("hello"));
        assert_eq!(rx2.next().await, Some("hello"));
    }

    #[tokio::test]
    async fn different_subjects_are_isolated() {
        let bus = LocalEventBus::<&'static str>::new();
        let mut rx_a = bus.subscribe("a").await.unwrap();
        let mut rx_b = bus.subscribe("b").await.unwrap();

        bus.publish("a", "only a").await.unwrap();

        assert_eq!(rx_a.next().await, Some("only a"));
        tokio::time::timeout(Duration::from_millis(50), rx_b.next())
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn publish_without_subscribers_does_not_error() {
        let bus = LocalEventBus::<&'static str>::new();
        bus.publish("void", "into the void").await.unwrap();
    }

    #[tokio::test]
    async fn remove_cleans_up_channel() {
        let bus = LocalEventBus::<&'static str>::new();
        let mut rx = bus.subscribe("greetings").await.unwrap();
        bus.publish("greetings", "before remove").await.unwrap();
        assert_eq!(rx.next().await, Some("before remove"));
        drop(rx);

        bus.remove("greetings").await;

        let mut rx = bus.subscribe("greetings").await.unwrap();
        tokio::time::timeout(Duration::from_millis(50), rx.next())
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn concurrent_traffic_does_not_deadlock() {
        let bus = Arc::new(LocalEventBus::<usize>::new());
        let mut handles = vec![];

        for i in 0..10 {
            let bus = bus.clone();
            handles.push(tokio::spawn(async move {
                let mut rx = bus.subscribe(&format!("topic-{i}")).await.unwrap();
                bus.publish(&format!("topic-{i}"), i).await.unwrap();
                assert_eq!(rx.next().await, Some(i));
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
    }
}
