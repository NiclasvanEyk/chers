use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    move_piece, moves_available::autocomplete_to, CantMovePiece, Coordinate, Engine, Event, Move,
    State,
};

use serde_wasm_bindgen as bridge;

#[derive(Serialize, Deserialize)]
pub enum MoveError {
    RequiresPromotion,
    InvalidMove,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub error: MoveError,
}

#[derive(Serialize, Deserialize)]
pub struct MoveResult {
    pub next_state: State,
    pub check: bool,
    pub mate: bool,
}

#[wasm_bindgen]
pub fn new_game() -> JsValue {
    let state = Engine::new().start();

    bridge::to_value(&state).unwrap()
}

#[wasm_bindgen]
pub fn available_moves(unsafe_state: JsValue, from: Coordinate) -> JsValue {
    let state: State = bridge::from_value(unsafe_state).unwrap();
    let moves = autocomplete_to(&state, from);

    bridge::to_value(&moves).unwrap()
}

#[wasm_bindgen]
pub fn next_state(unsafe_state: JsValue, the_move: Move) -> JsValue {
    let state: State = bridge::from_value(unsafe_state).unwrap();

    match move_piece(&state, the_move) {
        Err(error) => bridge::to_value(&Error {
            error: match error {
                CantMovePiece::NoPieceToMove => MoveError::InvalidMove,
                CantMovePiece::ItBelongsToOtherPlayer => MoveError::InvalidMove,
                CantMovePiece::RequiresPromotion => MoveError::RequiresPromotion,
                CantMovePiece::IllegalMove {
                    attempted: _,
                    legal: _,
                } => MoveError::InvalidMove,
            },
        })
        .unwrap(),
        Ok((next_state, events)) => {
            bridge::to_value(&MoveResult {
                next_state,
                check: events.iter().any(|x| match x {
                    Event::Capture { at, captured, by } => false,
                    Event::Promotion { to } => false,
                    Event::Check { by } => true,
                    Event::Mate => false,
                }), // TODO
                mate: events.iter().any(|x| match x {
                    Event::Capture { at, captured, by } => false,
                    Event::Promotion { to } => false,
                    Event::Check { by } => false,
                    Event::Mate => true,
                }), // TODO
            })
            .unwrap()
        }
    }
}
