use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::actor::handler;
use crate::communication::bus::EventBus;
use crate::communication::command::{
    Command, GameCommand, LobbyCommand, PostGameCommand, ReceivedStream,
};
use crate::communication::event::Event;
use crate::room::Room as RoomState;
use crate::room::storage::Storage;

use super::lease;
use super::proxy::PublisherScope;

/// Extract user_id and connection_id from a command for logging purposes.
fn cmd_identity(cmd: &Command) -> (Option<&str>, Option<&str>) {
    match cmd {
        Command::Lobby(LobbyCommand::Join { connection_id, .. }) => (None, Some(connection_id.as_str())),
        Command::Lobby(LobbyCommand::Leave { user, connection_id }) => {
            (Some(user.id.as_str()), Some(connection_id.as_str()))
        }
        Command::Game(GameCommand::Reconnect { connection_id, .. }) => {
            (None, Some(connection_id.as_str()))
        }
        Command::Game(GameCommand::Leave { user, connection_id }) => {
            (Some(user.id.as_str()), Some(connection_id.as_str()))
        }
        Command::Game(GameCommand::MakeMove { user, .. }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::Game(GameCommand::Resign { user }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::Reconnect { connection_id, .. }) => {
            (None, Some(connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::Leave { user, connection_id }) => {
            (Some(user.id.as_str()), Some(connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::OfferRematch { user }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::AcceptRematch { user }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::DeclineRematch { user }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::Lobby(LobbyCommand::ChangeName { user, .. }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::Lobby(LobbyCommand::ChangeReady { user, .. }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::RequestState { user } => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::Leave { user, connection_id } => {
            (Some(user.id.as_str()), Some(connection_id.as_str()))
        }
    }
}

/// Log a debug summary for each event (always includes room_id).
fn log_event(room_id: &str, event: &Event) {
    match event {
        Event::Lobby(e) => tracing::debug!(room_id, ?e, "lobby event"),
        Event::Game(e) => tracing::debug!(room_id, ?e, "game event"),
        Event::PostGame(e) => tracing::debug!(room_id, ?e, "post_game event"),
        Event::System(e) => tracing::debug!(room_id, ?e, "system event"),
    }
}

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

                let (user_id, conn_id) = cmd_identity(&incoming.command);
                tracing::debug!(
                    room_id = %room_id,
                    user_id,
                    connection_id = conn_id,
                    ?incoming.command,
                    "command received",
                );

                let result = handler::handle_command(incoming.command, &mut state);

                incoming.response_channel.respond(result.response).await;

                for event in result.events {
                    log_event(&room_id, &event);

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
