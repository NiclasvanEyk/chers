use std::collections::HashMap;
use std::sync::RwLock;

use crate::{
    room::{Room, RoomId},
    utils::AnyResult,
};

/// Stores rooms using an in-memory [HashMap] behind a [RwLock].
pub struct InMemoryStorage {
    rooms: RwLock<HashMap<RoomId, Room>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Storage for InMemoryStorage {
    async fn get_by_id(&self, id: &RoomId) -> AnyResult<Option<Room>> {
        let rooms = self.rooms.read().unwrap_or_else(|e| e.into_inner());
        Ok(rooms.get(id).cloned())
    }

    async fn persist(&self, room: &Room) -> AnyResult<()> {
        let mut rooms = self.rooms.write().unwrap_or_else(|e| e.into_inner());
        rooms.insert(room.id.clone(), room.clone());
        Ok(())
    }

    async fn delete(&self, id: &RoomId) -> AnyResult<()> {
        let mut rooms = self.rooms.write().unwrap_or_else(|e| e.into_inner());
        rooms.remove(id);
        Ok(())
    }
}
