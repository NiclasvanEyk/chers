use std::sync::Arc;
use std::time::Duration;

use tokio_stream::StreamExt;

use crate::actor::handler;
use crate::communication::bus::EventBus;
use crate::communication::command::{
    Command, GameCommand, LobbyCommand, PostGameCommand, ReceivedStream,
};
use crate::communication::event::Event;
use crate::room::storage::Storage;
use crate::room::{Phase, Room as RoomState};

use super::lease;
use super::proxy::PublisherScope;

/// Extract user_id and connection_id from a command for logging purposes.
fn cmd_identity(cmd: &Command) -> (Option<&str>, Option<&str>) {
    match cmd {
        Command::Lobby(LobbyCommand::Join { connection_id, .. }) => {
            (None, Some(connection_id.as_str()))
        }
        Command::Lobby(LobbyCommand::Leave {
            user,
            connection_id,
        }) => (Some(user.id.as_str()), Some(connection_id.as_str())),
        Command::Game(GameCommand::Reconnect { connection_id, .. }) => {
            (None, Some(connection_id.as_str()))
        }
        Command::Game(GameCommand::Leave {
            user,
            connection_id,
        }) => (Some(user.id.as_str()), Some(connection_id.as_str())),
        Command::Game(GameCommand::MakeMove { user, .. }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::Game(GameCommand::Resign { user }) => {
            (Some(user.id.as_str()), Some(user.connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::Reconnect { connection_id, .. }) => {
            (None, Some(connection_id.as_str()))
        }
        Command::PostGame(PostGameCommand::Leave {
            user,
            connection_id,
        }) => (Some(user.id.as_str()), Some(connection_id.as_str())),
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
        Command::Leave {
            user,
            connection_id,
        } => (Some(user.id.as_str()), Some(connection_id.as_str())),
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

fn phase_label(phase: &Phase) -> &'static str {
    match phase {
        Phase::Lobby { .. } => "lobby",
        Phase::Game { .. } => "game",
        Phase::PostGame { .. } => "post_game",
    }
}

/// Derive a human-readable label for a command, suitable for span attributes.
fn cmd_label(cmd: &Command) -> &'static str {
    match cmd {
        Command::Lobby(LobbyCommand::Join { .. }) => "Lobby:Join",
        Command::Lobby(LobbyCommand::Leave { .. }) => "Lobby:Leave",
        Command::Lobby(LobbyCommand::ChangeName { .. }) => "Lobby:ChangeName",
        Command::Lobby(LobbyCommand::ChangeReady { .. }) => "Lobby:ChangeReady",
        Command::Game(GameCommand::Reconnect { .. }) => "Game:Reconnect",
        Command::Game(GameCommand::Leave { .. }) => "Game:Leave",
        Command::Game(GameCommand::MakeMove { .. }) => "Game:MakeMove",
        Command::Game(GameCommand::Resign { .. }) => "Game:Resign",
        Command::PostGame(PostGameCommand::Reconnect { .. }) => "PostGame:Reconnect",
        Command::PostGame(PostGameCommand::Leave { .. }) => "PostGame:Leave",
        Command::PostGame(PostGameCommand::OfferRematch { .. }) => "PostGame:OfferRematch",
        Command::PostGame(PostGameCommand::AcceptRematch { .. }) => "PostGame:AcceptRematch",
        Command::PostGame(PostGameCommand::DeclineRematch { .. }) => "PostGame:DeclineRematch",
        Command::RequestState { .. } => "RequestState",
        Command::Leave { .. } => "Leave",
    }
}

/// Main event loop for a single room actor.
pub async fn run_actor<S, B>(
    actor_id: String,
    publisher: PublisherScope<B>,
    storage: Arc<S>,
    mut cmd_stream: ReceivedStream<Command>,
    mut guard: lease::Lease,
) where
    S: Storage,
    B: EventBus<Item = Event>,
{
    let room_id = publisher.room_id().clone();
    let actor_span = tracing::info_span!("actor", actor_id = %actor_id, room_id = %room_id);
    let _actor_guard = actor_span.enter();

    let mut state = match storage.get_by_id(&room_id).await {
        Ok(Some(room)) => room,
        Ok(None) => RoomState::new(room_id.clone()),
        Err(err) => {
            tracing::error!("failed to load room {room_id}: {err}");
            return;
        }
    };

    loop {
        let empty = state.players.is_empty();
        let empty_timeout = state.empty_shutdown_secs;

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

                let phase = phase_label(&state.phase);
                let result = tracing::debug_span!("command", cmd_type = %cmd_label(&incoming.command), %phase, room_id = %room_id)
                    .in_scope(|| handler::handle_command(incoming.command, &mut state));

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
            _ = async {
                match empty_timeout {
                    Some(secs) => tokio::time::sleep(Duration::from_secs(secs)).await,
                    None => std::future::pending().await,
                }
            }, if empty => {
                tracing::info!(room_id = %room_id, "shutting down empty room");
                break;
            }
        }
    }

    if let Err(err) = storage.persist(&state).await {
        tracing::error!("failed final persist for room {room_id}: {err}");
    }
    publisher.remove().await;
}

#[cfg(test)]
mod tests {
    use std::future;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::mpsc;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    use crate::actor::lease::Lease;
    use crate::actor::proxy::PublisherScope;
    use crate::communication::bus::local::LocalEventBus;
    use crate::communication::command::{Command, ReceivedCommand};
    use crate::communication::event::Event;
    use crate::room::Room;
    use crate::room::storage::{InMemoryStorage, Storage};

    use super::run_actor;

    #[tokio::test]
    async fn actor_shuts_down_after_empty_timeout() {
        let room_id = "test-empty-shutdown".to_string();
        let mut room = Room::new(room_id.clone());
        room.empty_shutdown_secs = Some(1);

        let storage = Arc::new(InMemoryStorage::new());
        storage.persist(&room).await.unwrap();

        let (_tx, rx) = mpsc::unbounded_channel::<ReceivedCommand<Command>>();
        let cmd_stream = Box::pin(UnboundedReceiverStream::new(rx));

        let event_bus = Arc::new(LocalEventBus::<Event>::new());
        let publisher = PublisherScope::new(event_bus, room_id.clone());
        let guard = Lease::new(None, Box::pin(future::pending()));

        tokio::time::timeout(
            Duration::from_secs(5),
            run_actor("test-actor".into(), publisher, storage, cmd_stream, guard),
        )
        .await
        .expect("actor should have shut down within empty_shutdown_secs");
    }
}
