use tokio::sync::broadcast;

use chers_server_api::{PrivateEvent, PublicEvent};

pub struct MatchChannels {
    /// Public events visible to all players and spectators
    pub public_tx: broadcast::Sender<PublicEvent>,
    /// Private events for player 1 (white or black)
    pub player1_tx: broadcast::Sender<PrivateEvent>,
    /// Private events for player 2 (the other color)
    pub player2_tx: broadcast::Sender<PrivateEvent>,
}

impl MatchChannels {
    pub fn new() -> Self {
        // Public channel with capacity for 100 spectators
        let (public_tx, _) = broadcast::channel(100);

        // Private channels - only 1 receiver per player
        let (player1_tx, _) = broadcast::channel(16);
        let (player2_tx, _) = broadcast::channel(16);

        Self {
            public_tx,
            player1_tx,
            player2_tx,
        }
    }
}

impl Default for MatchChannels {
    fn default() -> Self {
        Self::new()
    }
}
