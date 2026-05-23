use crate::room::{Room, RoomId};
use crate::utils::AnyResult;

pub mod in_memory;

#[cfg(feature = "redis")]
pub mod redis;

pub use in_memory::InMemoryStorage;
#[cfg(feature = "redis")]
pub use redis::RedisStorage;

#[cfg(feature = "nats")]
pub mod nats;
#[cfg(feature = "nats")]
pub use nats::NatsStorage;

#[enum_dispatch::enum_dispatch(Storage)]
pub enum AnyStorage {
    InMemory(InMemoryStorage),
    #[cfg(feature = "redis")]
    Redis(RedisStorage),
    #[cfg(feature = "nats")]
    Nats(NatsStorage),
}

// Manual impl because `enum_dispatch` doesn't support `impl Future` in trait methods.
impl Storage for AnyStorage {
    async fn get_by_id(&self, id: &RoomId) -> AnyResult<Option<Room>> {
        match self {
            AnyStorage::InMemory(storage) => storage.get_by_id(id).await,
            #[cfg(feature = "redis")]
            AnyStorage::Redis(storage) => storage.get_by_id(id).await,
            #[cfg(feature = "nats")]
            AnyStorage::Nats(storage) => storage.get_by_id(id).await,
        }
    }

    async fn persist(&self, room: &Room) -> AnyResult<()> {
        match self {
            AnyStorage::InMemory(storage) => storage.persist(room).await,
            #[cfg(feature = "redis")]
            AnyStorage::Redis(storage) => storage.persist(room).await,
            #[cfg(feature = "nats")]
            AnyStorage::Nats(storage) => storage.persist(room).await,
        }
    }

    async fn delete(&self, id: &RoomId) -> AnyResult<()> {
        match self {
            AnyStorage::InMemory(storage) => storage.delete(id).await,
            #[cfg(feature = "redis")]
            AnyStorage::Redis(storage) => storage.delete(id).await,
            #[cfg(feature = "nats")]
            AnyStorage::Nats(storage) => storage.delete(id).await,
        }
    }
}

pub trait Storage: Send + Sync {
    /// Retrieves data for a room by its id.
    fn get_by_id(&self, id: &RoomId) -> impl Future<Output = AnyResult<Option<Room>>> + Send;

    /// Stores the data for this room.
    fn persist(&self, room: &Room) -> impl Future<Output = AnyResult<()>> + Send;

    /// Clears the data for a room.
    fn delete(&self, id: &RoomId) -> impl Future<Output = AnyResult<()>> + Send;
}
