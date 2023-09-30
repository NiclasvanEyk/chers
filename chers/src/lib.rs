/// The core structs like [`Board`], [`Piece`] or [`Player`].
mod structs;

/// Responsible for the game loop and higher-level processes.
mod engine;

/// Low-level coordinate movements
mod coordinates;

/// Computes valid moves given a game state and a starting position.
mod moves;

/// Parses a description in Forsyth–Edwards Notation.
pub mod fen;

pub use coordinates::*;
pub use engine::*;
pub use structs::*;
