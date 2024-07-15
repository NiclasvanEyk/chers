use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    matches::{communication::Commands, Match},
    AppState,
};

use chers_server_api::PendingGameMessages;

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
async fn play(mut socket: WebSocket, context: Option<Arc<Match>>) {
    // VALIDATION PHASE
    // ========================================================================
    // This is where we check if the game even exists, etc.
    let mut commands = Commands::new(&mut socket);
    let Some(match_context) = context else {
        let _ = commands
            .send(PendingGameMessages::GameDoesNotExist {})
            .await;
        return;
    };

    // LOBBY PHASE
    // ========================================================================
    // Wait in the lobby, until there are two players, who both indicated that
    // they are ready to start the match.

    // GAME PHASE
    // ========================================================================
    // The two players take turns and mutate the board state. We somehow need to
    // find a nice way to "merge" the two websocket connections, have them take
    // turns (e.g. do not accept "move" messages from white while its blacks
    // turn) and support reconnections.

    // POST-GAME PHASE
    // ========================================================================
    // Do whatever here. I just think that the players should not be kicked out
    // right away after one wins.

    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Text("FOOO".to_string())).await.is_ok() {
        tracing::info!("Pinged who...");
    } else {
        tracing::error!("Could not send ping who!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    let match_id = match_context.id;
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            tracing::info!("who disconnected from match {match_id}");
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            tracing::info!("who disconnected from match {match_id}");
            // client disconnected
            return;
        }
    }
}
