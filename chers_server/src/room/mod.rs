use std::collections::HashMap;

pub use chers_server_api::v2::types::RoomId;

use crate::auth::{User, UserId};

pub mod storage;

/// Phase-independent container for players, spectators, etc.
///
/// Basically everything exists in the context of a [Room].
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    Lobby {
        ready_players: Vec<UserId>,
    },
    Game {
        /// The current board state.
        ///
        /// This contains all the information needed to reconstruct the game:
        /// - Board position (8x8 grid of pieces)
        /// - Current player (whose turn it is)
        /// - Castling rights
        /// - En passant target square (if any)
        /// - Halfmove clock (for 50-move rule)
        /// - Fullmove number
        state: chers::State,
        /// The player ID controlling the white pieces.
        white_player_id: UserId,
        /// The player ID controlling the black pieces.
        black_player_id: UserId,
        /// The sequence of moves made so far.
        ///
        /// Moves are stored in order — even indices are White's moves,
        /// odd indices are Black's moves. Replaying from the initial
        /// position reconstructs the board at any point.
        move_history: Vec<chers::Move>,
    },
    PostGame {
        winner: Option<UserId>,
        reason: Option<chers_server_api::v2::events::GameEndReason>,
        /// Snapshot of players at game end, persists across leaves.
        players: Vec<User>,
        /// Final board state at the moment the game ended.
        final_state: chers::State,
        /// The player who played white.
        white_player_id: UserId,
        /// The player who played black.
        black_player_id: UserId,
        /// The complete sequence of moves from the game.
        move_history: Vec<chers::Move>,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub phase: Phase,
    pub players: Vec<User>,
    pub auth: RoomAuth,
    /// Seconds of emptiness before the actor shuts down.
    /// `None` means never shut down automatically.
    pub empty_shutdown_secs: Option<u64>,
}

/// Per-room authentication state, persisted alongside the room.
///
/// Maps secrets → user identities. Unknown secrets are registered on first
/// use, keeping the protocol simple (no pre-registration needed).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RoomAuth {
    entries: HashMap<String, AuthEntry>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthEntry {
    pub user_id: UserId,
    pub name: String,
}

impl RoomAuth {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Resolve a secret to a user identity.
    ///
    /// If the secret is known the existing entry is returned (reconnect).
    /// Otherwise a new [`UserId`] is generated and a fresh entry is created,
    /// up to [`MAX_PLAYERS`] total entries.
    pub fn authenticate(&mut self, secret: &str, name: &str) -> Option<&AuthEntry> {
        // Reconnecting user — secret already known
        if self.entries.contains_key(secret) {
            return self.entries.get(secret);
        }

        // New user — only allow if we have capacity
        if self.entries.len() >= MAX_PLAYERS {
            return None;
        }

        let user_id = uuid::Uuid::now_v7().to_string();
        self.entries.insert(
            secret.to_owned(),
            AuthEntry {
                user_id,
                name: name.to_owned(),
            },
        );
        self.entries.get(secret)
    }

    /// Look up a secret without creating a new entry (used for reconnection).
    pub fn lookup(&self, secret: &str) -> Option<&AuthEntry> {
        self.entries.get(secret)
    }

    /// Directly insert an entry (useful for tests and pre-seeding).
    pub fn insert(&mut self, secret: &str, user_id: UserId, name: &str) {
        self.entries.insert(
            secret.to_owned(),
            AuthEntry {
                user_id,
                name: name.to_owned(),
            },
        );
    }
}

pub const MAX_PLAYERS: usize = 2;

impl Room {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            phase: Phase::Lobby {
                ready_players: Vec::new(),
            },
            players: Vec::with_capacity(MAX_PLAYERS),
            auth: RoomAuth::new(),
            empty_shutdown_secs: Some(60),
        }
    }
}
