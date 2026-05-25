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

use tracing::Instrument;

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

/// Create an info-level span for an event.
///
/// When `otel` feature is enabled, adds structured nested attributes
/// (e.g. `event.type`, `event.payload.user.id`) to the underlying OTel span.
/// When `otel` is disabled, records the full JSON payload string for console logging.
fn event_span(room_id: &str, event: &Event) -> tracing::Span {
    let span = match event {
        Event::Lobby(_) => {
            tracing::info_span!("Event:Lobby", room_id, payload = tracing::field::Empty)
        }
        Event::Game(_) => {
            tracing::info_span!("Event:Game", room_id, payload = tracing::field::Empty)
        }
        Event::PostGame(_) => {
            tracing::info_span!("Event:PostGame", room_id, payload = tracing::field::Empty)
        }
        Event::System(_) => {
            tracing::info_span!("Event:System", room_id, payload = tracing::field::Empty)
        }
    };

    #[cfg(not(feature = "otel"))]
    span.record(
        "payload",
        &tracing::field::display(serialized_payload(event)),
    );

    #[cfg(feature = "otel")]
    crate::telemetry::otel::add_event_attributes(&span, event);

    span
}

/// Serialize a value to a JSON string, redacting sensitive fields.
#[cfg(not(feature = "otel"))]
fn serialized_payload<T: serde::Serialize>(value: &T) -> String {
    let mut json = serde_json::to_value(value).unwrap_or_default();
    crate::telemetry::redact_value(&mut json);
    json.to_string()
}

fn phase_label(phase: &Phase) -> &'static str {
    match phase {
        Phase::Lobby { .. } => "lobby",
        Phase::Game { .. } => "game",
        Phase::PostGame { .. } => "post_game",
    }
}

/// Create an info-level span whose name is the human-readable command label.
///
/// When `otel` feature is enabled, adds structured nested attributes
/// (e.g. `command.type`, `command.payload.user.id`) to the underlying OTel span.
/// When `otel` is disabled, records the full JSON payload string for console logging.
fn cmd_span(
    cmd: &Command,
    phase: &str,
    room_id: &str,
    user_id: Option<&str>,
    connection_id: Option<&str>,
) -> tracing::Span {
    let user_id = user_id.unwrap_or("");
    let connection_id = connection_id.unwrap_or("");
    let span = match cmd {
        Command::Lobby(LobbyCommand::Join { .. }) => {
            tracing::info_span!("Lobby:Join", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Lobby(LobbyCommand::Leave { .. }) => {
            tracing::info_span!("Lobby:Leave", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Lobby(LobbyCommand::ChangeName { .. }) => {
            tracing::info_span!("Lobby:ChangeName", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Lobby(LobbyCommand::ChangeReady { .. }) => {
            tracing::info_span!("Lobby:ChangeReady", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Game(GameCommand::Reconnect { .. }) => {
            tracing::info_span!("Game:Reconnect", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Game(GameCommand::Leave { .. }) => {
            tracing::info_span!("Game:Leave", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Game(GameCommand::MakeMove { .. }) => {
            tracing::info_span!("Game:MakeMove", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Game(GameCommand::Resign { .. }) => {
            tracing::info_span!("Game:Resign", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::PostGame(PostGameCommand::Reconnect { .. }) => {
            tracing::info_span!("PostGame:Reconnect", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::PostGame(PostGameCommand::Leave { .. }) => {
            tracing::info_span!("PostGame:Leave", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::PostGame(PostGameCommand::OfferRematch { .. }) => {
            tracing::info_span!("PostGame:OfferRematch", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::PostGame(PostGameCommand::AcceptRematch { .. }) => {
            tracing::info_span!("PostGame:AcceptRematch", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::PostGame(PostGameCommand::DeclineRematch { .. }) => {
            tracing::info_span!("PostGame:DeclineRematch", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::RequestState { .. } => {
            tracing::info_span!("RequestState", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
        Command::Leave { .. } => {
            tracing::info_span!("Leave", %phase, room_id, %user_id, %connection_id, payload = tracing::field::Empty)
        }
    };

    #[cfg(not(feature = "otel"))]
    span.record("payload", &tracing::field::display(serialized_payload(cmd)));

    #[cfg(feature = "otel")]
    crate::telemetry::otel::add_command_attributes(&span, cmd);

    span
}

/// Log a shutdown message including the OTEL trace ID when available.
#[cfg(feature = "otel")]
fn log_actor_shutdown(room_id: &str) {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let cx = tracing::Span::current().context();
    let trace_id = cx.span().span_context().trace_id();
    tracing::info!(room_id, otel_trace_id = %trace_id, "actor shutting down");
}

/// Log a shutdown message (otel-disabled variant).
#[cfg(not(feature = "otel"))]
fn log_actor_shutdown(room_id: &str) {
    tracing::info!(room_id, "actor shutting down");
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
    let actor_span =
        tracing::info_span!(parent: None, "actor", actor_id = %actor_id, room_id = %room_id);

    let mut state = match storage.get_by_id(&room_id).await {
        Ok(Some(room)) => room,
        Ok(None) => RoomState::new(room_id.clone()),
        Err(err) => {
            tracing::error!("failed to load room {room_id}: {err}");
            return;
        }
    };

    let _ = async {
        tracing::info!(actor_id = %actor_id, room_id = %room_id, "actor started");

        let mut was_empty = state.players.is_empty();
        if was_empty {
            tracing::info!(room_id = %room_id, "starting empty room shutdown timer");
        }

        loop {
            let empty = state.players.is_empty();
            let empty_timeout = state.empty_shutdown_secs;

            if empty && !was_empty {
                tracing::info!(room_id = %room_id, "starting empty room shutdown timer");
            } else if !empty && was_empty {
                tracing::info!(room_id = %room_id, "cancelling empty room shutdown timer");
            }
            was_empty = empty;

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
                    let result = cmd_span(&incoming.command, phase, &room_id, user_id, conn_id)
                        .in_scope(|| handler::handle_command(incoming.command, &mut state));

                    incoming.response_channel.respond(result.response).await;

                    for event in result.events {
                        let span = event_span(&room_id, &event);
                        async {
                            if let Err(err) = publisher.publish(event).await {
                                tracing::warn!("failed to publish event for room {room_id}: {err}");
                            }
                        }
                        .instrument(span)
                        .await;
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

        log_actor_shutdown(&room_id);

        publisher.remove().await;
    }
    .instrument(actor_span)
    .await;
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
