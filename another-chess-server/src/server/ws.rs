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
};
use crate::communication::event::{Event, GameEvent, LobbyEvent, PostGameEvent};
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
}

// ---------------------------------------------------------------------------
// Command helpers
// ---------------------------------------------------------------------------

/// Inject the authenticated [`User`] into commands from the client.
fn command_with_user(raw: serde_json::Value, user: &User) -> AnyResult<Command> {
    let mut raw = raw;
    if let Some(phase) = raw.as_object_mut().and_then(|m| m.values_mut().next()) {
        if let Some(variant) = phase.as_object_mut().and_then(|m| m.values_mut().next()) {
            if let Some(args) = variant.as_object_mut() {
                args.insert("user".into(), serde_json::to_value(user)?);
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
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    secret: &str,
    name: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::Lobby(LobbyCommand::Join {
        secret: secret.to_owned(),
        name: name.to_owned(),
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
        CommandResponse::Rejected { ref reason }
            if reason == "room is full" || reason == "already joined" =>
        {
            send_error(sender, reason).await;
            None
        }
        _ => None,
    }
}

async fn try_game_reconnect<B, T>(
    room: &Room<B, T>,
    events: &mut EventStream,
    _sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    secret: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::Game(GameCommand::Reconnect {
        secret: secret.to_owned(),
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
    _sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    secret: &str,
) -> Option<User>
where
    B: EventBus<Item = Event>,
    T: CommandBus<Cmd = Command>,
{
    let cmd = Command::PostGame(PostGameCommand::Reconnect {
        secret: secret.to_owned(),
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

    // ---- Auth handshake ----
    let auth = {
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
        let mut user = try_join(&room, &mut events, &mut sender, &secret, &name).await;

        // If lobby Join didn't match, try game reconnect.
        if user.is_none() {
            user = try_game_reconnect(&room, &mut events, &mut sender, &secret).await;
        }

        // If game reconnect didn't match, try post-game reconnect.
        if user.is_none() {
            user = try_post_game_reconnect(&room, &mut events, &mut sender, &secret).await;
        }

        match user {
            Some(u) => u,
            None => {
                send_error(&mut sender, "authentication failed").await;
                return;
            }
        }
    };

    // ---- Re-subscribe (the auth step consumed some events) ----
    let room = match state.registry.get_or_create(room_id.clone()).await {
        Ok(room) => room,
        Err(_) => return,
    };
    let mut events = match room.subscribe().await {
        Ok(s) => s,
        Err(_) => return,
    };

    // ---- Main loop ----
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

                if let Ok(json) = serde_json::to_string(&ServerMessage::Event { payload: event }) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            else => break,
        }
    }
}
