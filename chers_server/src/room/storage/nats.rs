use async_nats::jetstream::kv::Store;
use bytes::Bytes;

use crate::room::{Room, RoomId};
use crate::utils::AnyResult;

use super::Storage;

const KEY_PREFIX: &str = "chers.room.";

/// Stores serialized room data using the [NATS Key/Value Store](https://docs.nats.io/nats-concepts/jetstream/key-value-store):
///
/// | Key         | Value |
/// |-------------|-------|
/// | `chers.room.123e4567-e89b-12d3-a456-426614174000` | `{"id": "123e4567-e89b-12d3-a456-426614174000", "phase": "..."} |
/// | `chers.room.467ce712-2077-420e-b599-a65ac6723f4b` | `{"id": "467ce712-2077-420e-b599-a65ac6723f4b", "phase": "..."} |
pub struct NatsStorage {
    kv: Store,
}

impl NatsStorage {
    pub fn new(kv: Store) -> Self {
        Self { kv }
    }
}

impl Storage for NatsStorage {
    async fn get_by_id(&self, id: &RoomId) -> AnyResult<Option<Room>> {
        let key = format!("{KEY_PREFIX}{id}");
        match self.kv.get(&key).await? {
            Some(raw) => Ok(Some(serde_json::from_slice(&raw)?)),
            None => Ok(None),
        }
    }

    async fn persist(&self, room: &Room) -> AnyResult<()> {
        let key = format!("{KEY_PREFIX}{}", room.id);
        let raw = serde_json::to_vec(room)?;
        self.kv.put(&key, Bytes::from(raw)).await?;
        Ok(())
    }

    async fn delete(&self, id: &RoomId) -> AnyResult<()> {
        let key = format!("{KEY_PREFIX}{id}");
        self.kv.delete(&key).await?;
        Ok(())
    }
}
