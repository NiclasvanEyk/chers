use chers::Color;
use serde::Serialize;

/// All messages sent before the game starts.
enum PendingGameMessages {
    /// A new player joined.
    PlayerJoined(),

    /// The player has defined a name for themselves.
    PlayerIdentified(),

    /// Both players agreed to start the game, switching the games state to
    /// "progressing".
    GameStarted(),
}

/// All messages sent while the game is in progress.
#[derive(Debug, Serialize)]
enum ProgressingGameMessages {
    /// Broadcasted when a legal move was made. Contains the updated board
    /// state, as well as any events that happened.
    Move(),

    /// One players offers the other a draw. This is only sent to the
    /// other player.
    OfferToDraw(),

    /// One of the players resigned.
    Resignation { who: Color },

    /// The other player agreed to end the game in a draw.
    AgreementToDraw(),

    /// One of the players disconnected.
    PlayerDisconnected { who: Color },

    /// One of the players reconnected.
    PlayerReconnected { who: Color },

    /// Broadcasted when there is no activity for a certain amount of time, or
    /// if the game just keeps going for too long.
    GameTimedOut(),
}
