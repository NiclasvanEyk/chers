/// Local in-memory event bus implementation.
pub mod local;
pub use local::LocalEventBus;

/// Redis-based event bus suitable for distributed deployments.
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "redis")]
pub use redis::RedisEventBus;

/// NATS-based event bus suitable for distributed deployments.
#[cfg(feature = "nats")]
pub mod nats;
#[cfg(feature = "nats")]
pub use nats::NatsEventBus;

use std::pin::Pin;
use tokio_stream::Stream;

use crate::utils::AnyResult;

/// An enum dispatching to the available [`EventBus`] implementations.
#[enum_dispatch::enum_dispatch(EventBus)]
pub enum AnyEventBus<T> {
    Local(LocalEventBus<T>),
    #[cfg(feature = "redis")]
    Redis(RedisEventBus<T>),
    #[cfg(feature = "nats")]
    Nats(NatsEventBus<T>),
}

// Manual impl because `enum_dispatch` doesn't support `impl Future` in trait methods.
impl<T: Clone + Send + Sync + 'static> Clone for AnyEventBus<T> {
    fn clone(&self) -> Self {
        match self {
            AnyEventBus::Local(bus) => AnyEventBus::Local(bus.clone()),
            #[cfg(feature = "redis")]
            AnyEventBus::Redis(bus) => AnyEventBus::Redis(bus.clone()),
            #[cfg(feature = "nats")]
            AnyEventBus::Nats(bus) => AnyEventBus::Nats(bus.clone()),
        }
    }
}

#[cfg(any(feature = "redis", feature = "nats"))]
impl<T: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned> EventBus
    for AnyEventBus<T>
{
    type Item = T;

    fn publish(&self, subject: &str, item: T) -> impl Future<Output = AnyResult<()>> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.publish(subject, item).await,
                #[cfg(feature = "redis")]
                AnyEventBus::Redis(bus) => bus.publish(subject, item).await,
                #[cfg(feature = "nats")]
                AnyEventBus::Nats(bus) => bus.publish(subject, item).await,
            }
        }
    }

    fn subscribe(
        &self,
        subject: &str,
    ) -> impl Future<Output = AnyResult<Pin<Box<dyn Stream<Item = T> + Send>>>> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.subscribe(subject).await,
                #[cfg(feature = "redis")]
                AnyEventBus::Redis(bus) => bus.subscribe(subject).await,
                #[cfg(feature = "nats")]
                AnyEventBus::Nats(bus) => bus.subscribe(subject).await,
            }
        }
    }

    fn remove(&self, subject: &str) -> impl Future<Output = ()> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.remove(subject).await,
                #[cfg(feature = "redis")]
                AnyEventBus::Redis(bus) => bus.remove(subject).await,
                #[cfg(feature = "nats")]
                AnyEventBus::Nats(bus) => bus.remove(subject).await,
            }
        }
    }
}

#[cfg(not(any(feature = "redis", feature = "nats")))]
impl<T: Clone + Send + Sync + 'static> EventBus for AnyEventBus<T> {
    type Item = T;

    fn publish(&self, subject: &str, item: T) -> impl Future<Output = AnyResult<()>> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.publish(subject, item).await,
            }
        }
    }

    fn subscribe(
        &self,
        subject: &str,
    ) -> impl Future<Output = AnyResult<Pin<Box<dyn Stream<Item = T> + Send>>>> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.subscribe(subject).await,
            }
        }
    }

    fn remove(&self, subject: &str) -> impl Future<Output = ()> + Send {
        async move {
            match self {
                AnyEventBus::Local(bus) => bus.remove(subject).await,
            }
        }
    }
}

/// A subject-based pub/sub bus for typed messages.
///
/// Messages are addressed by subject string (e.g. `"evt.{room_id}"`).
/// Multiple subscribers can listen on the same subject; each published
/// message is delivered to all current subscribers.
pub trait EventBus: Clone + Send + Sync + 'static {
    type Item: Clone + Send + 'static;

    fn publish(
        &self,
        subject: &str,
        item: Self::Item,
    ) -> impl Future<Output = AnyResult<()>> + Send;

    fn subscribe(
        &self,
        subject: &str,
    ) -> impl Future<Output = AnyResult<Pin<Box<dyn Stream<Item = Self::Item> + Send>>>> + Send;

    /// Release any bus-internal resources for this subject.
    fn remove(&self, subject: &str) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventBusDriver {
    Local,
    #[cfg(feature = "redis")]
    Redis,
    #[cfg(feature = "nats")]
    Nats,
}
