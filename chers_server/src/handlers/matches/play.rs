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
use tracing::{debug, error, info, instrument, warn};

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

#[derive(Deserialize, Debug)]
pub struct PlayPathParams {
    id: String, // UUID as string
}

#[instrument(skip(ws, state))]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(path): Path<PlayPathParams>,
) -> impl IntoResponse {
    let Some(match_id) = parse_match_id(&path.id) else {
        warn!(match_id = %path.id, "❌ Invalid match ID format");
        return (axum::http::StatusCode::BAD_REQUEST, "Invalid match ID").into_response();
    };

    let match_arc = state.matches.get(match_id);

    info!(match_id = %path.id, "🔌 WebSocket connection attempt");
    ws.on_upgrade(move |socket| handle_connection(socket, match_arc, path.id))
}

async fn handle_connection(
    mut socket: WebSocket,
    match_arc: Option<Arc<tokio::sync::RwLock<Match>>>,
    match_id: String,
) {
    // 1. Validate match exists
    let Some(match_arc) = match_arc else {
        warn!("❌ Match not found: {}", match_id);
        let close_msg = Message::Close(Some(axum::extract::ws::CloseFrame {
            code: 1008, // Policy violation
            reason: Utf8Bytes::from_static("Match not found"),
        }));
        let _ = socket.send(close_msg).await;
        return;
    };

    info!("✅ Match {} found, waiting for authentication", match_id);

    // 2. Wait for authentication message
    let (token, name) = wait_for_authentication(&mut socket).await;
    if token.is_empty() {
        warn!("❌ Authentication failed for match {}: empty token", match_id);
        return;
    }

    info!("🔑 Player '{}' (token: {}) authenticating in match {}", name, token, match_id);

    // 3. Check if this is a reconnection
    let is_reconnection = {
        let match_guard = match_arc.read().await;
        let reconnecting = match_guard.get_player_color(&token).is_some() && !match_guard.is_player_connected(&token);
        if reconnecting {
            info!("🔄 Detected reconnection for player {} in match {}", token, match_id);
        }
        reconnecting
    };

    let (context, mut private_rx, mut public_rx) = if is_reconnection {
        // Handle reconnection
        match handle_reconnection(&mut socket, &match_arc, &match_id, token, name).await {
            Some(data) => data,
            None => return,
        }
    } else {
        // Handle new connection
        match authenticate_player(&mut socket, &match_arc, &match_id, token, name).await {
            Some(data) => data,
            None => return,
        }
    };

    info!(
        "🎮 Player {} ({:?}) ready in match {}. Starting game loop.",
        context.name, context.color, match_id
    );

    // 4. Start game loop with player context
    game_loop(
        &mut socket,
        &match_arc,
        &match_id,
        &context,
        &mut private_rx,
        &mut public_rx,
    )
    .await;

    // 5. Handle disconnection after game loop ends
    info!("👋 Player {} disconnecting from match {}", context.name, match_id);
    handle_disconnection(&match_arc, &match_id, &context).await;
}

async fn wait_for_authentication(socket: &mut WebSocket) -> (String, String) {
    let msg = match socket.recv().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(Message::Close(_))) | None => {
            info!("⏹️  Client disconnected before authentication");
            return (String::new(), String::new());
        }
        _ => {
            send_error(socket, "Expected text message").await;
            return (String::new(), String::new());
        }
    };

    debug!("📨 Received message: {}", msg);

    match serde_json::from_str::<ClientMessage>(&msg) {
        Ok(ClientMessage::Authenticate { token, name }) => {
            info!("✅ Authentication received - Name: {}, Token: {}", name, token);
            (token, name)
        }
        Ok(_) => {
            warn!("❌ First message was not Authenticate");
            send_error(socket, "First message must be Authenticate").await;
            (String::new(), String::new())
        }
        Err(e) => {
            error!("❌ Failed to parse authentication message: {}", e);
            send_error(socket, "Invalid message format").await;
            (String::new(), String::new())
        }
    }
}

async fn handle_reconnection(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    match_id: &str,
    token: String,
    name: String,
) -> Option<(
    PlayerContext,
    tokio::sync::broadcast::Receiver<PrivateEvent>,
    tokio::sync::broadcast::Receiver<PublicEvent>,
)> {
    info!("🔄 Processing reconnection for player {} in match {}", token, match_id);
    
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
            info!(
                "✅ Reconnection successful for player {:?} in match {}",
                result.player, match_id
            );
            
            // Send StateSync to reconnecting player
            let (game_result, game_end_reason) = result.game_result.as_ref()
                .map(|r| r.to_api())
                .map(|(gr, ger)| (Some(gr), Some(ger)))
                .unwrap_or((None, None));
            
            let sync_event = PrivateEvent::StateSync {
                game_state: result.state,
                current_turn: result.current_turn,
                white_player: chers_server_api::server::PlayerInfo {
                    name: result.white_name,
                    connected: result.white_connected,
                },
                black_player: chers_server_api::server::PlayerInfo {
                    name: result.black_name,
                    connected: result.black_connected,
                },
                draw_offered_by: None,
                move_count: result.move_history.len() as u32,
                your_color: result.player,
                game_result,
                game_end_reason,
            };

            let msg = ServerMessage::Private(sync_event);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send(Message::Text(json.into())).await;
                debug!("📤 Sent StateSync to reconnecting player");
            }

            // If game is finished, don't start game loop - just return
            if result.game_result.is_some() {
                info!("🏁 Game already finished, closing connection after StateSync");
                // Give time for StateSync to be received
                tokio::time::sleep(Duration::from_millis(500)).await;
                let _ = socket.close().await;
                return None;
            }

            // Reconnection complete for active game - StateSync contains color info
            info!(
                "📢 Broadcast player reconnection: {:?} in match {}",
                result.player, match_id
            );
            
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
            warn!("❌ Reconnection failed for {}: {}", token, reason);
            send_auth_failed(socket, reason).await;
            None
        }
    }
}

async fn authenticate_player(
    socket: &mut WebSocket,
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    match_id: &str,
    token: String,
    name: String,
) -> Option<(
    PlayerContext,
    tokio::sync::broadcast::Receiver<PrivateEvent>,
    tokio::sync::broadcast::Receiver<PublicEvent>,
)> {
    info!("🔐 Authenticating new player '{}' (token: {}) in match {}", name, token, match_id);
    
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
        warn!("❌ Cannot join match {}: {}", match_id, reason);
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
                warn!("❌ Failed to assign player in match {}: {}", match_id, reason);
                send_auth_failed(socket, reason).await;
                return None;
            }
        }
    };

    info!(
        "✅ Player '{}' assigned to slot {} in match {}. Both connected: {}",
        name, assigned_slot, match_id, should_start
    );

    // Get current lobby state for the response
    let (player1_info, player2_info, p1_ready, p2_ready) = {
        let match_guard = match_arc.read().await;
        match &match_guard.state {
            MatchState::Lobby(lobby) => {
                let p1 = lobby.player1.as_ref().map(|p| chers_server_api::server::PlayerInfo {
                    name: p.name.clone(),
                    connected: p.connected,
                });
                let p2 = lobby.player2.as_ref().map(|p| chers_server_api::server::PlayerInfo {
                    name: p.name.clone(),
                    connected: p.connected,
                });
                (p1, p2, lobby.player1_ready, lobby.player2_ready)
            }
            _ => (None, None, false, false),
        }
    };

    // Player is waiting in lobby - send LobbyJoined with full lobby state
    let lobby_joined = ServerMessage::Private(PrivateEvent::LobbyJoined { 
        slot: assigned_slot,
        player1: player1_info,
        player2: player2_info,
        player1_ready: p1_ready,
        player2_ready: p2_ready,
    });
    if let Ok(json) = serde_json::to_string(&lobby_joined) {
        let _ = socket.send(Message::Text(json.into())).await;
        debug!("📤 Sent LobbyJoined with lobby state to slot {}", assigned_slot);
    }

    info!("⏳ Player '{}' waiting in lobby for match {}", name, match_id);

    // Context without color (will be assigned when game starts via ColorsAssigned)
    let context = PlayerContext {
        token: token.clone(),
        slot: assigned_slot,
        color: Color::White, // Temporary placeholder, will be overwritten by ColorsAssigned
        name,
    };

    Some((context, private_rx, public_rx))
}

async fn try_start_game(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
) -> Option<(crate::matches::state::ActiveGame, bool)> {
    let mut match_guard = match_arc.write().await;
    match match_guard.start_game() {
        Ok((game, player1_is_white)) => Some((game, player1_is_white)),
        Err(StartError::NotReady) => None,
    }
}

async fn broadcast_game_started(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    match_id: &str,
    game: &crate::matches::state::ActiveGame,
) {
    info!(
        "📢 Broadcasting GameStarted in match {} - White: {}, Black: {}",
        match_id, game.white.name, game.black.name
    );
    
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
    match_id: &str,
    context: &PlayerContext,
    private_rx: &mut tokio::sync::broadcast::Receiver<PrivateEvent>,
    public_rx: &mut tokio::sync::broadcast::Receiver<PublicEvent>,
) {
    debug!(
        "🔄 Game loop started for player {} ({:?}) in match {}",
        context.name, context.color, match_id
    );
    
    loop {
        tokio::select! {
            // Handle WebSocket messages from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("📨 Received message from {}: {}", context.name, text);
                        match handle_client_message(socket, match_arc, context, &text).await {
                            MessageHandlingResult::Continue => {}
                            MessageHandlingResult::GameOver => {
                                info!("🏁 Game over in match {}", match_id);
                                // Give time for final messages to send
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                break;
                            }
                            MessageHandlingResult::Error => break,
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("👋 Player {} disconnected gracefully from match {}", context.name, match_id);
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
                    debug!("📤 Sent public event to {}", context.name);
                }
            }

            // Handle private events
            Ok(event) = private_rx.recv() => {
                let msg = ServerMessage::Private(event);
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send(Message::Text(json.into())).await;
                    debug!("📤 Sent private event to {}", context.name);
                }
            }
        }
    }
    
    debug!("🛑 Game loop ended for player {} in match {}", context.name, match_id);
}

async fn handle_disconnection(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    match_id: &str,
    context: &PlayerContext,
) {
    info!(
        "🔌 Handling disconnection for player {} (slot {}) in match {}",
        context.name, context.slot, match_id
    );
    
    // Check if we're in lobby or game state
    let is_lobby = {
        let match_guard = match_arc.read().await;
        matches!(match_guard.state, MatchState::Lobby(_))
    };
    
    if is_lobby {
        // In lobby: immediately remove player and notify others
        let removed_slot = {
            let mut match_guard = match_arc.write().await;
            match_guard.remove_player_from_lobby(&context.token)
        };
        
        if let Some(slot) = removed_slot {
            info!(
                "👋 Player {} left lobby slot {} in match {} (immediate removal)",
                context.name, slot, match_id
            );
            
            // Broadcast that player left
            let leave_event = PublicEvent::PlayerLeftLobby { slot };
            {
                let match_guard = match_arc.read().await;
                let _ = match_guard.channels.public_tx.send(leave_event);
            }
        }
        
        return;
    }
    
    // In game: use grace period logic (existing code)
    let disconnect_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.handle_disconnection(&context.token)
    };

    let Some(disconnect_info) = disconnect_result else {
        warn!(
            "⚠️  No disconnect info for player {} in match {}",
            context.name, match_id
        );
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

    info!(
        "📢 Player {} ({:?}) disconnected from match {}. Grace period: {}s, both_disconnected: {}",
        context.name,
        disconnect_info.player,
        match_id,
        disconnect_info.grace_period.as_secs(),
        disconnect_info.both_disconnected
    );

    // Start grace period timer
    let match_weak = Arc::downgrade(match_arc);
    let token = context.token.clone();
    let player_color = disconnect_info.player;
    let both = disconnect_info.both_disconnected;
    let grace_period = disconnect_info.grace_period;
    let match_id_owned = match_id.to_string();

    let timer = tokio::spawn(async move {
        tokio::time::sleep(grace_period).await;

        if let Some(match_arc) = match_weak.upgrade() {
            let match_guard = match_arc.read().await;
            let should_end = !match_guard.is_player_connected(&token);

            if should_end {
                drop(match_guard);
                let mut match_guard = match_arc.write().await;
                let still_disconnected = !match_guard.is_player_connected(&token);

                if still_disconnected {
                    let result = if both {
                        GameResult::Draw(GameEndReason::Abandoned)
                    } else {
                        match player_color {
                            Color::White => GameResult::BlackWins(GameEndReason::Abandoned),
                            Color::Black => GameResult::WhiteWins(GameEndReason::Abandoned),
                        }
                    };

                    match_guard.end_game(result.clone());

                    let (api_result, api_reason) = convert_game_result(&result);
                    let game_over_event = PublicEvent::GameOver {
                        result: api_result,
                        reason: api_reason,
                    };

                    let _ = match_guard.channels.public_tx.send(game_over_event);

                    info!(
                        "⏰ Grace period expired. Game ended in match {} due to abandonment. Player: {:?}, Both disconnected: {}, Result: {:?}",
                        match_id_owned, player_color, both, result
                    );
                }
            }
        }
    });

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
        }) => {
            info!(
                "♟️  Move attempt by {} ({:?}): {} -> {} (promotion: {:?})",
                context.name, context.color, format_coords(from), format_coords(to), promotion
            );
            handle_make_move(socket, match_arc, context, from, to, promotion).await
        }
        Ok(ClientMessage::Heartbeat) => {
            debug!("💓 Heartbeat from {}", context.name);
            // Send heartbeat acknowledgment
            let ack = ServerMessage::Private(PrivateEvent::HeartbeatAck);
            if let Ok(json) = serde_json::to_string(&ack) {
                let _ = socket.send(Message::Text(json.into())).await;
            }
            MessageHandlingResult::Continue
        }
        Ok(ClientMessage::UpdateName { name }) => {
            info!("✏️  Name update request from {} to '{}'", context.name, name);
            handle_update_name(match_arc, context, name).await
        }
        Ok(ClientMessage::Ready { ready }) => {
            info!("✅ Ready toggle from {}: {}", context.name, ready);
            handle_ready(match_arc, context, ready).await
        }
        Ok(other) => {
            warn!("⚠️  Unsupported message from {}: {:?}", context.name, other);
            send_move_rejected(socket, "Message type not supported").await;
            MessageHandlingResult::Continue
        }
        Err(e) => {
            error!("❌ Failed to parse client message from {}: {}", context.name, e);
            send_move_rejected(socket, "Invalid message format").await;
            MessageHandlingResult::Continue
        }
    }
}

fn format_coords(coord: Coordinate) -> String {
    let file = (b'a' + coord.x as u8) as char;
    let rank = 8 - coord.y;
    format!("{}{}", file, rank)
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
            info!(
                "✅ Move successful by {}: {} -> {} (check: {}, checkmate: {})",
                context.name,
                format_coords(from),
                format_coords(to),
                move_result.is_check,
                move_result.is_checkmate
            );
            
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

                info!(
                    "🏁 Game over! Result: {:?}, Reason: {:?}",
                    api_result, api_reason
                );

                // Give more time for messages to propagate to both players
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Close connection gracefully
                let _ = socket.close().await;
                return MessageHandlingResult::GameOver;
            }

            MessageHandlingResult::Continue
        }
        Err(e) => {
            let reason = match e {
                MoveError::NotYourTurn => {
                    warn!("⛔ Move rejected for {}: Not your turn", context.name);
                    chers_server_api::server::MoveRejectionReason::NotYourTurn
                }
                MoveError::InvalidMove => {
                    warn!("⛔ Move rejected for {}: Illegal move", context.name);
                    chers_server_api::server::MoveRejectionReason::IllegalMove
                }
                MoveError::GameNotInProgress => {
                    warn!("⛔ Move rejected for {}: Game not in progress", context.name);
                    chers_server_api::server::MoveRejectionReason::GameOver
                }
                MoveError::GamePaused => {
                    warn!("⛔ Move rejected for {}: Game paused (disconnected)", context.name);
                    chers_server_api::server::MoveRejectionReason::GameOver
                }
                _ => {
                    warn!("⛔ Move rejected for {}: Invalid notation", context.name);
                    chers_server_api::server::MoveRejectionReason::InvalidNotation
                }
            };

            let rejection = PrivateEvent::MoveRejected { reason };
            let msg = ServerMessage::Private(rejection);

            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send(Message::Text(json.into())).await;
                debug!("📤 Sent MoveRejected to {}", context.name);
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

async fn handle_update_name(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
    new_name: String,
) -> MessageHandlingResult {
    // Try to update the name
    let update_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.update_player_name(&context.token, new_name.clone())
    };

    match update_result {
        Ok(player_color) => {
            info!(
                "✅ Name updated for player {:?} to '{}'",
                player_color, new_name
            );
            
            // Broadcast name change to all clients
            let event = PublicEvent::NameChanged {
                player: player_color,
                slot: context.slot,
                name: new_name,
            };

            {
                let match_guard = match_arc.read().await;
                let _ = match_guard.channels.public_tx.send(event);
            }
            
            MessageHandlingResult::Continue
        }
        Err(error_msg) => {
            warn!(
                "⚠️  Name update failed for {}: {}",
                context.name, error_msg
            );
            // We could send an error message to the client here, but for now just log it
            MessageHandlingResult::Continue
        }
    }
}

async fn handle_ready(
    match_arc: &Arc<tokio::sync::RwLock<Match>>,
    context: &PlayerContext,
    ready: bool,
) -> MessageHandlingResult {
    // Try to toggle ready status
    let toggle_result = {
        let mut match_guard = match_arc.write().await;
        match_guard.toggle_ready(&context.token, ready)
    };

    match toggle_result {
        Ok((player_color, new_ready_status, both_ready)) => {
            info!(
                "✅ Ready status toggled for player {:?}: {} (both ready: {})",
                player_color, new_ready_status, both_ready
            );
            
            // Broadcast ready status change to all clients
            let event = PublicEvent::PlayerReady {
                player: player_color,
                slot: context.slot,
                ready: new_ready_status,
            };

            {
                let match_guard = match_arc.read().await;
                let _ = match_guard.channels.public_tx.send(event);
            }
            
            // If both players are ready, start the countdown
            if both_ready {
                info!("🎮 Both players ready! Starting 5-second countdown...");
                start_countdown(match_arc).await;
            }
            
            MessageHandlingResult::Continue
        }
        Err(error_msg) => {
            warn!(
                "⚠️  Ready toggle failed for {}: {}",
                context.name, error_msg
            );
            MessageHandlingResult::Continue
        }
    }
}

async fn start_countdown(match_arc: &Arc<tokio::sync::RwLock<Match>>) {
    // Spawn countdown task
    let match_weak = Arc::downgrade(match_arc);
    
    tokio::spawn(async move {
        // Countdown from 5 to 1
        for seconds in (1..=5).rev() {
            // Send countdown tick
            if let Some(match_arc) = match_weak.upgrade() {
                let match_guard = match_arc.read().await;
                let event = PublicEvent::Countdown { seconds };
                let _ = match_guard.channels.public_tx.send(event);
                drop(match_guard);
            }
            
            // Wait 1 second
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            // Check if we should still proceed (both still ready)
            if let Some(match_arc) = match_weak.upgrade() {
                let match_guard = match_arc.read().await;
                let should_proceed = if let MatchState::Lobby(lobby) = &match_guard.state {
                    lobby.ready_to_start()
                } else {
                    false
                };
                drop(match_guard);
                
                if !should_proceed {
                    info!("⏹️  Countdown cancelled - players no longer ready");
                    return;
                }
            } else {
                return;
            }
        }
        
        // Countdown complete - start the game
        if let Some(match_arc) = match_weak.upgrade() {
            info!("🚀 Countdown complete! Starting game...");
            
            // Send GameStarting event
            {
                let match_guard = match_arc.read().await;
                let event = PublicEvent::GameStarting;
                let _ = match_guard.channels.public_tx.send(event);
                drop(match_guard);
            }
            
            // Start the game
            let start_result = {
                let mut match_guard = match_arc.write().await;
                match_guard.start_game()
            };
            
            match start_result {
                Ok((active_game, player1_is_white)) => {
                    info!("🎮 Game started successfully! (player1_is_white: {})", player1_is_white);
                    
                    // Broadcast GameStarted event
                    let match_guard = match_arc.read().await;
                    let event = PublicEvent::GameStarted {
                        game_state: active_game.state.clone(),
                        white_player: chers_server_api::server::PlayerInfo {
                            name: active_game.white.name.clone(),
                            connected: active_game.white.connected,
                        },
                        black_player: chers_server_api::server::PlayerInfo {
                            name: active_game.black.name.clone(),
                            connected: active_game.black.connected,
                        },
                    };
                    let _ = match_guard.channels.public_tx.send(event);
                    
                    // Send ColorsAssigned to both players via correct channels
                    // player1 channel gets assigned to whoever is in slot 1
                    // player2 channel gets assigned to whoever is in slot 2
                    if player1_is_white {
                        let _ = match_guard.channels.player1_tx.send(chers_server_api::PrivateEvent::ColorsAssigned { player: Color::White });
                        let _ = match_guard.channels.player2_tx.send(chers_server_api::PrivateEvent::ColorsAssigned { player: Color::Black });
                    } else {
                        let _ = match_guard.channels.player1_tx.send(chers_server_api::PrivateEvent::ColorsAssigned { player: Color::Black });
                        let _ = match_guard.channels.player2_tx.send(chers_server_api::PrivateEvent::ColorsAssigned { player: Color::White });
                    }
                }
                Err(e) => {
                    error!("❌ Failed to start game: {:?}", e);
                }
            }
        }
    });
}
