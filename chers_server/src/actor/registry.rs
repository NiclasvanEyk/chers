use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::communication::bus::EventBus;
use crate::communication::command::{Command, CommandBus};
use crate::communication::event::Event;
use crate::room::RoomId;
use crate::room::storage::Storage;

use super::lease::{self, LeaseError};
use super::proxy::{PublisherScope, Room, RoomEventSubscriber};

/// Manages the lifecycle of room actors.
///
/// Each room has at most one actor across the cluster, guaranteed by
/// the lease provider. The actor owns room state, processes commands,
/// and publishes events to the bus.
pub struct RoomRegistry<L, S, B, C> {
    lease: Arc<L>,
    storage: Arc<S>,
    event_bus: Arc<B>,
    command_bus: Arc<C>,
    active_rooms: Arc<RwLock<HashSet<RoomId>>>,
}

/// Errors that can occur when trying to spawn an actor for a room.
#[derive(Debug)]
pub enum RegistryError {
    LeaseHeldElsewhere(RoomId),
    Lease(LeaseError),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeaseHeldElsewhere(id) => write!(f, "lease held elsewhere for room {id}"),
            Self::Lease(err) => write!(f, "lease error: {err}"),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<LeaseError> for RegistryError {
    fn from(err: LeaseError) -> Self {
        match err {
            LeaseError::HeldByOther => RegistryError::LeaseHeldElsewhere(String::new()),
            other => RegistryError::Lease(other),
        }
    }
}

impl<
    L: lease::Provider,
    S: Storage + 'static,
    B: EventBus<Item = Event> + 'static,
    C: CommandBus<Cmd = Command> + 'static,
> RoomRegistry<L, S, B, C>
{
    pub fn new(lease: Arc<L>, storage: Arc<S>, event_bus: Arc<B>, command_bus: Arc<C>) -> Self {
        Self {
            lease,
            storage,
            event_bus,
            command_bus,
            active_rooms: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Number of currently active room actors on this server.
    pub fn active_count(&self) -> usize {
        self.active_rooms.read().len()
    }

    /// Get or create a room, spawning an actor if needed.
    ///
    /// Returns `LeaseHeldElsewhere` if another server holds the lease.
    pub async fn get_or_create(&self, room_id: RoomId) -> Result<Room<B, C>, RegistryError> {
        if !self.active_rooms.read().contains(&room_id) {
            let guard = self
                .lease
                .try_acquire(&room_id)
                .await
                .map_err(|e| match e {
                    LeaseError::HeldByOther => RegistryError::LeaseHeldElsewhere(room_id.clone()),
                    err => RegistryError::Lease(err),
                })?;

            let cmd_stream = self
                .command_bus
                .subscribe(&room_id)
                .await
                .expect("local command bus subscribe should not fail");

            let publisher = PublisherScope::new(Arc::clone(&self.event_bus), room_id.clone());
            let storage = Arc::clone(&self.storage);
            let active_rooms = Arc::clone(&self.active_rooms);
            let rid = room_id.clone();
            let actor_id = uuid::Uuid::now_v7().to_string();

            self.active_rooms.write().insert(room_id.clone());

            tokio::spawn(async move {
                super::loop_::run_actor(actor_id, publisher, storage, cmd_stream, guard).await;
                active_rooms.write().remove(&rid);
            });
        }

        Ok(Room {
            room_id,
            subscriber: RoomEventSubscriber::new(Arc::clone(&self.event_bus)),
            command_bus: C::clone(&self.command_bus),
        })
    }
}
