use std::time::Duration;

use chers::{Color, Coordinate, Game, PromotedFigure, State};
use jiff::Timestamp;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use chers_server_api::PrivateEvent;

use crate::matches::player::{PlayerInfo, PlayerSlot};

use super::channels::MatchChannels;
use super::MatchId;

pub struct Match {
    pub id: MatchId,
    pub created_at: Timestamp,
    pub state: MatchState,
    pub channels: MatchChannels,
    pub move_count: u32,
}

impl Match {
    pub fn new(id: MatchId) -> Self {
        Self {
            id,
            created_at: Timestamp::now(),
            state: MatchState::Lobby(LobbyState::default()),
            channels: MatchChannels::new(),
            move_count: 0,
        }
    }

    /// Assign a player to slot 1 or 2.
    /// Returns (slot_number, both_connected).
    pub fn assign_player(
        &mut self,
        token: String,
        name: String,
        private_tx: broadcast::Sender<PrivateEvent>,
    ) -> Result<(u8, bool), JoinError> {
        let lobby = match &mut self.state {
            MatchState::Lobby(lobby) => lobby,
            _ => return Err(JoinError::MatchAlreadyStarted),
        };

        // Check for duplicate tokens
        if lobby.token_in_use(&token) {
            return Err(JoinError::DuplicateToken);
        }

        let slot = PlayerSlot {
            name,
            token,
            connected: true,
            last_seen_at: Timestamp::now(),
            tx: Some(private_tx),
        };

        // Assign to player1 or player2
        if lobby.player1.is_none() {
            lobby.player1 = Some(slot);
            let both_connected = lobby.player2.as_ref().map(|p| p.connected).unwrap_or(false);
            Ok((1, both_connected))
        } else if lobby.player2.is_none() {
            lobby.player2 = Some(slot);
            let both_connected = lobby.player1.as_ref().map(|p| p.connected).unwrap_or(true);
            Ok((2, both_connected))
        } else {
            Err(JoinError::MatchFull)
        }
    }

    /// Start the game when both players are connected.
    /// Randomly assigns White/Black to player1/player2.
    pub fn start_game(&mut self) -> Result<ActiveGame, StartError> {
        let lobby = match &self.state {
            MatchState::Lobby(lobby) if lobby.ready_to_start() => lobby,
            _ => return Err(StartError::NotReady),
        };

        // Extract player info
        let p1 = lobby.player1.as_ref().unwrap();
        let p2 = lobby.player2.as_ref().unwrap();

        // Random assignment: true = player1 is White, false = player2 is White
        let player1_is_white = rand::random::<bool>();

        let (white, black) = if player1_is_white {
            (
                PlayerInfo {
                    name: p1.name.clone(),
                    connected: true,
                    token: p1.token.clone(),
                    disconnected_at: None,
                },
                PlayerInfo {
                    name: p2.name.clone(),
                    connected: true,
                    token: p2.token.clone(),
                    disconnected_at: None,
                },
            )
        } else {
            (
                PlayerInfo {
                    name: p2.name.clone(),
                    connected: true,
                    token: p2.token.clone(),
                    disconnected_at: None,
                },
                PlayerInfo {
                    name: p1.name.clone(),
                    connected: true,
                    token: p1.token.clone(),
                    disconnected_at: None,
                },
            )
        };

        let game = ActiveGame {
            started_at: Timestamp::now(),
            white,
            black,
            state: Game::new().start(),
            game: Game::new(),
            move_history: Vec::new(),
            disconnection_timer: None,
            paused: false,
        };

        self.state = MatchState::InProgress(game.clone());

        Ok(game)
    }

    /// Attempt to make a move.
    /// Validates turn order and move legality.
    pub fn try_move(
        &mut self,
        player_token: &str,
        from: Coordinate,
        to: Coordinate,
        promotion: Option<PromotedFigure>,
    ) -> Result<MoveResult, MoveError> {
        let active = match &mut self.state {
            MatchState::InProgress(active) => active,
            _ => return Err(MoveError::GameNotInProgress),
        };

        // Check if game is paused due to disconnection
        if active.paused {
            return Err(MoveError::GamePaused);
        }

        // Determine which color this player is
        let player_color = if active.white.token == player_token {
            Color::White
        } else if active.black.token == player_token {
            Color::Black
        } else {
            return Err(MoveError::PlayerNotFound);
        };

        // Check turn
        if active.state.player != player_color {
            return Err(MoveError::NotYourTurn);
        }

        // Validate and apply move
        let chess_move = chers::Move {
            from,
            to,
            promotion,
        };

        match active.game.move_piece(&active.state, chess_move) {
            Ok((new_state, events)) => {
                // Record the move
                let record = MoveRecord {
                    move_number: self.move_count + 1,
                    player: player_color,
                    from,
                    to,
                    promotion,
                    timestamp: Timestamp::now(),
                };
                active.move_history.push(record);

                // Update game state
                active.state = new_state.clone();
                self.move_count += 1;

                // Check for game end using the check module
                let is_checkmate = events.iter().any(|e| matches!(e, chers::Event::Mate));
                let is_check = !is_checkmate
                    && events
                        .iter()
                        .any(|e| matches!(e, chers::Event::Check { .. }));

                // Check for stalemate - need to see if player has any legal moves
                let is_stalemate = false; // TODO: Implement proper stalemate detection

                let game_over = if is_checkmate {
                    Some(match player_color {
                        Color::White => GameResult::WhiteWins(GameEndReason::Checkmate),
                        Color::Black => GameResult::BlackWins(GameEndReason::Checkmate),
                    })
                } else {
                    None
                };

                Ok(MoveResult {
                    new_state,
                    is_check,
                    is_checkmate,
                    is_stalemate,
                    game_over,
                })
            }
            Err(_) => Err(MoveError::InvalidMove),
        }
    }

    /// Handle player disconnection
    /// Returns information needed to start grace period timer
    pub fn handle_disconnection(&mut self, token: &str) -> Option<DisconnectionResult> {
        let active = match &mut self.state {
            MatchState::InProgress(active) => active,
            _ => return None,
        };

        // Find player and mark disconnected
        let player_color = if active.white.token == token {
            Color::White
        } else if active.black.token == token {
            Color::Black
        } else {
            return None;
        };

        let player = if player_color == Color::White {
            &mut active.white
        } else {
            &mut active.black
        };

        // Check if already disconnected
        if !player.connected {
            return None;
        }

        // Mark as disconnected
        player.connected = false;
        player.disconnected_at = Some(Timestamp::now());

        // Pause the game
        active.paused = true;

        // Cancel any existing timer
        if let Some(timer) = active.disconnection_timer.take() {
            timer.abort();
        }

        // Check if both disconnected
        let both_disconnected = !active.white.connected && !active.black.connected;

        Some(DisconnectionResult {
            player: player_color,
            both_disconnected,
            grace_period: Duration::from_secs(120), // 2 minutes
        })
    }

    /// Handle player reconnection
    /// Validates token and restores player connection
    pub fn handle_reconnection(
        &mut self,
        token: &str,
        _private_tx: broadcast::Sender<PrivateEvent>,
    ) -> Result<ReconnectionResult, ReconnectError> {
        let active = match &mut self.state {
            MatchState::InProgress(active) => active,
            _ => return Err(ReconnectError::GameNotInProgress),
        };

        // Find which player this token belongs to
        let player_color = if active.white.token == token {
            Color::White
        } else if active.black.token == token {
            Color::Black
        } else {
            return Err(ReconnectError::InvalidToken);
        };

        let player = if player_color == Color::White {
            &mut active.white
        } else {
            &mut active.black
        };

        // Verify not already connected
        if player.connected {
            return Err(ReconnectError::AlreadyConnected);
        }

        // Mark reconnected
        player.connected = true;
        player.disconnected_at = None;
        // Note: We don't store the channel in PlayerInfo - it's passed separately
        // The channel subscription is managed by the WebSocket handler

        // Cancel disconnection timer
        if let Some(timer) = active.disconnection_timer.take() {
            timer.abort();
        }

        // Check if both players now connected
        let both_connected = active.white.connected && active.black.connected;

        if both_connected {
            // Resume game
            active.paused = false;
        }

        Ok(ReconnectionResult {
            player: player_color,
            game_resumed: both_connected,
            state: active.state.clone(),
            move_history: active.move_history.clone(),
            current_turn: active.state.player,
            white_connected: active.white.connected,
            black_connected: active.black.connected,
        })
    }

    /// End the game with a result
    pub fn end_game(&mut self, result: GameResult) {
        let active = match &self.state {
            MatchState::InProgress(active) => active,
            _ => return,
        };

        // Cancel any pending timer
        if let Some(timer) = &active.disconnection_timer {
            timer.abort();
        }

        self.state = MatchState::Finished(result, Timestamp::now());
    }

    /// Set the disconnection timer
    pub fn set_disconnection_timer(&mut self, timer: JoinHandle<()>) {
        if let MatchState::InProgress(active) = &mut self.state {
            // Cancel existing timer if any
            if let Some(existing) = active.disconnection_timer.take() {
                existing.abort();
            }
            active.disconnection_timer = Some(timer);
        }
    }

    /// Get player color by token
    pub fn get_player_color(&self, token: &str) -> Option<Color> {
        match &self.state {
            MatchState::InProgress(active) => {
                if active.white.token == token {
                    Some(Color::White)
                } else if active.black.token == token {
                    Some(Color::Black)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if game is paused
    pub fn is_game_paused(&self) -> bool {
        match &self.state {
            MatchState::InProgress(active) => active.paused,
            _ => false,
        }
    }

    /// Check if player is connected
    pub fn is_player_connected(&self, token: &str) -> bool {
        match &self.state {
            MatchState::InProgress(active) => {
                if active.white.token == token {
                    active.white.connected
                } else if active.black.token == token {
                    active.black.connected
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get disconnected players
    pub fn get_disconnected_players(&self) -> Vec<(Color, &PlayerInfo)> {
        let mut disconnected = Vec::new();

        if let MatchState::InProgress(active) = &self.state {
            if !active.white.connected {
                disconnected.push((Color::White, &active.white));
            }
            if !active.black.connected {
                disconnected.push((Color::Black, &active.black));
            }
        }

        disconnected
    }
}

pub enum MatchState {
    Lobby(LobbyState),
    InProgress(ActiveGame),
    Finished(GameResult, Timestamp),
}

pub struct LobbyState {
    pub player1: Option<PlayerSlot>,
    pub player2: Option<PlayerSlot>,
}

impl Default for LobbyState {
    fn default() -> Self {
        Self {
            player1: None,
            player2: None,
        }
    }
}

impl LobbyState {
    /// True when both slots are filled and both players are connected
    pub fn ready_to_start(&self) -> bool {
        matches!(
            (&self.player1, &self.player2),
            (Some(p1), Some(p2)) if p1.connected && p2.connected
        )
    }

    /// Check if a token is already in use (prevent duplicates)
    pub fn token_in_use(&self, token: &str) -> bool {
        self.find_player(token).is_some()
    }

    /// Find player slot by token
    pub fn find_player(&self, token: &str) -> Option<&PlayerSlot> {
        [&self.player1, &self.player2]
            .into_iter()
            .filter_map(|opt| opt.as_ref())
            .find(|p| p.token == token)
    }

    /// Find mutable player slot by token
    pub fn find_player_mut(&mut self, token: &str) -> Option<&mut PlayerSlot> {
        [&mut self.player1, &mut self.player2]
            .into_iter()
            .filter_map(|opt| opt.as_mut())
            .find(|p| p.token == token)
    }

    /// Get player number (1 or 2) for a token
    pub fn get_player_number(&self, token: &str) -> Option<u8> {
        if self
            .player1
            .as_ref()
            .map(|p| p.token == token)
            .unwrap_or(false)
        {
            Some(1)
        } else if self
            .player2
            .as_ref()
            .map(|p| p.token == token)
            .unwrap_or(false)
        {
            Some(2)
        } else {
            None
        }
    }
}

pub struct ActiveGame {
    pub started_at: Timestamp,
    pub white: PlayerInfo,
    pub black: PlayerInfo,
    pub state: State,
    pub game: Game,
    pub move_history: Vec<MoveRecord>,
    pub disconnection_timer: Option<JoinHandle<()>>,
    pub paused: bool,
}

impl Clone for ActiveGame {
    fn clone(&self) -> Self {
        Self {
            started_at: self.started_at,
            white: self.white.clone(),
            black: self.black.clone(),
            state: self.state.clone(),
            game: self.game.clone(),
            move_history: self.move_history.clone(),
            disconnection_timer: None, // Can't clone JoinHandle
            paused: self.paused,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub move_number: u32,
    pub player: Color,
    pub from: Coordinate,
    pub to: Coordinate,
    pub promotion: Option<PromotedFigure>,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug)]
pub struct MoveResult {
    pub new_state: State,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_stalemate: bool,
    pub game_over: Option<GameResult>,
}

pub enum MoveError {
    GameNotInProgress,
    NotYourTurn,
    InvalidMove,
    PlayerNotFound,
    GamePaused,
}

pub struct DisconnectionResult {
    pub player: Color,
    pub both_disconnected: bool,
    pub grace_period: Duration,
}

pub struct ReconnectionResult {
    pub player: Color,
    pub game_resumed: bool,
    pub state: State,
    pub move_history: Vec<MoveRecord>,
    pub current_turn: Color,
    pub white_connected: bool,
    pub black_connected: bool,
}

pub enum ReconnectError {
    GameNotInProgress,
    InvalidToken,
    AlreadyConnected,
}

#[derive(Clone, Debug)]
pub enum GameResult {
    WhiteWins(GameEndReason),
    BlackWins(GameEndReason),
    Draw(GameEndReason),
}

#[derive(Clone, Debug)]
pub enum GameEndReason {
    Checkmate,
    Stalemate,
    Resignation,
    DrawAgreement,
    Timeout,
    Abandoned,
}

pub enum JoinError {
    MatchNotFound,
    MatchAlreadyStarted,
    MatchFull,
    DuplicateToken,
}

pub enum AuthError {
    InvalidToken,
    MatchNotFound,
    MatchAlreadyStarted,
    AlreadyConnected,
}

pub enum StartError {
    NotReady,
}
