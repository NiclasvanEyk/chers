use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::actor::handler;
use crate::communication::bus::EventBus;
use crate::communication::command::{Command, ReceivedStream};
use crate::communication::event::Event;
use crate::room::Room as RoomState;
use crate::room::storage::Storage;

use super::lease;
use super::proxy::PublisherScope;

/// Main event loop for a single room actor.
pub async fn run_actor<S, B>(
    publisher: PublisherScope<B>,
    storage: Arc<S>,
    mut cmd_stream: ReceivedStream<Command>,
    mut guard: lease::Lease,
) where
    S: Storage,
    B: EventBus<Item = Event>,
{
    let room_id = publisher.room_id().clone();
    let mut state = match storage.get_by_id(&room_id).await {
        Ok(Some(room)) => room,
        Ok(None) => RoomState::new(room_id.clone()),
        Err(err) => {
            tracing::error!("failed to load room {room_id}: {err}");
            return;
        }
    };

    loop {
        tokio::select! {
            incoming = async { cmd_stream.next().await } => {
                let Some(incoming) = incoming else { break };

                let result = handler::handle_command(incoming.command, &mut state);

                incoming.response_channel.respond(result.response).await;

                for event in result.events {
                    if let Err(err) = publisher.publish(event).await {
                        tracing::warn!("failed to publish event for room {room_id}: {err}");
                    }
                }

                if let Err(err) = storage.persist(&state).await {
                    tracing::error!("failed to persist room {room_id}: {err}");
                }
            }
            _ = guard.expired() => {
                tracing::warn!("lease expired for room {room_id}");
                break;
            }
        }
    }

    if let Err(err) = storage.persist(&state).await {
        tracing::error!("failed final persist for room {room_id}: {err}");
    }
    publisher.remove().await;
}
