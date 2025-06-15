use std::{
    error::Error,
    sync::{Arc, Mutex, RwLock},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    communication::announce,
    room::{states::lobby::server::Lobby, Room, State as RoomState},
    AppState,
};

use chers_server_api::LobbyAnnouncement;

#[derive(Deserialize)]
pub struct PlayPathParams {
    id: u32,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(path): Path<PlayPathParams>,
) -> impl IntoResponse {
    let matches = state.matches.lock().await;
    let found = matches.find(path.id);

    tracing::info!("`addr connected to play match {}.", path.id);
    ws.on_upgrade(move |socket| play(socket, found))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn play<'a>(mut socket: WebSocket, potential_room: Option<Arc<RwLock<Room>>>) {
    // VALIDATION PHASE
    // ========================================================================
    // This is where we check if the game even exists, etc.
    let Some(room) = potential_room else {
        let _ = announce(&mut socket, LobbyAnnouncement::MatchDoesNotExist {}).await;
        return;
    };

    let shared_socket = Arc::new(socket);

    let locked_room = room.clone();
    let room = locked_room.read().unwrap();

    let locked_state = room.state.clone();
    let state = match locked_state.read() {
        Ok(state) => state,
        Err(error) => {
            return;
        }
    };

    let inner_state = *state;

    let foo = match inner_state {
        RoomState::Lobby(l) => {}
        RoomState::Game(_) => todo!(),
        RoomState::PostGame { duration, winner } => todo!(),
    };

    if shared_socket
        .send(Message::Text("Hello there!".to_string()))
        .await
        .is_ok()
    {
        tracing::info!("Pinged who...");
    } else {
        tracing::error!("Could not send ping who!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }
}

fn on_join(room_ref: Arc<RwLock<Room>>) -> Result<(), String> {
    let room_lock = room_ref.clone();
    let Ok(room) = room_lock.read() else {
        // TODO: Handle these errors properly
        return Err(String::from("Failed to accquire room lock!"));
    };
    let state_lock = room.state.clone();
    let Ok(state) = state_lock.write() else {
        return Err(String::from("Failed to accquire room state lock!"));
    };

    match *state {
        RoomState::Lobby(ref lobby_lock) => on_join_during_lobby_phase(lobby_lock.clone()),
        RoomState::Game(_) => todo!(),
        RoomState::PostGame {
            duration: _,
            winner: _,
        } => todo!(),
    }
}

fn on_join_during_lobby_phase(lobby_lock: Arc<Mutex<Lobby>>) -> Result<(), String> {
    let Ok(mut lobby) = lobby_lock.lock() else {
        return Err(String::from("Failed to accquire lobby lock!"));
    };

    let Some(player_id) = lobby.try_join() else {
        return Err(String::from("Lobby full!"));
    };

    // TODO: Send the player id and secret to the lobby

    Ok(())
}

fn on_join_during_game_phase() -> Result<(), Box<dyn Error>> {
    todo!()
}

fn on_join_during_postgame_phase() -> Result<(), Box<dyn Error>> {
    todo!()
}
