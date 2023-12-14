/// A locally played game, where both players input their moves separately.
pub mod local;

/// A remotely played game over a TCP connection.
///
/// Players take turns and one has to wait for each others moves.
pub mod remote;
