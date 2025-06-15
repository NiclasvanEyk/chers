/// Logic for each state a room can be in.
pub mod states;

/// Storage layer for the matches.
pub mod repository;

use self::states::lobby::server::Lobby;
use chers::{Game, Player};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub type RoomId = u32;

pub struct ConnectionMetadata {
    /// A secret string that comes in helpful when one or both players
    /// disconnect. In that case, they (more like "their client apps behind the
    /// scenes") can present this token to the server to proof that
    recovery_token: String,
}

pub enum Room {
    /// The match has not started yet, maybe it is not even clear who plays
    /// against each other.
    Lobby(Arc<Mutex<Lobby>>),
    Game(MatchProgress),
    PostGame {
        duration: Duration,
        winner: Player,
    },
}

pub fn new_room() -> Room {
    Room::Lobby(Arc::new(Mutex::new(Lobby::default())))
}

struct MatchProgress {
    started_at: Instant,
    game: Game,
    white: ConnectionMetadata,
    black: ConnectionMetadata,
}
