use crate::auth::User;

/// Top-level event type, grouping events by phase.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Lobby(LobbyEvent),
    Game(GameEvent),
    PostGame(PostGameEvent),
    System(SystemEvent),
}

/// Events that are not tied to a specific phase.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum SystemEvent {
    CommandRejected { user: User, reason: String },
}

/// All events published during the lobby phase.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LobbyEvent {
    PlayerJoined { user: User },
    PlayerNameChanged { user: User },
    PlayerReadyChanged { user: User, is_ready: bool },
    PlayerLeft { user: User },
    JoinRejected { user: User, reason: String },
}

/// All events published while playing a game of chess.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum GameEvent {
    PlayerReconnected { user: User },
    PlayerLeft { user: User },
    GameStarted { white: User, black: User },
    Turn { author: User, turn: String },
    GameEnded { winner: User },
    MoveRejected { author: User, reason: String },
}

/// All events published after the winner has been decided.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum PostGameEvent {
    PlayerReconnected { user: User },
    PlayerLeft { user: User },
    RematchOffered { by: User },
    RematchAccepted { by: User },
    RematchDeclined { by: User },
}
