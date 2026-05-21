use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt as FuturesStreamExt};
use serde::{Deserialize, Serialize};

use crate::actor::proxy::{EventStream, Room};
use crate::actor::registry::RegistryError;
use crate::auth::User;
use crate::communication::bus::EventBus;
use crate::communication::command::{
    Command, CommandBus, CommandResponse, GameCommand, LobbyCommand, PostGameCommand,
    RoomStateMirror,
};
use crate::communication::event::{Event, GameEvent, LobbyEvent, PostGameEvent, SystemEvent};
use crate::room::RoomId;
use crate::utils::AnyResult;

use super::AppState;

// ---------------------------------------------------------------------------
// Wire protocol envelope
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Authenticate { secret: String, name: String },
    Command { payload: serde_json::Value },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Event { payload: Event },
    Error { reason: String },
    State { payload: RoomStateMirror },
}

// ---------------------------------------------------------------------------
// Command helpers
// ---------------------------------------------------------------------------

/// Inject the authenticated [`User`] into commands from the client.
fn command_with_user(raw: serde_json::Value, user: &User) -> AnyResult<Command> {
    let mut raw = raw;
    if let Some(phase) = raw.as_object_mut().and_then(|m| m.values_mut().next()) {
        if let Some(phase_obj) = phase.as_object_mut() {
            if phase_obj.contains_key("user") {
                // Flat struct variant (e.g. RequestState { user }) — the phase
                // object IS the field map. Inject at this level.
                phase_obj.insert("user".into(), serde_json::to_value(user)?);
            } else if let Some(variant) = phase_obj.values_mut().next() {
                // Newtype variant (e.g. Game(MakeMove { user, move_ })) — the
                // phase wraps a variant tag; descend one more level to find the
                // actual command fields.
                if let Some(args) = variant.as_object_mut() {
                    args.insert("user".into(), serde_json::to_value(user)?);
                }
            }
        }
    }
    Ok(serde_json::from_value(raw)?)
}

async fn send_error(sender: &mut futures::stream::SplitSink<WebSocket, Message>, reason: &str) {
    let msg = ServerMessage::Error {
        reason: reason.to_owned(),
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }
}

// ---------------------------------------------------------------------------
// Auth helpers (generic over EventBus + CommandBus)
// ---------------------------------------------------------------------------

async fn try_join<B, T>(
    room: &Room<B, T>,
    events: &mut EventStream,
    secret: &str,
    name: &str,
    connection_id: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::Lobby(LobbyCommand::Join {
        secret: secret.to_owned(),
        name: name.to_owned(),
        connection_id: connection_id.to_owned(),
    });

    let resp = room.send(cmd).await.ok()?;
    match resp {
        CommandResponse::Accepted => loop {
            match tokio_stream::StreamExt::next(&mut *events).await {
                Some(Event::Lobby(LobbyEvent::PlayerJoined { user })) => return Some(user),
                Some(_) => continue,
                None => return None,
            }
        },
        CommandResponse::Rejected { ref reason } if reason == "room is full" => None,
        _ => None,
    }
}

async fn try_game_reconnect<B, T>(
    room: &Room<B, T>,
    events: &mut EventStream,
    secret: &str,
    connection_id: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::Game(GameCommand::Reconnect {
        secret: secret.to_owned(),
        connection_id: connection_id.to_owned(),
    });

    let resp = room.send(cmd).await.ok()?;
    match resp {
        CommandResponse::Accepted => loop {
            match tokio_stream::StreamExt::next(&mut *events).await {
                Some(Event::Game(GameEvent::PlayerReconnected { user })) => return Some(user),
                Some(_) => continue,
                None => return None,
            }
        },
        _ => None,
    }
}

async fn try_post_game_reconnect<B, T>(
    room: &Room<B, T>,
    events: &mut EventStream,
    secret: &str,
    connection_id: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::PostGame(PostGameCommand::Reconnect {
        secret: secret.to_owned(),
        connection_id: connection_id.to_owned(),
    });

    let resp = room.send(cmd).await.ok()?;
    match resp {
        CommandResponse::Accepted => loop {
            match tokio_stream::StreamExt::next(&mut *events).await {
                Some(Event::PostGame(PostGameEvent::PlayerReconnected { user })) => {
                    return Some(user);
                }
                Some(_) => continue,
                None => return None,
            }
        },
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn handle_socket<P, S, B, T>(
    socket: WebSocket,
    state: Arc<AppState<P, S, B, T>>,
    room_id: RoomId,
) where
    P: crate::actor::lease::Provider + Send + Sync + 'static,
    S: crate::room::storage::Storage + 'static,
    B: EventBus<Item = Event> + Send + Sync + 'static,
    T: CommandBus<Cmd = Command> + 'static,
{
    let (mut sender, mut receiver) = socket.split();

    // Each WebSocket connection gets a unique connection_id.
    // This is stored on the User in the actor and used to:
    //   a) validate that commands originate from the active connection
    //   b) detect when a new connection supersedes this one
    let connection_id = uuid::Uuid::new_v4().to_string();

    // ---- Auth handshake ----
    let msg = match FuturesStreamExt::next(&mut receiver).await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            send_error(&mut sender, "expected 'authenticate' message").await;
            return;
        }
    };

    let client_msg: ClientMessage = match serde_json::from_str(&msg) {
        Ok(m) => m,
        Err(_) => {
            send_error(&mut sender, "invalid 'authenticate' message").await;
            return;
        }
    };

    let (secret, name) = match client_msg {
        ClientMessage::Authenticate { secret, name } => (secret, name),
        _ => {
            send_error(&mut sender, "expected 'authenticate' message").await;
            return;
        }
    };

    let room = match state.registry.get_or_create(room_id.clone()).await {
        Ok(room) => room,
        Err(RegistryError::LeaseHeldElsewhere(_)) => {
            send_error(&mut sender, "room is active on another server").await;
            return;
        }
        Err(e) => {
            send_error(&mut sender, &format!("registry error: {e}")).await;
            return;
        }
    };

    let mut events = match room.subscribe().await {
        Ok(s) => s,
        Err(e) => {
            send_error(&mut sender, &format!("failed to subscribe: {e}")).await;
            return;
        }
    };

    // Try lobby Join first, then fall through to reconnect commands.
    let mut user =
        try_join(&room, &mut events, &secret, &name, &connection_id).await;
    let mut is_reconnect = false;

    // If lobby Join didn't match, try game reconnect.
    if user.is_none() {
        user = try_game_reconnect(&room, &mut events, &secret, &connection_id).await;
        if user.is_some() {
            is_reconnect = true;
        }
    }

    // If game reconnect didn't match, try post-game reconnect.
    if user.is_none() {
        user = try_post_game_reconnect(&room, &mut events, &secret, &connection_id).await;
        if user.is_some() {
            is_reconnect = true;
        }
    }

    // Forward the auth event to the client so it knows the user and can request state sync
    if let Some(ref u) = user {
        let auth_event = if is_reconnect {
            Event::Game(GameEvent::PlayerReconnected { user: u.clone() })
        } else {
            Event::Lobby(LobbyEvent::PlayerJoined { user: u.clone() })
        };
        if let Ok(json) = serde_json::to_string(&ServerMessage::Event { payload: auth_event }) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    let auth = match user {
        Some(u) => u,
        None => {
            send_error(&mut sender, "authentication failed").await;
            return;
        }
    };

    // ---- Main loop ----
    let mut superseded = false;
    loop {
        tokio::select! {
            msg = FuturesStreamExt::next(&mut receiver) => {
                let raw = match msg {
                    Some(Ok(Message::Text(text))) => text,
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => continue,
                };

                let client_msg: ClientMessage = match serde_json::from_str(&raw) {
                    Ok(m) => m,
                    Err(_) => {
                        send_error(&mut sender, "invalid message format").await;
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::Command { payload } => {
                        let cmd = match command_with_user(payload, &auth) {
                            Ok(cmd) => cmd,
                            Err(e) => {
                                send_error(&mut sender, &format!("invalid command: {e}")).await;
                                continue;
                            }
                        };

                        match room.send(cmd).await {
                            Ok(CommandResponse::Accepted) => {}
                            Ok(CommandResponse::State(state)) => {
                                // Send state back to client as a successful response
                                let msg = ServerMessage::State { payload: state };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Ok(CommandResponse::Rejected { reason }) => {
                                send_error(&mut sender, &reason).await;
                            }
                            Err(e) => {
                                send_error(&mut sender, &format!("transport error: {e}")).await;
                                break;
                            }
                        }
                    }
                    _ => {
                        send_error(&mut sender, "unexpected message type").await;
                    }
                }
            }
            event = tokio_stream::StreamExt::next(&mut events) => {
                let Some(event) = event else { break };

                // If our connection has been superseded by a newer one, close gracefully.
                if let Event::System(SystemEvent::ConnectionSuperseded { old_connection_id, .. }) = &event {
                    if *old_connection_id == connection_id {
                        superseded = true;
                        break;
                    }
                }

                if let Ok(json) = serde_json::to_string(&ServerMessage::Event { payload: event }) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            else => break,
        }
    }

    // ---- Cleanup ----
    if !superseded {
        // The connection is truly gone — remove the player from the room.
        let _ = room
            .send(Command::Leave {
                user: auth.clone(),
                connection_id: connection_id.clone(),
            })
            .await;
    }
    // If superseded, the new connection is already handling this user.
    // Sending a Leave here would race with the new connection — skip it.
}
