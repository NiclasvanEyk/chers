/// Lists all possible commands received and sent over the websocket connection.
pub mod communication;

/// Storage layer for the matches.
pub mod repository;

/// Everything before the game starts and how two players find each other.
pub mod negotiation;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chers::{Game, Player};

pub type MatchId = u32;

pub struct ConnectionMetadata {
    /// A secret string that comes in helpful when one or both players
    /// disconnect. In that case, they (more like "their client apps behind the
    /// scenes") can present this token to the server to proof that
    recovery_token: String,

    /// Can be used to measure the duration between now and the last time the
    /// client signified, that they are still connected and present.
    last_heartbeat_at: Instant,
}

impl ConnectionMetadata {
    pub fn token_matches(self, other: String) -> bool {
        self.recovery_token == other
    }

    pub fn duration_since_last_heartbeat(self) -> Duration {
        return Instant::now().duration_since(self.last_heartbeat_at);
    }
}

pub struct Match {
    pub id: MatchId,
    pub state: MatchState,
}

impl Match {
    pub fn new(id: MatchId) -> Match {
        return Match {
            id,
            state: MatchState::Pending(),
        };
    }
}

enum MatchState {
    /// The match has not started yet, maybe it is not even clear who plays
    /// against each other.
    Pending(),
    Progressing(MatchProgress),
    Finished {
        duration: Duration,
        winner: Player,
    },
}

struct MatchProgress {
    started_at: Instant,
    game: Game,
    white: ConnectionMetadata,
    black: ConnectionMetadata,
}
