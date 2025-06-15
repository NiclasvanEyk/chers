use crate::room::{Room, RoomId};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::new_room;

#[derive(Default)]
pub struct InMemoryRoomRepository {
    last_id: u32,
    matches: HashMap<RoomId, Arc<RwLock<Room>>>,
}

impl InMemoryRoomRepository {
    fn generate_id(&mut self) -> RoomId {
        let id = self.last_id.wrapping_add(1);
        self.last_id = id;
        id
    }

    pub fn start(&mut self) -> Arc<RwLock<Room>> {
        let id = self.generate_id();
        let m = Arc::new(RwLock::new(new_room()));
        self.matches.insert(id, m.clone());
        m
    }

    pub fn find(&self, id: RoomId) -> Option<Arc<RwLock<Room>>> {
        match self.matches.get(&id) {
            Some(the_match) => Some(the_match.clone()),
            None => todo!(),
        }
    }
}
