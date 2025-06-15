/// LOBBY PHASE
/// ========================================================================
/// Wait in the lobby, until there are two players, who both indicated that
/// they are ready to start the match.
pub mod lobby;

/// GAME PHASE
/// ========================================================================
/// The two players take turns and mutate the board state. We somehow need to
/// find a nice way to "merge" the two websocket connections, have them take
/// turns (e.g. do not accept "move" messages from white while its blacks
/// turn) and support reconnections.
pub mod game;

/// POST-GAME PHASE
/// ========================================================================
/// Do whatever here. I just think that the players should not be kicked out
/// right away after one wins.
pub mod postgame;
