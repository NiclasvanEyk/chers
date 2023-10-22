/// The core structs like [`Board`], [`Piece`] or [`Player`].
mod structs;

/// Responsible for the game loop and higher-level processes.
mod engine;

/// Low-level coordinate movements
mod coordinates;

/// Computes valid moves given a game state and a starting position.
mod moves_available;

/// TODO: Document
mod movement_patterns;

/// Computes the resulting state changes of a move.
mod move_execution;

/// Computes whether a given state represents check or even mate.
mod check;

/// Parses a description in Forsyth–Edwards Notation.
pub mod fen;

/// WASM bindings
pub mod wasm;

pub use coordinates::*;
pub use engine::*;
pub use move_execution::*;
pub use structs::*;
