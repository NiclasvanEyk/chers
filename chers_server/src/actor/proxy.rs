use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::Stream;

use crate::communication::bus::EventBus;
use crate::communication::command::{Command, CommandBus, CommandBusError, CommandResponse};
use crate::communication::event::Event;
use crate::room::RoomId;
use crate::utils::AnyResult;

/// A boxed, sendable stream of [Event] items.
pub type EventStream = Pin<Box<dyn Stream<Item = Event> + Send>>;

// ---------------------------------------------------------------------------
// RoomEventSubscriber — read-only event access
// ---------------------------------------------------------------------------

/// Read-only event subscriber for rooms.
///
/// Can subscribe to any room's event stream but cannot publish events.
/// WS handlers receive one of these to consume events.
pub struct RoomEventSubscriber<B> {
    bus: Arc<B>,
}

impl<B> Clone for RoomEventSubscriber<B> {
    fn clone(&self) -> Self {
        Self {
            bus: Arc::clone(&self.bus),
        }
    }
}

impl<B> RoomEventSubscriber<B> {
    pub fn new(bus: Arc<B>) -> Self {
        Self { bus }
    }
}

impl<B: EventBus<Item = Event>> RoomEventSubscriber<B> {
    /// Subscribe to events for a specific room.
    pub async fn subscribe(&self, room_id: &RoomId) -> AnyResult<EventStream> {
        self.bus.subscribe(&format!("evt.{room_id}")).await
    }
}

// ---------------------------------------------------------------------------
// PublisherScope — write-only event publishing for a specific room
// ---------------------------------------------------------------------------

/// Write-only publisher scoped to a single room.
///
/// Only the room's actor holds this. It can publish events and clean up
/// the bus subject, but it cannot subscribe.
pub struct PublisherScope<B> {
    bus: Arc<B>,
    room_id: RoomId,
}

impl<B> PublisherScope<B> {
    pub fn new(bus: Arc<B>, room_id: RoomId) -> Self {
        Self { bus, room_id }
    }

    /// The room this publisher is scoped to.
    pub fn room_id(&self) -> &RoomId {
        &self.room_id
    }
}

impl<B: EventBus<Item = Event>> PublisherScope<B> {
    /// Publish an event to the room's event subject.
    pub async fn publish(&self, event: Event) -> AnyResult<()> {
        self.bus
            .publish(&format!("evt.{}", self.room_id), event)
            .await
    }

    /// Remove the room's event subject, releasing bus resources.
    pub async fn remove(&self) {
        self.bus.remove(&format!("evt.{}", self.room_id)).await
    }
}

// ---------------------------------------------------------------------------
// Room — public handle for interacting with a room
// ---------------------------------------------------------------------------

/// A handle to a room that lets you send commands and subscribe to events.
///
/// This is a thin, cloneable proxy — the actual actor runs in a background
/// task managed by [`RoomRegistry`](super::registry::RoomRegistry).
pub struct Room<B, T> {
    pub(crate) room_id: RoomId,
    pub(crate) subscriber: RoomEventSubscriber<B>,
    pub(crate) command_bus: T,
}

impl<B: EventBus<Item = Event>, T: CommandBus<Cmd = Command>> Room<B, T> {
    pub fn room_id(&self) -> &RoomId {
        &self.room_id
    }

    /// Subscribe to events broadcast by this room's actor.
    pub async fn subscribe(&self) -> AnyResult<EventStream> {
        self.subscriber.subscribe(&self.room_id).await
    }

    /// Send a command to the room's actor and wait for a response.
    pub async fn send(&self, cmd: Command) -> Result<CommandResponse, CommandBusError> {
        self.command_bus.send(&self.room_id, cmd).await
    }
}

impl<B, T: Clone> Clone for Room<B, T> {
    fn clone(&self) -> Self {
        Self {
            room_id: self.room_id.clone(),
            subscriber: self.subscriber.clone(),
            command_bus: self.command_bus.clone(),
        }
    }
}
