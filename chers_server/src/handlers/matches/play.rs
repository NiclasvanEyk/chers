use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use chers::{Color, Coordinate, PromotedFigure};
use chers_server_api::{ClientMessage, PrivateEvent, PromotionPiece, PublicEvent, ServerMessage};
use futures::sink::SinkExt;
use serde::Deserialize;

use crate::{
    matches::{
        parse_match_id,
        state::{GameEndReason, GameResult, MatchState, MoveError, ReconnectError, StartError},
        Match,
    },
    AppState,
};

/// Player context stored per WebSocket connection
struct PlayerContext {
    token: String,
    slot: u8,     // 1 or 2
    color: Color, // White or Black
    name: String,
}

#[derive(Deserialize)]
pub struct PlayPathParams {
    id: String, // UUID as string
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(path): Path<PlayPathParams>,
) -> impl IntoResponse {
    let Some(match_id) = parse_match_id(&path.id) else {
        return (axum::http::StatusCode::BAD_REQUEST, "Invalid match ID").into_response();
    };

    let match_arc = state.matches.get(match_id);

    tracing::info!("WebSocket connection for match {}", path.id);
    ws.on_upgrade(move |socket| handle_connection(socket, match_arc))
}

async fn handle_connection(
    mut socket: WebSocket,
    match_arc: Option<Arc<tokio::sync::RwLock<Match>>>,
) {
    // 1. Validate match exists
    let Some(match_arc) = match_arc else {
        let close_msg = Message::Close(Some(axum::extract::ws::CloseFrame {
            code: 1008, // Policy violation
            reason: Utf8Bytes::from_static("Match not found"),
        }));
        let _ = socket.send(close_msg).await;
        return;
    };

    // 2. Wait for authentication message
    let (token, name) = wait_for_authentication(&mut socket).await;
    if token.is_empty() {
        return;
    }

    // 3. Check if this is a reconnection
    let is_reconnection = {
        let match_guard = match_arc.read().await;
        match_guard.get_player_color(&token).is_some() && !match_guard.is_player_connected(&token)
    };

    let (context, mut private_rx, mut public_rx) = if is_reconnection {
        // Handle reconnection
        match handle_reconnection(&mut socket, &match_arc, token, name).await {
            Some(data) => data,
            None => return,
        }
    } else {
        // Handle new connection
        match authenticate_player(&mut socket, &match_arc, token, name).await {
            Some(data) => data,
            None => return,
        }
    };

    // 4. Start game loop with player context
    game_loop(
        &mut socket,
        &match_arc,
        &context,
        &mut private_rx,
        &mut public_rx,
    )
    .await;

    // 5. Handle disconnection after game loop ends
    handle_disconnection(&match_arc, &context).await;
}

async fn wait_for_authentication(socket: &mut WebSocket) -> (String, String) {
    let msg = match socket.recv().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(Message::Close(_))) | None => {
            tracing::info!("Client disconnected before authentication");
            return (String::new(), String::new());
        }
        _ => {
            send_error(socket, "Expected text message").await;
            return (String::new(), String::new());
        }
    };

    match serde_json::from_str::<ClientMessage>(&msg) {
        Ok(ClientMessage::Authenticate { token, name }) => (token, name),
        Ok(_) => {
            send_error(socket, "First message must be Authenticate").await;
            (String::new(), String::new())
        }
        Err(e) => {
            tracing::error!("Failed to parse message: {}", e);
            send_error(socket, "Invalid message format").await;
            (String::new(), String::new())
        }
    }
}

async fn handle_reconnection(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    token: String,
    name: String,
) -> Option<(
    PlayerContext,
    tokio::sync::broadcast::Receiver<PrivateEvent>,
    tokio::sync::broadcast::Receiver<PublicEvent>,
)> {
    // Get channels for reconnection
    let (public_tx, player_tx, _player_color) = {
        let match_guard = match_arc.read().await;

        // Determine which player is reconnecting
        let color = match_guard.get_player_color(&token)?;
        let tx = if color == Color::White {
            match_guard.channels.player1_tx.clone()
        } else {
            match_guard.channels.player2_tx.clone()
        };

        (match_guard.channels.public_tx.clone(), tx, color)
    };

    let public_rx = public_tx.subscribe();
    let private_rx = player_tx.subscribe();

    // Attempt reconnection
    let reconnect_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.handle_reconnection(&token, player_tx)
    };

    match reconnect_result {
        Ok(result) => {
            // Send StateSync to reconnecting player
            let sync_event = PrivateEvent::StateSync {
                game_state: result.state,
                current_turn: result.current_turn,
                white_player: chers_server_api::server::PlayerInfo {
                    name: if result.player == Color::White {
                        name.clone()
                    } else {
                        "Opponent".to_string()
                    },
                    connected: result.white_connected,
                },
                black_player: chers_server_api::server::PlayerInfo {
                    name: if result.player == Color::Black {
                        name.clone()
                    } else {
                        "Opponent".to_string()
                    },
                    connected: result.black_connected,
                },
                draw_offered_by: None,
                move_count: result.move_history.len() as u32,
            };

            let msg = ServerMessage::Private(sync_event);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send(Message::Text(json.into())).await;
            }

            // Send re-authentication success
            let auth_success = ServerMessage::Private(PrivateEvent::Authenticated {
                player: result.player,
            });
            if let Ok(json) = serde_json::to_string(&auth_success) {
                let _ = socket.send(Message::Text(json.into())).await;
            }

            // Broadcast reconnection to opponent
            let status_event = PublicEvent::PlayerStatusChanged {
                player: result.player,
                status: chers_server_api::server::PlayerConnectionStatus::Connected,
            };

            {
                let match_guard = match_arc.read().await;
                let _ = match_guard.channels.public_tx.send(status_event);
            }

            let context = PlayerContext {
                token: token.clone(),
                slot: if result.player == Color::White { 1 } else { 2 },
                color: result.player,
                name,
            };

            Some((context, private_rx, public_rx))
        }
        Err(e) => {
            let reason = match e {
                ReconnectError::GameNotInProgress => "Game not in progress",
                ReconnectError::InvalidToken => "Invalid token",
                ReconnectError::AlreadyConnected => "Player already connected",
            };
            send_auth_failed(socket, reason).await;
            None
        }
    }
}

async fn authenticate_player(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    token: String,
    name: String,
) -> Option<(
    PlayerContext,
    tokio::sync::broadcast::Receiver<PrivateEvent>,
    tokio::sync::broadcast::Receiver<PublicEvent>,
)> {
    // Check match state
    let check_result = {
        let match_guard = match_arc.read().await;
        match &match_guard.state {
            MatchState::Lobby(_) => Ok(()),
            MatchState::InProgress(_) => Err("Game already started"),
            MatchState::Finished(_, _) => Err("Game already finished"),
        }
    };

    if let Err(reason) = check_result {
        send_auth_failed(socket, reason).await;
        return None;
    }

    // Get channels
    let (public_tx, player1_tx, player2_tx) = {
        let match_guard = match_arc.read().await;
        (
            match_guard.channels.public_tx.clone(),
            match_guard.channels.player1_tx.clone(),
            match_guard.channels.player2_tx.clone(),
        )
    };

    let public_rx = public_tx.subscribe();

    // Determine slot and private channel
    let (_slot, private_tx, private_rx) = {
        let match_guard = match_arc.read().await;
        match &match_guard.state {
            MatchState::Lobby(lobby) => {
                let slot1_occupied = lobby.player1.is_some();
                let slot = if !slot1_occupied { 1 } else { 2 };
                let private_tx = if slot == 1 { player1_tx } else { player2_tx };
                let private_rx = private_tx.subscribe();
                (slot, private_tx, private_rx)
            }
            _ => {
                send_auth_failed(socket, "Game already started").await;
                return None;
            }
        }
    };

    // Assign player
    let (assigned_slot, should_start) = {
        let mut match_guard = match_arc.write().await;
        match match_guard.assign_player(token.clone(), name.clone(), private_tx) {
            Ok((s, both)) => (s, both),
            Err(e) => {
                let reason = match e {
                    crate::matches::state::JoinError::MatchNotFound => "Match not found",
                    crate::matches::state::JoinError::MatchAlreadyStarted => "Game already started",
                    crate::matches::state::JoinError::MatchFull => "Match is full",
                    crate::matches::state::JoinError::DuplicateToken => "Token already in use",
                };
                send_auth_failed(socket, reason).await;
                return None;
            }
        }
    };

    // Start game if both connected
    if should_start {
        if let Some(game) = try_start_game(match_arc).await {
            broadcast_game_started(match_arc, &game).await;
        }
    }

    // Determine player color
    let player_color = {
        let match_guard = match_arc.read().await;
        match &match_guard.state {
            MatchState::InProgress(game) => {
                if game.white.token == token {
                    Color::White
                } else {
                    Color::Black
                }
            }
            _ => {
                // For lobby state before game starts, default to White
                Color::White
            }
        }
    };

    // Send authentication success
    let auth_success = ServerMessage::Private(PrivateEvent::Authenticated {
        player: player_color,
    });

    if let Ok(json) = serde_json::to_string(&auth_success) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    let context = PlayerContext {
        token: token.clone(),
        slot: assigned_slot,
        color: player_color,
        name,
    };

    Some((context, private_rx, public_rx))
}

async fn try_start_game(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
) -> Option<crate::matches::state::ActiveGame> {
    let mut match_guard = match_arc.write().await;
    match match_guard.start_game() {
        Ok(game) => Some(game),
        Err(StartError::NotReady) => None,
    }
}

async fn broadcast_game_started(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    game: &crate::matches::state::ActiveGame,
) {
    let event = PublicEvent::GameStarted {
        game_state: game.state.clone(),
        white_player: chers_server_api::server::PlayerInfo {
            name: game.white.name.clone(),
            connected: true,
        },
        black_player: chers_server_api::server::PlayerInfo {
            name: game.black.name.clone(),
            connected: true,
        },
    };

    let match_guard = match_arc.read().await;
    let _ = match_guard.channels.public_tx.send(event);
}

async fn game_loop(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
    private_rx: &mut tokio::sync::broadcast::Receiver<PrivateEvent>,
    public_rx: &mut tokio::sync::broadcast::Receiver<PublicEvent>,
) {
    loop {
        tokio::select! {
            // Handle WebSocket messages from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match handle_client_message(socket, match_arc, context, &text).await {
                            MessageHandlingResult::Continue => {}
                            MessageHandlingResult::GameOver => {
                                // Give time for final messages to send
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                break;
                            }
                            MessageHandlingResult::Error => break,
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("Player {} disconnected gracefully", context.token);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }

            // Handle public events
            Ok(event) = public_rx.recv() => {
                let msg = ServerMessage::Public(event);
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }

            // Handle private events
            Ok(event) = private_rx.recv() => {
                let msg = ServerMessage::Private(event);
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }
        }
    }
}

async fn handle_disconnection(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
) {
    // Mark player as disconnected
    let disconnect_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.handle_disconnection(&context.token)
    };

    let Some(disconnect_info) = disconnect_result else {
        return;
    };

    // Broadcast disconnection
    let status_event = PublicEvent::PlayerStatusChanged {
        player: disconnect_info.player,
        status: chers_server_api::server::PlayerConnectionStatus::Disconnected,
    };

    {
        let match_guard = match_arc.read().await;
        let _ = match_guard.channels.public_tx.send(status_event);
    }

    tracing::info!(
        "Player {} ({:?}) disconnected. Grace period: {}s, both_disconnected: {}",
        context.token,
        disconnect_info.player,
        disconnect_info.grace_period.as_secs(),
        disconnect_info.both_disconnected
    );

    // Start grace period timer
    let match_weak = Arc::downgrade(match_arc);
    let token = context.token.clone();
    let player_color = disconnect_info.player;
    let both = disconnect_info.both_disconnected;
    let grace_period = disconnect_info.grace_period;

    let timer = tokio::spawn(async move {
        tokio::time::sleep(grace_period).await;

        // Check if still disconnected
        if let Some(match_arc) = match_weak.upgrade() {
            let match_guard = match_arc.read().await;

            let should_end = match_guard.is_player_connected(&token);
            let should_end = !should_end; // Game should end if NOT connected

            if should_end {
                drop(match_guard); // Release read lock

                let mut match_guard = match_arc.write().await;

                // Verify again that player is still disconnected
                let still_disconnected = !match_guard.is_player_connected(&token);

                if still_disconnected {
                    // End the game
                    let result = if both {
                        GameResult::Draw(GameEndReason::Abandoned)
                    } else {
                        match player_color {
                            Color::White => GameResult::BlackWins(GameEndReason::Abandoned),
                            Color::Black => GameResult::WhiteWins(GameEndReason::Abandoned),
                        }
                    };

                    match_guard.end_game(result.clone());

                    // Broadcast game over
                    let (api_result, api_reason) = convert_game_result(&result);
                    let game_over_event = PublicEvent::GameOver {
                        result: api_result,
                        reason: api_reason,
                    };

                    let _ = match_guard.channels.public_tx.send(game_over_event);

                    tracing::info!(
                        "Game ended due to abandonment. Player: {:?}, Both disconnected: {}, Result: {:?}",
                        player_color, both, result
                    );
                }
            }
        }
    });

    // Store timer
    {
        let mut match_guard = match_arc.write().await;
        match_guard.set_disconnection_timer(timer);
    }
}

enum MessageHandlingResult {
    Continue,
    GameOver,
    Error,
}

async fn handle_client_message(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
    text: &str,
) -> MessageHandlingResult {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::MakeMove {
            from,
            to,
            promotion,
        }) => handle_make_move(socket, match_arc, context, from, to, promotion).await,
        Ok(ClientMessage::Heartbeat) => {
            // Send heartbeat acknowledgment
            let ack = ServerMessage::Private(PrivateEvent::HeartbeatAck);
            if let Ok(json) = serde_json::to_string(&ack) {
                let _ = socket.send(Message::Text(json.into())).await;
            }
            MessageHandlingResult::Continue
        }
        Ok(_) => {
            // Other messages not yet supported
            send_move_rejected(socket, "Message type not supported").await;
            MessageHandlingResult::Continue
        }
        Err(e) => {
            tracing::error!("Failed to parse client message: {}", e);
            send_move_rejected(socket, "Invalid message format").await;
            MessageHandlingResult::Continue
        }
    }
}

async fn handle_make_move(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
    from: Coordinate,
    to: Coordinate,
    promotion: Option<PromotionPiece>,
) -> MessageHandlingResult {
    // Convert promotion
    let promoted = promotion.map(|p| match p {
        PromotionPiece::Q => PromotedFigure::Queen,
        PromotionPiece::R => PromotedFigure::Rook,
        PromotionPiece::B => PromotedFigure::Bishop,
        PromotionPiece::N => PromotedFigure::Knight,
    });

    // Try the move (single write lock)
    let move_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.try_move(&context.token, from, to, promoted)
    };

    match move_result {
        Ok(move_result) => {
            // Broadcast MoveMade
            let event = PublicEvent::MoveMade {
                author: context.color,
                from,
                to,
                promotion,
                new_state: move_result.new_state,
                is_check: move_result.is_check,
                is_checkmate: move_result.is_checkmate,
            };

            {
                let match_guard = match_arc.read().await;
                let _ = match_guard.channels.public_tx.send(event);
            }

            // Handle game over
            if let Some(result) = move_result.game_over {
                let (api_result, api_reason) = convert_game_result(&result);

                let game_over_event = PublicEvent::GameOver {
                    result: api_result,
                    reason: api_reason,
                };

                {
                    let match_guard = match_arc.read().await;
                    let _ = match_guard.channels.public_tx.send(game_over_event);
                }

                // Give time for messages to propagate
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Close connection
                let _ = socket.close().await;
                return MessageHandlingResult::GameOver;
            }

            MessageHandlingResult::Continue
        }
        Err(e) => {
            // Send private rejection
            let reason = match e {
                MoveError::NotYourTurn => {
                    chers_server_api::server::MoveRejectionReason::NotYourTurn
                }
                MoveError::InvalidMove => {
                    chers_server_api::server::MoveRejectionReason::IllegalMove
                }
                MoveError::GameNotInProgress => {
                    chers_server_api::server::MoveRejectionReason::GameOver
                }
                MoveError::GamePaused => chers_server_api::server::MoveRejectionReason::GameOver,
                _ => chers_server_api::server::MoveRejectionReason::InvalidNotation,
            };

            let rejection = PrivateEvent::MoveRejected { reason };
            let msg = ServerMessage::Private(rejection);

            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send(Message::Text(json.into())).await;
            }

            MessageHandlingResult::Continue
        }
    }
}

fn convert_game_result(
    result: &GameResult,
) -> (
    chers_server_api::server::GameResult,
    chers_server_api::server::GameEndReason,
) {
    match result {
        GameResult::WhiteWins(reason) => {
            let api_reason = match reason {
                GameEndReason::Checkmate => chers_server_api::server::GameEndReason::Checkmate,
                GameEndReason::Abandoned => chers_server_api::server::GameEndReason::Abandoned,
                _ => chers_server_api::server::GameEndReason::Abandoned,
            };
            (chers_server_api::server::GameResult::WhiteWins, api_reason)
        }
        GameResult::BlackWins(reason) => {
            let api_reason = match reason {
                GameEndReason::Checkmate => chers_server_api::server::GameEndReason::Checkmate,
                GameEndReason::Abandoned => chers_server_api::server::GameEndReason::Abandoned,
                _ => chers_server_api::server::GameEndReason::Abandoned,
            };
            (chers_server_api::server::GameResult::BlackWins, api_reason)
        }
        GameResult::Draw(reason) => {
            let api_reason = match reason {
                GameEndReason::Stalemate => chers_server_api::server::GameEndReason::Stalemate,
                GameEndReason::Abandoned => chers_server_api::server::GameEndReason::Abandoned,
                _ => chers_server_api::server::GameEndReason::Abandoned,
            };
            (chers_server_api::server::GameResult::Draw, api_reason)
        }
    }
}

async fn send_move_rejected(_socket: &mut WebSocket, _message: &str) {
    // For now, just log - we can implement full error messages later
    tracing::warn!("Move rejected: {}", _message);
}

async fn send_error(socket: &mut WebSocket, message: &str) {
    let error_json = serde_json::json!({
        "error": message
    });
    let _ = socket
        .send(Message::Text(error_json.to_string().into()))
        .await;
    let _ = socket
        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
            code: 1008,
            reason: Utf8Bytes::from_static("Error"),
        })))
        .await;
}

async fn send_auth_failed(socket: &mut WebSocket, _reason: &str) {
    let msg = ServerMessage::Private(PrivateEvent::AuthenticationFailed {
        reason: chers_server_api::server::AuthFailureReason::InvalidToken,
    });
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = socket.send(Message::Text(json.into())).await;
    }
    let _ = socket
        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
            code: 1008,
            reason: Utf8Bytes::from_static("Authentication failed"),
        })))
        .await;
}
