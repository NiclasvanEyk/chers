use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    move_piece, moves_available::autocomplete_to, CantMovePiece, Coordinate, Event, Game, Move,
    State,
};

use serde_wasm_bindgen as bridge;

#[derive(Tsify, Serialize, Deserialize)]
pub struct MoveExecutionError {
    pub error: String,
}

#[derive(Tsify, Serialize, Deserialize)]
pub struct MoveExecutionResult {
    pub next_state: State,
    pub events: Vec<Event>,
    pub check: bool,
    pub mate: bool,
}

#[wasm_bindgen]
pub fn new_game() -> Result<JsValue, JsError> {
    let state = Game::new().start();
    bridge::to_value(&state).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn available_moves(state: JsValue, from: JsValue) -> Result<JsValue, JsError> {
    let state: State = bridge::from_value(state)
        .map_err(|e| JsError::new(&format!("Failed to deserialize state: {}", e)))?;
    let from: Coordinate = bridge::from_value(from)
        .map_err(|e| JsError::new(&format!("Failed to deserialize coordinate: {}", e)))?;

    let moves = autocomplete_to(&state, from);
    bridge::to_value(&moves).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn next_state(state: JsValue, the_move: JsValue) -> Result<JsValue, JsError> {
    let state: State = bridge::from_value(state)
        .map_err(|e| JsError::new(&format!("Failed to deserialize state: {}", e)))?;
    let the_move: Move = bridge::from_value(the_move)
        .map_err(|e| JsError::new(&format!("Failed to deserialize move: {}", e)))?;

    let result = match move_piece(&state, the_move) {
        Err(error) => {
            let error_str = match error {
                CantMovePiece::NoPieceToMove => "No piece to move",
                CantMovePiece::ItBelongsToOtherPlayer => "Piece belongs to opponent",
                CantMovePiece::RequiresPromotion => "Promotion required",
                CantMovePiece::IllegalMove { .. } => "Illegal move",
            };
            bridge::to_value(&MoveExecutionError {
                error: error_str.to_string(),
            })
        }
        Ok((next_state, events)) => bridge::to_value(&MoveExecutionResult {
            next_state,
            check: is_check(&events),
            mate: is_mate(&events),
            events,
        }),
    };

    result.map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

fn is_check(events: &[Event]) -> bool {
    events.iter().any(|x| match x {
        Event::Capture {
            at: _,
            captured: _,
            by: _,
        } => false,
        Event::Move {
            piece: _,
            from: _,
            to: _,
        } => false,
        Event::Promotion { to: _ } => false,
        Event::Check { by: _ } => true,
        Event::Mate => false,
    })
}

fn is_mate(events: &[Event]) -> bool {
    events.iter().any(|x| match x {
        Event::Capture {
            at: _,
            captured: _,
            by: _,
        } => false,
        Event::Move {
            piece: _,
            from: _,
            to: _,
        } => false,
        Event::Promotion { to: _ } => false,
        Event::Check { by: _ } => false,
        Event::Mate => true,
    })
}
