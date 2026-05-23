use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use crate::room::{Room, RoomId};
use crate::utils::AnyResult;

use super::Storage;

const KEY_PREFIX: &str = "chers:room:";

/// Stores serialized room data using the [Redis Key/Value Store](https://redis.io/nosql/key-value-databases/):
///
/// | Key         | Value |
/// |-------------|-------|
/// | `chers:room:123e4567-e89b-12d3-a456-426614174000` | `{"id": "123e4567-e89b-12d3-a456-426614174000", "phase": "..."} |
/// | `chers:room:467ce712-2077-420e-b599-a65ac6723f4b` | `{"id": "467ce712-2077-420e-b599-a65ac6723f4b", "phase": "..."} |
pub struct RedisStorage {
    conn: ConnectionManager,
}

impl RedisStorage {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

impl Storage for RedisStorage {
    async fn get_by_id(&self, id: &RoomId) -> AnyResult<Option<Room>> {
        let mut conn = self.conn.clone();
        let key = format!("{KEY_PREFIX}{id}");
        let raw: Option<Vec<u8>> = conn.get(&key).await?;
        match raw {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    async fn persist(&self, room: &Room) -> AnyResult<()> {
        let mut conn = self.conn.clone();
        let key = format!("{KEY_PREFIX}{}", room.id);
        let raw = serde_json::to_vec(room)?;
        let _: () = conn.set(&key, raw).await?;
        Ok(())
    }

    async fn delete(&self, id: &RoomId) -> AnyResult<()> {
        let mut conn = self.conn.clone();
        let key = format!("{KEY_PREFIX}{id}");
        let _: () = conn.del(&key).await?;
        Ok(())
    }
}
